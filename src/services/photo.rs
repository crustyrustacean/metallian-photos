// Shared, recoverable photo operations for both browser and API routes.

use crate::conversion::convert_heic_to_jpeg;
use crate::database::{DatabaseBackend, DatabaseError, PhotoStatus};
use crate::domain::{Exif, Photo};
use crate::exif::{get_raw_exif, parse_exif};
use crate::storage::StorageBackend;
use crate::telemetry::spawn_blocking_with_tracing;
use crate::utils::{e400, e500};
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};
use uuid::Uuid;

/// Coordinates photo operations within one application process. Locks are held
/// from before the first database write through the final storage operation.
/// Recovery uses the same locks and skips photos currently being processed.
#[derive(Default)]
pub struct PhotoOperations {
    locks: Mutex<HashMap<Uuid, Weak<AsyncMutex<()>>>>,
}

impl PhotoOperations {
    fn lock_for(&self, id: Uuid) -> Arc<AsyncMutex<()>> {
        let mut locks = self.locks.lock().unwrap_or_else(|e| e.into_inner());
        locks.retain(|_, lock| lock.strong_count() > 0);
        if let Some(lock) = locks.get(&id).and_then(Weak::upgrade) {
            return lock;
        }
        let lock = Arc::new(AsyncMutex::new(()));
        locks.insert(id, Arc::downgrade(&lock));
        lock
    }

    async fn lock(&self, id: Uuid) -> OwnedMutexGuard<()> {
        self.lock_for(id).lock_owned().await
    }

    /// Convert and persist one photo, publishing it only after storage succeeds.
    pub async fn create_photo_from_bytes(
        &self,
        photo_bytes: Bytes,
        band: String,
        tour: String,
        venue: String,
        database: &dyn DatabaseBackend,
        storage: &dyn StorageBackend,
    ) -> Result<Uuid, actix_web::Error> {
        let id = Uuid::new_v4();
        let (exif_data, jpeg_bytes) = spawn_blocking_with_tracing(move || {
            let exif_data = match get_raw_exif(&photo_bytes) {
                Ok(raw) => parse_exif(&raw),
                Err(_) => Exif::default(),
            };
            convert_heic_to_jpeg(&photo_bytes).map(|jpeg| (exif_data, Bytes::from(jpeg)))
        })
        .await
        .map_err(e500)?
        .map_err(e400)?;

        let _guard = self.lock(id).await;
        let photo = Photo {
            id,
            band,
            tour,
            venue,
            exif_data,
        };
        if let Err(error) = database.create_uploading(photo).await {
            // An error can be reported even if the insert committed.
            self.try_cleanup(id, database, storage).await;
            return Err(error.into());
        }
        if let Err(error) = storage.save(id, jpeg_bytes).await {
            // A timed-out write may nevertheless have reached storage.
            self.try_cleanup(id, database, storage).await;
            return Err(error.into());
        }

        let transition = database
            .transition_status(id, PhotoStatus::Uploading, PhotoStatus::Ready)
            .await;
        match transition {
            Ok(true) => Ok(id),
            result => {
                // If the update committed but its result was lost, keep the
                // published photo rather than deleting a valid object.
                if matches!(
                    database.photo_status(id).await,
                    Ok(Some(PhotoStatus::Ready))
                ) {
                    return Ok(id);
                }
                self.try_cleanup(id, database, storage).await;
                match result {
                    Err(error) => Err(error.into()),
                    Ok(false) => Err(e500(anyhow::anyhow!("photo could not be published: {id}"))),
                    Ok(true) => unreachable!(),
                }
            }
        }
    }

    /// Hide a photo first; storage and row removal may be retried safely.
    pub async fn delete_photo(
        &self,
        id: Uuid,
        database: &dyn DatabaseBackend,
        storage: &dyn StorageBackend,
    ) -> Result<(), actix_web::Error> {
        let _guard = self.lock(id).await;
        match database.photo_status(id).await? {
            None => return Ok(()),
            Some(PhotoStatus::Ready) => {
                if !database
                    .transition_status(id, PhotoStatus::Ready, PhotoStatus::Deleting)
                    .await?
                {
                    return Err(e500(anyhow::anyhow!(
                        "photo state changed during deletion: {id}"
                    )));
                }
            }
            Some(PhotoStatus::Deleting) => {}
            Some(PhotoStatus::Uploading) => {
                return Err(actix_web::error::ErrorConflict(
                    "photo upload is incomplete",
                ));
            }
        }
        self.cleanup_locked(id, database, storage)
            .await
            .map_err(e500)
    }

    /// Retry abandoned operations. Individual failures are logged and left for
    /// the next pass; only failure to enumerate pending work aborts the pass.
    pub async fn recover_incomplete(
        &self,
        database: &dyn DatabaseBackend,
        storage: &dyn StorageBackend,
    ) -> Result<(), DatabaseError> {
        for (id, _) in database.list_incomplete().await? {
            let Ok(_guard) = self.lock_for(id).try_lock_owned() else {
                continue;
            };
            self.try_cleanup(id, database, storage).await;
        }
        Ok(())
    }

    async fn try_cleanup(
        &self,
        id: Uuid,
        database: &dyn DatabaseBackend,
        storage: &dyn StorageBackend,
    ) {
        if let Err(error) = self.cleanup_locked(id, database, storage).await {
            tracing::warn!(photo_id = %id, error = ?error, "photo cleanup failed; will retry");
        }
    }

    // Caller holds the per-photo lock. Never remove storage for a ready photo,
    // even if a previous result was ambiguous or recovery had a stale list.
    async fn cleanup_locked(
        &self,
        id: Uuid,
        database: &dyn DatabaseBackend,
        storage: &dyn StorageBackend,
    ) -> anyhow::Result<()> {
        match database.photo_status(id).await? {
            Some(PhotoStatus::Uploading) => {
                if !database
                    .transition_status(id, PhotoStatus::Uploading, PhotoStatus::Deleting)
                    .await?
                {
                    return Ok(());
                }
            }
            Some(PhotoStatus::Deleting) => {}
            Some(PhotoStatus::Ready) | None => return Ok(()),
        }
        storage.delete(id).await?;
        database
            .delete_incomplete(id, PhotoStatus::Deleting)
            .await?;
        tracing::info!(photo_id = %id, "completed photo cleanup");
        Ok(())
    }
}

use async_trait::async_trait;
use bytes::Bytes;
use metallian_photos::configuration::DatabaseSettings;
use metallian_photos::database::{DatabaseBackend, DatabaseError, PhotoStatus, SqliteRepository};
use metallian_photos::domain::{Exif, Gallery, Photo, UpdatePhoto};
use metallian_photos::services::PhotoOperations;
use metallian_photos::storage::{StorageBackend, StorageError};
use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Notify, Semaphore};
use uuid::Uuid;

struct FaultStorage {
    objects: Mutex<HashMap<Uuid, Bytes>>,
    fail_save_after_write: AtomicBool,
    fail_delete: AtomicBool,
    pause_save: AtomicBool,
    save_started: Notify,
    resume_save: Semaphore,
}

impl Default for FaultStorage {
    fn default() -> Self {
        Self {
            objects: Mutex::new(HashMap::new()),
            fail_save_after_write: AtomicBool::new(false),
            fail_delete: AtomicBool::new(false),
            pause_save: AtomicBool::new(false),
            save_started: Notify::new(),
            resume_save: Semaphore::new(0),
        }
    }
}

#[async_trait]
impl StorageBackend for FaultStorage {
    async fn save(&self, id: Uuid, bytes: Bytes) -> Result<(), StorageError> {
        if self.pause_save.load(Ordering::SeqCst) {
            self.save_started.notify_one();
            self.resume_save.acquire().await.unwrap().forget();
        }
        self.objects.lock().unwrap().insert(id, bytes);
        if self.fail_save_after_write.load(Ordering::SeqCst) {
            return Err(StorageError::Operation(anyhow::anyhow!(
                "injected save error"
            )));
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), StorageError> {
        if self.fail_delete.load(Ordering::SeqCst) {
            return Err(StorageError::Operation(anyhow::anyhow!(
                "injected delete error"
            )));
        }
        self.objects.lock().unwrap().remove(&id);
        Ok(())
    }

    async fn find(&self, id: Uuid) -> Result<Bytes, StorageError> {
        self.objects
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(id.to_string()))
    }
}

struct FaultDatabase {
    inner: SqliteRepository,
    fail_create_after: AtomicBool,
    fail_publish_before: AtomicBool,
    fail_publish_after: AtomicBool,
    fail_delete_transition_after: AtomicBool,
    fail_cleanup_after: AtomicBool,
    fail_status: AtomicBool,
}

impl FaultDatabase {
    fn new(inner: SqliteRepository) -> Self {
        Self {
            inner,
            fail_create_after: AtomicBool::new(false),
            fail_publish_before: AtomicBool::new(false),
            fail_publish_after: AtomicBool::new(false),
            fail_delete_transition_after: AtomicBool::new(false),
            fail_cleanup_after: AtomicBool::new(false),
            fail_status: AtomicBool::new(false),
        }
    }
}

fn db_failure() -> DatabaseError {
    DatabaseError::Operation(anyhow::anyhow!("injected database error"))
}

#[async_trait]
impl DatabaseBackend for FaultDatabase {
    async fn create(&self, photo: Photo) -> Result<Uuid, DatabaseError> {
        self.inner.create(photo).await
    }
    async fn create_uploading(&self, photo: Photo) -> Result<Uuid, DatabaseError> {
        let id = self.inner.create_uploading(photo).await?;
        if self.fail_create_after.load(Ordering::SeqCst) {
            return Err(db_failure());
        }
        Ok(id)
    }
    async fn photo_status(&self, id: Uuid) -> Result<Option<PhotoStatus>, DatabaseError> {
        if self.fail_status.load(Ordering::SeqCst) {
            return Err(db_failure());
        }
        self.inner.photo_status(id).await
    }
    async fn transition_status(
        &self,
        id: Uuid,
        from: PhotoStatus,
        to: PhotoStatus,
    ) -> Result<bool, DatabaseError> {
        if to == PhotoStatus::Ready && self.fail_publish_before.load(Ordering::SeqCst) {
            return Err(db_failure());
        }
        let result = self.inner.transition_status(id, from, to).await?;
        if (to == PhotoStatus::Ready && self.fail_publish_after.load(Ordering::SeqCst))
            || (to == PhotoStatus::Deleting
                && self.fail_delete_transition_after.load(Ordering::SeqCst))
        {
            return Err(db_failure());
        }
        Ok(result)
    }
    async fn list_incomplete(&self) -> Result<Vec<(Uuid, PhotoStatus)>, DatabaseError> {
        self.inner.list_incomplete().await
    }
    async fn delete_incomplete(
        &self,
        id: Uuid,
        status: PhotoStatus,
    ) -> Result<bool, DatabaseError> {
        let result = self.inner.delete_incomplete(id, status).await?;
        if self.fail_cleanup_after.load(Ordering::SeqCst) {
            return Err(db_failure());
        }
        Ok(result)
    }
    async fn read(&self, id: Uuid) -> Result<Photo, DatabaseError> {
        self.inner.read(id).await
    }
    async fn list(&self) -> Result<Vec<Photo>, DatabaseError> {
        self.inner.list().await
    }
    async fn update(&self, id: Uuid, photo: UpdatePhoto) -> Result<(), DatabaseError> {
        self.inner.update(id, photo).await
    }
    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        self.inner.delete(id).await
    }
    async fn list_galleries(&self) -> Result<Vec<Gallery>, DatabaseError> {
        self.inner.list_galleries().await
    }
    async fn list_photos_by_band(&self, band: &str) -> Result<Vec<Photo>, DatabaseError> {
        self.inner.list_photos_by_band(band).await
    }
}

struct SharedStorage(Arc<FaultStorage>);

#[async_trait]
impl StorageBackend for SharedStorage {
    async fn save(&self, id: Uuid, bytes: Bytes) -> Result<(), StorageError> {
        self.0.save(id, bytes).await
    }
    async fn find(&self, id: Uuid) -> Result<Bytes, StorageError> {
        self.0.find(id).await
    }
    async fn delete(&self, id: Uuid) -> Result<(), StorageError> {
        self.0.delete(id).await
    }
}

async fn repository() -> SqliteRepository {
    SqliteRepository::new(&DatabaseSettings {
        path: ":memory:".into(),
        max_connections: Some(1),
    })
    .await
    .unwrap()
}

fn photo(id: Uuid, band: &str) -> Photo {
    Photo {
        id,
        band: band.into(),
        tour: "Tour".into(),
        venue: "Venue".into(),
        exif_data: Exif::default(),
    }
}

#[tokio::test]
async fn application_startup_recovers_abandoned_photos_before_serving() {
    use metallian_photos::configuration::get_configuration;
    use metallian_photos::startup::Application;
    use metallian_photos::template::tera::TeraRenderer;

    let db = repository().await;
    let storage = Arc::new(FaultStorage::default());
    let pending = Uuid::new_v4();
    let deleting = Uuid::new_v4();
    let ready = Uuid::new_v4();
    db.create_uploading(photo(pending, "Band")).await.unwrap();
    db.create(photo(deleting, "Band")).await.unwrap();
    db.transition_status(deleting, PhotoStatus::Ready, PhotoStatus::Deleting)
        .await
        .unwrap();
    db.create(photo(ready, "Band")).await.unwrap();
    for id in [pending, deleting, ready] {
        storage
            .save(id, Bytes::from_static(b"image"))
            .await
            .unwrap();
    }
    let mut config = get_configuration().unwrap();
    config.application.port = 0;
    config.admin.password = "testpassword".into();
    let app = Application::build(
        config,
        Box::new(TeraRenderer::new().unwrap()),
        Box::new(db.clone()),
        Box::new(SharedStorage(storage.clone())),
    )
    .await
    .unwrap();
    assert_eq!(db.photo_status(pending).await.unwrap(), None);
    assert_eq!(db.photo_status(deleting).await.unwrap(), None);
    assert_eq!(
        db.photo_status(ready).await.unwrap(),
        Some(PhotoStatus::Ready)
    );
    assert!(storage.find(pending).await.is_err());
    assert!(storage.find(deleting).await.is_err());
    assert!(storage.find(ready).await.is_ok());
    drop(app);
}

#[tokio::test]
async fn status_migration_preserves_existing_photos_and_can_be_reverted() {
    use sqlx::Connection;
    let mut connection = sqlx::SqliteConnection::connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::raw_sql(include_str!("../migrations/20260625045653_photos.up.sql"))
        .execute(&mut connection)
        .await
        .unwrap();
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO photos (id, band, tour, venue) VALUES (?, 'Band', 'Tour', 'Venue')")
        .bind(&id)
        .execute(&mut connection)
        .await
        .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261028000000_photo_status.up.sql"
    ))
    .execute(&mut connection)
    .await
    .unwrap();
    let row: (String, String) = sqlx::query_as("SELECT band, status FROM photos WHERE id = ?")
        .bind(&id)
        .fetch_one(&mut connection)
        .await
        .unwrap();
    assert_eq!(row, ("Band".into(), "ready".into()));
    sqlx::raw_sql(include_str!(
        "../migrations/20261028000000_photo_status.down.sql"
    ))
    .execute(&mut connection)
    .await
    .unwrap();
    let band: String = sqlx::query_scalar("SELECT band FROM photos WHERE id = ?")
        .bind(&id)
        .fetch_one(&mut connection)
        .await
        .unwrap();
    assert_eq!(band, "Band");
}

#[tokio::test]
async fn only_ready_photos_are_visible_in_reads_and_galleries() {
    let db = repository().await;
    let ready = Uuid::new_v4();
    let pending = Uuid::new_v4();
    let deleting = Uuid::new_v4();
    db.create(photo(ready, "Band")).await.unwrap();
    db.create_uploading(photo(pending, "Band")).await.unwrap();
    db.create(photo(deleting, "Other Band")).await.unwrap();
    assert!(
        db.transition_status(deleting, PhotoStatus::Ready, PhotoStatus::Deleting)
            .await
            .unwrap()
    );

    assert!(matches!(
        db.read(pending).await,
        Err(DatabaseError::NotFound(_))
    ));
    assert!(matches!(
        db.read(deleting).await,
        Err(DatabaseError::NotFound(_))
    ));
    assert_eq!(db.list().await.unwrap().len(), 1);
    assert_eq!(db.list_photos_by_band("Band").await.unwrap().len(), 1);
    let galleries = db.list_galleries().await.unwrap();
    assert_eq!(galleries.len(), 1);
    assert_eq!(galleries[0].photo_count, 1);
    assert_eq!(galleries[0].cover_photo_id, Some(ready));

    db.update(
        pending,
        UpdatePhoto {
            band: "Changed".into(),
            tour: "Changed".into(),
            venue: "Changed".into(),
        },
    )
    .await
    .unwrap();
    assert!(
        db.transition_status(pending, PhotoStatus::Uploading, PhotoStatus::Ready)
            .await
            .unwrap()
    );
    assert_eq!(db.read(pending).await.unwrap().band, "Band");
}

#[tokio::test]
async fn state_transitions_and_cleanup_are_conditional() {
    let db = repository().await;
    let id = Uuid::new_v4();
    db.create_uploading(photo(id, "Band")).await.unwrap();
    assert!(
        !db.transition_status(id, PhotoStatus::Ready, PhotoStatus::Deleting)
            .await
            .unwrap()
    );
    assert!(
        !db.delete_incomplete(id, PhotoStatus::Deleting)
            .await
            .unwrap()
    );
    assert_eq!(
        db.photo_status(id).await.unwrap(),
        Some(PhotoStatus::Uploading)
    );
    assert!(
        db.transition_status(id, PhotoStatus::Uploading, PhotoStatus::Deleting)
            .await
            .unwrap()
    );
    assert!(
        db.delete_incomplete(id, PhotoStatus::Deleting)
            .await
            .unwrap()
    );
    assert_eq!(db.photo_status(id).await.unwrap(), None);
}

fn heic() -> Bytes {
    Bytes::from_static(include_bytes!("fixtures/IMG_2215.HEIC"))
}

#[tokio::test]
async fn ambiguous_upload_failure_is_cleaned_up_and_retried_after_storage_recovers() {
    let db = repository().await;
    let storage = FaultStorage::default();
    let operations = PhotoOperations::default();
    storage.fail_save_after_write.store(true, Ordering::SeqCst);
    storage.fail_delete.store(true, Ordering::SeqCst);
    assert!(
        operations
            .create_photo_from_bytes(
                heic(),
                "Band".into(),
                "Tour".into(),
                "Venue".into(),
                &db,
                &storage
            )
            .await
            .is_err()
    );
    let pending = db.list_incomplete().await.unwrap();
    assert_eq!(pending.len(), 1);
    let id = pending[0].0;
    assert_eq!(pending[0].1, PhotoStatus::Deleting);
    assert!(storage.find(id).await.is_ok());
    operations.recover_incomplete(&db, &storage).await.unwrap();
    assert_eq!(
        db.photo_status(id).await.unwrap(),
        Some(PhotoStatus::Deleting)
    );
    storage.fail_delete.store(false, Ordering::SeqCst);
    operations.recover_incomplete(&db, &storage).await.unwrap();
    operations.recover_incomplete(&db, &storage).await.unwrap();
    assert_eq!(db.photo_status(id).await.unwrap(), None);
    assert!(matches!(
        storage.find(id).await,
        Err(StorageError::NotFound(_))
    ));
}

#[tokio::test]
async fn interrupted_delete_is_hidden_and_recovered() {
    let db = repository().await;
    let storage = FaultStorage::default();
    let operations = PhotoOperations::default();
    let id = Uuid::new_v4();
    db.create(photo(id, "Band")).await.unwrap();
    storage
        .save(id, Bytes::from_static(b"image"))
        .await
        .unwrap();
    storage.fail_delete.store(true, Ordering::SeqCst);
    assert!(operations.delete_photo(id, &db, &storage).await.is_err());
    assert_eq!(
        db.photo_status(id).await.unwrap(),
        Some(PhotoStatus::Deleting)
    );
    assert!(matches!(db.read(id).await, Err(DatabaseError::NotFound(_))));
    storage.fail_delete.store(false, Ordering::SeqCst);
    operations.recover_incomplete(&db, &storage).await.unwrap();
    assert_eq!(db.photo_status(id).await.unwrap(), None);
    assert!(storage.find(id).await.is_err());
    operations.delete_photo(id, &db, &storage).await.unwrap();
}

#[tokio::test]
async fn recovery_skips_upload_in_progress() {
    let db = Arc::new(repository().await);
    let storage = Arc::new(FaultStorage::default());
    let operations = Arc::new(PhotoOperations::default());
    storage.pause_save.store(true, Ordering::SeqCst);
    let task = {
        let (db, storage, operations) = (db.clone(), storage.clone(), operations.clone());
        tokio::spawn(async move {
            operations
                .create_photo_from_bytes(
                    heic(),
                    "Band".into(),
                    "Tour".into(),
                    "Venue".into(),
                    db.as_ref(),
                    storage.as_ref(),
                )
                .await
                .unwrap()
        })
    };
    storage.save_started.notified().await;
    let id = db.list_incomplete().await.unwrap()[0].0;
    operations
        .recover_incomplete(db.as_ref(), storage.as_ref())
        .await
        .unwrap();
    assert_eq!(
        db.photo_status(id).await.unwrap(),
        Some(PhotoStatus::Uploading)
    );
    storage.resume_save.add_permits(1);
    assert_eq!(task.await.unwrap(), id);
    assert_eq!(db.photo_status(id).await.unwrap(), Some(PhotoStatus::Ready));
    assert!(storage.find(id).await.is_ok());
}

#[tokio::test]
async fn recovery_cleans_up_an_abandoned_upload() {
    let db = repository().await;
    let storage = FaultStorage::default();
    let operations = PhotoOperations::default();
    let id = Uuid::new_v4();
    db.create_uploading(photo(id, "Band")).await.unwrap();
    storage
        .save(id, Bytes::from_static(b"image"))
        .await
        .unwrap();
    operations.recover_incomplete(&db, &storage).await.unwrap();
    assert_eq!(db.photo_status(id).await.unwrap(), None);
    assert!(storage.find(id).await.is_err());
}

#[tokio::test]
async fn committed_insert_error_cleans_up_its_row() {
    let db = FaultDatabase::new(repository().await);
    let storage = FaultStorage::default();
    let ops = PhotoOperations::default();
    db.fail_create_after.store(true, Ordering::SeqCst);
    assert!(
        ops.create_photo_from_bytes(
            heic(),
            "Band".into(),
            "Tour".into(),
            "Venue".into(),
            &db,
            &storage
        )
        .await
        .is_err()
    );
    assert!(db.list_incomplete().await.unwrap().is_empty());
    assert!(storage.objects.lock().unwrap().is_empty());
}

#[tokio::test]
async fn committed_publish_error_keeps_the_ready_photo() {
    let db = FaultDatabase::new(repository().await);
    let storage = FaultStorage::default();
    let ops = PhotoOperations::default();
    db.fail_publish_after.store(true, Ordering::SeqCst);
    let id = ops
        .create_photo_from_bytes(
            heic(),
            "Band".into(),
            "Tour".into(),
            "Venue".into(),
            &db,
            &storage,
        )
        .await
        .unwrap();
    assert_eq!(db.photo_status(id).await.unwrap(), Some(PhotoStatus::Ready));
    assert!(storage.find(id).await.is_ok());
    ops.recover_incomplete(&db, &storage).await.unwrap();
    assert!(storage.find(id).await.is_ok());
}

#[tokio::test]
async fn unknown_publish_outcome_never_deletes_a_committed_ready_photo() {
    let db = FaultDatabase::new(repository().await);
    let storage = FaultStorage::default();
    let ops = PhotoOperations::default();
    db.fail_publish_after.store(true, Ordering::SeqCst);
    db.fail_status.store(true, Ordering::SeqCst);
    assert!(
        ops.create_photo_from_bytes(
            heic(),
            "Band".into(),
            "Tour".into(),
            "Venue".into(),
            &db,
            &storage
        )
        .await
        .is_err()
    );
    let photos = db.list().await.unwrap();
    assert_eq!(photos.len(), 1);
    let id = photos[0].id;
    db.fail_status.store(false, Ordering::SeqCst);
    ops.recover_incomplete(&db, &storage).await.unwrap();
    assert_eq!(db.photo_status(id).await.unwrap(), Some(PhotoStatus::Ready));
    assert!(storage.find(id).await.is_ok());
}

#[tokio::test]
async fn unknown_uncommitted_publish_is_cleaned_up_later() {
    let db = FaultDatabase::new(repository().await);
    let storage = FaultStorage::default();
    let ops = PhotoOperations::default();
    db.fail_publish_before.store(true, Ordering::SeqCst);
    db.fail_status.store(true, Ordering::SeqCst);
    assert!(
        ops.create_photo_from_bytes(
            heic(),
            "Band".into(),
            "Tour".into(),
            "Venue".into(),
            &db,
            &storage
        )
        .await
        .is_err()
    );
    let id = db.list_incomplete().await.unwrap()[0].0;
    assert!(storage.find(id).await.is_ok());
    db.fail_status.store(false, Ordering::SeqCst);
    ops.recover_incomplete(&db, &storage).await.unwrap();
    assert_eq!(db.photo_status(id).await.unwrap(), None);
    assert!(storage.find(id).await.is_err());
}

#[tokio::test]
async fn uncommitted_publish_error_cleans_up_the_uploaded_object() {
    let db = FaultDatabase::new(repository().await);
    let storage = FaultStorage::default();
    let ops = PhotoOperations::default();
    db.fail_publish_before.store(true, Ordering::SeqCst);
    assert!(
        ops.create_photo_from_bytes(
            heic(),
            "Band".into(),
            "Tour".into(),
            "Venue".into(),
            &db,
            &storage
        )
        .await
        .is_err()
    );
    assert!(db.list_incomplete().await.unwrap().is_empty());
    assert!(storage.objects.lock().unwrap().is_empty());
}

#[tokio::test]
async fn committed_delete_transition_error_is_recovered() {
    let db = FaultDatabase::new(repository().await);
    let storage = FaultStorage::default();
    let ops = PhotoOperations::default();
    let id = Uuid::new_v4();
    db.create(photo(id, "Band")).await.unwrap();
    storage
        .save(id, Bytes::from_static(b"image"))
        .await
        .unwrap();
    db.fail_delete_transition_after
        .store(true, Ordering::SeqCst);
    assert!(ops.delete_photo(id, &db, &storage).await.is_err());
    assert_eq!(
        db.photo_status(id).await.unwrap(),
        Some(PhotoStatus::Deleting)
    );
    assert!(storage.find(id).await.is_ok());
    db.fail_delete_transition_after
        .store(false, Ordering::SeqCst);
    ops.recover_incomplete(&db, &storage).await.unwrap();
    assert_eq!(db.photo_status(id).await.unwrap(), None);
    assert!(storage.find(id).await.is_err());
}

#[tokio::test]
async fn committed_final_delete_error_is_safe_to_retry() {
    let db = FaultDatabase::new(repository().await);
    let storage = FaultStorage::default();
    let ops = PhotoOperations::default();
    let id = Uuid::new_v4();
    db.create(photo(id, "Band")).await.unwrap();
    storage
        .save(id, Bytes::from_static(b"image"))
        .await
        .unwrap();
    db.fail_cleanup_after.store(true, Ordering::SeqCst);
    assert!(ops.delete_photo(id, &db, &storage).await.is_err());
    assert_eq!(db.photo_status(id).await.unwrap(), None);
    assert!(storage.find(id).await.is_err());
    ops.delete_photo(id, &db, &storage).await.unwrap();
}

#[tokio::test]
async fn cancelled_upload_is_available_to_recovery() {
    let db = Arc::new(repository().await);
    let storage = Arc::new(FaultStorage::default());
    let ops = Arc::new(PhotoOperations::default());
    storage.pause_save.store(true, Ordering::SeqCst);
    let task = {
        let (db, storage, ops) = (db.clone(), storage.clone(), ops.clone());
        tokio::spawn(async move {
            ops.create_photo_from_bytes(
                heic(),
                "Band".into(),
                "Tour".into(),
                "Venue".into(),
                db.as_ref(),
                storage.as_ref(),
            )
            .await
            .unwrap()
        })
    };
    storage.save_started.notified().await;
    let id = db.list_incomplete().await.unwrap()[0].0;
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    ops.recover_incomplete(db.as_ref(), storage.as_ref())
        .await
        .unwrap();
    assert_eq!(db.photo_status(id).await.unwrap(), None);
}

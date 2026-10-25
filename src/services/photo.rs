// src/services/photo.rs
//
// Shared photo-creation pipeline. Extracted from the original `create_photo`
// API handler so that both the JSON API and the browser upload form can call
// it without duplicating the save logic.

use crate::conversion::convert_heic_to_jpeg;
use crate::database::DatabaseBackend;
use crate::domain::{Exif, Photo};
use crate::exif::{get_raw_exif, parse_exif};
use crate::storage::StorageBackend;
use crate::telemetry::spawn_blocking_with_tracing;
use crate::utils::{e400, e500};
use bytes::Bytes;
use uuid::Uuid;

/// Persist a new photo from raw image bytes and its metadata.
///
/// This is the single save pipeline shared by `POST /api/photos` and
/// `POST /upload`. It:
///
/// 1. Generates a fresh UUID for the photo.
/// 2. Extracts EXIF metadata from the original bytes (defaults to an empty
///    `Exif` when the bytes contain no parseable EXIF — a common case).
/// 3. Converts HEIC → JPEG for storage.
/// 4. Writes the metadata row to the database, then the JPEG bytes to storage.
///
/// Steps 2 and 3 are CPU-bound (~0.65 s per iPhone photo in a release build)
/// and run on Tokio's blocking thread pool. Running them inline would stall
/// the async worker — and on a single-CPU machine there is only one, so every
/// other request (public galleries, blog image fetches) would wait.
///
/// Returns the new photo's UUID so the caller decides the response shape —
/// JSON for the API route, a redirect for the browser form.
pub async fn create_photo_from_bytes(
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
    // The blocking task panicked or was cancelled — a server fault.
    .map_err(e500)?
    // The bytes weren't a decodable HEIC image — the client's fault.
    .map_err(e400)?;

    let photo = Photo {
        id,
        band,
        tour,
        venue,
        exif_data,
    };

    database.create(photo).await?;
    storage.save(id, jpeg_bytes).await?;

    Ok(id)
}

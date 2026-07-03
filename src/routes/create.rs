// src/routes/create.rs

use crate::storage::StorageBackend;
// dependencies
use crate::database::DatabaseBackend;
use crate::domain::Photo;
use crate::exif::{get_raw_exif, parse_exif};
use crate::utils::{e400, e500};
use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_web::HttpResponse;
use actix_web::web::Data;
use std::fs;
use uuid::Uuid;

#[derive(Debug, MultipartForm)]
pub struct CreateFormData {
    band: Text<String>,
    tour: Text<String>,
    venue: Text<String>,
    #[multipart(rename = "file")]
    photo_file: Vec<TempFile>,
}

/// create endpoint
pub async fn create(
    MultipartForm(form): MultipartForm<CreateFormData>,
    database: Data<Box<dyn DatabaseBackend>>,
    storage: Data<Box<dyn StorageBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::new_v4();
    let photo_file = fs::read(form.photo_file[0].file.path()).map_err(e400)?;
    let photo_file_bytes = photo_file.into();
    let raw_exif = get_raw_exif(&photo_file_bytes).map_err(e500)?;
    let photo = Photo {
        id,
        band: form.band.into_inner(),
        tour: form.tour.into_inner(),
        venue: form.venue.into_inner(),
        exif_data: parse_exif(&raw_exif),
    };

    database.create(photo).await.map_err(e500)?;
    storage.save(id, photo_file_bytes).await.map_err(e500)?;

    Ok(HttpResponse::Ok().json(id))
}

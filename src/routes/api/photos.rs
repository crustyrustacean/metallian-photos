// src/routes/api/photos.rs

use crate::database::DatabaseBackend;
use crate::domain::{Exif, Photo, UpdatePhoto};
use crate::exif::{get_raw_exif, parse_exif};
use crate::storage::StorageBackend;
use crate::utils::e400;
use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Form, Path};
use std::fs;
use uuid::Uuid;

#[derive(Debug, MultipartForm)]
pub struct CreatePhotoForm {
    band: Text<String>,
    tour: Text<String>,
    venue: Text<String>,
    #[multipart(rename = "file")]
    photo_file: Vec<TempFile>,
}

/// POST /api/photos
pub async fn create_photo(
    MultipartForm(form): MultipartForm<CreatePhotoForm>,
    database: Data<Box<dyn DatabaseBackend>>,
    storage: Data<Box<dyn StorageBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::new_v4();
    let photo_file = form
        .photo_file
        .into_iter()
        .next()
        .ok_or_else(|| e400("a file upload is required"))?;
    let photo_file_bytes = fs::read(photo_file.file.path()).map_err(e400)?.into();
    let exif_data = match get_raw_exif(&photo_file_bytes) {
        Ok(raw) => parse_exif(&raw),
        Err(_) => Exif::default(),
    };
    let photo = Photo {
        id,
        band: form.band.into_inner(),
        tour: form.tour.into_inner(),
        venue: form.venue.into_inner(),
        exif_data,
    };

    database.create(photo).await?;
    storage.save(id, photo_file_bytes).await?;

    Ok(HttpResponse::Ok().json(id))
}

/// GET /api/photos/{id}
pub async fn read_photo(
    path: Path<String>,
    database: Data<Box<dyn DatabaseBackend>>,
    _storage: Data<Box<dyn StorageBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;
    let photo = database.read(id).await?;

    Ok(HttpResponse::Ok().json(photo))
}

/// PUT /api/photos/{id}
pub async fn update_photo(
    path: Path<String>,
    form: Form<UpdatePhoto>,
    database: Data<Box<dyn DatabaseBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;
    let Form(form_data) = form;
    let updated_photo = UpdatePhoto {
        band: form_data.band,
        tour: form_data.tour,
        venue: form_data.venue,
    };

    database.update(id, updated_photo).await?;

    Ok(HttpResponse::Ok().finish())
}

/// DELETE /api/photos/{id}
pub async fn delete_photo(
    path: Path<String>,
    database: Data<Box<dyn DatabaseBackend>>,
    storage: Data<Box<dyn StorageBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;
    database.delete(id).await?;
    storage.delete(id).await?;

    Ok(HttpResponse::Ok().status(StatusCode::NO_CONTENT).finish())
}

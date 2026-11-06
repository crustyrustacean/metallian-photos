// src/routes/api/photos.rs

use crate::auth::require_login;
use crate::database::DatabaseBackend;
use crate::domain::UpdatePhoto;
use crate::services::PhotoOperations;
use crate::storage::StorageBackend;
use crate::utils::{e400, e500};
use actix_identity::Identity;
use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use actix_web::web::{self, Data, Form, Path};
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
    database: Data<dyn DatabaseBackend>,
    storage: Data<dyn StorageBackend>,
    operations: Data<PhotoOperations>,
    identity: Option<Identity>,
) -> Result<HttpResponse, actix_web::Error> {
    require_login(&identity)?;
    let photo_file = form
        .photo_file
        .into_iter()
        .next()
        .ok_or_else(|| e400("a file upload is required"))?;
    // Read the temp file on the blocking pool so the async worker stays free.
    let photo_file_bytes = web::block(move || fs::read(photo_file.file.path()))
        .await
        .map_err(e500)?
        .map_err(e400)?
        .into();

    let id = operations
        .create_photo_from_bytes(
            photo_file_bytes,
            form.band.into_inner(),
            form.tour.into_inner(),
            form.venue.into_inner(),
            database.get_ref(),
            storage.get_ref(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(id))
}

/// GET /api/photos/{id}
pub async fn read_photo(
    path: Path<String>,
    database: Data<dyn DatabaseBackend>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;
    let photo = database.read(id).await?;

    Ok(HttpResponse::Ok().json(photo))
}

/// GET /api/photos/{id}/image
///
/// Serves the stored JPEG bytes for a photo. This is the public read endpoint
/// that the gallery `<img>` tags and the blog will both pull from.
pub async fn get_photo_image(
    path: Path<String>,
    database: Data<dyn DatabaseBackend>,
    storage: Data<dyn StorageBackend>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;
    database.read(id).await?;
    let bytes = storage.find(id).await?;

    Ok(HttpResponse::Ok().content_type("image/jpeg").body(bytes))
}

/// PUT /api/photos/{id}
pub async fn update_photo(
    path: Path<String>,
    form: Form<UpdatePhoto>,
    database: Data<dyn DatabaseBackend>,
    identity: Option<Identity>,
) -> Result<HttpResponse, actix_web::Error> {
    require_login(&identity)?;
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
    database: Data<dyn DatabaseBackend>,
    storage: Data<dyn StorageBackend>,
    operations: Data<PhotoOperations>,
    identity: Option<Identity>,
) -> Result<HttpResponse, actix_web::Error> {
    require_login(&identity)?;
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;
    operations
        .delete_photo(id, database.get_ref(), storage.get_ref())
        .await?;

    Ok(HttpResponse::Ok().status(StatusCode::NO_CONTENT).finish())
}

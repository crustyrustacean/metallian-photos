// src/routes/frontend/upload.rs

use crate::auth::require_login;
use crate::database::DatabaseBackend;
use crate::services::create_photo_from_bytes;
use crate::storage::StorageBackend;
use crate::template::TemplateRenderer;
use crate::utils::e400;
use actix_identity::Identity;
use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_web::HttpResponse;

use actix_web::web::Data;
use serde::Serialize;
use std::fs;

#[derive(Serialize)]
struct PageContext<'a> {
    title: &'a str,
    header: &'a str,
    sub_header: &'a str,
}

/// GET /upload — render the upload form.
pub async fn get_upload_page(
    templates: Data<Box<dyn TemplateRenderer>>,
    identity: Identity,
) -> Result<HttpResponse, actix_web::Error> {
    require_login(&identity)?;
    let context = PageContext {
        title: "Upload",
        header: "Metallian Photos",
        sub_header: "Concert Photo Archive"
    };

    let json_context = serde_json::to_value(&context)?;

    let html = templates.render("upload.html", &json_context)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    band: Text<String>,
    tour: Text<String>,
    venue: Text<String>,
    #[multipart(rename = "file")]
    photo_file: Vec<TempFile>,
}

/// POST /upload — browser-facing upload. Saves the photo via the shared
/// pipeline, then redirects to the gallery on success.
///
/// Unlike `POST /api/photos` (which returns JSON), this route returns a
/// 303 See Other redirect so a standard HTML form gets a sensible
/// post-submit experience.
pub async fn post_upload(
    MultipartForm(form): MultipartForm<UploadForm>,
    database: Data<Box<dyn DatabaseBackend>>,
    storage: Data<Box<dyn StorageBackend>>,
    identity: Identity,
) -> Result<HttpResponse, actix_web::Error> {
    require_login(&identity)?;
    let photo_file = form
        .photo_file
        .into_iter()
        .next()
        .ok_or_else(|| e400("a file upload is required"))?;
    let photo_file_bytes = fs::read(photo_file.file.path()).map_err(e400)?.into();

    create_photo_from_bytes(
        photo_file_bytes,
        form.band.into_inner(),
        form.tour.into_inner(),
        form.venue.into_inner(),
        database.as_ref().as_ref(),
        storage.as_ref().as_ref(),
    )
    .await?;

    Ok(HttpResponse::SeeOther()
        .insert_header(("Location", "/gallery?status=success"))
        .finish())
}
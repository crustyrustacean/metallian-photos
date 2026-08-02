// src/routes/frontend/galleries.rs

use crate::database::DatabaseBackend;
use crate::domain::{Gallery, Photo};
use crate::template::TemplateRenderer;
use actix_web::{HttpResponse, web::{Data, Path}};
use serde::Serialize;

/// GET /g/{slug} — public read-only gallery for a single band.
///
/// The slug is derived from the band name (e.g. "iron-maiden"). We reverse
/// it by matching against the gallery list, since the slug is lossy
/// (non-ASCII chars are dropped, spaces collapsed).
pub async fn get_public_gallery(
    slug: Path<String>,
    templates: Data<Box<dyn TemplateRenderer>>,
    database: Data<Box<dyn DatabaseBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let slug = slug.into_inner();
    let galleries = database.list_galleries().await?;

    let gallery = galleries
        .into_iter()
        .find(|g| g.slug == slug)
        .ok_or_else(|| {
            actix_web::error::ErrorNotFound(format!("No gallery found for slug: {slug}"))
        })?;

    let photos = database.list_photos_by_band(&gallery.band).await?;

    let context = GalleryPageContext {
        title: &gallery.band,
        header: "Metallian Photos",
        sub_header: "Concert Photo Archive",
        band: &gallery.band,
        photo_count: gallery.photo_count,
        photos,
    };

    let json_context = serde_json::to_value(&context)?;
    let html = templates.render("public_gallery.html", &json_context)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

#[derive(Serialize)]
struct GalleryPageContext<'a> {
    title: &'a str,
    header: &'a str,
    sub_header: &'a str,
    band: &'a str,
    photo_count: i64,
    photos: Vec<Photo>,
}

/// GET /galleries — admin-facing list of all galleries (for the nav link).
pub async fn get_galleries_index(
    templates: Data<Box<dyn TemplateRenderer>>,
    database: Data<Box<dyn DatabaseBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let galleries = database.list_galleries().await?;

    let context = GalleriesIndexContext {
        title: "Galleries",
        header: "Metallian Photos",
        sub_header: "Concert Photo Archive",
        galleries,
    };

    let json_context = serde_json::to_value(&context)?;
    let html = templates.render("galleries_index.html", &json_context)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

#[derive(Serialize)]
struct GalleriesIndexContext<'a> {
    title: &'a str,
    header: &'a str,
    sub_header: &'a str,
    galleries: Vec<Gallery>,
}

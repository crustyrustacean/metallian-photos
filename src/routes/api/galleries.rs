// src/routes/api/galleries.rs

use crate::database::DatabaseBackend;
use actix_web::{HttpResponse, web::Data};

/// GET /api/galleries
///
/// Returns a JSON array of all galleries (one per unique band), with each
/// gallery containing the band name, a URL slug, the photo count, and the
/// cover photo id (most recent photo for that band).
///
/// This is the endpoint the blog fetches to render its "Galleries" page.
pub async fn list_galleries(
    database: Data<Box<dyn DatabaseBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let galleries = database.list_galleries().await?;
    Ok(HttpResponse::Ok().json(galleries))
}

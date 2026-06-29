// src/routes/create.rs

use crate::storage::StorageBackend;
// dependencies
use crate::database::DatabaseBackend;
use crate::domain::{Exif, Photo};
use crate::utils::e500;
use actix_web::HttpResponse;
use actix_web::web::{Data, Form};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    band: String,
    tour: String,
    venue: String,
}

/// create endpoint
pub async fn create(
    form: Form<FormData>,
    database: Data<Box<dyn DatabaseBackend>>,
    _storage: Data<Box<dyn StorageBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::new_v4();
    let Form(form_data) = form;
    let photo = Photo {
        id,
        band: form_data.band,
        tour: form_data.tour,
        venue: form_data.venue,
        exif_data: Exif {
            date_time_original: None,
            make: None,
            model: None,
            lens_make: None,
            lens_model: None,
        },
    };

    database.create(photo).await.map_err(e500)?;
    
    Ok(HttpResponse::Ok().json(id))
}

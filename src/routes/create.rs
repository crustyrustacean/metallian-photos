// src/routes/create.rs

use crate::storage::StorageBackend;
// dependencies
use crate::domain::{Exif, Photo};
use crate::response::ApiResponse;
use crate::{database::DatabaseBackend, error::ApiError};
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
) -> Result<ApiResponse<Uuid>, ApiError> {
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

    database.create(photo).await?;

    Ok(ApiResponse::success(id))
}

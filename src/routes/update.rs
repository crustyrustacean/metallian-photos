// src/routes/update.rs

// dependencies
use crate::database::DatabaseBackend;
use crate::domain::{Exif, Photo};
use crate::storage::StorageBackend;
use crate::utils::e400;
use actix_web::HttpResponse;
use actix_web::web::{Data, Form, Path};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct UpdateFormData {
    band: String,
    tour: String,
    venue: String,
}

/// update endpoint
pub async fn update(
    path: Path<String>,
    form: Form<UpdateFormData>,
    database: Data<Box<dyn DatabaseBackend>>,
    _storage: Data<Box<dyn StorageBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;
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

    database.update(photo).await?;

    Ok(HttpResponse::Ok().finish())
}

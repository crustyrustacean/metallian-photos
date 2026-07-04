// src/routes/update.rs

// dependencies
use crate::database::DatabaseBackend;
use crate::domain::UpdatePhoto;
use crate::utils::e400;
use actix_web::HttpResponse;
use actix_web::web::{Data, Form, Path};
use uuid::Uuid;

/// update endpoint
pub async fn update(
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

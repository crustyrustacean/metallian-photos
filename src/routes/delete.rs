// src/routes/delete.rs

// dependencies
use crate::database::DatabaseBackend;
use crate::storage::StorageBackend;
use crate::utils::{e400, e500};
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use uuid::Uuid;

/// create endpoint
pub async fn delete(
    path: Path<String>,
    database: Data<Box<dyn DatabaseBackend>>,
    _storage: Data<Box<dyn StorageBackend>>,
) -> Result<HttpResponse, actix_web::Error> {
    let id_string = path.into_inner();
    let id = Uuid::parse_str(&id_string).map_err(e400)?;
    database.delete(id).await.map_err(e500)?;

    Ok(HttpResponse::Ok().status(StatusCode::NO_CONTENT).finish())
}
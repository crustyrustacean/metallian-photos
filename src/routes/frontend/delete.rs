// src/routes/frontend/delete.rs

use super::sse::patch_response;
use crate::auth::require_login;
use crate::database::DatabaseBackend;
use crate::services::PhotoOperations;
use crate::storage::StorageBackend;
use crate::utils::e400;
use actix_identity::Identity;
use actix_web::Responder;
use actix_web::web::{Data, Path};
use datastar::prelude::PatchElements;
use uuid::Uuid;

/// DELETE /gallery/{id} — browser-facing delete. Removes the photo from
/// storage and database, then returns a Datastar SSE event that removes
/// the photo's `<li>` from the DOM without a page reload.
pub async fn delete_photo(
    path: Path<String>,
    database: Data<dyn DatabaseBackend>,
    storage: Data<dyn StorageBackend>,
    operations: Data<PhotoOperations>,
    identity: Option<Identity>,
) -> Result<impl Responder, actix_web::Error> {
    require_login(&identity)?;
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;

    operations
        .delete_photo(id, database.get_ref(), storage.get_ref())
        .await?;

    Ok(patch_response(PatchElements::new_remove(format!(
        "#photo-{id}"
    ))))
}

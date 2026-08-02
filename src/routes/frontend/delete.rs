// src/routes/frontend/delete.rs

use crate::auth::require_login;
use crate::database::DatabaseBackend;
use crate::storage::StorageBackend;
use crate::utils::e400;
use actix_identity::Identity;
use actix_web::web::{Data, Path};
use datastar::actix::Sse;
use datastar::prelude::PatchElements;
use uuid::Uuid;

/// DELETE /gallery/{id} — browser-facing delete. Removes the photo from
/// storage and database, then returns a Datastar SSE event that removes
/// the photo's `<li>` from the DOM without a page reload.
pub async fn delete_photo(
    path: Path<String>,
    database: Data<Box<dyn DatabaseBackend>>,
    storage: Data<Box<dyn StorageBackend>>,
    identity: Identity,
) -> Result<Sse, actix_web::Error> {
    require_login(&identity)?;
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;

    storage.delete(id).await?;
    database.delete(id).await?;

    Ok(PatchElements::new_remove(format!("#photo-{id}")).into())
}
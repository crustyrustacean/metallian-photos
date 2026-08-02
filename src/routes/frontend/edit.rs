// src/routes/frontend/edit.rs
//
// Inline metadata editing for gallery photos. Three handlers power a
// two-state morph: the caption area (#caption-{id}) starts as a display
// `<p>`, morphs into an edit form on "Edit", and morphs back on "Save"
// or "Cancel". All three return Datastar SSE patch-elements events so
// the page never reloads.
//
// HTML fragments are rendered via Tera partials (templates/partials/),
// not hand-built format!() strings. Tera's auto-escaping handles the
// HTML attribute layer — the class of injection bug that raw format!()
// strings are vulnerable to.

use crate::auth::require_login;
use crate::database::DatabaseBackend;
use crate::domain::UpdatePhoto;
use crate::template::TemplateRenderer;
use crate::utils::e400;
use actix_identity::Identity;
use actix_web::web::{Data, Path};
use datastar::actix::{ReadSignals, Sse};
use datastar::prelude::PatchElements;
use uuid::Uuid;

/// Build the Tera context for a photo caption fragment.
///
/// The `signals` field is pre-serialized JSON so Tera's auto-escaping
/// handles the HTML attribute layer correctly when it renders into
/// `data-signals="{{ signals }}"`.
fn caption_context(id: Uuid, band: &str, tour: &str, venue: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "band": band,
        "tour": tour,
        "venue": venue,
        "signals": serde_json::json!({
            "band": band,
            "tour": tour,
            "venue": venue,
        }).to_string(),
    })
}

/// GET /gallery/{id}/edit
///
/// Morphs the caption display into an inline edit form. The form uses
/// `data-signals` to seed the inputs with the photo's current metadata,
/// `data-bind` for two-way binding, and `@put` to save via the update
/// handler below.
pub async fn edit_photo(
    path: Path<String>,
    database: Data<Box<dyn DatabaseBackend>>,
    templates: Data<Box<dyn TemplateRenderer>>,
    identity: Identity,
) -> Result<Sse, actix_web::Error> {
    require_login(&identity)?;
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;
    let photo = database.read(id).await?;

    let context = caption_context(id, &photo.band, &photo.tour, &photo.venue);
    let html = templates.render("partials/edit_form.html", &context)?;

    Ok(PatchElements::new(html).into())
}

/// PUT /gallery/{id}
///
/// Reads the edited metadata from Datastar signals via `ReadSignals<UpdatePhoto>`,
/// updates the database, then morphs the form back into the display caption
/// with the new values.
///
/// This is the first handler to use `ReadSignals<T>` — signals flowing
/// from browser to server.
pub async fn update_photo(
    path: Path<String>,
    ReadSignals(updates): ReadSignals<UpdatePhoto>,
    database: Data<Box<dyn DatabaseBackend>>,
    templates: Data<Box<dyn TemplateRenderer>>,
    identity: Identity,
) -> Result<Sse, actix_web::Error> {
    require_login(&identity)?;
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;

    database.update(id, updates).await?;
    let photo = database.read(id).await?;

    let context = serde_json::json!({
        "id": id,
        "band": photo.band,
        "tour": photo.tour,
        "venue": photo.venue,
    });
    let html = templates.render("partials/caption.html", &context)?;

    Ok(PatchElements::new(html).into())
}

/// GET /gallery/{id}/cancel
///
/// Morphs the edit form back into the display caption without saving.
pub async fn cancel_edit(
    path: Path<String>,
    database: Data<Box<dyn DatabaseBackend>>,
    templates: Data<Box<dyn TemplateRenderer>>,
    identity: Identity,
) -> Result<Sse, actix_web::Error> {
    require_login(&identity)?;
    let id = Uuid::parse_str(&path.into_inner()).map_err(e400)?;
    let photo = database.read(id).await?;

    let context = serde_json::json!({
        "id": id,
        "band": photo.band,
        "tour": photo.tour,
        "venue": photo.venue,
    });
    let html = templates.render("partials/caption.html", &context)?;

    Ok(PatchElements::new(html).into())
}

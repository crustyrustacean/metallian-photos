// src/routes/frontend/sse.rs

use actix_web::Responder;
use actix_web_lab::sse::{Event, Sse};
use datastar::prelude::PatchElements;
use futures_util::stream;

/// Return a single Datastar patch as an SSE response.
pub(super) fn patch_response(patch: PatchElements) -> impl Responder {
    let event: Event = patch.into();
    Sse::from_infallible_stream(stream::iter([event]))
}

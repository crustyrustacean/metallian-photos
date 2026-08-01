// src/services.rs
//
// Shared backend logic used by multiple routes. Both the JSON API
// (`POST /api/photos`) and the browser upload form (`POST /upload`) are
// consumers of the same save pipeline — the functions here are the single
// source of truth for it.

pub mod photo;

pub use photo::*;

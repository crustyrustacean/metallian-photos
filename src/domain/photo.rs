// src/domain/photo.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Photo {
    pub id: Uuid,
    pub band: String,
    pub tour: String,
    pub venue: String,
    pub exif_data: Exif,
}

#[derive(Debug, Default, Serialize)]
pub struct Exif {
    pub date_time_original: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePhoto {
    pub band: String,
    pub tour: String,
    pub venue: String,
}

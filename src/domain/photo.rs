use uuid::Uuid;

pub struct Photo {
    pub id: Uuid,
    pub band: String,
    pub tour: String,
    pub venue: String,
    pub exif_data: Exif,
}

pub struct Exif {
    pub date_time_original: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
}

// src/routes/frontend/gallery.rs

use crate::database::DatabaseBackend;
use crate::domain::Photo;
use crate::template::TemplateRenderer;
use actix_web::{HttpResponse, web::Data};
use serde::Serialize;

#[derive(Serialize)]
struct PageContext<'a> {
    title: &'a str,
    header: &'a str,
    sub_header: &'a str,
    photos: Vec<Photo>,
}

pub async fn get_gallery_page(templates: Data<Box<dyn TemplateRenderer>>, database: Data<Box<dyn DatabaseBackend>>) -> Result<HttpResponse, actix_web::Error> {
    let photos = database.list().await?;
    
    let context = PageContext {
        title: "Gallery",
        header: "Metallian Photos",
        sub_header: "Concert Photo Archive",
        photos
    };

    let json_context = serde_json::to_value(&context)?;

    let html = templates.render("gallery.html", &json_context)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}
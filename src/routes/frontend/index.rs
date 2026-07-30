// src/routes/frontend/index.rs

use crate::template::TemplateRenderer;
use actix_web::{HttpResponse, web::Data};
use serde::Serialize;

#[derive(Serialize)]
struct PageContext<'a> {
    title: &'a str,
    header: &'a str,
}

/// GET /
pub async fn get_index_page(templates: Data<Box<dyn TemplateRenderer>>) -> HttpResponse {
    let context = PageContext {
        title: "Home",
        header: "Hello, World!",
    };

    let json_context = serde_json::to_value(&context).unwrap();

    match templates.render("index.html", &json_context) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

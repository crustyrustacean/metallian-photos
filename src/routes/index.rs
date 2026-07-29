// src/routes/index.rs

use actix_web::{web, HttpResponse};
use crate::template::TemplateRenderer;
use serde::Serialize;

#[derive(Serialize)]
struct PageContext<'a> {
    title: &'a str,
    content: &'a str,
}

pub async fn get_index_page(templates: web::Data<Box<dyn TemplateRenderer>>) -> HttpResponse {
    let context = PageContext {
        title: "Home",
        content: "Welcome!!",
    };
    
    let json_context = serde_json::to_value(&context).unwrap();
    
    match templates.render("index.html", &json_context) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
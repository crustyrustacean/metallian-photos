// src/routes/frontend/index.rs

use crate::template::TemplateRenderer;
use actix_web::{HttpResponse, web::Data};
use serde::Serialize;

#[derive(Serialize)]
struct PageContext<'a> {
    title: &'a str,
    header: &'a str,
    sub_header: &'a str,
}

pub async fn get_index_page(templates: Data<Box<dyn TemplateRenderer>>) -> Result<HttpResponse, actix_web::Error> {
    let context = PageContext {
        title: "Home",
        header: "R2 Photo API",
        sub_header: "Landing Page"
    };

    let json_context = serde_json::to_value(&context)?;

     let html = templates.render("index.html", &json_context)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}
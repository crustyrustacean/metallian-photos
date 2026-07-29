// src/routes/index.rs

use actix_web::web::{Data, Html};
use tera::Tera;

pub async fn get_index_page(tera_templates:Data<Tera>) -> Result<Html, actix_web::Error> {
    todo!()
}
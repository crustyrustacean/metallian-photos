// src/template/tera.rs

use crate::template::{TemplateError, TemplateRenderer};
use anyhow::Context;
use serde::Serialize;
use tera:: Tera;

#[derive(Debug, Serialize)]
struct PageContext<'a> {
    title: &'a str,
    content: &'a str,
}

struct TeraTemplate<'a, T: Serialize> {
    template_name: &'a str,
    context: &'a T,
}

impl TemplateRenderer for TeraTemplate<'_, PageContext<'_>> {
    fn render<T: Serialize>(&self, template_name: &str, context: &T) -> Result<String, TemplateError> {
        todo!()
    }
}
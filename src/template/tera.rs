// src/template/tera.rs

use anyhow::Context;
use crate::template::{TemplateError, TemplateRenderer};
use serde::Serialize;
use tera::{Context as TeraContext, Tera};

// Keep your page context for use elsewhere, but the renderer 
// doesn't need to know about it.
#[derive(Debug)]
pub struct PageContext<'a> {
    pub title: &'a str,
    pub content: &'a str,
}

// The renderer only needs to hold the Tera engine state
pub struct TeraRenderer {
    pub engine: Tera,
}

// Implement the trait generally so it can accept ANY serializable context
impl TemplateRenderer for TeraRenderer {
    fn render(&self, template_name: &str, context: &serde_json::Value) -> Result<String, TemplateError> {
        
        let template_context = TeraContext::from_serialize(context)
            .context("Unable to build the page context")?;

        let output = self.engine.render(template_name, &template_context)
            .context("Tera engine failed to render template")?;

        Ok(output)
    }
}
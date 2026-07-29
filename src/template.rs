// src/template.rs

use crate::utils::error_chain_fmt;
use serde::Serialize;
use actix_web::{ResponseError, http::StatusCode};
use thiserror::Error;

mod tera;

#[derive(Error)]
pub enum TemplateError {
    #[error("template not found: {0}")]
    NotFound(String),

    #[error("A failure was encountered while trying to render the template.")]
    Operation(#[from] anyhow::Error),
}

impl std::fmt::Debug for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for TemplateError {
    fn status_code(&self) -> StatusCode {
        match self {
            TemplateError::NotFound(_) => StatusCode::NOT_FOUND,
            TemplateError::Operation(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub trait TemplateRenderer: Send + Sync {
    fn render<T: Serialize>(&self, template_name: &str, context: &T) -> Result<String, TemplateError>;
}
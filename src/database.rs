// src/database.rs

use crate::domain::Photo;
use crate::utils::error_chain_fmt;
use actix_web::{ResponseError, http::StatusCode};
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

pub mod sqlite;

pub use sqlite::*;

#[derive(Error)]
pub enum DatabaseError {
    #[error("record not found: {0}")]
    NotFound(String),

    #[error(transparent)]
    Operation(#[from] anyhow::Error),
}

impl std::fmt::Debug for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for DatabaseError {
    fn status_code(&self) -> StatusCode {
        match self {
            DatabaseError::NotFound(_) => StatusCode::NOT_FOUND,
            DatabaseError::Operation(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[async_trait]
pub trait DatabaseBackend: Send + Sync {
    async fn create(&self, photo: Photo) -> Result<Uuid, DatabaseError>;
    async fn read(&self, id: Uuid) -> Result<Photo, DatabaseError>;
    async fn update(&self, photo: Photo) -> Result<(), DatabaseError>;
    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError>;
}

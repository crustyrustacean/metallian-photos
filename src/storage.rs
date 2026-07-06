// src/storage.rs

use crate::utils::error_chain_fmt;
use actix_web::{ResponseError, http::StatusCode};
use async_trait::async_trait;
use bytes::Bytes;
use thiserror::Error;
use uuid::Uuid;

mod in_memory;
mod opendal;

pub use in_memory::*;
pub use opendal::*;

#[derive(Error)]
pub enum StorageError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error(transparent)]
    Operation(#[from] anyhow::Error),
}

impl std::fmt::Debug for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for StorageError {
    fn status_code(&self) -> StatusCode {
        match self {
            StorageError::NotFound(_) => StatusCode::NOT_FOUND,
            StorageError::Operation(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn delete(&self, id: Uuid) -> Result<(), StorageError>;
    async fn find(&self, id: Uuid) -> Result<Bytes, StorageError>;
    async fn save(&self, id: Uuid, bytes: Bytes) -> Result<(), StorageError>;
}

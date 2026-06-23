// src/storage.rs

use async_trait::async_trait;
use bytes::Bytes;
use thiserror::Error;
use uuid::Uuid;

mod opendal;

pub use opendal::*;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("storage operation failed: {0}")]
    Operation(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn delete(&self, id: Uuid) -> Result<(), StorageError>;
    async fn find(&self, id: Uuid) -> Result<Bytes, StorageError>;
    async fn save(&self, id: Uuid, bytes: Bytes) -> Result<(), StorageError>;
}

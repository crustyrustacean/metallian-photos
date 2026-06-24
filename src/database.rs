// src/database.rs

use crate::domain::Photo;
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

pub mod sqlite;

pub use sqlite::*;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("record not found: {0}")]
    NotFound(String),

    #[error("database operation failed: {0}")]
    Operation(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[async_trait]
pub trait DatabaseBackend: Send + Sync {
    async fn create(&self, photo: Photo) -> Result<Uuid, DatabaseError>;
    async fn read(&self, id: Uuid) -> Result<Photo, DatabaseError>;
    async fn update(&self, photo: Photo) -> Result<(), DatabaseError>;
    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError>;
}

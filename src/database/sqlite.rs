// src/database/sqlite.rs

use crate::configuration::DatabaseSettings;
use crate::database::{DatabaseBackend, DatabaseError};
use crate::domain::Photo;
use async_trait::async_trait;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn new(db_configuration: DatabaseSettings) -> Result<Self, anyhow::Error> {
        let db_path = format!("sqlite:{}", db_configuration.path);
        let options = SqliteConnectOptions::from_str(&db_path)?.create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl DatabaseBackend for SqliteRepository {
    async fn create(&self, photo: Photo) -> Result<Uuid, DatabaseError> {
        sqlx::query(
            "INSERT INTO photos (id, band, tour, venue, date_time_original, make, model, lens_make, lens_model)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(photo.id)
        .bind(photo.band)
        .bind(photo.tour)
        .bind(photo.venue)
        .bind(photo.exif_data.date_time_original)
        .bind(photo.exif_data.make)
        .bind(photo.exif_data.model)
        .bind(photo.exif_data.lens_make)
        .bind(photo.exif_data.lens_model)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::Operation(e.into()))?;

        Ok(photo.id)
    }

    async fn read(&self, id: Uuid) -> Result<Photo, DatabaseError> {
        let result: Photo = sqlx::query_as("SELECT * FROM photos WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::NotFound(e.to_string()))?;

        Ok(result)
    }

    async fn update(&self, photo: Photo) -> Result<(), DatabaseError> {
        sqlx::query(
             "UPDATE photos SET band = ?, tour = ?, venue = ?, date_time_original = ?, make = ?, model = ?, lens_make = ?, lens_model = ? WHERE id = ?"
        )
        .bind(photo.band)
        .bind(photo.tour)
        .bind(photo.venue)
        .bind(photo.exif_data.date_time_original)
        .bind(photo.exif_data.make)
        .bind(photo.exif_data.model)
        .bind(photo.exif_data.lens_make)
        .bind(photo.exif_data.lens_model)
        .bind(photo.id)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::Operation(e.into()))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM photos WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::Operation(e.into()))?;

        Ok(())
    }
}

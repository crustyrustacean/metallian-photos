// src/database/sqlite.rs

use crate::configuration::DatabaseSettings;
use crate::database::{DatabaseBackend, DatabaseError};
use crate::domain::{Exif, Photo};
use async_trait::async_trait;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{FromRow, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct PhotoRow {
    pub id: Uuid,
    pub band: String,
    pub tour: String,
    pub venue: String,
    pub date_time_original: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
}

impl From<PhotoRow> for Photo {
    fn from(row: PhotoRow) -> Self {
        Photo {
            id: row.id,
            band: row.band,
            tour: row.tour,
            venue: row.venue,
            exif_data: Exif {
                date_time_original: row.date_time_original,
                make: row.make,
                model: row.model,
                lens_make: row.lens_make,
                lens_model: row.lens_model,
            },
        }
    }
}

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
        let result: PhotoRow = sqlx::query_as("SELECT * FROM photos WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => DatabaseError::NotFound(id.to_string()),
                _ => DatabaseError::Operation(e.into()),
            })?;

        Ok(result.into())
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

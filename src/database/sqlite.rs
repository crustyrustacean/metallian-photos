// src/database/sqlite.rs

use crate::configuration::DatabaseSettings;
use crate::database::{DatabaseBackend, DatabaseError};
use crate::domain::{Exif, Photo};
use anyhow::Context;
use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{FromRow, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;
use uuid::fmt::Hyphenated;

// `photos.id` is declared `TEXT PRIMARY KEY` in the schema, so the SQLite
// layer stores the UUID as its canonical hyphenated string form. sqlx encodes
// `Uuid` itself as a 16-byte BLOB on SQLite, which would never compare equal
// to a TEXT value; using `Hyphenated` keeps the on-disk representation
// human-readable *and* consistent with the string the API exposes.
//
// Bind a `Uuid` as hyphenated text so it matches the stored column type.
fn id_as_text(id: Uuid) -> Hyphenated {
    id.hyphenated()
}

#[derive(Debug, FromRow)]
struct PhotoRow {
    pub id: Hyphenated,
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
            id: row.id.into_uuid(),
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
    pub async fn new(db_configuration: &DatabaseSettings) -> Result<Self, anyhow::Error> {
        let db_path = format!("sqlite:{}", db_configuration.path);
        let mut pool_opts = SqlitePoolOptions::new();
        if let Some(max) = db_configuration.max_connections {
            pool_opts = pool_opts.max_connections(max);
        }
        let options = SqliteConnectOptions::from_str(&db_path)?.create_if_missing(true);
        let pool = pool_opts.connect_with(options).await?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("Failed to run the database migrations.")?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl DatabaseBackend for SqliteRepository {
    async fn create(&self, photo: Photo) -> Result<Uuid, DatabaseError> {
        sqlx::query(
            "INSERT INTO photos (id, band, tour, venue, date_time_original, make, model, lens_make, lens_model)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id_as_text(photo.id))
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
        .context("Failed to create the photo.")?;

        Ok(photo.id)
    }

    async fn read(&self, id: Uuid) -> Result<Photo, DatabaseError> {
        let row: Option<PhotoRow> = sqlx::query_as("SELECT * FROM photos WHERE id = ?")
            .bind(id_as_text(id))
            .fetch_optional(&self.pool)
            .await
            .context("Failed to fetch the photo.")?;
        let row = row.ok_or_else(|| DatabaseError::NotFound(id.to_string()))?;

        Ok(row.into())
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
        .bind(id_as_text(photo.id))
        .execute(&self.pool)
        .await
        .context("Failed to update the photo.")?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM photos WHERE id = ?")
            .bind(id_as_text(id))
            .execute(&self.pool)
            .await
            .context("Failed to delete the photo.")?;

        Ok(())
    }
}

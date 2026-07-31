// src/database/sqlite.rs

use crate::configuration::DatabaseSettings;
use crate::database::{DatabaseBackend, DatabaseError};
use crate::domain::{Exif, Photo, UpdatePhoto};
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

    async fn list(&self) -> Result<Vec<Photo>, DatabaseError> {
        let rows: Vec<PhotoRow> = sqlx::query_as::<_, PhotoRow>("SELECT * FROM photos")
            .fetch_all(&self.pool)
            .await
            .context("Failed to list the photos.")?;

        Ok(rows.into_iter().map(Photo::from).collect())
    }

    async fn update(&self, id: Uuid, updated_photo: UpdatePhoto) -> Result<(), DatabaseError> {
        sqlx::query("UPDATE photos SET band = ?, tour = ?, venue = ? WHERE id = ?")
            .bind(updated_photo.band)
            .bind(updated_photo.tour)
            .bind(updated_photo.venue)
            .bind(id_as_text(id))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str) -> PhotoRow {
        PhotoRow {
            id: Uuid::parse_str(id).unwrap().hyphenated(),
            band: "Band".into(),
            tour: "Tour".into(),
            venue: "Venue".into(),
            date_time_original: Some("2024-01-02 03:04:05".into()),
            make: Some("Apple".into()),
            model: Some("iPhone 16 Pro Max".into()),
            lens_make: Some("Apple".into()),
            lens_model: Some("iPhone 16 Pro Max back camera".into()),
        }
    }

    #[test]
    fn photo_row_maps_all_fields_including_exif() {
        let r = row("550e8400-e29b-41d4-a716-446655440000");
        let p: Photo = r.into();

        assert_eq!(
            p.id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(p.band, "Band");
        assert_eq!(p.tour, "Tour");
        assert_eq!(p.venue, "Venue");
        assert_eq!(
            p.exif_data.date_time_original.as_deref(),
            Some("2024-01-02 03:04:05")
        );
        assert_eq!(p.exif_data.make.as_deref(), Some("Apple"));
        assert_eq!(p.exif_data.model.as_deref(), Some("iPhone 16 Pro Max"));
        assert_eq!(p.exif_data.lens_make.as_deref(), Some("Apple"));
        assert_eq!(
            p.exif_data.lens_model.as_deref(),
            Some("iPhone 16 Pro Max back camera")
        );
    }

    #[test]
    fn photo_row_maps_when_all_exif_fields_are_absent() {
        let mut r = row("550e8400-e29b-41d4-a716-446655440000");
        r.date_time_original = None;
        r.make = None;
        r.model = None;
        r.lens_make = None;
        r.lens_model = None;
        let p: Photo = r.into();

        assert!(p.exif_data.date_time_original.is_none());
        assert!(p.exif_data.make.is_none());
        assert!(p.exif_data.model.is_none());
        assert!(p.exif_data.lens_make.is_none());
        assert!(p.exif_data.lens_model.is_none());
    }

    #[test]
    fn id_as_text_round_trips_through_parse() {
        let id = Uuid::new_v4();
        let encoded = id_as_text(id).to_string();
        assert_eq!(encoded.parse::<Uuid>().unwrap(), id);
    }

    // ------------------------------------------------------------------
    // CRUD tests — exercise the DatabaseBackend trait against an in-memory
    // SQLite database (max_connections = 1 so every op hits the same DB).
    // ------------------------------------------------------------------

    async fn test_repo() -> SqliteRepository {
        SqliteRepository::new(&DatabaseSettings {
            path: ":memory:".to_string(),
            max_connections: Some(1),
        })
        .await
        .expect("failed to build test repo")
    }

    fn sample_photo(id: Uuid) -> Photo {
        Photo {
            id,
            band: "The Band".into(),
            tour: "Tour of Champions".into(),
            venue: "Best Ever".into(),
            exif_data: Exif::default(),
        }
    }

    // --- create ---

    #[tokio::test]
    async fn create_persists_and_returns_id() {
        let db = test_repo().await;
        let id = Uuid::new_v4();

        let returned = db.create(sample_photo(id)).await.unwrap();
        assert_eq!(returned, id);

        // round-trip: it can be read back
        let read = db.read(id).await.unwrap();
        assert_eq!(read.band, "The Band");
    }

    #[tokio::test]
    async fn create_with_duplicate_id_errors() {
        let db = test_repo().await;
        let id = Uuid::new_v4();
        db.create(sample_photo(id)).await.unwrap();

        let result = db.create(sample_photo(id)).await;
        assert!(matches!(result, Err(DatabaseError::Operation(_))));
    }

    // --- read ---

    #[tokio::test]
    async fn read_returns_stored_photo() {
        let db = test_repo().await;
        let id = Uuid::new_v4();
        db.create(sample_photo(id)).await.unwrap();

        let photo = db.read(id).await.unwrap();
        assert_eq!(photo.id, id);
        assert_eq!(photo.band, "The Band");
        assert_eq!(photo.tour, "Tour of Champions");
        assert_eq!(photo.venue, "Best Ever");
    }

    #[tokio::test]
    async fn list_returns_empty_for_no_photos() {
        let db = test_repo().await;
        
        let photos = db.list().await.unwrap();
        assert!(photos.is_empty());
    }

    #[tokio::test]
    async fn list_returns_all_stored_photos() {
        let db = test_repo().await;
        db.create(sample_photo(Uuid::new_v4())).await.unwrap();
        db.create(sample_photo(Uuid::new_v4())).await.unwrap();
        
        let photos = db.list().await.unwrap();
        assert_eq!(photos.len(), 2);
        assert_eq!(photos[0].band, "The Band");
        assert_eq!(photos[1].band, "The Band");
    }

    #[tokio::test]
    async fn read_on_missing_id_returns_not_found() {
        let db = test_repo().await;
        let result = db.read(Uuid::new_v4()).await;
        assert!(matches!(result, Err(DatabaseError::NotFound(_))));
    }

    // --- update ---

    #[tokio::test]
    async fn update_changes_stored_fields() {
        let db = test_repo().await;
        let id = Uuid::new_v4();
        db.create(sample_photo(id)).await.unwrap();

        db.update(
            id,
            UpdatePhoto {
                band: "New Band".into(),
                tour: "New Tour".into(),
                venue: "New Venue".into(),
            },
        )
        .await
        .unwrap();

        let updated = db.read(id).await.unwrap();
        assert_eq!(updated.band, "New Band");
        assert_eq!(updated.tour, "New Tour");
        assert_eq!(updated.venue, "New Venue");
    }

    #[tokio::test]
    async fn update_on_missing_id_succeeds_silently() {
        // SQL `UPDATE ... WHERE id = ?` on a non-existent row affects 0 rows
        // but is not itself an error. This documents that the repo mirrors
        // SQL's behaviour: it returns Ok.
        let db = test_repo().await;
        let result = db
            .update(
                Uuid::new_v4(),
                UpdatePhoto {
                    band: "Ghost".into(),
                    tour: "Ghost".into(),
                    venue: "Ghost".into(),
                },
            )
            .await;
        assert!(result.is_ok());
    }

    // --- delete ---

    #[tokio::test]
    async fn delete_removes_photo() {
        let db = test_repo().await;
        let id = Uuid::new_v4();
        db.create(sample_photo(id)).await.unwrap();

        db.delete(id).await.unwrap();

        // subsequent read fails with NotFound
        let result = db.read(id).await;
        assert!(matches!(result, Err(DatabaseError::NotFound(_))));
    }

    #[tokio::test]
    async fn delete_on_missing_id_succeeds_silently() {
        // SQL `DELETE ... WHERE id = ?` on a non-existent row affects 0 rows
        // but is not itself an error. This documents that the repo mirrors
        // SQL's behaviour: it returns Ok.
        let db = test_repo().await;
        let result = db.delete(Uuid::new_v4()).await;
        assert!(result.is_ok());
    }
}
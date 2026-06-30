// tests/api/update.rs

// dependencies
use crate::helpers::spawn_app;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
struct PhotoData {
    band: String,
    tour: String,
    venue: String,
}

#[allow(dead_code)]
#[derive(FromRow)]
struct StoredPhotoData {
    id: String,
    band: String,
    tour: String,
    venue: String,
}

#[tokio::test]
async fn update_photo_endpoint_updates_photo_and_returns_200_ok() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let photo_data = PhotoData {
        band: "The Band".to_string(),
        tour: "Tour of Champions".to_string(),
        venue: "Best Ever".to_string(),
    };

    // Act — create the photo first so we have an id to update
    let response = client
        .post(&format!("{}/photos", &app.address))
        .form(&photo_data)
        .send()
        .await
        .expect("Failed to execute request.");

    let id: Uuid = response
        .json()
        .await
        .expect("Unable to obtain response body.");

    let updated_photo_data = PhotoData {
        band: "The Real Band".to_string(),
        tour: "The Real Tour of Champions".to_string(),
        venue: "Second Best Ever".to_string(),
    };

    let response = client
        .put(&format!("{}/photos/{}", &app.address, id.to_string()))
        .form(&updated_photo_data)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());

    let db_pool = app.database.pool();
    let result: StoredPhotoData = sqlx::query_as("SELECT * FROM photos WHERE id = ?")
        .bind(&id.to_string())
        .fetch_one(db_pool)
        .await
        .unwrap();

    assert_eq!(result.band, updated_photo_data.band);
    assert_eq!(result.tour, updated_photo_data.tour);
    assert_eq!(result.venue, updated_photo_data.venue);
}

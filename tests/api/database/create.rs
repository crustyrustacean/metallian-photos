// tests/api/create.rs

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

#[derive(FromRow)]
struct StoredPhotoData {
    id: String,
    band: String,
    tour: String,
    venue: String,
}

#[tokio::test]
async fn create_photo_endpoint_returns_200_ok_and_id() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let photo_data = PhotoData {
        band: "The Band".to_string(),
        tour: "Tour of Champions".to_string(),
        venue: "Best Ever".to_string(),
    };

    // Act
    let response = client
        .post(&format!("{}/photos", &app.address))
        .form(&photo_data)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());

    let id: Uuid = response
        .json()
        .await
        .expect("Unable to obtain response body.");

    let id_string = id.to_string();
    let db_pool = app.database.pool();

    let result: StoredPhotoData = sqlx::query_as("SELECT * FROM photos WHERE id = ?")
        .bind(&id_string)
        .fetch_one(db_pool)
        .await
        .unwrap();

    assert_eq!(result.id, id_string);
    assert_eq!(result.band, photo_data.band);
    assert_eq!(result.tour, photo_data.tour);
    assert_eq!(result.venue, photo_data.venue);
}

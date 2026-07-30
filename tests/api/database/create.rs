// tests/api/create.rs

// dependencies
use crate::helpers::{create_photo, spawn_app};
use reqwest::multipart;
use sqlx::FromRow;

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

    // Act
    let id = create_photo(&client, &app.address).await;

    // Assert — the photo is persisted with the submitted metadata
    let id_string = id.to_string();
    let db_pool = app.database.pool();

    let result: StoredPhotoData = sqlx::query_as("SELECT * FROM photos WHERE id = ?")
        .bind(&id_string)
        .fetch_one(db_pool)
        .await
        .unwrap();

    assert_eq!(result.id, id_string);
    assert_eq!(result.band, "The Band");
    assert_eq!(result.tour, "Tour of Champions");
    assert_eq!(result.venue, "Best Ever");
}

#[tokio::test]
async fn create_photo_endpoint_returns_400_for_bad_multipart_form_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let form = multipart::Form::new()
        .text("band", "The Band")
        .text("tour", "Tour of Champions")
        .text("venue", "Best Ever");

    // Act
    let response = client
        .post(&format!("{}/api/photos", &app.address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
    
}
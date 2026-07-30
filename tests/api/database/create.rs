// tests/api/create.rs

// dependencies
use crate::helpers::{create_photo, create_photo_with_fixture, spawn_app};
use reqwest::multipart;
use serde::Deserialize;
use sqlx::FromRow;

#[derive(FromRow)]
struct StoredPhotoData {
    id: String,
    band: String,
    tour: String,
    venue: String,
}

#[derive(Deserialize)]
struct PhotoResponse {
    id: String,
    band: String,
    tour: String,
    venue: String,
    exif_data: ExifData,
}

#[derive(Deserialize)]
struct ExifData {
    date_time_original: Option<String>,
    make: Option<String>,
    model: Option<String>,
    lens_make: Option<String>,
    lens_model: Option<String>,
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

#[tokio::test]
async fn create_photo_with_fixture_extracts_and_persists_exif_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act — create a photo using the real HEIC fixture, then read it back
    let id = create_photo_with_fixture(&client, &app.address).await;

    let response = client
        .get(&format!("{}/api/photos/{}", &app.address, id))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert — the EXIF data extracted from the fixture is persisted and
    // returned by the read endpoint. These values are captured from the
    // IMG_2215.HEIC fixture (same as the exif.rs unit test).
    let photo: PhotoResponse = response
        .json()
        .await
        .expect("Unable to deserialize response body.");

    assert_eq!(photo.exif_data.date_time_original.as_deref(), Some("2026:01:24 19:55:19"));
    assert_eq!(photo.exif_data.make.as_deref(), Some("Apple"));
    assert_eq!(photo.exif_data.model.as_deref(), Some("iPhone 16 Pro Max"));
    assert_eq!(photo.exif_data.lens_make.as_deref(), Some("Apple"));
    assert_eq!(
        photo.exif_data.lens_model.as_deref(),
        Some("iPhone 16 Pro Max back triple camera 15.66mm f/2.8")
    );
}
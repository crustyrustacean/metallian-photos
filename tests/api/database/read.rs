// tests/api/read.rs

// dependencies
use crate::helpers::{create_photo, login, spawn_app};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct StoredPhotoData {
    id: String,
    band: String,
    tour: String,
    venue: String,
}

#[tokio::test]
async fn read_photo_endpoint_returns_200_ok_and_a_single_photo() {
    // Arrange
    let app = spawn_app().await;
    login(&app).await;

    let id: Uuid = create_photo(&app.admin_client, &app.address).await;

    // Act
    let response = reqwest::Client::new()
        .get(&format!("{}/api/photos/{}", &app.address, id))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    let stored_photo: StoredPhotoData = response
        .json()
        .await
        .expect("Unable to deserialize response body.");

    assert_eq!(stored_photo.id, id.to_string());
    assert_eq!(stored_photo.band, "The Band");
    assert_eq!(stored_photo.tour, "Tour of Champions");
    assert_eq!(stored_photo.venue, "Best Ever");
}

#[tokio::test]
async fn read_photo_endpoint_with_malformed_uuid_returns_400() {
    // Arrange
    let app = spawn_app().await;
    login(&app).await;

    // Act
    let response = reqwest::Client::new()
        .get(&format!(
            "{}/api/photos/{}",
            &app.address, "not-a-valid-uuid"
        ))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn read_photo_endpoint_with_unknown_uuid_returns_404() {
    // Arrange
    let app = spawn_app().await;
    login(&app).await;

    let missing_id = Uuid::new_v4();

    // Act
    let response = reqwest::Client::new()
        .get(&format!("{}/api/photos/{}", &app.address, missing_id))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn get_photo_image_returns_jpeg_bytes() {
    // Arrange
    let app = spawn_app().await;
    login(&app).await;

    let id: Uuid = create_photo(&app.admin_client, &app.address).await;

    // Act
    let response = reqwest::Client::new()
        .get(&format!("{}/api/photos/{}/image", &app.address, id))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert — the image endpoint returns JPEG bytes
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        response.headers().get("CONTENT-TYPE").unwrap(),
        "image/jpeg"
    );

    let bytes = response.bytes().await.expect("Failed to read body");
    assert!(!bytes.is_empty(), "image body should not be empty");
    assert_eq!(bytes[0], 0xFF, "JPEG magic byte");
    assert_eq!(bytes[1], 0xD8, "JPEG magic byte");
}

#[tokio::test]
async fn get_photo_image_with_unknown_uuid_returns_404() {
    // Arrange
    let app = spawn_app().await;
    login(&app).await;

    let missing_id = Uuid::new_v4();

    // Act
    let response = reqwest::Client::new()
        .get(&format!("{}/api/photos/{}/image", &app.address, missing_id))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 404);
}

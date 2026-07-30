// tests/api/read.rs

// dependencies
use crate::helpers::{create_photo, spawn_app};
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
    let client = reqwest::Client::new();

    let id: Uuid = create_photo(&client, &app.address).await;

    // Act
    let response = client
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
    let client = reqwest::Client::new();

    // Act
    let response = client
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
    let client = reqwest::Client::new();

    let missing_id = Uuid::new_v4();

    // Act
    let response = client
        .get(&format!("{}/api/photos/{}", &app.address, missing_id))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 404);
}

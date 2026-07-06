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
        .get(&format!("{}/photos/{}", &app.address, id))
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

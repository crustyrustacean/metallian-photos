// tests/api/read.rs

// dependencies
use crate::helpers::spawn_app;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
struct PhotoData {
    band: String,
    tour: String,
    venue: String,
}

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

    let response = client
        .get(&format!("{}/photos/{}", &app.address, id))
        .send()
        .await
        .expect("Failed to execute request");

    let stored_photo: StoredPhotoData = response
        .json()
        .await
        .expect("Unable to deserialize response body.");

    assert_eq!(stored_photo.id, id.to_string());
    assert_eq!(stored_photo.band, photo_data.band);
    assert_eq!(stored_photo.tour, photo_data.tour);
    assert_eq!(stored_photo.venue, photo_data.venue);
}

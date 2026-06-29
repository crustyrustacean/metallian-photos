// tests/api/create.rs

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

#[tokio::test]
async fn delete_photo_endpoint_returns_204_no_content() {
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
        .delete(&format!("{}/photos/{}", &app.address, id.to_string()))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 204);
    assert_eq!(Some(0), response.content_length());
}

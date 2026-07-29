// tests/api/delete.rs

// dependencies
use crate::helpers::{create_photo, spawn_app};

#[tokio::test]
async fn delete_photo_endpoint_returns_204_no_content() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let id = create_photo(&client, &app.address).await;

    // Act
    let response = client
        .delete(&format!("{}/photos/{}", &app.address, id))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 204);
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn delete_photo_endpoint_with_malformed_uuid_returns_400() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .delete(&format!("{}/photos/{}", &app.address, "not-a-valid-uuid"))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

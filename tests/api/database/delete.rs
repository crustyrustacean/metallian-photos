// tests/api/delete.rs

// dependencies
use crate::helpers::{create_photo, login, spawn_app};

#[tokio::test]
async fn delete_photo_endpoint_returns_204_no_content() {
    // Arrange
    let app = spawn_app().await;
    login(&app).await;

    let id = create_photo(&app.admin_client, &app.address).await;

    // Act
    let response = app.admin_client
        .delete(&format!("{}/api/photos/{}", &app.address, id))
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
    login(&app).await;

    // Act
    let response = app.admin_client
        .delete(&format!(
            "{}/api/photos/{}",
            &app.address, "not-a-valid-uuid"
        ))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

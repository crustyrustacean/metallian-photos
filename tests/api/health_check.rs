// tests/api/health_check.rs

// dependencies
use crate::helpers::spawn_app;
use r2_photo_api::response::ApiResponse;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());

    let body: ApiResponse<()> = response
        .json()
        .await
        .expect("Failed to deserialize response.");

    assert!(body.success);
    assert!(body.data.is_none());
    assert!(body.error.is_none());
}

// tests/api/health.rs

use crate::helpers::spawn_app;

#[tokio::test]
async fn index_page_returns_200_ok_and_renders_html_content() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());

    let body = response.text().await.unwrap();
    assert!(body.contains("<!DOCTYPE html>"));
}

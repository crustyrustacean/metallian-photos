// tests/api/upload.rs

use crate::helpers::spawn_app;
use metallian_photos::database::DatabaseBackend;
use reqwest::multipart;

#[tokio::test]
async fn upload_form_returns_400_for_missing_file() {
    // Arrange
    let app = spawn_app().await;

    let form = multipart::Form::new()
        .text("band", "The Band")
        .text("tour", "Tour of Champions")
        .text("venue", "Best Ever");

    // Act
    let response = app
        .api_client
        .post(&format!("{}/upload", &app.address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn upload_redirects_to_gallery_and_persists_photo() {
    // Arrange
    let app = spawn_app().await;

    let fixture = include_bytes!("../fixtures/IMG_2215.HEIC");
    let file_part = multipart::Part::bytes(fixture.as_slice().to_vec())
        .file_name("IMG_2215.HEIC")
        .mime_str("image/heic")
        .unwrap();

    let form = multipart::Form::new()
        .text("band", "The Band")
        .text("tour", "Tour of Champions")
        .text("venue", "Best Ever")
        .part("file", file_part);

    // Act — POST to the browser upload endpoint. The test client has
    // redirects disabled, so we observe the raw 303 rather than following it.
    let response = app
        .api_client
        .post(&format!("{}/upload", &app.address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute upload request");

    // Assert — 303 See Other redirect to /gallery
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("LOCATION").unwrap(), "/gallery?status=success");

    // The photo was persisted through the shared save pipeline.
    let photos = app
        .database
        .list()
        .await
        .expect("Failed to list photos");
    assert_eq!(photos.len(), 1);
    assert_eq!(photos[0].band, "The Band");
    assert_eq!(photos[0].venue, "Best Ever");
}

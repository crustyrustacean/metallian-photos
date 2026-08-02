// tests/api/upload.rs

use crate::helpers::{login, spawn_app};
use metallian_photos::database::DatabaseBackend;
use reqwest::multipart;

#[tokio::test]
async fn upload_form_returns_400_for_missing_file() {
    // Arrange
    let app = spawn_app().await;
    login(&app).await;
    let form = multipart::Form::new()
        .text("band", "The Band")
        .text("tour", "Tour of Champions")
        .text("venue", "Best Ever");

    // Act
    let response = app
        .admin_client
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
    login(&app).await;
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

    // Act — POST to the browser upload endpoint. Use the no-redirect
    // client with the session cookie manually attached.
    // We can't use admin_client (follows redirects → sees 200 not 303).
    // Instead, extract the cookie from admin_client's jar isn't directly
    // possible, so we log in via api_client's cookie store... but api_client
    // doesn't have one. Simplest: build a dedicated no-redirect cookie client.
    let noredirect_client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    // Log in with this client too
    noredirect_client
        .post(&format!("{}/login", &app.address))
        .form(&[("username", "testadmin"), ("password", "testpassword")])
        .send()
        .await
        .expect("Failed to log in");

    let response = noredirect_client
        .post(&format!("{}/upload", &app.address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute upload request");

    // Assert — 303 See Other redirect to /gallery
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(
        response.headers().get("LOCATION").unwrap(),
        "/gallery?status=success"
    );

    // The photo was persisted through the shared save pipeline.
    let photos = app.database.list().await.expect("Failed to list photos");
    assert_eq!(photos.len(), 1);
    assert_eq!(photos[0].band, "The Band");
    assert_eq!(photos[0].venue, "Best Ever");
}

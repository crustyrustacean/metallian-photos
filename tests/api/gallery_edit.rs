// tests/api/gallery_edit.rs

use crate::helpers::{create_photo, spawn_app};
use metallian_photos::database::DatabaseBackend;

#[tokio::test]
async fn edit_returns_form_with_current_metadata() {
    // Arrange
    let app = spawn_app().await;
    let id = create_photo(&app.api_client, &app.address).await;

    // Act — GET the edit form
    let response = app
        .api_client
        .get(&format!("{}/gallery/{}/edit", &app.address, id))
        .send()
        .await
        .expect("Failed to execute edit request");

    // Assert — SSE with a form containing the current metadata
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        response.headers().get("CONTENT-TYPE").unwrap(),
        "text/event-stream"
    );

    let body = response.text().await.expect("Failed to read body");
    assert!(body.contains("datastar-patch-elements"));
    assert!(body.contains("data-bind:band"));
    assert!(body.contains("The Band"));
    assert!(body.contains("Best Ever"));
}

#[tokio::test]
async fn update_saves_changes_and_returns_caption() {
    // Arrange
    let app = spawn_app().await;
    let id = create_photo(&app.api_client, &app.address).await;

    // Act — PUT with updated metadata as Datastar JSON signals.
    // Datastar sends all signals in the JSON body for non-GET requests.
    let signals = serde_json::json!({
        "band": "Updated Band",
        "tour": "New Tour",
        "venue": "Different Venue",
        "flash": false
    });

    let response = app
        .api_client
        .put(&format!("{}/gallery/{}", &app.address, id))
        .header("Content-Type", "application/json")
        .json(&signals)
        .send()
        .await
        .expect("Failed to execute update request");

    // Assert — SSE response with the updated caption
    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().await.expect("Failed to read body");
    assert!(body.contains("datastar-patch-elements"));
    assert!(body.contains("Updated Band"));
    assert!(body.contains("New Tour"));
    assert!(body.contains("Different Venue"));

    // The database was actually updated
    let photos = app.database.list().await.expect("Failed to list photos");
    assert_eq!(photos.len(), 1);
    assert_eq!(photos[0].band, "Updated Band");
    assert_eq!(photos[0].venue, "Different Venue");
}

#[tokio::test]
async fn cancel_returns_original_caption_without_saving() {
    // Arrange
    let app = spawn_app().await;
    let id = create_photo(&app.api_client, &app.address).await;

    // Act — GET the cancel endpoint
    let response = app
        .api_client
        .get(&format!("{}/gallery/{}/cancel", &app.address, id))
        .send()
        .await
        .expect("Failed to execute cancel request");

    // Assert — SSE with original metadata, unchanged
    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().await.expect("Failed to read body");
    assert!(body.contains("datastar-patch-elements"));
    assert!(body.contains("The Band"));
    assert!(body.contains("Best Ever"));

    // Database unchanged
    let photos = app.database.list().await.expect("Failed to list photos");
    assert_eq!(photos[0].band, "The Band");
}

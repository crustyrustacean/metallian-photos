// tests/api/gallery_delete.rs

use crate::helpers::{create_photo, login, spawn_app};
use metallian_photos::database::DatabaseBackend;

#[tokio::test]
async fn gallery_delete_removes_photo_and_returns_sse() {
    // Arrange
    let app = spawn_app().await;
    login(&app).await;
    let id = create_photo(&app.admin_client, &app.address).await;

    // Act — DELETE via the frontend Datastar endpoint
    let response = app
        .admin_client
        .delete(&format!("{}/gallery/{}", &app.address, id))
        .send()
        .await
        .expect("Failed to execute delete request");

    // Assert — the response is SSE with the patch-elements event
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        response.headers().get("CONTENT-TYPE").unwrap(),
        "text/event-stream"
    );

    let body = response.text().await.expect("Failed to read body");
    assert!(
        body.contains("datastar-patch-elements"),
        "response should contain a patch-elements event"
    );
    assert!(
        body.contains("mode remove"),
        "response should use remove mode"
    );
    assert!(
        body.contains(&format!("#photo-{id}")),
        "response should target the photo's list item"
    );

    // The photo was actually deleted from the database
    let photos = app.database.list().await.expect("Failed to list photos");
    assert!(photos.is_empty(), "photo should be deleted from database");
}
// tests/api/create.rs

// dependencies
use crate::helpers::{create_photo, spawn_app};
use sqlx::FromRow;

#[derive(FromRow)]
struct StoredPhotoData {
    id: String,
    band: String,
    tour: String,
    venue: String,
}

#[tokio::test]
async fn create_photo_endpoint_returns_200_ok_and_id() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let id = create_photo(&client, &app.address).await;

    // Assert — the photo is persisted with the submitted metadata
    let id_string = id.to_string();
    let db_pool = app.database.pool();

    let result: StoredPhotoData = sqlx::query_as("SELECT * FROM photos WHERE id = ?")
        .bind(&id_string)
        .fetch_one(db_pool)
        .await
        .unwrap();

    assert_eq!(result.id, id_string);
    assert_eq!(result.band, "The Band");
    assert_eq!(result.tour, "Tour of Champions");
    assert_eq!(result.venue, "Best Ever");
}

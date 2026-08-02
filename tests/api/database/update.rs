// tests/api/update.rs

// dependencies
use crate::helpers::{create_photo, login, spawn_app};
use serde::Serialize;
use sqlx::FromRow;

#[allow(dead_code)]
#[derive(FromRow)]
struct StoredPhotoData {
    id: String,
    band: String,
    tour: String,
    venue: String,
}

#[derive(Serialize)]
struct UpdatePhotoData {
    band: String,
    tour: String,
    venue: String,
}

#[tokio::test]
async fn update_photo_endpoint_updates_photo_and_returns_200_ok() {
    // Arrange
    let app = spawn_app().await;
    login(&app).await;

    let id = create_photo(&app.admin_client, &app.address).await;

    let updated_photo_data = UpdatePhotoData {
        band: "The Real Band".to_string(),
        tour: "The Real Tour of Champions".to_string(),
        venue: "Second Best Ever".to_string(),
    };

    // Act — the update endpoint takes urlencoded form data
    let response = app.admin_client
        .put(&format!("{}/api/photos/{}", &app.address, id))
        .form(&updated_photo_data)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());

    let db_pool = app.database.pool();
    let result: StoredPhotoData = sqlx::query_as("SELECT * FROM photos WHERE id = ?")
        .bind(&id.to_string())
        .fetch_one(db_pool)
        .await
        .unwrap();

    assert_eq!(result.band, updated_photo_data.band);
    assert_eq!(result.tour, updated_photo_data.tour);
    assert_eq!(result.venue, updated_photo_data.venue);
}

#[tokio::test]
async fn update_photo_endpoint_with_malformed_uuid_returns_400() {
    // Arrange
    let app = spawn_app().await;
    login(&app).await;

    let updated_photo_data = UpdatePhotoData {
        band: "The Real Band".to_string(),
        tour: "The Real Tour of Champions".to_string(),
        venue: "Second Best Ever".to_string(),
    };

    // Act
    let response = app.admin_client
        .put(&format!(
            "{}/api/photos/{}",
            &app.address, "not-a-valid-uuid"
        ))
        .form(&updated_photo_data)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

use actix_web::{App, test, web};
use metallian_photos::configuration::DatabaseSettings;
use metallian_photos::database::{DatabaseBackend, SqliteRepository};
use metallian_photos::domain::{Exif, Photo};
use metallian_photos::routes::api;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn galleries_handler_extracts_unsized_database_data() {
    let repository = SqliteRepository::new(&DatabaseSettings {
        path: ":memory:".into(),
        max_connections: Some(1),
    })
    .await
    .unwrap();
    repository
        .create(Photo {
            id: Uuid::new_v4(),
            band: "Iron Maiden".into(),
            tour: "Tour".into(),
            venue: "Venue".into(),
            exif_data: Exif::default(),
        })
        .await
        .unwrap();

    // Exercise the same Box -> Arc -> Data conversion used at startup.
    let backend: Box<dyn DatabaseBackend> = Box::new(repository);
    let database: web::Data<dyn DatabaseBackend> = web::Data::from(Arc::from(backend));
    let app = test::init_service(
        App::new()
            .app_data(database)
            .route("/api/galleries", web::get().to(api::list_galleries)),
    )
    .await;
    let response = test::call_service(
        &app,
        test::TestRequest::get().uri("/api/galleries").to_request(),
    )
    .await;
    assert_eq!(response.status(), 200);
    let galleries: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(galleries[0]["band"], "Iron Maiden");
    assert_eq!(galleries[0]["photo_count"], 1);
}

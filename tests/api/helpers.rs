// tests/api/helpers.rs

// dependencies
use metallian_photos::configuration::get_configuration;
use metallian_photos::database::DatabaseBackend;
use metallian_photos::database::SqliteRepository;
use metallian_photos::startup::Application;
use metallian_photos::storage::InMemoryStorageBackend;
use metallian_photos::storage::StorageBackend;
use metallian_photos::telemetry::{get_subscriber, init_subscriber};
use metallian_photos::template::TemplateRenderer;
use reqwest::multipart;
use std::sync::LazyLock;
use uuid::Uuid;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

#[allow(dead_code)]
pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub api_client: reqwest::Client,
    pub database: SqliteRepository,
}

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);
    dotenvy::dotenv().expect("Unable to read environment variables.");
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");

        c.application.port = 0;
        c.database.path = ":memory:".to_string();
        c.database.max_connections = Some(1);

        c
    };

    // build the templates
    let template_renderer: Box<dyn TemplateRenderer> = Box::new(
        metallian_photos::template::tera::TeraRenderer::new()
            .expect("Failed to build the template renderer."),
    );

    // build the database backend
    let database = SqliteRepository::new(&configuration.database)
        .await
        .expect("Unable to build the database backend.");
    let database_for_test = database.clone();

    let database_backend: Box<dyn DatabaseBackend> = Box::new(database);

    // build the storage backend
    let storage_backend: Box<dyn StorageBackend> = Box::new(InMemoryStorageBackend::new());

    // Launch the application as a background task
    let application = Application::build(
        configuration.clone(),
        template_renderer,
        database_backend,
        storage_backend,
    )
    .await
    .expect("Failed to build application.");
    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let test_app = TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        api_client: client,
        database: database_for_test,
    };

    test_app
}

/// Create a photo via the multipart `/api/photos` endpoint and return its id.
///
/// Uses the real HEIC fixture so the full pipeline runs: HEIC decode → EXIF
/// extraction → JPEG conversion → storage. Used as test setup by the
/// read/update/delete tests, and exercised directly by the create test.
pub async fn create_photo(client: &reqwest::Client, address: &str) -> Uuid {
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

    post_photo_form(client, address, form).await
}

/// Shared POST logic for both helpers — sends the multipart form and
/// returns the created photo's id.
async fn post_photo_form(client: &reqwest::Client, address: &str, form: multipart::Form) -> Uuid {
    let response = client
        .post(&format!("{}/api/photos", address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute create request.");

    assert!(
        response.status().is_success(),
        "create photo failed with status: {}",
        response.status()
    );

    response
        .json()
        .await
        .expect("Unable to obtain response body.")
}
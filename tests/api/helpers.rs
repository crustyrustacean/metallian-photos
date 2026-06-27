// tests/api/helpers.rs

// dependencies
use r2_photo_api::configuration::get_configuration;
use r2_photo_api::database::DatabaseBackend;
use r2_photo_api::database::SqliteRepository;
use r2_photo_api::startup::Application;
use r2_photo_api::storage::OpendalStorageBackend;
use r2_photo_api::storage::StorageBackend;
use r2_photo_api::telemetry::{get_subscriber, init_subscriber};
use std::sync::LazyLock;

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

    // build the database backend
    let database = SqliteRepository::new(&configuration.database)
        .await
        .expect("Unable to build the database backend.");
    let database_for_test = database.clone();

    let database_backend: Box<dyn DatabaseBackend> = Box::new(database);

    // build the storage backend
    let storage_backend: Box<dyn StorageBackend> = Box::new(
        OpendalStorageBackend::new(&configuration.storage)
            .expect("Unable to build storage backend."),
    );

    // Launch the application as a background task
    let application = Application::build(configuration.clone(), database_backend, storage_backend)
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

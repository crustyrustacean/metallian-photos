// src/main.rs

// dependencies
use r2_photo_api::configuration::get_configuration;
use r2_photo_api::database::DatabaseBackend;
use r2_photo_api::database::SqliteRepository;
use r2_photo_api::startup::Application;
use r2_photo_api::storage::OpendalStorageBackend;
use r2_photo_api::storage::StorageBackend;
use r2_photo_api::telemetry::{get_subscriber, init_subscriber};
use tera::Tera;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    let subscriber = get_subscriber("actix-web-starter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // get environment variables
    dotenvy::dotenv()?;

    // build the configuration
    let configuration = get_configuration().expect("Failed to read configuration.");

    // build the templates
    let mut tera_templates = Tera::default();
    tera_templates.load_from_glob("templates/**/*.html")
        .expect("Unable to load the Tera templates.");

    // build the database backend
    let database_backend: Box<dyn DatabaseBackend> =
        Box::new(SqliteRepository::new(&configuration.database).await?);

    // build the storage backend
    let storage_backend: Box<dyn StorageBackend> =
        Box::new(OpendalStorageBackend::new(&configuration.storage)?);

    // build the application by passing the configuration, database, and storage backends
    let application =
        Application::build(configuration.clone(), tera_templates,database_backend, storage_backend).await?;

    // run the application
    application.run_until_stopped().await?;

    Ok(())
}

// src/main.rs

// dependencies
use metallian_photos::configuration::get_configuration;
use metallian_photos::database::DatabaseBackend;
use metallian_photos::database::SqliteRepository;
use metallian_photos::startup::Application;
use metallian_photos::storage::{InMemoryStorageBackend, OpendalStorageBackend};
use metallian_photos::storage::StorageBackend;
use metallian_photos::telemetry::{get_subscriber, init_subscriber};
use metallian_photos::template::{TemplateRenderer, tera::TeraRenderer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    let subscriber = get_subscriber("metallian-photos".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // get environment variables
    dotenvy::dotenv()?;

    // build the configuration
    let configuration = get_configuration().expect("Failed to read configuration.");

    // build the template engine
    let template_renderer: Box<dyn TemplateRenderer> = Box::new(TeraRenderer::new()?);

    // build the database backend
    let database_backend: Box<dyn DatabaseBackend> =
        Box::new(SqliteRepository::new(&configuration.database).await?);

    // build the storage backend — selects the implementation based on the
    // `storage.backend` config value: "memory" (default) for local dev,
    // "r2" for production against Cloudflare R2.
    let storage_backend: Box<dyn StorageBackend> = match configuration.storage.backend.as_str() {
        "memory" => Box::new(InMemoryStorageBackend::new()),
        "r2" => Box::new(OpendalStorageBackend::new(&configuration.storage)?),
        other => panic!("Unknown storage backend: '{other}'. Use 'memory' or 'r2'."),
    };

    // build the application by passing the configuration, database, and storage backends
    let application = Application::build(
        configuration.clone(),
        template_renderer,
        database_backend,
        storage_backend,
    )
    .await?;

    // run the application
    application.run_until_stopped().await?;

    Ok(())
}
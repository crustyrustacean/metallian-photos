// src/startup.rs

// dependencies
use crate::configuration::Settings;
use crate::database::{DatabaseBackend, SqliteRepository};
use crate::routes::health_check;
use crate::storage::{OpendalStorageBackend, StorageBackend};
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web, web::Data};

use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let database_repository: Box<dyn DatabaseBackend> =
            Box::new(SqliteRepository::new(configuration.database).await?);
        let storage_backend: Box<dyn StorageBackend> =
            Box::new(OpendalStorageBackend::new(configuration.storage)?);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        let server = run(
            listener,
            database_repository,
            storage_backend,
            configuration.application.base_url,
        )
        .await?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct ApplicationBaseUrl(pub String);

async fn run(
    listener: TcpListener,
    database: Box<dyn DatabaseBackend>,
    storage: Box<dyn StorageBackend>,
    base_url: String,
) -> Result<Server, anyhow::Error> {
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let database_repository = Data::new(database);
    let storage_backend = Data::new(storage);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .app_data(base_url.clone())
            .app_data(database_repository.clone())
            .app_data(storage_backend.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

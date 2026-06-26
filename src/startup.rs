// src/startup.rs

// dependencies
use crate::configuration::Settings;
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
        let db_pool = create_database_pool(configuration.database).await?;
        let storage_backend: Box<dyn StorageBackend> =
            Box::new(OpendalStorageBackend::new(configuration.storage)?);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        let server = run(
            listener,
            db_pool,
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
    pool: SqlitePool,
    storage: Box<dyn StorageBackend>,
    base_url: String,
) -> Result<Server, anyhow::Error> {
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let db_pool = Data::new(pool);
    let storage_backend = Data::new(storage);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .app_data(base_url.clone())
            .app_data(db_pool.clone())
            .app_data(storage_backend.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

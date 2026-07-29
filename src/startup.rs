// src/startup.rs

// dependencies
use crate::configuration::Settings;
use crate::database::DatabaseBackend;
use crate::routes::{create, delete, health_check, read, update, get_index_page};
use crate::storage::StorageBackend;
use crate::template::TemplateRenderer;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web, web::Data};

use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(
        configuration: Settings,
        template_renderer: Box<dyn TemplateRenderer>,
        database_backend: Box<dyn DatabaseBackend>,
        storage_backend: Box<dyn StorageBackend>,
    ) -> Result<Self, anyhow::Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        let server = run(
            listener,
            template_renderer,
            database_backend,
            storage_backend,
            configuration.application.base_url.clone(),
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
    template_renderer: Box<dyn TemplateRenderer>,
    database: Box<dyn DatabaseBackend>,
    storage: Box<dyn StorageBackend>,
    base_url: String,
) -> Result<Server, anyhow::Error> {
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let template_renderer = Data::new(template_renderer);
    let database_repository = Data::new(database);
    let storage_backend = Data::new(storage);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(get_index_page))
            .route("/health_check", web::get().to(health_check))
            .route("/photos", web::post().to(create))
            .route("/photos/{id}", web::get().to(read))
            .route("/photos/{id}", web::put().to(update))
            .route("/photos/{id}", web::delete().to(delete))
            .app_data(base_url.clone())
            .app_data(template_renderer.clone())
            .app_data(database_repository.clone())
            .app_data(storage_backend.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

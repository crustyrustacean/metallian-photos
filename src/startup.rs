// src/startup.rs

// dependencies
use crate::configuration::Settings;
use crate::database::DatabaseBackend;
use crate::routes::{api, frontend};
use crate::storage::StorageBackend;
use crate::template::TemplateRenderer;
use actix_files::Files;
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
            // frontend routes — browser-facing HTML pages
            .route("/", web::get().to(frontend::get_index_page))
            .service(Files::new("/static", "static").prefer_utf8(true))
            // api routes — machine-facing JSON endpoints
            .service(
                web::scope("/api")
                    .route("/health_check", web::get().to(api::health_check))
                    .route("/photos", web::post().to(api::create_photo))
                    .route("/photos/{id}", web::get().to(api::read_photo))
                    .route("/photos/{id}", web::put().to(api::update_photo))
                    .route("/photos/{id}", web::delete().to(api::delete_photo)),
            )
            .app_data(base_url.clone())
            .app_data(template_renderer.clone())
            .app_data(database_repository.clone())
            .app_data(storage_backend.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
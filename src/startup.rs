// src/startup.rs

// dependencies
use crate::auth;
use crate::configuration::Settings;
use crate::database::DatabaseBackend;
use crate::routes::{api, frontend};
use crate::services::PhotoOperations;
use crate::storage::StorageBackend;
use crate::template::TemplateRenderer;
use actix_cors::Cors;
use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web, web::Data};

use std::net::TcpListener;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
    database: Data<dyn DatabaseBackend>,
    storage: Data<dyn StorageBackend>,
    operations: Data<PhotoOperations>,
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

        // Derive a 64-byte session signing key from the admin password.
        // The cookie crate requires at least 32 bytes; we repeat the password
        // bytes to fill 64. This is pragmatic for a single-user app — the key
        // changes if the password changes (invalidating old sessions), and we
        // don't need a separate secret.
        let key_bytes: Vec<u8> = configuration
            .admin
            .password
            .as_bytes()
            .iter()
            .cycle()
            .take(64)
            .copied()
            .collect();
        let session_key = Key::from(&key_bytes);

        let database: Data<dyn DatabaseBackend> = Data::from(Arc::from(database_backend));
        let storage: Data<dyn StorageBackend> = Data::from(Arc::from(storage_backend));
        let operations = Data::new(PhotoOperations::default());
        // Finish abandoned operations before serving requests. A temporary
        // backend outage should not prevent the app from starting.
        if let Err(error) = operations
            .recover_incomplete(database.get_ref(), storage.get_ref())
            .await
        {
            tracing::warn!(error = ?error, "startup photo recovery failed; will retry");
        }
        let server = run(
            listener,
            template_renderer,
            database.clone(),
            storage.clone(),
            operations.clone(),
            configuration.admin.clone(),
            session_key,
        )
        .await?;
        Ok(Self {
            port,
            server,
            database,
            storage,
            operations,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        let Self {
            server,
            database,
            storage,
            operations,
            ..
        } = self;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
        interval.tick().await; // Startup already performed the first pass.
        tokio::pin!(server);
        loop {
            tokio::select! {
                result = &mut server => return result,
                _ = interval.tick() => {
                    if let Err(error) = operations.recover_incomplete(database.get_ref(), storage.get_ref()).await {
                        tracing::warn!(error = ?error, "photo recovery failed; will retry");
                    }
                }
            }
        }
    }
}

async fn run(
    listener: TcpListener,
    template_renderer: Box<dyn TemplateRenderer>,
    database: Data<dyn DatabaseBackend>,
    storage: Data<dyn StorageBackend>,
    operations: Data<PhotoOperations>,
    admin: crate::configuration::AdminSettings,
    session_key: Key,
) -> Result<Server, anyhow::Error> {
    let template_renderer = Data::new(template_renderer);
    let admin = Data::new(admin);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                session_key.clone(),
            ))
            .wrap(Cors::permissive().allowed_methods(["GET", "POST", "PUT", "DELETE"]))
            // --- public routes (no auth required) ---
            .route("/", web::get().to(frontend::get_index_page))
            .route("/galleries", web::get().to(frontend::get_galleries_index))
            .route("/g/{slug}", web::get().to(frontend::get_public_gallery))
            .service(Files::new("/static", "static").prefer_utf8(true))
            // auth routes
            .route("/login", web::get().to(auth::get_login_page))
            .route("/login", web::post().to(auth::post_login))
            .route("/logout", web::get().to(auth::get_logout))
            // --- admin routes (auth required) ---
            .route("/gallery", web::get().to(frontend::get_gallery_page))
            .route("/upload", web::get().to(frontend::get_upload_page))
            .route("/upload", web::post().to(frontend::post_upload))
            .route("/gallery/{id}", web::delete().to(frontend::delete_photo))
            .route("/gallery/{id}/edit", web::get().to(frontend::edit_photo))
            .route("/gallery/{id}", web::put().to(frontend::update_photo))
            .route("/gallery/{id}/cancel", web::get().to(frontend::cancel_edit))
            // --- api routes ---
            // Public reads (blog fetches these):
            .service(
                web::scope("/api")
                    .route("/health_check", web::get().to(api::health_check))
                    .route("/galleries", web::get().to(api::list_galleries))
                    .route("/photos/{id}", web::get().to(api::read_photo))
                    .route("/photos/{id}/image", web::get().to(api::get_photo_image))
                    // Writes (admin only — identity checked in handler):
                    .route("/photos", web::post().to(api::create_photo))
                    .route("/photos/{id}", web::put().to(api::update_photo))
                    .route("/photos/{id}", web::delete().to(api::delete_photo)),
            )
            // --- catch-all 404 ---
            .default_service(web::to(frontend::not_found))
            .app_data(template_renderer.clone())
            .app_data(database.clone())
            .app_data(storage.clone())
            .app_data(operations.clone())
            .app_data(admin.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

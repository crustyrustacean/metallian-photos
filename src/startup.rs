// src/startup.rs

// dependencies
use crate::configuration::{DatabaseSettings, Settings};
use crate::routes::health_check;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web, web::Data};
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::net::TcpListener;
use std::str::FromStr;
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
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        let server = run(listener, db_pool, configuration.application.base_url).await?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

async fn create_database_pool(
    db_configuration: DatabaseSettings,
) -> Result<SqlitePool, anyhow::Error> {
    let db_path = format!("sqlite:{}", db_configuration.path);
    let options = SqliteConnectOptions::from_str(&db_path)?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;

    Ok(pool)
}

pub struct ApplicationBaseUrl(pub String);

async fn run(
    listener: TcpListener,
    pool: SqlitePool,
    base_url: String,
) -> Result<Server, anyhow::Error> {
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let db_pool = Data::new(pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .app_data(base_url.clone())
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

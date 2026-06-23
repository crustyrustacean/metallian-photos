// src/main.rs

// dependencies
use r2_photo_api::configuration::get_configuration;
use r2_photo_api::startup::Application;
use r2_photo_api::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("actix-web-starter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    dotenvy::dotenv()?;
    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration.clone()).await?;
    application.run_until_stopped().await?;

    Ok(())
}

// src/routes/health.rs

// dependencies
use actix_web::HttpResponse;

/// health check endpoint
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

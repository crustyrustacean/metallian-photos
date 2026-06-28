// src/routes/health_check.rs

// dependencies
use crate::response::ApiResponse;
use actix_web::Responder;

/// health check endpoint
pub async fn health_check() -> impl Responder {
    ApiResponse::success(())
}

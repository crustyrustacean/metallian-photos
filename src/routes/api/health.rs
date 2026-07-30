// src/routes/api/health.rs

use actix_web::HttpResponse;

/// GET /api/health_check
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

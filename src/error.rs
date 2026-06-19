// src/lib/errors.rs

// dependencies
use crate::response::ApiResponse;
use actix_web::http::{StatusCode, header::ContentType};
use actix_web::{HttpResponse, ResponseError, body::BoxBody};

// enum type to represent the possible error variants for the API
#[derive(thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal server error: {0}")]
    Internal(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
}

// implement the IntoResponse trait for the ApiError type
impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let user_message = self.to_string();

        let body = ApiResponse::<()>::error(&user_message);

        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(body)
    }
}

// implement the Debug trait for ApiError
impl std::fmt::Debug for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

// utility function to presereve the error chain in the error message
pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

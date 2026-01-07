use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("internal error")]
    Internal,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorBody {
            error: self.to_string(),
        })
    }
}

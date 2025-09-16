use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    InternalServerError(String),
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    details: Option<String>,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status, error, details) = match self {
            ApiError::InternalServerError(details) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error", Some(details.clone()))
            }
            ApiError::BadRequest(error) => (StatusCode::BAD_REQUEST, error.as_str(), None),
            ApiError::Unauthorized(error) => (StatusCode::UNAUTHORIZED, error.as_str(), None),
            ApiError::NotFound(error) => (StatusCode::NOT_FOUND, error.as_str(), None),
        };

        let response_body = ErrorResponse {
            error: error.to_string(),
            details,
        };

        HttpResponse::build(status).json(response_body)
    }
}

// Implementasi Display trait agar bisa digunakan dengan .to_string()
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::InternalServerError(s) => write!(f, "Internal Server Error: {}", s),
            ApiError::BadRequest(s) => write!(f, "Bad Request: {}", s),
            ApiError::Unauthorized(s) => write!(f, "Unauthorized: {}", s),
            ApiError::NotFound(s) => write!(f, "Not Found: {}", s),
        }
    }
}
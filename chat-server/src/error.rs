use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]

pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("password hash error: {0}")]
    HashError(#[from] argon2::password_hash::Error),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),
    #[error("http header parse error: {0}")]
    HttpHeaderError(#[from] axum::http::header::InvalidHeaderValue),
    #[error("create chat error: {0}")]
    CreateChatError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("multipart error: {0}")]
    MultipartError(#[from] axum::extract::multipart::MultipartError),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let status = match &self {
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::HashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::JwtError(_) => StatusCode::FORBIDDEN,
            Self::HttpHeaderError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            Self::CreateChatError(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MultipartError(_) => StatusCode::BAD_REQUEST,
        };

        (status, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}

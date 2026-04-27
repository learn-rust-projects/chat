use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use jwt_simple::reexports::serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]

pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("password hash error: {0}")]
    HashError(#[from] argon2::password_hash::Error),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let status = match &self {
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::HashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::JwtError(_) => StatusCode::FORBIDDEN,
        };

        (status, Json(json!({"error": self.to_string()}))).into_response()
    }
}

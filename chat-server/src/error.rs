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

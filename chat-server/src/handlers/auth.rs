use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    AppError, AppState,
    models::{CreateUser, SigninUser, User},
};

pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::create(&input, &state.db).await?;
    let token = state.ek.sign(user)?;
    Ok((StatusCode::CREATED, token))
}
pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify_password(&input, &state.db).await?;
    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::CREATED, token).into_response())
        }
        None => Ok((StatusCode::FORBIDDEN, "Invalid email or password").into_response()),
    }
}

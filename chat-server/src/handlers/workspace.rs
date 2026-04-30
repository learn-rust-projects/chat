use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{AppError, AppState, User};

pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_all_chat_users(user.ws_id as _).await?;
    Ok((StatusCode::OK, Json(users)).into_response())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;
    use crate::models::CreateUser;
    #[tokio::test]
    async fn list_chat_users_should_work() -> Result<()> {
        let (_tdb, state) = crate::AppState::new_for_test().await?;
        let ws = state.create_workspace("test", 0).await.unwrap();
        let input = CreateUser::new(&ws.name, "Tyr Chen", "tchen@acme.org", "test");
        let user1 = state.create_user(&input).await.unwrap();
        let input = CreateUser::new(&ws.name, "Alice Wang", "alice@acme.org", "test");
        let _user2 = state.create_user(&input).await.unwrap();
        let ret = list_chat_users_handler(Extension(user1), State(state.clone()))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::OK);

        Ok(())
    }
}

use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{AppError, AppState, User, models::Workspace};

pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = Workspace::fetch_all_chat_users(user.ws_id as _, &state.db).await?;
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
        let pool = &state.db;
        let ws = Workspace::create("test", 0, pool).await.unwrap();
        let input = CreateUser::new(&ws.name, "Tyr Chen", "tchen@acme.org", "test");
        let user1 = User::create(&input, pool).await.unwrap();
        let input = CreateUser::new(&ws.name, "Alice Wang", "alice@acme.org", "test");
        let _user2 = User::create(&input, pool).await.unwrap();
        let ret = list_chat_users_handler(Extension(user1), State(state.clone()))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::OK);

        Ok(())
    }
}

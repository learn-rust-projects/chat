use chat_core::Workspace;

use crate::{AppError, AppState};
impl AppState {
    pub async fn create_workspace(&self, name: &str, owner_id: i64) -> Result<Workspace, AppError> {
        let ws = sqlx::query_as(
            r#"
            INSERT INTO workspaces (name, owner_id)
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(name)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(ws)
    }
    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ws)
    }
    #[allow(dead_code)]
    pub async fn find_workspace_by_id(&self, id: u64) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id, name, owner_id, created_at
        FROM workspaces
        WHERE id = $1
        "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(ws)
    }

    pub async fn update_owner_by_id(
        &self,
        new_owner_id: i64,
        workspace_id: i64,
    ) -> Result<Workspace, AppError> {
        // 保证owner属于该组织下
        let ws = sqlx::query_as(
            r#"
        UPDATE workspaces
        SET owner_id = $1
        WHERE id = $2 and (SELECT ws_id FROM users WHERE id = $1) = $2
        RETURNING id, name, owner_id, created_at
        "#,
        )
        .bind(new_owner_id)
        .bind(workspace_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(ws)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;
    use crate::models::CreateUser;
    #[tokio::test]
    async fn workspace_should_create_and_set_owner() -> Result<()> {
        let (_tdb, app_state) = AppState::new_for_test().await?;
        // 1.创建workspace
        let ws = app_state.create_workspace("test", 0).await?;
        assert_eq!(ws.name, "test");
        // 2.通过create_user创建用户,传入上面的组织
        let input = CreateUser::new("Jack Chen", "jack@acme.org", "test", &ws.name);
        let user = app_state.create_user(&input).await.unwrap();
        assert_eq!(user.ws_id, ws.id);
        // 3.更新组织owner_id为上面创建的用户
        let ws = app_state.update_owner_by_id(user.id, ws.id).await?;
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_find_by_name() -> Result<()> {
        let (_tdb, app_state) = AppState::new_for_test().await?;
        let ws = app_state.find_workspace_by_name("acme").await?;
        assert_eq!(ws.unwrap().name, "acme");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> Result<()> {
        let (_tdb, app_state) = AppState::new_for_test().await?;
        // 查询组织下的所有用户
        let users = app_state.fetch_all_chat_users(1).await?;
        assert_eq!(users.len(), 5);
        Ok(())
    }
}

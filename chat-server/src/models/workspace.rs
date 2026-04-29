use sqlx::PgPool;

use crate::{
    AppError,
    models::{ChatUser, Workspace},
};
impl Workspace {
    pub async fn create(name: &str, owner_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
            INSERT INTO workspaces (name, owner_id)
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(name)
        .bind(owner_id)
        .fetch_one(pool)
        .await?;
        Ok(ws)
    }
    pub async fn update_owner(
        &self,
        new_owner_id: i64,
        db: &sqlx::PgPool,
    ) -> Result<Self, AppError> {
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
        .bind(self.id)
        .fetch_one(db)
        .await?;
        Ok(ws)
    }
    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;
        Ok(ws)
    }
    #[allow(dead_code)]
    pub async fn find_by_id(id: u64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id, name, owner_id, created_at
        FROM workspaces
        WHERE id = $1
        "#,
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }
    #[allow(dead_code)]
    pub async fn fetch_all_chat_users(id: i64, pool: &PgPool) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
        SELECT id, fullname, email
        FROM users
        WHERE ws_id = $1 order by id
        "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;
    use crate::models::{CreateUser, User};
    #[tokio::test]
    async fn workspace_should_create_and_set_owner() -> Result<()> {
        let (_tdb, state) = crate::AppState::new_for_test().await?;
        // 1.创建workspace
        let ws = Workspace::create("test", 0, &state.db).await?;
        assert_eq!(ws.name, "test");
        // 2.通过create_user创建用户,传入上面的组织
        let input = CreateUser::new(&ws.name, "Tyr Chen", "tchen@acme.org", "test");
        let user = User::create(&input, &state.db).await.unwrap();
        assert_eq!(user.ws_id, ws.id);
        // 3.更新组织owner_id为上面创建的用户
        let ws = ws.update_owner(user.id, &state.db).await?;
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_find_by_name() -> Result<()> {
        let (_tdb, state) = crate::AppState::new_for_test().await?;
        let pool = &state.db;
        let _ws = Workspace::create("test", 0, pool).await?;
        let ws = Workspace::find_by_name("test", pool).await?;

        assert_eq!(ws.unwrap().name, "test");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> Result<()> {
        let (_tdb, state) = crate::AppState::new_for_test().await?;
        let pool = &state.db;
        let ws = Workspace::create("test", 0, pool).await?;
        let input = CreateUser::new(&ws.name, "Tyr Chen", "tchen@acme.org", "test");
        let user1 = User::create(&input, pool).await?;
        let input = CreateUser::new(&ws.name, "Alice Wang", "alice@acme.org", "test");
        let user2 = User::create(&input, pool).await?;
        // 查询组织下的所有用户
        let users = Workspace::fetch_all_chat_users(ws.id, pool).await?;
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].id, user1.id);
        assert_eq!(users[1].id, user2.id);

        Ok(())
    }
}

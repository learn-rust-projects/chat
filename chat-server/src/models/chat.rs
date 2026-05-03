use chat_core::{Chat, ChatType};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{AppError, AppState};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}
impl From<UpdateChat> for CreateChat {
    fn from(value: UpdateChat) -> Self {
        Self {
            name: value.name,
            members: value.members,
            public: value.public,
        }
    }
}

#[allow(dead_code)]
impl AppState {
    pub async fn check_chat_input(&self, input: &CreateChat) -> Result<(), AppError> {
        // check size of members must be at least 2
        let len = input.members.len();
        if len < 2 {
            return Err(AppError::CreateChatError(
                "Chat must have at least 2 members".to_string(),
            ));
        }
        // chat has a name if it's a group chat with more than 8 members
        if len > 8 && input.name.is_none() {
            return Err(AppError::CreateChatError(
                "Group chat with more than 8 members must have a name".to_string(),
            ));
        }
        // verify if all members exist
        let users = self.fetch_chat_users_by_ids(&input.members).await?;
        if users.len() != len {
            return Err(AppError::CreateChatError(
                "Some members do not exist".to_string(),
            ));
        }
        Ok(())
    }
    pub async fn create_chat(&self, input: CreateChat, ws_id: i64) -> Result<Chat, AppError> {
        self.check_chat_input(&input).await?;
        let chat_type = match (&input.name, input.members.len()) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };

        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, type, members, created_at
            "#,
        )
        .bind(ws_id)
        .bind(input.name)
        .bind(chat_type)
        .bind(&input.members)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }
    /// fetch all chats in a workspace
    pub async fn fetch_all_chats(&self, ws_id: i64) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(chats)
    }
    pub async fn get_chat_by_id(&self, id: i64) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(chat)
    }
    pub async fn update_chat_by_id(
        &self,
        id: i64,
        input: UpdateChat,
    ) -> Result<Option<Chat>, AppError> {
        self.check_chat_input(&input.clone().into()).await?;
        let chat_type = match (&input.name, input.members.len()) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };
        let chat = sqlx::query_as(
            r#"
            UPDATE chats
            SET name = $1, type = $2, members = $3
            WHERE id = $4
            RETURNING id, ws_id, name, type, members, created_at
            "#,
        )
        .bind(input.name)
        .bind(chat_type)
        .bind(&input.members)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(chat)
    }
    pub async fn delete_chat_by_id(&self, id: i64) -> Result<bool, AppError> {
        let chat = sqlx::query(
            r#"
            DELETE FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(chat.is_some())
    }
    pub async fn is_chat_member(&self, chat_id: u64, user_id: u64) -> Result<bool, AppError> {
        let is_member: Option<sqlx::postgres::PgRow> = sqlx::query(
            r#"
            SELECT 1
            FROM chats
            WHERE id = $1 AND $2 = ANY(members)
            "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(is_member.is_some())
    }
}
#[cfg(test)]
impl CreateChat {
    pub fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}

#[cfg(test)]
impl UpdateChat {
    pub fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn create_single_chat_should_work() -> Result<(), AppError> {
        let (_tdb, app_state) = AppState::new_for_test().await?;
        let input = CreateChat::new("", &[1, 2], false);
        let chat = app_state
            .create_chat(input, 1)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::Single);
        Ok(())
    }

    #[tokio::test]
    async fn create_public_named_chat_should_work() -> Result<(), AppError> {
        let (_tdb, app_state) = AppState::new_for_test().await?;
        let input = CreateChat::new("general", &[1, 2, 3], true);
        let chat = app_state
            .create_chat(input, 1)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        Ok(())
    }

    #[tokio::test]
    async fn chat_get_by_id_should_work() -> Result<(), AppError> {
        let (_tdb, app_state) = AppState::new_for_test().await?;
        let chat = app_state
            .get_chat_by_id(1)
            .await
            .expect("get chat by id failed")
            .unwrap();

        assert_eq!(chat.id, 1);
        assert_eq!(chat.name.unwrap(), "general");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 5);
        Ok(())
    }

    #[tokio::test]
    async fn chat_fetch_all_should_work() -> Result<(), AppError> {
        let (_tdb, app_state) = AppState::new_for_test().await?;
        let chats = app_state
            .fetch_all_chats(1)
            .await
            .expect("fetch all chats failed");

        assert_eq!(chats.len(), 4);
        Ok(())
    }
    #[tokio::test]
    async fn chat_update_by_id_should_work() -> Result<(), AppError> {
        let (_tdb, app_state) = AppState::new_for_test().await?;
        let input = UpdateChat::new("random", &[1, 2], false);
        let chat = app_state
            .update_chat_by_id(1, input)
            .await
            .expect("update chat by id failed")
            .unwrap();
        assert_eq!(chat.id, 1);
        assert_eq!(chat.name.unwrap(), "random");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);
        Ok(())
    }
    #[tokio::test]
    async fn chat_delete_by_id_should_work() -> Result<(), AppError> {
        let (_tdb, app_state) = AppState::new_for_test().await?;
        app_state
            .delete_chat_by_id(2)
            .await
            .expect("delete chat by id failed");

        if let Some(chat) = app_state.get_chat_by_id(2).await? {
            panic!("chat should be deleted but still exists: {:?}", chat);
        }
        Ok(())
    }
    #[tokio::test]
    async fn chat_is_member_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let is_member = state.is_chat_member(1, 1).await.expect("is member failed");
        assert!(is_member);

        // user 6 doesn't exist
        let is_member = state.is_chat_member(1, 6).await.expect("is member failed");
        assert!(!is_member);

        // chat 10 doesn't exist
        let is_member = state.is_chat_member(10, 1).await.expect("is member failed");
        assert!(!is_member);

        // user 4 is not a member of chat 2
        let is_member = state.is_chat_member(2, 4).await.expect("is member failed");
        assert!(!is_member);

        // user 2 is a member of chat 2
        let is_member = state.is_chat_member(2, 2).await.expect("is member failed");
        assert!(is_member);
        Ok(())
    }
}

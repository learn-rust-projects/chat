use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::{Chat, ChatType};
use crate::{AppError, models::ChatUser};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
    pub id: i64,
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
impl Chat {
    pub async fn check(input: &CreateChat, _ws_id: i64, pool: &PgPool) -> Result<(), AppError> {
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
        let users = ChatUser::fetch_by_ids(&input.members, pool).await?;
        if users.len() != len {
            return Err(AppError::CreateChatError(
                "Some members do not exist".to_string(),
            ));
        }
        Ok(())
    }
    pub async fn create(input: CreateChat, ws_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        Self::check(&input, ws_id, pool).await?;
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
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }
    /// fetch all chats in a workspace
    pub async fn fetch_all(ws_id: i64, pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id)
        .fetch_all(pool)
        .await?;

        Ok(chats)
    }
    pub async fn get_by_id(id: i64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(chat)
    }
    pub async fn update_by_id(input: UpdateChat, pool: &PgPool) -> Result<Self, AppError> {
        Self::check(&input.clone().into(), 0, pool).await?;
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
        .bind(input.id)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }
    pub async fn delete_by_id(id: i64, pool: &PgPool) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
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
    pub fn new(id: i64, name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        Self {
            id,
            name,
            members: members.to_vec(),
            public,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::get_test_pool;

    #[tokio::test]
    async fn create_single_chat_should_work() -> Result<(), AppError> {
        let (_tdb, pool) = get_test_pool().await?;
        let input = CreateChat::new("", &[1, 2], false);
        let chat = Chat::create(input, 1, &pool)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::Single);
        Ok(())
    }

    #[tokio::test]
    async fn create_public_named_chat_should_work() -> Result<(), AppError> {
        let (_tdb, pool) = get_test_pool().await?;
        let input = CreateChat::new("general", &[1, 2, 3], true);
        let chat = Chat::create(input, 1, &pool)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        Ok(())
    }

    #[tokio::test]
    async fn chat_get_by_id_should_work() -> Result<(), AppError> {
        let (_tdb, pool) = get_test_pool().await?;
        let chat = Chat::get_by_id(1, &pool)
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
        let (_tdb, pool) = get_test_pool().await?;
        let chats = Chat::fetch_all(1, &pool)
            .await
            .expect("fetch all chats failed");

        assert_eq!(chats.len(), 4);
        Ok(())
    }
    #[tokio::test]
    async fn chat_update_by_id_should_work() -> Result<(), AppError> {
        let (_tdb, pool) = get_test_pool().await?;
        let input = UpdateChat::new(1, "random", &[1, 2], false);
        let chat = Chat::update_by_id(input, &pool)
            .await
            .expect("update chat by id failed");
        assert_eq!(chat.id, 1);
        assert_eq!(chat.name.unwrap(), "random");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);
        Ok(())
    }
    #[tokio::test]
    async fn chat_delete_by_id_should_work() -> Result<(), AppError> {
        let (_tdb, pool) = get_test_pool().await?;
        Chat::delete_by_id(1, &pool)
            .await
            .expect("delete chat by id failed");

        if let Some(chat) = Chat::get_by_id(1, &pool).await? {
            panic!("chat should be deleted but still exists: {:?}", chat);
        }
        Ok(())
    }
}

use std::{collections::HashSet, sync::Arc};

use chat_core::{Chat, Message};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tracing::{info, warn};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AppEvent {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}
#[derive(Debug)]
struct Notification {
    // users being impacted, so we should send the notification to them
    user_ids: HashSet<i64>,
    event: Arc<AppEvent>,
}
// pg_notify('chat_updated', json_build_object('op', TG_OP, 'old', OLD, 'new',
// NEW)::text);
#[derive(Debug, Serialize, Deserialize)]
struct ChatUpdated {
    op: String,
    old: Option<Chat>,
    new: Option<Chat>,
}

// pg_notify('chat_message_created', row_to_json(NEW)::text);
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessageCreated {
    messages: Message,
    members: Vec<i64>,
}
pub async fn setup_pg_listener(state: AppState) -> anyhow::Result<()> {
    let mut listener = PgListener::connect(&state.config.server.db_url).await?;
    listener.listen("chat_updated").await?;
    // chat_message_created
    listener.listen("chat_message_created").await?;

    let mut stream = listener.into_stream();
    tokio::spawn(async move {
        while let Some(Ok(message)) = stream.next().await {
            info!("Received message: {:?}", message);
            let users = &state.inner.users;
            let notification = match Notification::load(message.channel(), message.payload()) {
                Ok(notification) => notification,
                Err(e) => {
                    warn!("Failed to load notification: {}", e);
                    continue;
                }
            };
            info!("Received notification: {:?}", notification);
            for use_id in notification.user_ids {
                if let Some(tx) = users.get(&use_id)
                    && let Err(e) = tx.send(notification.event.clone())
                {
                    warn!("Failed to send notification to user {}: {}", use_id, e);
                }
            }
        }
        info!("PgListener closed");
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}

impl Notification {
    fn load(r#type: &str, payload: &str) -> anyhow::Result<Self> {
        match r#type {
            "chat_updated" => {
                let payload: ChatUpdated = serde_json::from_str(payload)?;
                info!("ChatUpdated: {:?}", payload);
                let user_ids =
                    get_affected_chat_user_ids(payload.old.as_ref(), payload.new.as_ref());
                let event = match payload.op.as_str() {
                    "INSERT" => AppEvent::NewChat(payload.new.expect("new should exist")),
                    "UPDATE" => AppEvent::AddToChat(payload.new.expect("new should exist")),
                    "DELETE" => AppEvent::RemoveFromChat(payload.old.expect("old should exist")),
                    _ => return Err(anyhow::anyhow!("Invalid operation")),
                };
                Ok(Self {
                    user_ids,
                    event: Arc::new(event),
                })
            }
            "chat_message_created" => {
                let payload: ChatMessageCreated = serde_json::from_str(payload)?;
                let user_ids = payload.members.iter().copied().collect();
                Ok(Self {
                    user_ids,
                    event: Arc::new(AppEvent::NewMessage(payload.messages)),
                })
            }
            _ => Err(anyhow::anyhow!("Invalid notification type")),
        }
    }
}

fn get_affected_chat_user_ids(old: Option<&Chat>, new: Option<&Chat>) -> HashSet<i64> {
    match (old, new) {
        (Some(old), Some(new)) => {
            // diff old/new members, if identical, no need to notify, otherwise notify the
            // union of both
            let old_user_ids: HashSet<_> = old.members.iter().copied().collect();
            let new_user_ids: HashSet<_> = new.members.iter().copied().collect();
            if old_user_ids == new_user_ids {
                HashSet::new()
            } else {
                // notify the union of both
                old_user_ids.union(&new_user_ids).copied().collect()
            }
        }
        // delete chat, notify all members in the chat
        (Some(old), None) => old.members.iter().copied().collect(),
        // create chat, notify all members in the chat
        (None, Some(new)) => new.members.iter().copied().collect(),
        _ => HashSet::new(),
    }
}

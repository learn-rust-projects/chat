use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::{
    Extension,
    extract::State,
    response::{Sse, sse::Event},
};
use chat_core::User;
use futures_util::stream::Stream;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use tracing::{debug, info, warn};

use crate::{AppState, notify::AppEvent};

const CHANNEL_CAPACITY: usize = 256;

struct StreamWithCleanup<S> {
    inner: S,
    user_id: i64,
    users: Arc<dashmap::DashMap<i64, tokio::sync::broadcast::Sender<Arc<AppEvent>>>>,
}

impl<S> Stream for StreamWithCleanup<S>
where
    S: Stream + Unpin,
{
    type Item = S::Item;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::pin::Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl<S> Drop for StreamWithCleanup<S> {
    fn drop(&mut self) {
        let user_id = self.user_id;
        let user_map = self.users.clone();
        // std::thread::spawn(move || {
        let is_removed = if let Some(tx) = user_map.get(&user_id) {
            let active = tx.receiver_count();
            debug!(
                "User {} stream dropped, active receivers: {}",
                user_id, active
            );
            if active == 1 {
                warn!(
                    "User {} no active receivers, removing from user_map",
                    user_id
                );
                true
            } else {
                false
            }
        } else {
            false
        };
        if is_removed {
            user_map.remove(&user_id);
        }
        // });
    }
}

pub(crate) async fn sse_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("`{}` connected", user.id);

    let user_id = user.id;
    let users = state.users.clone();

    let rx = if let Some(tx) = users.get(&user_id) {
        tx.subscribe()
    } else {
        let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
        users.insert(user_id, tx);
        rx
    };

    let stream = BroadcastStream::new(rx).filter_map(|v| v.ok()).map(|v| {
        let name = match v.as_ref() {
            AppEvent::NewChat(_) => "NewChat",
            AppEvent::AddToChat(_) => "AddToChat",
            AppEvent::RemoveFromChat(_) => "RemoveFromChat",
            AppEvent::NewMessage(_) => "NewMessage",
        };
        let v = serde_json::to_string(&v).expect("serde_json::to_string failed");
        Ok(Event::default().event(name).data(v))
    });

    let stream = StreamWithCleanup {
        inner: stream,
        user_id,
        users,
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep alive text"),
    )
}

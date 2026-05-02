use std::{convert::Infallible, time::Duration};

use axum::{
    Extension,
    extract::State,
    response::{Sse, sse::Event},
};
use chat_core::User;
use futures_util::stream::Stream;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use tracing::info;

use crate::{AppState, notify::AppEvent};

const CHANNEL_CAPACITY: usize = 256;
pub(crate) async fn sse_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("`{}` connected", user.id);

    let user_id = user.id;
    let users = &state.users;

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

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep alive text"),
    )
}

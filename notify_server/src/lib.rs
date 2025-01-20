mod sse;

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chat_core::{Chat, Message};
use futures::StreamExt;
use sqlx::postgres::PgListener;
use sse::*;
use tracing::info;

pub enum Event {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

const INDEX_HTML: &str = include_str!("../index.html");

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
}

pub async fn setup_pg_listener() -> anyhow::Result<()> {
    let mut listener =
        PgListener::connect("postgres://postgres:nyh196511@localhost:5432/chat").await?;

    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;

    let mut stream = listener.into_stream();

    tokio::spawn(async move {
        while let Some(Ok(notification)) = stream.next().await {
            info!("Received notification: {notification:?}");
        }
    });

    Ok(())
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

#[cfg(test)]
mod tests {
    #[test]
    fn notify_server_test() {}
}

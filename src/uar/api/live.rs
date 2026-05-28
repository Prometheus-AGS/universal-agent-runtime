//! Server-Sent Events endpoint for the live-query bus.
//!
//! `GET /api/live/{topic}` streams change notifications from the underlying
//! SurrealDB `.select().live()` bus to a single browser client. Auth is
//! enforced by the global JWT middleware applied to `/api`.

use std::convert::Infallible;
use std::str::FromStr;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Sse, sse::Event, sse::KeepAlive},
};
use futures::Stream;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::AppState;
use crate::uar::realtime::{EntityTopic, LiveAction, LiveEvent};

pub async fn live_stream(
    State(state): State<AppState>,
    Path(topic): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    let topic = EntityTopic::from_str(&topic)
        .map_err(|_| (StatusCode::NOT_FOUND, format!("unknown topic '{topic}'")))?;

    let bus = state.live_bus.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "live-query bus not initialized (non-surreal persistence backend?)".to_string(),
    ))?;

    let rx = bus.subscribe(topic).ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("topic '{topic}' has no broadcaster"),
    ))?;

    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(event) => Some(Ok(sse_event(&event))),
        Err(_) => None, // skip lagged messages
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

fn sse_event(event: &LiveEvent) -> Event {
    let name = match event.action {
        LiveAction::Create => "create",
        LiveAction::Update => "update",
        LiveAction::Delete => "delete",
    };
    let payload = serde_json::json!({
        "topic": event.topic.as_str(),
        "id": event.id,
        "data": event.data,
    });
    Event::default()
        .event(name)
        .data(payload.to_string())
}

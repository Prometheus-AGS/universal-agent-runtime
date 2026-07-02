//! Server-Sent Events endpoint for the live-query bus.
//!
//! `GET /api/live/{topic}` streams change notifications from the underlying
//! SurrealDB `.select().live()` bus to a single browser client. Auth is
//! enforced by the global JWT middleware applied to `/api`.

use std::convert::Infallible;
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Sse, sse::Event, sse::KeepAlive},
};
use futures::Stream;
use futures::stream::select_all;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

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

/// Multiplexed live stream: `GET /api/live` fans **every** enrolled topic over a
/// single SSE connection.
///
/// Browsers cap HTTP/1.1 connections at 6 per origin. The original design opened
/// one stream per topic (10 of them), which exhausted that budget and starved
/// every other request — including the PGlite WASM the SPA needs to boot — so
/// the app hung on startup. Multiplexing keeps realtime to a single connection.
/// Each event still carries its `topic` field, so the client demultiplexes.
pub async fn live_stream_all(
    State(state): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    let bus = state.live_bus.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "live-query bus not initialized (non-surreal persistence backend?)".to_string(),
    ))?;

    // Merge each enrolled topic's broadcast receiver into one SSE feed. Boxed so
    // the per-topic `filter_map` closures unify into a single stream type.
    let mut streams: Vec<Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>> =
        Vec::with_capacity(EntityTopic::ALL.len());
    for &topic in EntityTopic::ALL {
        if let Some(rx) = bus.subscribe(topic) {
            let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
                Ok(event) => Some(Ok(sse_event(&event))),
                Err(_) => None, // skip lagged messages
            });
            streams.push(Box::pin(stream));
        }
    }

    Ok(
        Sse::new(select_all(streams))
            .keep_alive(KeepAlive::new().interval(Duration::from_secs(15))),
    )
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
    Event::default().event(name).data(payload.to_string())
}

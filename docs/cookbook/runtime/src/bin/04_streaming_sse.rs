//! Cookbook: a minimal Axum SSE streaming endpoint.

use axum::{
    Router,
    response::sse::{Event, Sse},
    routing::get,
};
use futures::stream::{self, Stream, StreamExt};
use std::convert::Infallible;
use std::time::Duration;
use tokio::net::TcpListener;

fn stream_events() -> impl Stream<Item = Result<Event, Infallible>> {
    stream::iter(vec!["connected", "processing", "done"])
        .enumerate()
        .then(|(i, msg)| async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(Event::default().event("status").data(format!("{msg} #{i}")))
        })
}

async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    Sse::new(stream_events()).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text(""),
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/events", get(sse_handler));
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let url = format!("http://{addr}/events");
    println!("SSE endpoint: {url}");

    // Start server in the background.
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start.
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Consume the SSE endpoint with a plain HTTP client.
    let http = reqwest::Client::new();
    let response = http
        .get(&url)
        .header("accept", "text/event-stream")
        .send()
        .await?;
    let bytes = response.bytes_stream();
    let mut chunks = Vec::new();

    tokio::pin!(bytes);
    while let Some(chunk) = bytes.next().await {
        let chunk = chunk?;
        chunks.extend_from_slice(&chunk);
        if String::from_utf8_lossy(&chunks).contains("done #2") {
            break;
        }
    }

    let text = String::from_utf8_lossy(&chunks);
    for line in text.lines() {
        println!("{line}");
    }
    println!("SSE stream consumed successfully.");
    Ok(())
}

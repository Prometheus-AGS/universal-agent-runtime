//! Full-harness frames retain the root identity captured by kernel assembly.

use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::response::sse::{Event, Sse};
use futures::{Stream, StreamExt};

use super::FullHarnessTaskAuthority;
use crate::uar::{api::sse::to_agui_event, runtime::manager::StreamEvent};

pub(super) fn build_response<S>(
    stream: S,
    authority: Arc<FullHarnessTaskAuthority>,
    task_id: String,
) -> Sse<impl Stream<Item = Result<Event, Infallible>> + Send>
where
    S: Stream<Item = StreamEvent> + Send + 'static,
{
    let frames = stream.then(move |event| {
        let authority = Arc::clone(&authority);
        let task_id = task_id.clone();
        async move {
            if crate::uar::api::routes::is_terminal_stream_event(&event) {
                // Settle the owning receipt before this frame can close the stream.
                let _ = authority.refresh_receipt(&task_id).await;
            }
            let root_run_id = authority.capture_root(&task_id).await;
            to_agui_event(&event.event).map(|(name, mut payload)| {
                if let Some(root_run_id) = root_run_id {
                    payload["root_run_id"] = serde_json::json!(root_run_id);
                }
                Ok(Event::default().event(name).id(event.id.to_string()).data(payload.to_string()))
            })
        }
    }).filter_map(|frame| async move { frame });
    Sse::new(frames)
        .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)))
}

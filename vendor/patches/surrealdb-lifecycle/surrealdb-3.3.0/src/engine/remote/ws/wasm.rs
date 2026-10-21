use std::collections::HashSet;
use std::sync::Arc;
use async_channel::{Receiver, Sender};
use futures::stream::{SplitSink, SplitStream};
use futures::{FutureExt, SinkExt, StreamExt};
use tokio::sync::{RwLock, watch};
use tokio_tungstenite_wasm::{Message, WebSocketStream, connect_with_protocols};
use wasm_bindgen_futures::spawn_local;

use super::{
	HandleResult, PATH, SessionState, WsMessage, all_sessions_replay_ready, handle_response,
	handle_route, handle_session, replay_session, reset_sessions,
};
use crate::conn::{self, ReconnectRequest, Route, Router};
use crate::engine::SessionError;
use crate::method::BoxFuture;
use crate::opt::{Endpoint, WaitFor};
use crate::types::HashMap;
use crate::{Error, ExtraFeatures, Result, SessionClone, SessionId, Surreal};

type MessageStream = SplitStream<WebSocketStream>;
type MessageSink = SplitSink<WebSocketStream, Message>;
type Sessions = HashMap<uuid::Uuid, std::result::Result<Arc<SessionState>, SessionError>>;

// ============================================================================
// Platform Implementation
// ============================================================================

impl WsMessage for Message {
	fn binary(payload: Vec<u8>) -> Self {
		Message::Binary(payload.into())
	}

	fn as_binary(&self) -> Option<&[u8]> {
		match self {
			Message::Binary(data) => Some(data),
			_ => None,
		}
	}

	fn should_process(&self) -> bool {
		matches!(self, Message::Binary(_))
	}

	fn log_description(&self) -> &'static str {
		match self {
			Message::Text(_) => "text message",
			Message::Binary(_) => "binary message",
			Message::Close(_) => "close message",
		}
	}
}

impl crate::Connection for super::Client {}
impl conn::Sealed for super::Client {
	#[allow(private_interfaces)]
	fn connect(
		mut address: Endpoint,
		capacity: usize,
		session_clone: Option<crate::SessionClone>,
	) -> BoxFuture<'static, Result<Surreal<Self>>> {
		Box::pin(async move {
			address.url = address.url.join(PATH).map_err(crate::std_error_to_types_error)?;

			let (route_tx, route_rx) = match capacity {
				0 => async_channel::unbounded(),
				capacity => async_channel::bounded(capacity),
			};

			let (conn_tx, conn_rx) = async_channel::bounded(1);
			let config = address.config.clone();
			let session_clone = session_clone.unwrap_or_else(SessionClone::new);
			let (transport_state, _) = watch::channel(crate::TransportState::Available);
			let transport_generation = Arc::new(std::sync::atomic::AtomicU64::new(0));
			let (reconnect_tx, reconnect_rx) = async_channel::unbounded();

			let driver = Arc::new(crate::conn::driver_scope::DriverScope::default());
			driver.spawn(run_router(
				address, conn_tx, route_rx, session_clone.receiver.clone(), transport_state.clone(),
				Arc::clone(&transport_generation), reconnect_rx,
			));

			conn_rx.recv().await.map_err(crate::std_error_to_types_error)??;

			let mut features = HashSet::new();
			features.insert(ExtraFeatures::LiveQueries);

			let waiter = watch::channel(Some(WaitFor::Connection));
			let mut router = Router::from_route_sender_with_reconnect(
				route_tx, features, config, transport_state, transport_generation, reconnect_tx,
			);

			router.driver = Some(driver);

			Ok((router, waiter, session_clone).into())
		})
	}
}

// ============================================================================
// Router State
// ============================================================================

/// Router state for WASM WebSocket connections.
struct RouterState {
	sessions: Sessions,
	sink: RwLock<MessageSink>,
	stream: RwLock<MessageStream>,
}

impl RouterState {
	fn new(sink: MessageSink, stream: MessageStream) -> Self {
		RouterState {
			sessions: HashMap::new(),
			sink: RwLock::new(sink),
			stream: RwLock::new(stream),
		}
	}

	async fn update_connection(&self, sink: MessageSink, stream: MessageStream) {
		*self.sink.write().await = sink;
		*self.stream.write().await = stream;
	}
}

// ============================================================================
// Router
// ============================================================================

pub(crate) async fn run_router(
	endpoint: Endpoint,
	conn_tx: Sender<Result<()>>,
	route_rx: Receiver<Route>,
	session_rx: Receiver<SessionId>,
	transport_state: watch::Sender<crate::TransportState>,
	transport_generation: Arc<std::sync::atomic::AtomicU64>,
	reconnect_rx: Receiver<ReconnectRequest>,
) {
	let socket = match connect_with_protocols(endpoint.url.as_str(), &["flatbuffers"]).await {
		Ok(socket) => socket,
		Err(error) => {
			let _ = conn_tx.send(Err(Error::connection(
				format!("WebSocket error: {}", error),
				surrealdb_types::ConnectionError::ConnectionFailed,
			))).await;
			return;
		}
	};
	if conn_tx.send(Ok(())).await.is_err() { return; }
	let (sink, stream) = socket.split();
	#[allow(clippy::arc_with_non_send_sync)]
	let state = Arc::new(RouterState::new(sink, stream));
	let mut unavailable = false;
	let mut reconnecting = None;
	let mut queued_sessions = Vec::new();

	loop {
		if unavailable {
			futures::select! {
				request = reconnect_rx.recv().fuse() => {
					let Ok(request) = request else { return };
					let generation = transport_generation.load(std::sync::atomic::Ordering::Acquire);
					if request.expected_generation != generation {
						let _ = request.response.send(Err(Error::internal("stale transport generation".to_owned()))).await;
						continue;
					}
					let result = async {
						let socket = connect_with_protocols(endpoint.url.as_str(), &["flatbuffers"]).await
							.map_err(|error| Error::connection(format!("WebSocket error: {}", error), surrealdb_types::ConnectionError::ConnectionFailed))?;
						let (sink, stream) = socket.split();
						state.update_connection(sink, stream).await;
						for (session_id, session) in state.sessions.to_vec() {
							if let Ok(session) = session {
								let _ = replay_session::<Message, _, _>(session_id, &session, &state.sink).await?;
							}
						}
						all_sessions_replay_ready(&state.sessions)
					}.await;
					match result {
						Ok(true) => {
							let next = transport_generation.fetch_add(1, std::sync::atomic::Ordering::AcqRel) + 1;
							transport_state.send_replace(crate::TransportState::Available);
							let _ = request.response.send(Ok(next)).await;
							unavailable = false;
						}
						Ok(false) => {
							// Replay is in flight. The existing stream must carry every
							// acknowledgement before this explicit reconnect can report success.
							reconnecting = Some(request);
							unavailable = false;
						}
						Err(error) => { let _ = request.response.send(Err(error)).await; }
					}
				}
				route = route_rx.recv().fuse() => {
					let Ok(route) = route else {
						transport_state.send_replace(crate::TransportState::Released);
						return;
					};
					let _ = route.response.send(Err(Error::connection(
						"remote transport unavailable; operation was not admitted".to_owned(),
						surrealdb_types::ConnectionError::ConnectionFailed,
					))).await;
				}
				session = session_rx.recv().fuse() => {
					let Ok(session) = session else { return };
					queued_sessions.push(session);
				}
			}
			continue;
		}

		if let Some(request) = reconnecting.take() {
			futures::select! {
				route = route_rx.recv().fuse() => {
					let Ok(route) = route else {
						let _ = request.response.send(Err(Error::connection(
							"reconnect interrupted by local shutdown".to_owned(),
							surrealdb_types::ConnectionError::ConnectionFailed,
						))).await;
						transport_state.send_replace(crate::TransportState::Released);
						return;
					};
					let _ = route.response.send(Err(Error::connection(
						"remote session replay is incomplete; operation was not admitted".to_owned(),
						surrealdb_types::ConnectionError::ConnectionFailed,
					))).await;
					reconnecting = Some(request);
				}
				session = session_rx.recv().fuse() => {
					let Ok(session) = session else {
						let _ = request.response.send(Err(Error::connection(
							"reconnect interrupted by session lifecycle closure".to_owned(),
							surrealdb_types::ConnectionError::ConnectionFailed,
						))).await;
						transport_state.send_replace(crate::TransportState::Released);
						return;
					};
					queued_sessions.push(session);
					reconnecting = Some(request);
				}
				new_request = reconnect_rx.recv().fuse() => {
					let Ok(new_request) = new_request else {
						let _ = request.response.send(Err(Error::connection(
							"reconnect control channel closed during session replay".to_owned(),
							surrealdb_types::ConnectionError::ConnectionFailed,
						))).await;
						transport_state.send_replace(crate::TransportState::Released);
						return;
					};
					let _ = new_request.response.send(Err(Error::internal(
						"a reconnect is already waiting for session replay".to_owned(),
					))).await;
					reconnecting = Some(request);
				}
				result = async { state.stream.write().await.next().await }.fuse() => {
					let disconnected = match result {
						Some(Ok(message)) => matches!(handle_response::<Message, _, _>(&message, None, &state.sessions, &state.sink).await, HandleResult::Disconnected),
						Some(Err(error)) => { trace!("WebSocket transport failed during session replay: {error}"); true }
						None => { trace!("WebSocket stream ended during session replay"); true }
					};
					if disconnected {
						reset_sessions(&state.sessions).await;
						let _ = request.response.send(Err(Error::connection(
							"connection lost during session replay; reconnect outcome is unknown".to_owned(),
							surrealdb_types::ConnectionError::ConnectionFailed,
						))).await;
						unavailable = true;
					} else {
						match all_sessions_replay_ready(&state.sessions) {
							Ok(true) => {
								let next = transport_generation.fetch_add(1, std::sync::atomic::Ordering::AcqRel) + 1;
								transport_state.send_replace(crate::TransportState::Available);
								let _ = request.response.send(Ok(next)).await;
							}
							Ok(false) => reconnecting = Some(request),
							Err(error) => {
								let _ = request.response.send(Err(error)).await;
								unavailable = true;
							}
						}
					}
				}
			}
			continue;
		}

		for session_id in queued_sessions.drain(..) {
			handle_session::<Message, _, _>(session_id, &state.sessions, &state.sink).await;
		}
		futures::select! {
			session = session_rx.recv().fuse() => {
				let Ok(session_id) = session else { return };
				handle_session::<Message, _, _>(session_id, &state.sessions, &state.sink).await;
			}
			route = route_rx.recv().fuse() => {
				let Ok(route) = route else {
					let _ = state.sink.write().await.close().await;
					transport_state.send_replace(crate::TransportState::Released);
					return;
				};
				while let Ok(session_id) = session_rx.try_recv() {
					handle_session::<Message, _, _>(session_id, &state.sessions, &state.sink).await;
				}
				if matches!(handle_route::<Message, _, _>(route, None, &state.sessions, &state.sink).await, HandleResult::Disconnected) {
					transport_state.send_replace(crate::TransportState::Unavailable);
					reset_sessions(&state.sessions).await;
					unavailable = true;
				}
			}
			result = async { state.stream.write().await.next().await }.fuse() => {
				let disconnected = match result {
					Some(Ok(message)) => matches!(handle_response::<Message, _, _>(&message, None, &state.sessions, &state.sink).await, HandleResult::Disconnected),
					Some(Err(error)) => { trace!("WebSocket transport failed: {error}"); true }
					None => { trace!("WebSocket stream ended"); true }
				};
				if disconnected {
					transport_state.send_replace(crate::TransportState::Unavailable);
					reset_sessions(&state.sessions).await;
					unavailable = true;
				}
			}
		}
	}
}

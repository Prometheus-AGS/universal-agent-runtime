use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use async_channel::Receiver;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use surrealdb_types::{ConnectionError, ValidationError};
use tokio::net::TcpStream;
use tokio::sync::{RwLock, watch};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::http::header::SEC_WEBSOCKET_PROTOCOL;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};
use uuid::Uuid;

use super::{
	HandleResult, PATH, SessionState, WsMessage, all_sessions_replay_ready, answer_encoded,
	handle_response, handle_route, handle_session, replay_session, reset_sessions,
	take_for_decoding_apart,
};
use crate::conn::driver_scope::DriverScope;
use crate::conn::{self, ReconnectRequest, Route, Router};
use crate::engine::SessionError;
use crate::method::BoxFuture;
#[cfg(any(feature = "native-tls", feature = "rustls"))]
use crate::opt::Tls;
use crate::opt::{Endpoint, WaitFor};
use crate::types::HashMap;
use crate::{Error, ExtraFeatures, SessionClone, Surreal, TransportState};

pub(crate) const NAGLE_ALG: bool = false;

type MessageSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type MessageStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type Sessions = HashMap<Uuid, Result<Arc<SessionState>, SessionError>>;

// ============================================================================
// Platform Implementation
// ============================================================================

impl WsMessage for Message {
	fn binary(payload: Vec<u8>) -> Self {
		Message::Binary(payload.into())
	}

	fn as_binary(&self) -> Option<&[u8]> {
		if let Message::Binary(data) = self {
			Some(data)
		} else {
			None
		}
	}

	fn should_process(&self) -> bool {
		matches!(self, Message::Binary(_))
	}

	fn log_description(&self) -> &'static str {
		match self {
			Message::Text(_) => "text message",
			Message::Binary(_) => "binary message",
			Message::Ping(_) => "ping",
			Message::Pong(_) => "pong",
			Message::Frame(_) => "raw frame",
			Message::Close(_) => "close message",
		}
	}
}

#[cfg(any(feature = "native-tls", feature = "rustls"))]
impl From<Tls> for Connector {
	fn from(tls: Tls) -> Self {
		match tls {
			#[cfg(feature = "native-tls")]
			Tls::Native(config) => Self::NativeTls(config),
			#[cfg(feature = "rustls")]
			Tls::Rust(config) => Self::Rustls(std::sync::Arc::new(config)),
		}
	}
}

pub(crate) async fn connect(
	endpoint: &Endpoint,
	config: Option<WebSocketConfig>,
	#[cfg_attr(not(any(feature = "native-tls", feature = "rustls")), expect(unused_variables))]
	maybe_connector: Option<Connector>,
) -> crate::Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
	let mut request = (&endpoint.url).into_client_request().map_err(|err| {
		Error::validation(format!("Invalid URL: {}", err), ValidationError::InvalidRequest)
	})?;

	request.headers_mut().insert(
		SEC_WEBSOCKET_PROTOCOL,
		HeaderValue::from_static(surrealdb_rpc::wire::PROTOCOLS_HEADER),
	);

	#[cfg(any(feature = "native-tls", feature = "rustls"))]
	let (socket, _) = tokio_tungstenite::connect_async_tls_with_config(
		request,
		config,
		NAGLE_ALG,
		maybe_connector,
	)
	.await
	.map_err(|err| {
		Error::connection(format!("WebSocket error: {}", err), ConnectionError::ConnectionFailed)
	})?;

	#[cfg(not(any(feature = "native-tls", feature = "rustls")))]
	let (socket, _) = tokio_tungstenite::connect_async_with_config(request, config, NAGLE_ALG)
		.await
		.map_err(|err| {
		Error::connection(format!("WebSocket error: {}", err), ConnectionError::ConnectionFailed)
	})?;

	Ok(socket)
}

impl crate::Connection for super::Client {}
impl conn::Sealed for super::Client {
	#[allow(private_interfaces)]
	fn connect(
		mut address: Endpoint,
		capacity: usize,
		session_clone: Option<crate::SessionClone>,
	) -> BoxFuture<'static, crate::Result<Surreal<Self>>> {
		Box::pin(async move {
			address.url = address
				.url
				.join(PATH)
				.map_err(|e| Error::validation(e.to_string(), ValidationError::InvalidRequest))?;
			#[cfg(any(feature = "native-tls", feature = "rustls"))]
			let maybe_connector = address.config.tls_config.clone().map(Connector::from);
			#[cfg(not(any(feature = "native-tls", feature = "rustls")))]
			let maybe_connector = None;

			let ws_config = WebSocketConfig::default()
				.read_buffer_size(address.config.websocket.read_buffer_size)
				.max_message_size(address.config.websocket.max_message_size)
				.max_frame_size(address.config.websocket.max_message_size)
				.max_write_buffer_size(address.config.websocket.max_write_buffer_size)
				.write_buffer_size(address.config.websocket.write_buffer_size);

			let socket = connect(&address, Some(ws_config), maybe_connector.clone()).await?;

			let (route_tx, route_rx) = match capacity {
				0 => async_channel::unbounded(),
				capacity => async_channel::bounded(capacity),
			};
			let config = address.config.clone();
			let session_clone = session_clone.unwrap_or_else(SessionClone::new);
			let (transport_state, _) = watch::channel(TransportState::Available);
			let transport_generation = Arc::new(AtomicU64::new(0));
			let (reconnect_tx, reconnect_rx) = async_channel::unbounded();

			let driver = Arc::new(DriverScope::default());
			driver.spawn(run_router(
				address,
				maybe_connector,
				ws_config,
				socket,
				route_rx,
				session_clone.router_receiver(),
				transport_state.clone(),
				Arc::clone(&transport_generation),
				reconnect_rx,
				Arc::clone(&driver),
			));

			let mut features = HashSet::new();
			features.insert(ExtraFeatures::LiveQueries);

			let waiter = watch::channel(Some(WaitFor::Connection));
			let mut router = Router::from_route_sender_with_reconnect(
				route_tx,
				features,
				config,
				transport_state,
				transport_generation,
				reconnect_tx,
			);

			router.driver = Some(driver);

			Ok((router, waiter, session_clone).into())
		})
	}
}

// ============================================================================
// Router State
// ============================================================================

/// Router state for native WebSocket connections.
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

async fn run_router(
	endpoint: Endpoint,
	maybe_connector: Option<Connector>,
	config: WebSocketConfig,
	socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
	route_rx: Receiver<Route>,
	session_rx: crate::RouterSessions,
	transport_state: watch::Sender<TransportState>,
	transport_generation: Arc<AtomicU64>,
	reconnect_rx: Receiver<ReconnectRequest>,
	driver: Arc<DriverScope>,
) {
	let (socket_sink, socket_stream) = socket.split();
	let state = RouterState::new(socket_sink, socket_stream);
	let mut unavailable = false;
	let mut reconnecting = None;
	let mut queued_sessions = VecDeque::new();

	'router: loop {
		if unavailable {
			tokio::select! {
				request = reconnect_rx.recv() => {
					let Ok(request) = request else {
						return;
					};
					let generation = transport_generation.load(Ordering::Acquire);
					if request.expected_generation != generation {
						let _ = request.response.send(Err(Error::internal(
							"stale transport generation".to_owned(),
						))).await;
						continue;
					}
					let result = async {
						let socket = connect(&endpoint, Some(config), maybe_connector.clone()).await?;
						let (sink, stream) = socket.split();
						state.update_connection(sink, stream).await;
						for (session_id, session) in state.sessions.to_vec() {
							if let Ok(session) = session
								&& !session.unattached.load(Ordering::SeqCst)
							{
								replay_session::<Message, _, _>(session_id, &session, &state.sink).await?;
							}
						}
						all_sessions_replay_ready(&state.sessions)
					}.await;
					match result {
						Ok(true) => {
							let next = transport_generation.fetch_add(1, Ordering::AcqRel) + 1;
							transport_state.send_replace(TransportState::Available);
							let _ = request.response.send(Ok(next)).await;
							unavailable = false;
						}
						Ok(false) => {
							// Replay is in flight. The existing stream must carry every
							// acknowledgement before this explicit reconnect can report success.
							reconnecting = Some(request);
							unavailable = false;
						}
						Err(error) => {
							let _ = request.response.send(Err(error)).await;
						}
					}
				}
				route = route_rx.recv() => {
					let Ok(route) = route else {
						return;
					};
					let _ = route.response.send(Err(Error::connection(
						"remote transport unavailable; operation was not admitted".to_owned(),
						ConnectionError::ConnectionFailed,
					))).await;
				}
				session = session_rx.recv() => {
					let Ok(session) = session else {
						return;
					};
					queued_sessions.push_back(session);
				}
			}
			continue;
		}

		if let Some(request) = reconnecting.take() {
			tokio::select! {
				route = route_rx.recv() => {
					let Ok(route) = route else {
						let _ = request.response.send(Err(Error::connection(
							"reconnect interrupted by local shutdown".to_owned(),
							ConnectionError::ConnectionFailed,
						))).await;
						return;
					};
					let _ = route.response.send(Err(Error::connection(
						"remote session replay is incomplete; operation was not admitted".to_owned(),
						ConnectionError::ConnectionFailed,
					))).await;
					reconnecting = Some(request);
				}
				session = session_rx.recv() => {
					let Ok(session) = session else {
						let _ = request.response.send(Err(Error::connection(
							"reconnect interrupted by session lifecycle closure".to_owned(),
							ConnectionError::ConnectionFailed,
						))).await;
						return;
					};
					queued_sessions.push_back(session);
					reconnecting = Some(request);
				}
				new_request = reconnect_rx.recv() => {
					let Ok(new_request) = new_request else {
						let _ = request.response.send(Err(Error::connection(
							"reconnect control channel closed during session replay".to_owned(),
							ConnectionError::ConnectionFailed,
						))).await;
						return;
					};
					let _ = new_request.response.send(Err(Error::internal(
						"a reconnect is already waiting for session replay".to_owned(),
					))).await;
					reconnecting = Some(request);
				}
				result = async { state.stream.write().await.next().await } => {
					let disconnected = match result {
						Some(Ok(message)) => {
							matches!(handle_response::<Message, _, _>(
								&message, config.max_message_size, &state.sessions, &state.sink,
							).await, HandleResult::Disconnected)
						}
						Some(Err(error)) => { trace!("WebSocket transport failed during session replay: {error}"); true }
						None => { trace!("WebSocket stream ended during session replay"); true }
					};
					if disconnected {
						reset_sessions(&state.sessions).await;
						let _ = request.response.send(Err(Error::connection(
							"connection lost during session replay; reconnect outcome is unknown".to_owned(),
							ConnectionError::ConnectionFailed,
						))).await;
						unavailable = true;
					} else {
						match all_sessions_replay_ready(&state.sessions) {
							Ok(true) => {
								let next = transport_generation.fetch_add(1, Ordering::AcqRel) + 1;
								transport_state.send_replace(TransportState::Available);
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

		while let Some(session_id) = queued_sessions.pop_front() {
			if handle_session::<Message, _, _>(session_id, &state.sessions, &state.sink).await
				== HandleResult::Disconnected
			{
				transport_state.send_replace(TransportState::Unavailable);
				reset_sessions(&state.sessions).await;
				unavailable = true;
				continue 'router;
			}
		}
		tokio::select! {
			biased;
			request = reconnect_rx.recv() => {
				let Ok(request) = request else {
					return;
				};
				let _ = request.response.send(Err(Error::internal(
					"remote transport is already available".to_owned(),
				))).await;
			}
			session = session_rx.recv() => {
				let Ok(session_id) = session else {
					return;
				};
				if handle_session::<Message, _, _>(session_id, &state.sessions, &state.sink).await
					== HandleResult::Disconnected
				{
					transport_state.send_replace(TransportState::Unavailable);
					reset_sessions(&state.sessions).await;
					unavailable = true;
				}
			}
			route = route_rx.recv() => {
				let Ok(route) = route else {
					let _ = state.sink.write().await.send(Message::Close(None)).await;
					return;
				};
				// A route may follow a clone or registration on the separate session
				// channel. Apply those events before admitting the request.
				while let Ok(session_id) = session_rx.try_recv() {
					if handle_session::<Message, _, _>(session_id, &state.sessions, &state.sink).await
						== HandleResult::Disconnected
					{
						transport_state.send_replace(TransportState::Unavailable);
						reset_sessions(&state.sessions).await;
						let _ = route.response.send(Err(Error::connection(
							"remote transport unavailable; operation was not admitted".to_owned(),
							ConnectionError::ConnectionFailed,
						))).await;
						unavailable = true;
						continue 'router;
					}
				}
				if handle_route::<Message, _, _>(route, config.max_message_size, &state.sessions, &state.sink).await
					== HandleResult::Disconnected
				{
					transport_state.send_replace(TransportState::Unavailable);
					reset_sessions(&state.sessions).await;
					unavailable = true;
				}
			}
			result = async { state.stream.write().await.next().await } => {
				let disconnected = match result {
					Some(Ok(message)) => {
						// The response already reached this generation. Decode it
						// outside the router while keeping it in the shutdown scope.
						if let Message::Binary(bytes) = &message
							&& let Some(pending) = take_for_decoding_apart(bytes, &state.sessions)
						{
							let bytes = bytes.clone();
							driver.spawn(async move { answer_encoded(&bytes, pending).await });
							continue;
						}
						matches!(handle_response::<Message, _, _>(
							&message, config.max_message_size, &state.sessions, &state.sink,
						).await, HandleResult::Disconnected)
					},
					Some(Err(error)) => { trace!("WebSocket transport failed: {error}"); true }
					None => { trace!("WebSocket stream ended"); true }
				};
				if disconnected {
					transport_state.send_replace(TransportState::Unavailable);
					reset_sessions(&state.sessions).await;
					unavailable = true;
				}
			}
		}
	}
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
	use std::io::Write;

	use flate2::Compression;
	use flate2::write::GzEncoder;
	use rand::Rng;
	use web_time::SystemTime;

	use crate::types::{Array, Value};

	#[test_log::test]
	fn large_vector_serialisation_bench() {
		let timed = |func: &dyn Fn() -> Vec<u8>| {
			let start = SystemTime::now();
			let r = func();
			(start.elapsed().unwrap(), r)
		};
		let compress = |v: &Vec<u8>| {
			let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
			encoder.write_all(v).unwrap();
			encoder.finish().unwrap()
		};
		let vector_size = if cfg!(debug_assertions) {
			200_000
		} else {
			2_000_000
		};
		let mut vector: Vec<i32> = Vec::new();
		let mut rng = rand::rng();
		for _ in 0..vector_size {
			vector.push(rng.random());
		}
		let mut results = vec![];

		let vector = Value::Array(Array::from(vector));

		const CBOR: &str = "CBOR Vec<Value>";
		const CBOR_COMPRESSED: &str = "CBOR Compressed Vec<Value>";
		{
			let (duration, payload) =
				timed(&|| surrealdb_rpc::wire::encode(vector.clone()).unwrap());
			results.push((payload.len(), CBOR, duration, 1.0));

			let (compression_duration, payload) = timed(&|| compress(&payload));
			let duration = duration + compression_duration;
			results.push((payload.len(), CBOR_COMPRESSED, duration, 1.0));
		}

		results.sort_by_key(|(a, _, _, _)| *a);
		for (size, name, duration, factor) in &results {
			info!("{name} - Size: {size} - Duration: {duration:?} - Factor: {factor}");
		}

		let results: Vec<&str> = results.into_iter().map(|(_, name, _, _)| name).collect();
		assert_eq!(results, vec![CBOR_COMPRESSED, CBOR])
	}
}

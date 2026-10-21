use std::collections::HashSet;
use std::sync::Arc;

use async_channel::{Receiver, Sender};
use futures::FutureExt;
use reqwest::ClientBuilder;
use tokio::sync::watch;
use wasm_bindgen_futures::spawn_local;

use super::{Client, RouterState};
use crate::conn::{Route, Router};
use crate::engine::remote::RemoteCommand;
use crate::engine::{SessionError, session_error_to_error};
use crate::method::BoxFuture;
use crate::opt::{Endpoint, WaitFor};
use crate::{Error, ExtraFeatures, Result, SessionClone, SessionId, Surreal, conn};

impl crate::Connection for Client {}
impl conn::Sealed for Client {
	#[allow(private_interfaces)]
	fn connect(
		address: Endpoint,
		capacity: usize,
		session_clone: Option<SessionClone>,
	) -> BoxFuture<'static, Result<Surreal<Self>>> {
		Box::pin(async move {
			let (route_tx, route_rx) = match capacity {
				0 => async_channel::unbounded(),
				capacity => async_channel::bounded(capacity),
			};

			let (conn_tx, conn_rx) = async_channel::bounded(1);
			let config = address.config.clone();
			let session_clone = session_clone.unwrap_or_else(SessionClone::new);
			let (transport_state, _) = watch::channel(crate::TransportState::Available);

			let driver = Arc::new(crate::conn::driver_scope::DriverScope::default());
			driver.spawn(run_router(
				address,
				conn_tx,
				route_rx,
				session_clone.receiver.clone(),
				transport_state.clone(),
				Arc::clone(&driver),
			));

			// The router owns the startup probe. If that probe fails after the
			// driver was registered, wait for the tracked router to finish before
			// returning the failure. Dropping the connect future must not leave a
			// browser task alive with a client that was never published.
			let startup = match conn_rx.recv().await {
				Ok(result) => result,
				Err(error) => Err(crate::std_error_to_types_error(error)),
			};
			if let Err(startup_error) = startup {
				return match driver.join().await {
					Ok(()) => Err(startup_error),
					Err(driver_error) => Err(Error::internal(format!(
						"HTTP startup failed: {startup_error}; driver release failed: {driver_error}"
					))),
				};
			}

			let mut features = HashSet::new();
			features.insert(ExtraFeatures::Backup);

			let waiter = watch::channel(Some(WaitFor::Connection));
			let mut router = Router::from_route_sender_with_transport_state(
				route_tx, features, config, transport_state,
			);

			router.driver = Some(driver);

			Ok((router, waiter, session_clone).into())
		})
	}
}

async fn create_client(base_url: &url::Url) -> Result<reqwest::Client> {
	let headers = super::default_headers();
	let builder = ClientBuilder::new().default_headers(headers);
	let client = builder.build().map_err(crate::std_error_to_types_error)?;
	let health = base_url.join("health").map_err(crate::std_error_to_types_error)?;
	super::health(client.get(health)).await?;
	Ok(client)
}

pub(crate) async fn run_router(
	address: Endpoint,
	conn_tx: Sender<Result<()>>,
	route_rx: Receiver<Route>,
	session_rx: Receiver<SessionId>,
	transport_state: watch::Sender<crate::TransportState>,
	driver: Arc<crate::conn::driver_scope::DriverScope>,
) {
	let base_url = address.url;

	let client = match create_client(&base_url).await {
		Ok(client) => {
			conn_tx.send(Ok(())).await.ok();
			client
		}
		Err(error) => {
			conn_tx.send(Err(error)).await.ok();
			return;
		}
	};

	let state = Arc::new(RouterState::new(client, base_url));

	loop {
		futures::select! {
			session = session_rx.recv().fuse() => {
				let Ok(session_id) = session else {
					break
				};
				state.handle_session(session_id).await;
			}
			route = route_rx.recv().fuse() => {
				let Ok(route) = route else {
					break
				};

				if *transport_state.borrow() != crate::TransportState::Available {
					route.response.send(Err(Error::connection(
						"remote HTTP transport unavailable; operation was not admitted".to_owned(),
						surrealdb_types::ConnectionError::ConnectionFailed,
					))).await.ok();
					continue;
				}

				// Apply any session-lifecycle events that were enqueued before this
				// query. The session channel is separate from the route channel, so a
				// freshly registered or cloned session may not have been processed yet;
				// receiving this route establishes a happens-before with the sender, so
				// those earlier events are now observable and one drain pass suffices.
				while let Ok(session_id) = session_rx.try_recv() {
					state.handle_session(session_id).await;
				}

				let session_id = route.request.session_id;
				let command = route.request.command.clone();

				// Get session state for this session_id
				let session_state = match state.sessions.get(&session_id) {
					Some(Ok(state)) => state,
					Some(Err(error)) => {
						route.response.send(Err(session_error_to_error(error))).await.ok();
						continue;
					}
					None => {
						let error = session_error_to_error(SessionError::NotFound(session_id));
						route.response.send(Err(error)).await.ok();
						continue;
					}
				};

				// Spawn the request handling in a background task
				// SessionState uses RwLock internally, so we can share the Arc directly
				let router_state = Arc::clone(&state);
				let transport_state = transport_state.clone();
				let request_driver = Arc::clone(&driver);
				driver.spawn(async move {
					let result = super::router(
						route.request,
						&router_state.base_url,
						&router_state.client,
						&session_state,
						&request_driver,
					)
					.await;

					if result.as_ref().is_err_and(|error| error.is_connection()) {
						transport_state.send_replace(crate::TransportState::Unavailable);
					}

					// On success, add replayable commands to the replay list.
					// boxcar::Vec is lock-free, so this is safe to do concurrently.
					// NOTE: see native.rs for why HTTP doesn't coalesce Use entries.
					if result.is_ok() && command.replayable() {
						session_state.replay.push(command);
					}

					match result {
						Ok(value) => {
							route.response.send(Ok(value)).await.ok();
						}
						Err(error) => {
							route.response.send(Err(error)).await.ok();
						}
					}
				});
			}
		}
	}
}

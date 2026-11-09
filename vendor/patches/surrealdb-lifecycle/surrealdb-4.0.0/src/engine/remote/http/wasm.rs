use std::collections::HashSet;
use std::sync::Arc;

use async_channel::{Receiver, Sender};
use futures::FutureExt;
use reqwest::ClientBuilder;
use tokio::sync::watch;

use super::{Client, RouterState};
use crate::conn::driver_scope::DriverScope;
use crate::conn::{Route, Router};
use crate::engine::{SessionError, session_error_to_error};
use crate::method::BoxFuture;
use crate::opt::{Endpoint, WaitFor};
use crate::{Error, ExtraFeatures, Result, SessionClone, Surreal, TransportState, conn};

impl crate::Connection for Client {}
impl conn::Sealed for Client {
	#[allow(private_interfaces)]
	fn connect(
		address: Endpoint,
		capacity: usize,
		session_clone: Option<SessionClone>,
	) -> BoxFuture<'static, Result<Surreal<Self>>> {
		Box::pin(async move {
			let config = address.config;
			let base_url = address.url;

			let (route_tx, route_rx) = match capacity {
				0 => async_channel::unbounded(),
				capacity => async_channel::bounded(capacity),
			};

			let (conn_tx, conn_rx) = async_channel::bounded(1);
			let session_clone = session_clone.unwrap_or_else(SessionClone::new);
			let (transport_state, _) = watch::channel(TransportState::Available);
			let driver = Arc::new(DriverScope::default());
			driver.spawn(run_router(
				base_url,
				conn_tx,
				route_rx,
				session_clone.router_receiver(),
				transport_state.clone(),
				Arc::clone(&driver),
			));

			let startup = conn_rx.recv().await.map_err(crate::std_error_to_types_error);
			if let Err(startup_error) = startup.and_then(|result| result) {
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
			let mut router =
				Router::from_route_sender_with_transport_state(route_tx, features, config, transport_state);
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

async fn run_router(
	base_url: url::Url,
	conn_tx: Sender<Result<()>>,
	route_rx: Receiver<Route>,
	session_rx: crate::RouterSessions,
	transport_state: watch::Sender<TransportState>,
	driver: Arc<DriverScope>,
) {
	// Browser fetch futures are local. Stop probing if the connect future drops.
	let startup = futures::select_biased! {
		() = conn_tx.closed().fuse() => return,
		result = create_client(&base_url).fuse() => result,
	};
	let client = match startup {
		Ok(client) => {
			if conn_tx.send(Ok(())).await.is_err() {
				return;
			}
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

				// Apply any session-lifecycle events that were enqueued before this
				// query. The session channel is separate from the route channel, so a
				// freshly registered or cloned session may not have been processed yet;
				// receiving this route establishes a happens-before with the sender, so
				// those earlier events are now observable and one drain pass suffices.
				while let Ok(session_id) = session_rx.try_recv() {
					state.handle_session(session_id).await;
				}

				if *transport_state.borrow() != TransportState::Available {
					route.response.send(Err(super::transport_unavailable_error())).await.ok();
					continue;
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
					if *transport_state.borrow() != TransportState::Available {
						route.response.send(Err(super::transport_unavailable_error())).await.ok();
						return;
					}

					let result = super::router(
						route.request,
						&router_state.base_url,
						&router_state.client,
						&session_state,
						&request_driver,
					)
					.await;

					if result.as_ref().is_err_and(|error| error.is_connection()) {
						transport_state.send_replace(TransportState::Unavailable);
					}

					// On success, record the command in the replay list.
					// boxcar::Vec is lock-free, so this is safe to do concurrently.
					// NOTE: see native.rs for why HTTP doesn't coalesce Use entries.
					if result.is_ok() {
						super::record_replay(&session_state, command).await;
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
	// The loop only ends once the SDK has closed its session channel, so no
	// further `Drop` events will arrive to detach what is left.
	state.detach_all().await;
}

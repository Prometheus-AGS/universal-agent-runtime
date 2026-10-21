#![recursion_limit = "256"]

//! This library provides a low-level database library implementation, a remote
//! client and a query language definition, for [SurrealDB](https://surrealdb.com), the ultimate cloud database for
//! tomorrow's applications. SurrealDB is a scalable, distributed,
//! collaborative, document-graph database for the realtime web.
//!
//! This library can be used to start an [embedded](crate::engine::local)
//! in-memory datastore, an embedded datastore persisted to disk, a
//! browser-based embedded datastore backed by IndexedDB, or for connecting to a distributed [TiKV](https://tikv.org) key-value store.
//!
//! It also enables simple and advanced querying of a
//! [remote](crate::engine::remote) SurrealDB server from server-side or
//! client-side code. WebSocket transports report loss and close local
//! admission; trusted hosts explicitly decide whether to construct a new client.

#![doc(html_favicon_url = "https://surrealdb.s3.amazonaws.com/favicon.png")]
#![doc(html_logo_url = "https://surrealdb.s3.amazonaws.com/icon.png")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(all(target_family = "wasm", feature = "ml"))]
compile_error!("The `ml` feature is not supported on Wasm.");

#[macro_use]
extern crate tracing;

pub mod engine;
#[doc(hidden)]
#[cfg(feature = "protocol-http")]
pub mod headers;
pub mod method;
pub mod opt;

mod conn;
mod notification;

#[doc(hidden)]
/// Channels for receiving a SurrealQL database export
pub mod channel {
	pub use async_channel::{Receiver, Sender, bounded, unbounded};
}

#[cfg(feature = "parse")]
pub mod parse {
	//! Parse SurrealQL text without connecting to a database.

	/// Parses a SurrealQL value from its textual representation.
	pub fn value(input: &str) -> anyhow::Result<crate::types::Value> {
		surrealdb_syn::value(input).map_err(anyhow::Error::new)
	}
}

#[doc(inline)]
pub use method::Stats;
#[doc(inline)]
pub use method::Stream;
#[doc(inline)]
pub use method::query::IndexedResults;
#[doc(inline)]
pub use surrealdb_types as types;

#[doc(inline)]
pub use crate::notification::Notification;

/// A specialized `Result` type
pub type Result<T> = std::result::Result<T, Error>;

/// The retained local transport state for a connected client.
///
/// `Unavailable` is authoritative evidence that this client stopped admitting
/// remote work. It does not claim anything about a remote transaction whose
/// response was lost. Reconnection creates a new client under the host's
/// lifecycle authority; this SDK never retries it on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportState {
	Available,
	Unavailable,
	Released,
}
use std::fmt;
use std::fmt::Debug;
use std::future::IntoFuture;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use async_channel::{Receiver, Sender};
use method::BoxFuture;
use semver::{Version, VersionReq};
use surrealdb_engine_api::SessionId;
#[doc(inline)]
pub use surrealdb_types::Error;
use surrealdb_types::NotAllowedError;
use tokio::sync::watch;
use uuid::Uuid;

use self::conn::Router;
use self::opt::{Endpoint, EndpointKind, WaitFor};

// Channel for waiters
type Waiter = (watch::Sender<Option<WaitFor>>, watch::Receiver<Option<WaitFor>>);

const SUPPORTED_VERSIONS: &str = ">=3.0.0-alpha.1, <4.0.0";

/// Connection trait implemented by supported engines
pub trait Connection: conn::Sealed {}

/// The future returned when creating a new SurrealDB instance
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Connect<C: Connection, Response> {
	surreal: Surreal<C>,
	address: Result<Endpoint>,
	capacity: usize,
	response_type: PhantomData<Response>,
}

impl<C, R> Connect<C, R>
where
	C: Connection,
{
	/// Sets the maximum capacity of the connection
	///
	/// This is used to set bounds of the channels used internally
	/// as well set the capacity of the `HashMap` used for routing
	/// responses in case of the WebSocket client.
	///
	/// Setting this capacity to `0` (the default) means that
	/// unbounded channels will be used. If your queries per second
	/// are so high that the client is running out of memory,
	/// it might be helpful to set this to a number that works best
	/// for you.
	///
	/// This applies to the remote engines. An embedded connection queues
	/// nothing between the caller and the datastore — each request is awaited
	/// where it is made — so there is no channel for it to bound, and it is
	/// ignored.
	///
	/// # Examples
	///
	/// ```no_run
	/// # #[tokio::main]
	/// # async fn main() -> surrealdb::Result<()> {
	/// use surrealdb::engine::remote::ws::Ws;
	/// use surrealdb::Surreal;
	///
	/// let db = Surreal::new::<Ws>("localhost:8000")
	///     .with_capacity(100_000)
	///     .await?;
	/// # Ok(())
	/// # }
	/// ```
	pub const fn with_capacity(mut self, capacity: usize) -> Self {
		self.capacity = capacity;
		self
	}
}

impl<Client> IntoFuture for Connect<Client, Surreal<Client>>
where
	Client: Connection,
{
	type Output = Result<Surreal<Client>>;
	type IntoFuture = BoxFuture<'static, Self::Output>;

	fn into_future(self) -> Self::IntoFuture {
		Box::pin(async move {
			let endpoint = self.address?;
			let endpoint_kind = EndpointKind::from(endpoint.url.scheme());
			let client = Client::connect(endpoint, self.capacity, None).await?;
			if endpoint_kind.is_remote() {
				client.verify_server_version().await?;
			}
			// Both ends of the channel are still alive at this point
			client.inner.waiter.0.send(Some(WaitFor::Connection)).ok();
			Ok(client)
		})
	}
}

impl<Client> IntoFuture for Connect<Client, ()>
where
	Client: Connection,
{
	type Output = Result<()>;
	type IntoFuture = BoxFuture<'static, Self::Output>;

	fn into_future(self) -> Self::IntoFuture {
		Box::pin(async move {
			if !self.surreal.inner.admission_open.load(Ordering::Acquire) {
				return Err(Error::internal("database lifecycle is closed".to_string()));
			}
			// Avoid establishing another connection if already connected
			if self.surreal.inner.router.get().is_some() {
				return Err(Error::connection(
					"Already connected".to_string(),
					Some(crate::types::ConnectionError::AlreadyConnected),
				));
			}
			let endpoint = self.address?;
			let endpoint_kind = EndpointKind::from(endpoint.url.scheme());
			let session_clone = self.surreal.inner.session_clone.clone();
			let client = Client::connect(endpoint, self.capacity, Some(session_clone)).await?;
			if endpoint_kind.is_remote() {
				client.verify_server_version().await?;
			}
			let router = client.inner.router.wait().clone();
			self.surreal.inner.router.set(router).map_err(|_| {
				Error::connection(
					"Already connected".to_string(),
					Some(crate::types::ConnectionError::AlreadyConnected),
				)
			})?;
			// Both ends of the channel are still alive at this point
			self.surreal.inner.waiter.0.send(Some(WaitFor::Connection)).ok();
			Ok(())
		})
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum ExtraFeatures {
	Backup,
	LiveQueries,
}

#[derive(Debug, Clone)]
struct SessionClone {
	sender: Sender<SessionId>,
	#[allow(dead_code)]
	receiver: Receiver<SessionId>,
}

/// A subscription retains its registering session without creating a new one.
/// Ordinary client Clone still creates an independently mutable session.
#[derive(Debug)]
struct SessionLease {
    id: Uuid,
    sender: Sender<SessionId>,
}

impl Drop for SessionLease {
    fn drop(&mut self) { let _ = self.sender.try_send(SessionId::Drop(self.id)); }
}

impl SessionClone {
	fn new() -> Self {
		let (sender, receiver) = async_channel::unbounded();
		Self {
			sender,
			receiver,
		}
	}
}

#[derive(Debug)]
struct Inner {
	router: OnceLock<Router>,
	waiter: Waiter,
	session_clone: SessionClone,
	admission_open: AtomicBool,
}

impl Inner {
	fn router_for_operation(&self) -> Result<&Router> {
		if !self.admission_open.load(Ordering::Acquire) {
			return Err(Error::internal("database lifecycle is closed".to_string()));
		}
		self.router.extract()
	}

	fn clone_session(&self, old: Uuid, new: Uuid) {
		if !self.admission_open.load(Ordering::Acquire) {
			return;
		}
		self.session_clone
			.sender
			.try_send(SessionId::Clone {
				old,
				new,
			})
			.ok();
	}
}

/// A database client instance for embedded or remote databases.
///
/// See [Running SurrealDB embedded in
/// Rust](crate#running-surrealdb-embedded-in-rust) for tips on how to optimize
/// performance for the client when working with embedded instances.
pub struct Surreal<C: Connection> {
	inner: Arc<Inner>,
	session_id: Uuid,
    session_lease: Arc<SessionLease>,
	engine: PhantomData<C>,
}

#[doc(hidden)]
impl<C> From<Arc<Inner>> for Surreal<C>
where
	C: Connection,
{
	fn from(inner: Arc<Inner>) -> Self {
		let session_id = Uuid::new_v4();
		if inner.admission_open.load(Ordering::Acquire) {
			inner.session_clone.sender.try_send(SessionId::Initial(session_id)).ok();
		}
		Surreal {
            session_lease: Arc::new(SessionLease { id: session_id, sender: inner.session_clone.sender.clone() }),
			inner,
			session_id,
			engine: PhantomData,
		}
	}
}

#[doc(hidden)]
impl<C> From<(OnceLock<Router>, Waiter, SessionClone)> for Surreal<C>
where
	C: Connection,
{
	fn from((router, waiter, session_clone): (OnceLock<Router>, Waiter, SessionClone)) -> Self {
		Arc::new(Inner {
			router,
			waiter,
			session_clone,
			admission_open: AtomicBool::new(true),
		})
		.into()
	}
}

#[doc(hidden)]
impl<C> From<(Router, Waiter, SessionClone)> for Surreal<C>
where
	C: Connection,
{
	fn from((router, waiter, session_clone): (Router, Waiter, SessionClone)) -> Self {
		let oncelock = OnceLock::with_value(router);
		(oncelock, waiter, session_clone).into()
	}
}

impl<C> Surreal<C>
where
	C: Connection,
{
    fn same_session(&self) -> Surreal<engine::any::Any> {
        Surreal {
            inner: Arc::clone(&self.inner), session_id: self.session_id,
            session_lease: Arc::clone(&self.session_lease), engine: PhantomData,
        }
    }
	/// Close this client's shared engine scope.
	///
	/// The call is explicit because dropping an SDK handle cannot await engine
	/// finalization. Engine implementations resolve this only after their local
	/// cleanup contract completes; remote engines report release of local state.
	pub async fn shutdown(&self) -> Result<()> {
		self.inner.admission_open.store(false, Ordering::Release);
		let router = self.inner.router.extract()?;
        let engine_result = router.engine.shutdown().await;
        let driver_result = match &router.driver {
            Some(driver) => driver.join().await,
            None => Ok(()),
        };
        let result = match (engine_result, driver_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
            (Err(engine), Err(driver)) => Err(Error::internal(format!("engine closure: {engine}; driver closure: {driver}"))),
        };
		if result.is_ok() {
			router.transport_state.send_replace(TransportState::Released);
		}
		result
	}

	/// Subscribe to retained local transport transitions.
	///
	/// A transition to [`TransportState::Unavailable`] asks the host to decide
	/// whether and when to construct a fresh client. It never certifies a lost
	/// remote write as rolled back.
	pub fn subscribe_transport_state(&self) -> Result<watch::Receiver<TransportState>> {
		Ok(self.inner.router.extract()?.transport_state.subscribe())
	}

	/// Current retained transport generation for explicit reconnect arbitration.
	pub fn transport_generation(&self) -> Result<u64> {
		Ok(self.inner.router.extract()?.transport_generation.load(Ordering::Acquire))
	}

	/// Request one host-authorized reconnect for the current unavailable generation.
	/// The request is rejected once this engine scope is closed or superseded.
	pub async fn reconnect(&self, expected_generation: u64) -> Result<u64> {
		if !self.inner.admission_open.load(Ordering::Acquire) {
			return Err(Error::internal("database lifecycle is closed".to_string()));
		}
		self.inner.router.extract()?.reconnect(expected_generation).await
	}

	async fn check_server_version(&self, version: &Version) -> Result<()> {
		// invalid version requirements should be caught during development
		let req = VersionReq::parse(SUPPORTED_VERSIONS).expect("valid supported versions");
		if !req.matches(version) {
			return Err(Error::internal(format!(
				"server version `{version}` does not match the range supported by the client `{SUPPORTED_VERSIONS}`"
			)));
		}

		Ok(())
	}

	// If the server denies the `version` RPC method via its capabilities, skip
	// the version check rather than failing to connect — the operator has
	// explicitly opted out of exposing the version, and the rest of the
	// connection is still usable.
	async fn verify_server_version(&self) -> Result<()> {
		match self.version().await {
			Ok(mut version) => {
				// we would like to be able to connect to pre-releases too
				version.pre = Default::default();
				self.check_server_version(&version).await
			}
			Err(e) if matches!(e.not_allowed_details(), Some(NotAllowedError::Method { .. })) => {
				debug!(
					"Skipping server version check; the `version` RPC method is denied by server capabilities"
				);
				Ok(())
			}
			Err(e) => Err(e),
		}
	}
}

impl<C> Clone for Surreal<C>
where
	C: Connection,
{
	fn clone(&self) -> Self {
		let session_id = Uuid::new_v4();
		self.inner.clone_session(self.session_id, session_id);
		Self {
			inner: Arc::clone(&self.inner),
			session_id,
            session_lease: Arc::new(SessionLease { id: session_id, sender: self.inner.session_clone.sender.clone() }),
			engine: self.engine,
		}
	}
}

impl<C> Debug for Surreal<C>
where
	C: Connection,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Surreal")
			.field("router", &self.inner.router)
			.field("session_id", &self.session_id)
			.field("engine", &self.engine)
			.finish()
	}
}

trait OnceLockExt {
	fn with_value(value: Router) -> OnceLock<Router> {
		let cell = OnceLock::new();
		match cell.set(value) {
			Ok(()) => cell,
			Err(_) => unreachable!("don't have exclusive access to `cell`"),
		}
	}

	fn extract(&self) -> Result<&Router>;
}

impl OnceLockExt for OnceLock<Router> {
	fn extract(&self) -> Result<&Router> {
		let router = self.get().ok_or_else(|| {
			Error::connection(
				"Connection uninitialised".to_string(),
				Some(crate::types::ConnectionError::Uninitialised),
			)
		})?;
		Ok(router)
	}
}

/// Used by engine code (HTTP, local, any, and WS on wasm) when converting std/io errors.
#[allow(dead_code)]
fn std_error_to_types_error(error: impl std::fmt::Display) -> Error {
	Error::internal(error.to_string())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_supported_versions() {
		let req = VersionReq::parse(SUPPORTED_VERSIONS).expect("valid supported versions");
		assert!(req.matches(&Version::parse("3.0.0-alpha.1").unwrap()));
		assert!(req.matches(&Version::parse("3.0.0-beta.3").unwrap()));
		assert!(req.matches(&Version::parse("3.0.0-rc.2").unwrap()));
		assert!(req.matches(&Version::parse("3.0.0").unwrap()));
		assert!(req.matches(&Version::parse("3.0.1").unwrap()));
		assert!(req.matches(&Version::parse("3.9.0").unwrap()));

		assert!(!req.matches(&Version::parse("2.9.0").unwrap()));
		assert!(!req.matches(&Version::parse("4.0.0").unwrap()));
	}
}

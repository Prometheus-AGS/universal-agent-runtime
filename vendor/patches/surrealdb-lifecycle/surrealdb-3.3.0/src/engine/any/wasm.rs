use std::collections::HashSet;

use tokio::sync::watch;

#[allow(unused_imports, reason = "Used by the DB engines.")]
use crate::ExtraFeatures;
use crate::conn::Router;
#[allow(unused_imports, reason = "Used by the DB engines.")]
use crate::engine;
use crate::engine::any::Any;
use crate::method::BoxFuture;
use crate::opt::{Endpoint, EndpointKind, WaitFor};
use crate::{Error, Result, SessionClone, Surreal, conn};

impl crate::Connection for Any {}
impl conn::Sealed for Any {
	#[allow(
		unused_variables,
		unreachable_code,
		unused_mut,
		private_interfaces,
		reason = "Thse are all used depending on the enabled features."
	)]
	fn connect(
		address: Endpoint,
		capacity: usize,
		session_clone: Option<crate::SessionClone>,
	) -> BoxFuture<'static, Result<Surreal<Self>>> {
		Box::pin(async move {
			let (route_tx, route_rx) = match capacity {
				0 => async_channel::unbounded(),
				capacity => async_channel::bounded(capacity),
			};

			let config = address.config.clone();
			let session_clone = session_clone.unwrap_or_else(SessionClone::new);
			let mut features = HashSet::new();

			let endpoint_kind = EndpointKind::from(address.url.scheme());
			match endpoint_kind {
				EndpointKind::IndxDb => {
					#[cfg(feature = "kv-indxdb")]
					{
						features.insert(ExtraFeatures::LiveQueries);
						let engine = surrealdb_engine_local::connect(
							engine::local::local_config(address),
							session_clone.receiver.clone(),
						)
						.await?;
						let router = Router::from_engine(engine, features, config);
						let waiter = watch::channel(Some(WaitFor::Connection));
						return Ok((router, waiter, session_clone).into());
					}

					#[cfg(not(feature = "kv-indxdb"))]
				return Err(
					Error::internal("Cannot connect to the `indxdb` storage engine as it is not enabled in this build of SurrealDB".to_owned())
				);
				}

				EndpointKind::Memory => {
					#[cfg(feature = "kv-mem")]
					{
						features.insert(ExtraFeatures::LiveQueries);
						let engine = surrealdb_engine_local::connect(
							engine::local::local_config(address),
							session_clone.receiver.clone(),
						)
						.await?;
						let router = Router::from_engine(engine, features, config);
						let waiter = watch::channel(Some(WaitFor::Connection));
						return Ok((router, waiter, session_clone).into());
					}

					#[cfg(not(feature = "kv-mem"))]
				return Err(
					Error::internal("Cannot connect to the `memory` storage engine as it is not enabled in this build of SurrealDB".to_owned())
				);
				}

				EndpointKind::RocksDb => {
					#[cfg(feature = "kv-rocksdb")]
					{
						features.insert(ExtraFeatures::LiveQueries);
						let engine = surrealdb_engine_local::connect(
							engine::local::local_config(address),
							session_clone.receiver.clone(),
						)
						.await?;
						let router = Router::from_engine(engine, features, config);
						let waiter = watch::channel(Some(WaitFor::Connection));
						return Ok((router, waiter, session_clone).into());
					}

					#[cfg(not(feature = "kv-rocksdb"))]
			return Err(Error::internal("Cannot connect to the `rocksdb` storage engine as it is not enabled in this build of SurrealDB".to_owned()));
				}

				EndpointKind::SurrealKv => {
					#[cfg(feature = "kv-surrealkv")]
					{
						features.insert(ExtraFeatures::LiveQueries);
						let engine = surrealdb_engine_local::connect(
							engine::local::local_config(address),
							session_clone.receiver.clone(),
						)
						.await?;
						let router = Router::from_engine(engine, features, config);
						let waiter = watch::channel(Some(WaitFor::Connection));
						return Ok((router, waiter, session_clone).into());
					}

					#[cfg(not(feature = "kv-surrealkv"))]
				return Err(Error::internal(
				"Cannot connect to the `surrealkv` storage engine as it is not enabled in this build of SurrealDB".to_owned(),
			));
				}

				EndpointKind::TiKv => {
					#[cfg(feature = "kv-tikv")]
					{
						features.insert(ExtraFeatures::LiveQueries);
						let engine = surrealdb_engine_local::connect(
							engine::local::local_config(address),
							session_clone.receiver.clone(),
						)
						.await?;
						let router = Router::from_engine(engine, features, config);
						let waiter = watch::channel(Some(WaitFor::Connection));
						return Ok((router, waiter, session_clone).into());
					}

					#[cfg(not(feature = "kv-tikv"))]
				return Err(
					Error::internal("Cannot connect to the `tikv` storage engine as it is not enabled in this build of SurrealDB".to_owned())
				);
				}

				EndpointKind::Http | EndpointKind::Https => {
					#[cfg(feature = "protocol-http")]
					{
						// Reuse the typed connector's retained driver, transport state
						// and reconnect authority rather than spawning an unowned copy.
						return <engine::remote::http::Client as conn::Sealed>::connect(
							address, capacity, Some(session_clone),
						).await.map(|client| client.same_session());
					}

					#[cfg(not(feature = "protocol-http"))]
				return Err(Error::internal(
					"Cannot connect to the `HTTP` remote engine as it is not enabled in this build of SurrealDB".to_owned(),
				));
				}

				EndpointKind::Ws | EndpointKind::Wss => {
					#[cfg(feature = "protocol-ws")]
					{
						// Reuse the typed connector's retained driver, transport state
						// and reconnect authority rather than spawning an unowned copy.
						return <engine::remote::ws::Client as conn::Sealed>::connect(
							address, capacity, Some(session_clone),
						).await.map(|client| client.same_session());
					}

					#[cfg(not(feature = "protocol-ws"))]
				return Err(Error::internal(
					"Cannot connect to the `WebSocket` remote engine as it is not enabled in this build of SurrealDB".to_owned(),
				));
				}

				EndpointKind::Unsupported(v) => {
					return Err(Error::configuration(format!("Unsupported scheme: {v}"), None));
				}
			}

			let waiter = watch::channel(Some(WaitFor::Connection));
			let router = Router::from_route_sender(route_tx, features, config);

			Ok((router, waiter, session_clone).into())
		})
	}
}

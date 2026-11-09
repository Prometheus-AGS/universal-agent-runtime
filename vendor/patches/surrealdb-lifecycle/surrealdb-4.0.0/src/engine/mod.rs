//! Different embedded and remote database engines

pub mod any;
#[cfg(any(feature = "kv-mem", feature = "kv-surrealkv",))]
pub mod local;
#[cfg(any(http_engine, ws_engine, feature = "protocol-grpc"))]
pub mod remote;
/// The embedded engine's background maintenance tasks. Requires the
/// `unstable` feature: these are the engine's own types, outside the SDK's
/// semver guarantees.
#[cfg(all(feature = "unstable", any(feature = "kv-mem", feature = "kv-surrealkv",)))]
#[doc(hidden)]
pub use surrealdb_engine_local::tasks;

#[cfg(any(http_engine, ws_engine))]
pub(crate) use surrealdb_engine_api::{SessionError, session_error_to_error};

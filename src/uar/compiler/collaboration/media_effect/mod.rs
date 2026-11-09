//! Durable media authority and effect claims. The trusted host owns all connector I/O.
mod connector;
#[cfg(feature = "server")]
mod service;

pub use connector::{MediaConnectorResult, MediaDispatchClaim, MediaHostConnector, MockMediaPeer};

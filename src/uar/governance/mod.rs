//! Governance policy engine for enforcing declarative authorization policies.
//!
//! Uses [Cedar](https://www.cedarpolicy.com/) to evaluate policies before
//! tool execution and request handling.

#[cfg(feature = "cedar-governance")]
pub mod engine;
#[cfg(not(feature = "cedar-governance"))]
#[path = "engine_disabled.rs"]
pub mod engine;
#[cfg(feature = "server")]
pub mod middleware;
#[cfg(feature = "cedar-governance")]
pub mod policy;

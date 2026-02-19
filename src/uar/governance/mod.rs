//! Governance policy engine for enforcing declarative authorization policies.
//!
//! Uses [Cedar](https://www.cedarpolicy.com/) to evaluate policies before
//! tool execution and request handling.

pub mod engine;
pub mod middleware;
pub mod policy;

//! Local, metadata-only observation of committed logical-agent transitions.

mod controller;
mod delivery;
mod error;

pub use controller::{ObserverController, ObserverSourceStatus, ObserverStatus};
pub use error::ObserverError;

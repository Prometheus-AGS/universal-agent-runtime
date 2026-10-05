//! Catalog-owned admission bridged to the ordinary actor/thread execution host.
mod controller;
mod epoch;
mod execution;
pub mod host;
mod recovery;

pub use controller::TeamExecutionRuntime;
pub(crate) use epoch::revalidate_member_binding;

mod workflows;

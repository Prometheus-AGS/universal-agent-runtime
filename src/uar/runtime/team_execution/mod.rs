//! Catalog-owned admission bridged to the ordinary actor/thread execution host.
mod controller;
mod epoch;
mod execution;

pub use controller::TeamExecutionRuntime;
pub(crate) use epoch::revalidate_member_binding;

//! Durable, owner-scoped logical agents built on the existing actor root host.

mod controller;
mod epoch;
mod error;
mod lifecycle;
mod pump;
mod reconcile;
mod representation;
mod view;

pub use controller::AgentInstanceController;
pub use epoch::InstanceEpochBinding;
pub use error::AgentInstanceError;
pub use reconcile::EffectDisposition;
pub use view::{
    AgentInstanceCommandView, AgentInstanceEventView, AgentInstanceLimits, AgentInstanceView,
    InstanceActivationProfile, InstanceCommandKind, InstanceCommandStatus, InstanceLifecycle,
    InstanceRecovery,
};

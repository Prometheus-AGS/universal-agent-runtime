//! Local, metadata-only observation of committed logical-agent transitions.

mod controller;
mod channel;
mod channel_authority;
mod delivery;
mod error;

pub use controller::{ObserverController, ObserverSourceStatus, ObserverStatus};
pub use channel::{ChannelCapability, ChannelDeliveryInput, ChannelObserverController, ChannelObserverError, FabricRoutedObserver};
pub use error::ObserverError;

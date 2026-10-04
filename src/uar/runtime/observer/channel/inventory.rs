//! Metadata-only inventory is independent of observation release and execution.

use serde::Serialize;

use super::{ChannelObserverController, ChannelObserverError};
use crate::uar::persistence::channel_observers::ChannelDeliveryMetadata;
use crate::uar::runtime::actor::messages::ActorOwner;

/// Current independent cursor and persisted delivery states, excluding protected projections.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelDeliveryInventory {
    pub subscription_id: String,
    pub cursor: Option<String>,
    pub deliveries: Vec<ChannelDeliveryMetadata>,
}

impl ChannelObserverController {
    /// Inspect the authenticated owner's exact subscription, including paused or revoked state.
    pub async fn delivery_inventory(
        &self,
        owner: &ActorOwner,
        workspace: &str,
        id: &str,
    ) -> Result<ChannelDeliveryInventory, ChannelObserverError> {
        self.require_available()?;
        let subscription = self.load(owner, workspace, id).await?;
        let entries = self
            .store
            .list_channel_inbox_entries(
                &subscription.owner_id,
                workspace,
                &subscription.subscription_id,
            )
            .await
            .map_err(ChannelObserverError::Store)?;
        Ok(ChannelDeliveryInventory {
            subscription_id: subscription.subscription_id,
            cursor: subscription.cursor,
            deliveries: entries
                .into_iter()
                .map(ChannelDeliveryMetadata::from)
                .collect(),
        })
    }
}

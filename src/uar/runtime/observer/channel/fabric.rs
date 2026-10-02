//! Fabric envelope conversion preserves the C08 channel-source contract.

use serde::Deserialize;
use serde_json::Value;

use super::{ChannelDeliveryInput, ChannelObserverError, stable_uuid};
use crate::uar::persistence::channel_observers::{
    CHANNEL_OBSERVER_PROFILE, ChannelProjectionClass, ChannelSourceScope,
};

/// Exact JSON shape carried by Fabric's `frf.routed-observer/1` payload.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FabricRoutedObserver {
    profile: String,
    source: FabricSource,
    selection: FabricSelection,
    delivery: FabricDelivery,
    causal: FabricCausal,
    projection: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FabricSource {
    occurrence_id: String,
    native_message_id: String,
    tenant_id: String,
    provider: String,
    account: String,
    workspace: String,
    room: String,
    thread: Option<String>,
    sender: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FabricSelection {
    route_id: String,
    handler_id: String,
    route_revision: String,
    binding_revision: String,
    policy_revision: String,
    original_principal: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FabricDelivery {
    delivery_id: String,
    subscriber_id: String,
    subscriber_cursor_id: String,
    classification: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FabricCausal {
    root_occurrence_id: String,
    parent_action_id: Option<String>,
    action_id: String,
    depth: u8,
    fanout: u8,
    visited_routes: Vec<String>,
}

impl FabricRoutedObserver {
    pub fn into_channel_input(
        self,
        subscription_id: &str,
    ) -> Result<ChannelDeliveryInput, ChannelObserverError> {
        if self.profile != "frf.routed-observer/1" || self.delivery.subscriber_id != subscription_id
        {
            return Err(ChannelObserverError::Unsupported(
                "fabric_subscriber_profile_mismatch",
            ));
        }
        let (classification, text_projection) = match self.delivery.classification.as_str() {
            "metadata_only" if self.projection.is_object() => {
                (ChannelProjectionClass::MetadataOnly, None)
            }
            "policy_filtered" => {
                let Some(object) = self.projection.as_object() else {
                    return Err(ChannelObserverError::Invalid(
                        "text projection must be an object",
                    ));
                };
                let Some(text) = object
                    .get("text")
                    .and_then(Value::as_str)
                    .filter(|_| object.len() == 1)
                else {
                    return Err(ChannelObserverError::Invalid(
                        "policy-filtered projection must contain only text",
                    ));
                };
                (
                    ChannelProjectionClass::PolicyFiltered,
                    Some(text.to_owned()),
                )
            }
            _ => {
                return Err(ChannelObserverError::Unsupported(
                    "channel_projection_class_unsupported",
                ));
            }
        };
        if self.source.native_message_id.trim().is_empty()
            || self.source.tenant_id.trim().is_empty()
            || self.causal.action_id.trim().is_empty()
            || self
                .causal
                .parent_action_id
                .as_ref()
                .is_some_and(|id| id.trim().is_empty())
            || self.causal.depth > 4
            || self.causal.fanout > 8
        {
            return Err(ChannelObserverError::Invalid(
                "Fabric source or causal identity is invalid",
            ));
        }
        Ok(ChannelDeliveryInput {
            profile: CHANNEL_OBSERVER_PROFILE.to_owned(),
            occurrence_id: self.source.occurrence_id,
            native_message_id: self.source.native_message_id,
            source_tenant_id: self.source.tenant_id,
            subscriber_cursor_id: self.delivery.subscriber_cursor_id,
            delivery_id: self.delivery.delivery_id,
            scope: ChannelSourceScope {
                provider: self.source.provider,
                account: self.source.account,
                workspace: self.source.workspace,
                room: self.source.room,
                thread: self.source.thread,
                sender: self.source.sender.clone(),
            },
            route_id: self.selection.route_id,
            route_revision: self.selection.route_revision,
            binding_revision: self.selection.binding_revision,
            policy_revision: self.selection.policy_revision,
            selected_handler_id: self.selection.handler_id,
            original_principal: self.selection.original_principal,
            original_actor: self.source.sender,
            grant_issuer: String::new(),
            grant_id: String::new(),
            classification,
            text_projection,
            root_occurrence_id: self.causal.root_occurrence_id,
            parent_action_id: self
                .causal
                .parent_action_id
                .as_deref()
                .map(|id| stable_uuid(&[id])),
            action_id: stable_uuid(&[&self.causal.action_id]),
            visited_routes: self.causal.visited_routes,
            remaining_depth: 4 - self.causal.depth,
            remaining_fanout: 8 - self.causal.fanout,
        })
    }
}

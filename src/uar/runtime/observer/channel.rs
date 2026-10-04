//! Explicit channel-source admission. This path never reads C07 instance outboxes.

use std::{collections::HashSet, sync::Arc};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::uar::{
    persistence::{PersistenceLayer, channel_observers::{
        CHANNEL_OBSERVER_PROFILE, ChannelInboxEntry, ChannelInboxStatus,
        ChannelProjectionClass, ChannelSourceScope, ChannelSubscription,
    }},
    runtime::{actor::messages::ActorOwner, instance::{AgentInstanceCommandView, AgentInstanceController}},
};

use super::channel_authority::ChannelGateClient;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelDeliveryInput {
    pub profile: String,
    pub occurrence_id: String,
    pub native_message_id: String,
    pub source_tenant_id: String,
    pub subscriber_cursor_id: String,
    pub delivery_id: String,
    pub scope: ChannelSourceScope,
    pub route_id: String,
    pub route_revision: String,
    pub binding_revision: String,
    pub policy_revision: String,
    pub selected_handler_id: String,
    pub original_principal: String,
    pub original_actor: String,
    pub grant_issuer: String,
    pub grant_id: String,
    #[serde(default)]
    pub classification: ChannelProjectionClass,
    #[serde(default)]
    pub text_projection: Option<String>,
    pub root_occurrence_id: String,
    pub parent_action_id: Option<Uuid>,
    pub action_id: Uuid,
    pub visited_routes: Vec<String>,
    /// Path hops remaining after the current action was admitted. Zero means
    /// this action is terminal; it does not invalidate the current effect.
    pub remaining_depth: u8,
    /// Root fanout remaining after the current action was admitted. Zero means
    /// no sibling may follow; it does not invalidate the current effect.
    pub remaining_fanout: u8,
}

mod fabric;
mod inventory;
pub use fabric::FabricRoutedObserver;
pub use inventory::ChannelDeliveryInventory;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelCapability {
    pub profile: &'static str,
    pub supported: bool,
    pub cross_host_supported: bool,
    pub reason: &'static str,
    pub fabric_profile: &'static str,
    pub gate_contract: &'static str,
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelObserverError {
    #[error("channel-source profile is unsupported: {0}")]
    Unsupported(&'static str),
    #[error("channel-source request is invalid: {0}")]
    Invalid(&'static str),
    #[error("channel-source identity or revision conflicts")]
    Conflict,
    #[error("channel-source resource not found")]
    NotFound,
    #[error("channel-source action was withheld: {0}")]
    Withheld(&'static str),
    #[error("channel-source action has uncertain outcome")]
    Uncertain,
    #[error("channel-source persistence failed")]
    Store(#[source] anyhow::Error),
}

impl ChannelObserverError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unsupported(_) => "channel_profile_unsupported",
            Self::Invalid(_) => "channel_profile_invalid",
            Self::Conflict => "channel_profile_conflict",
            Self::NotFound => "channel_profile_not_found",
            Self::Withheld(_) => "channel_effect_withheld",
            Self::Uncertain => "channel_effect_uncertain",
            Self::Store(_) => "channel_profile_store_error",
        }
    }

    pub fn status(&self) -> u16 {
        match self {
            Self::Unsupported(_) => 503,
            Self::Invalid(_) => 400,
            Self::Conflict | Self::Uncertain => 409,
            Self::NotFound => 404,
            Self::Withheld(_) => 403,
            Self::Store(_) => 500,
        }
    }
}

pub struct ChannelObserverController {
    instances: Arc<AgentInstanceController>,
    store: Arc<dyn PersistenceLayer>,
    gate: Option<ChannelGateClient>,
    admission: Mutex<()>,
}

impl ChannelObserverController {
    pub fn new(instances: Arc<AgentInstanceController>, store: Arc<dyn PersistenceLayer>) -> Arc<Self> {
        Arc::new(Self {
            instances, store, gate: ChannelGateClient::from_process_environment(),
            admission: Mutex::new(()),
        })
    }

    pub fn capabilities(&self) -> ChannelCapability {
        let store = self.store.supports_channel_observers();
        let gate = self.gate.is_some();
        ChannelCapability {
            profile: CHANNEL_OBSERVER_PROFILE,
            supported: store && gate,
            // An authenticated ingress is not a Fabric subscription/replay worker.
            cross_host_supported: false,
            reason: if !store { self.store.channel_observer_unavailable_reason() }
                else if !gate { "authenticated_gate_binding_missing" }
                else { "local_ingress_configured_fabric_consumer_not_wired" },
            fabric_profile: "frf.routed-observer/1",
            gate_contract: "afc.channel-authority/1",
        }
    }

    fn require_available(&self) -> Result<&ChannelGateClient, ChannelObserverError> {
        if !self.store.supports_channel_observers() {
            return Err(ChannelObserverError::Unsupported(self.store.channel_observer_unavailable_reason()));
        }
        self.gate.as_ref().ok_or(ChannelObserverError::Unsupported("authenticated_gate_binding_missing"))
    }

    pub async fn create_subscription(
        &self, owner: &ActorOwner, workspace: &str, observer_instance_id: &str,
        source: ChannelSourceScope, grant_issuer: &str, grant_id: &str,
    ) -> Result<ChannelSubscription, ChannelObserverError> {
        self.require_available()?;
        if !source.valid() || source.workspace != workspace || observer_instance_id.trim().is_empty()
            || grant_issuer.trim().is_empty() || grant_id.trim().is_empty() {
            return Err(ChannelObserverError::Invalid("subscription scope or grant is missing"));
        }
        let instance = self.instances.load(owner, workspace, observer_instance_id).await
            .map_err(|_| ChannelObserverError::NotFound)?;
        self.instances.bound(&instance).await.map_err(|_| ChannelObserverError::Conflict)?;
        let now = Utc::now();
        let record = ChannelSubscription {
            profile: CHANNEL_OBSERVER_PROFILE.to_owned(), subscription_id: Uuid::new_v4().to_string(),
            owner_id: owner.presentation_owner_key(), workspace_id: workspace.to_owned(),
            observer_instance_id: observer_instance_id.to_owned(), source,
            grant_issuer: grant_issuer.to_owned(), grant_id: grant_id.to_owned(), revision: 0,
            cursor: None, paused: false, revoked: false, created_at: now, updated_at: now,
        };
        self.store.create_channel_subscription(&record).await.map_err(ChannelObserverError::Store)
    }

    pub async fn list(&self, owner: &ActorOwner, workspace: &str) -> Result<Vec<ChannelSubscription>, ChannelObserverError> {
        self.require_available()?;
        self.store.list_channel_subscriptions(&owner.presentation_owner_key(), workspace).await.map_err(ChannelObserverError::Store)
    }

    pub async fn set_status(&self, owner: &ActorOwner, workspace: &str, id: &str,
        expected_revision: u64, paused: Option<bool>, revoke: bool,
    ) -> Result<ChannelSubscription, ChannelObserverError> {
        self.require_available()?;
        let _lock = self.admission.lock().await;
        let current = self.load(owner, workspace, id).await?;
        if current.revision != expected_revision || current.revoked { return Err(ChannelObserverError::Conflict); }
        let mut next = current.clone();
        next.paused = paused.unwrap_or(next.paused);
        next.revoked = revoke;
        next.revision = next.revision.checked_add(1).ok_or(ChannelObserverError::Conflict)?;
        next.updated_at = Utc::now();
        if !self.store.compare_and_swap_channel_subscription(&current, &next).await.map_err(ChannelObserverError::Store)? {
            return Err(ChannelObserverError::Conflict);
        }
        Ok(next)
    }

    pub async fn deliver(&self, owner: &ActorOwner, workspace: &str, subscription_id: &str,
        mut input: ChannelDeliveryInput,
    ) -> Result<ChannelInboxEntry, ChannelObserverError> {
        let gate = self.require_available()?;
        let _lock = self.admission.lock().await;
        let subscription = self.load(owner, workspace, subscription_id).await?;
        input.grant_issuer.clone_from(&subscription.grant_issuer);
        input.grant_id.clone_from(&subscription.grant_id);
        validate_input(&input, workspace)?;
        if owner.tenant_id().is_some_and(|tenant| tenant != input.source_tenant_id) {
            return Err(ChannelObserverError::Withheld("source_tenant_mismatch"));
        }
        if subscription.paused || subscription.revoked || subscription.source != input.scope
            || input.original_principal.trim().is_empty() {
            return Err(ChannelObserverError::Withheld("subscription_not_current"));
        }
        let instance = self.instances.load(owner, workspace, &subscription.observer_instance_id).await
            .map_err(|_| ChannelObserverError::NotFound)?;
        self.instances.bound(&instance).await.map_err(|_| ChannelObserverError::Withheld("observer_binding_changed"))?;
        let payload_sha256 = match input.classification {
            ChannelProjectionClass::PolicyFiltered => sha256_hex(input.text_projection.as_deref().unwrap_or_default().as_bytes()),
            ChannelProjectionClass::MetadataOnly => sha256_hex(&serde_json::to_vec(&(&input.occurrence_id, &input.native_message_id, &input.scope, &input.route_id, &input.subscriber_cursor_id)).map_err(|_| ChannelObserverError::Invalid("source encoding"))?),
        };
        let now = Utc::now();
        let pending = ChannelInboxEntry {
            subscription_id: subscription_id.to_owned(), owner_id: owner.presentation_owner_key(),
            workspace_id: workspace.to_owned(), occurrence_id: input.occurrence_id.clone(),
            native_message_id: input.native_message_id.clone(), source_tenant_id: input.source_tenant_id.clone(),
            subscriber_cursor_id: input.subscriber_cursor_id.clone(), delivery_id: input.delivery_id.clone(),
            route_id: input.route_id.clone(), route_revision: input.route_revision.clone(),
            binding_revision: input.binding_revision.clone(), policy_revision: input.policy_revision.clone(),
            original_actor: input.original_actor.clone(), original_principal: input.original_principal.clone(), payload_sha256: payload_sha256.clone(),
            classification: input.classification, text_projection: None,
            status: ChannelInboxStatus::PendingAuthority, gate_receipt_id: None,
            admitted_at: now, updated_at: now,
        };
        let existing = self.store.create_channel_inbox_entry(&pending).await.map_err(ChannelObserverError::Store)?;
        if !same_delivery(&pending, &existing) { return Err(ChannelObserverError::Conflict); }
        if existing.status != ChannelInboxStatus::PendingAuthority { return Ok(existing); }
        let request = gate_request(&input, &subscription.observer_instance_id,
            &input.selected_handler_id, "recipient_delivery", &payload_sha256);
        let receipt = gate.release(&request).await;
        let mut next = existing.clone();
        next.updated_at = Utc::now();
        match receipt {
            Ok(receipt) => {
                next.status = ChannelInboxStatus::Admitted;
                next.gate_receipt_id = Some(receipt);
                next.text_projection.clone_from(&input.text_projection);
            }
            Err("channel_effect_uncertain") => { next.status = ChannelInboxStatus::Uncertain; }
            Err("channel_gate_unreachable" | "channel_gate_response_invalid" | "channel_gate_configuration_invalid") => {
                return Err(ChannelObserverError::Unsupported("channel_gate_evaluation_unavailable"));
            }
            Err(_) => { next.status = ChannelInboxStatus::Withheld; }
        }
        if !self.store.compare_and_swap_channel_inbox_entry(&existing, &next).await.map_err(ChannelObserverError::Store)? {
            return Err(ChannelObserverError::Conflict);
        }
        Ok(next)
    }

    pub async fn acknowledge(&self, owner: &ActorOwner, workspace: &str, subscription_id: &str,
        delivery_id: &str, expected_revision: u64,
    ) -> Result<ChannelInboxEntry, ChannelObserverError> {
        self.require_available()?;
        let _lock = self.admission.lock().await;
        let current = self.load(owner, workspace, subscription_id).await?;
        if current.revision != expected_revision || current.paused || current.revoked { return Err(ChannelObserverError::Conflict); }
        let entry = self.store.load_channel_inbox_entry(&current.owner_id, workspace, subscription_id, delivery_id).await
            .map_err(ChannelObserverError::Store)?.ok_or(ChannelObserverError::NotFound)?;
        if entry.status != ChannelInboxStatus::Admitted && entry.status != ChannelInboxStatus::Acknowledged {
            return Err(ChannelObserverError::Withheld("delivery_not_admitted"));
        }
        let mut acknowledged = entry.clone();
        acknowledged.status = ChannelInboxStatus::Acknowledged;
        acknowledged.updated_at = Utc::now();
        if entry.status == ChannelInboxStatus::Admitted
            && !self.store.compare_and_swap_channel_inbox_entry(&entry, &acknowledged).await.map_err(ChannelObserverError::Store)? {
                return Err(ChannelObserverError::Conflict);
        }
        let mut next = current.clone();
        next.cursor = Some(entry.subscriber_cursor_id.clone());
        next.revision = next.revision.checked_add(1).ok_or(ChannelObserverError::Conflict)?;
        next.updated_at = Utc::now();
        if !self.store.compare_and_swap_channel_subscription(&current, &next).await.map_err(ChannelObserverError::Store)? {
            return Err(ChannelObserverError::Conflict);
        }
        Ok(acknowledged)
    }

    pub async fn execute_selected_handler(&self, owner: &ActorOwner, workspace: &str,
        input: ChannelDeliveryInput, prompt: &str,
    ) -> Result<Value, ChannelObserverError> {
        let gate = self.require_available()?;
        validate_input(&input, workspace)?;
        if owner.tenant_id().is_some_and(|tenant| tenant != input.source_tenant_id) {
            return Err(ChannelObserverError::Withheld("source_tenant_mismatch"));
        }
        if prompt.trim().is_empty() { return Err(ChannelObserverError::Invalid("handler prompt is empty")); }
        if input.classification == ChannelProjectionClass::PolicyFiltered
            && input.text_projection.as_deref() != Some(prompt) {
            return Err(ChannelObserverError::Invalid("handler prompt differs from authorized text projection"));
        }
        let _lock = self.admission.lock().await;
        let instance = self.instances.load(owner, workspace, &input.selected_handler_id).await
            .map_err(|_| ChannelObserverError::NotFound)?;
        self.instances.bound(&instance).await.map_err(|_| ChannelObserverError::Withheld("handler_binding_changed"))?;
        let digest = sha256_hex(prompt.as_bytes());
        let receipt_id = stable_uuid(&[&input.occurrence_id, &input.route_revision, &input.selected_handler_id, "handler"]);
        let now = Utc::now();
        let pending = ChannelInboxEntry {
            subscription_id: format!("handler:{}", input.selected_handler_id),
            owner_id: owner.presentation_owner_key(), workspace_id: workspace.to_owned(),
            occurrence_id: input.occurrence_id.clone(), native_message_id: input.native_message_id.clone(),
            source_tenant_id: input.source_tenant_id.clone(), subscriber_cursor_id: input.subscriber_cursor_id.clone(),
            delivery_id: receipt_id.to_string(), route_id: input.route_id.clone(),
            route_revision: input.route_revision.clone(), binding_revision: input.binding_revision.clone(),
            policy_revision: input.policy_revision.clone(), original_actor: input.original_actor.clone(),
            original_principal: input.original_principal.clone(),
            payload_sha256: digest.clone(), classification: input.classification,
            text_projection: None, status: ChannelInboxStatus::PendingAuthority,
            gate_receipt_id: None, admitted_at: now, updated_at: now,
        };
        let existing = self.store.create_channel_inbox_entry(&pending).await.map_err(ChannelObserverError::Store)?;
        if !same_delivery(&pending, &existing) { return Err(ChannelObserverError::Conflict); }
        let command_id = format!("channel:{}", receipt_id);
        if existing.status == ChannelInboxStatus::Admitted {
            let current = self.instances.load(owner, workspace, &input.selected_handler_id).await
                .map_err(|_| ChannelObserverError::Uncertain)?;
            let command = current.inbox.iter().find(|command| command.command_id == command_id)
                .ok_or(ChannelObserverError::Uncertain)?;
            return serde_json::to_value(AgentInstanceCommandView::from(command))
                .map_err(|_| ChannelObserverError::Invalid("command encoding"));
        }
        if existing.status != ChannelInboxStatus::PendingAuthority { return Err(ChannelObserverError::Uncertain); }
        let request = gate_request(&input, &input.selected_handler_id,
            &input.selected_handler_id, "handler_execution", &digest);
        let receipt = gate.release(&request).await;
        let mut next = existing.clone();
        next.updated_at = Utc::now();
        match receipt {
            Ok(receipt) => { next.status = ChannelInboxStatus::Admitted; next.gate_receipt_id = Some(receipt); }
            Err("channel_effect_uncertain") => { next.status = ChannelInboxStatus::Uncertain; }
            Err("channel_gate_unreachable" | "channel_gate_response_invalid" | "channel_gate_configuration_invalid") => {
                return Err(ChannelObserverError::Unsupported("channel_gate_evaluation_unavailable"));
            }
            Err(_) => { next.status = ChannelInboxStatus::Withheld; }
        }
        if !self.store.compare_and_swap_channel_inbox_entry(&existing, &next).await.map_err(ChannelObserverError::Store)? {
            return Err(ChannelObserverError::Conflict);
        }
        if next.status != ChannelInboxStatus::Admitted { return Err(ChannelObserverError::Withheld("gate_rejected_handler_execution")); }
        let view = self.instances.submit(owner, workspace, &input.selected_handler_id, &command_id, prompt).await
            .map_err(|_| ChannelObserverError::Uncertain)?;
        serde_json::to_value(view).map_err(|_| ChannelObserverError::Invalid("command encoding"))
    }

    async fn load(&self, owner: &ActorOwner, workspace: &str, id: &str) -> Result<ChannelSubscription, ChannelObserverError> {
        self.store.load_channel_subscription(&owner.presentation_owner_key(), workspace, id).await
            .map_err(ChannelObserverError::Store)?.ok_or(ChannelObserverError::NotFound)
    }
}

fn validate_input(input: &ChannelDeliveryInput, workspace: &str) -> Result<(), ChannelObserverError> {
    if input.profile != CHANNEL_OBSERVER_PROFILE { return Err(ChannelObserverError::Unsupported("channel_profile_version_mismatch")); }
    if !input.scope.valid() || input.scope.workspace != workspace
        || [&input.occurrence_id, &input.native_message_id, &input.source_tenant_id, &input.delivery_id, &input.subscriber_cursor_id, &input.route_id, &input.route_revision,
            &input.binding_revision, &input.policy_revision, &input.selected_handler_id,
            &input.original_principal, &input.original_actor, &input.grant_issuer, &input.grant_id, &input.root_occurrence_id]
            .into_iter().any(|value| value.trim().is_empty())
        || input.action_id.is_nil() || input.remaining_depth > 4 || input.remaining_fanout > 8
        || match input.classification {
            ChannelProjectionClass::MetadataOnly => input.text_projection.is_some(),
            ChannelProjectionClass::PolicyFiltered => input.text_projection.as_ref()
                .is_none_or(|text| text.as_bytes().len() > crate::uar::persistence::agent_instances::MAX_INSTANCE_COMMAND_BYTES - 1024),
        }
        || input.visited_routes.iter().any(|route| route.trim().is_empty() || route == &input.route_id)
        || input.visited_routes.iter().collect::<HashSet<_>>().len() != input.visited_routes.len() {
        return Err(ChannelObserverError::Invalid("source, route, grant, or causal budget is invalid"));
    }
    Ok(())
}

fn gate_request(input: &ChannelDeliveryInput, recipient: &str,
    handler: &str, action: &str, payload_sha256: &str) -> Value {
    json!({
        "protocol": "afc.channel-effect/1",
        "effect_id": stable_uuid(&[&input.occurrence_id, &input.delivery_id, action]).to_string(),
        "occurrence_id": input.occurrence_id,
        "action": action,
        "scope": {"provider": input.scope.provider, "account": input.scope.account,
            "workspace": input.scope.workspace, "room": input.scope.room,
            "thread": input.scope.thread, "sender": input.scope.sender},
        "recipient": recipient,
        "handler": handler,
        "route_revision": input.route_revision,
        "payload": {"algorithm": "sha256", "sha256": payload_sha256},
        "classification": input.classification.as_str(),
        "causality": {"root_occurrence_id": input.root_occurrence_id,
            "parent_action_id": input.parent_action_id,
            "action_id": stable_uuid(&[&input.action_id.to_string(), &input.delivery_id, action]),
            "route_identity": input.route_id, "visited_routes": input.visited_routes,
            "remaining_depth": input.remaining_depth, "remaining_fanout": input.remaining_fanout},
        "grant_issuer": input.grant_issuer, "grant_id": input.grant_id,
    })
}

fn stable_uuid(parts: &[&str]) -> Uuid {
    let mut hash = Sha256::new();
    for part in parts { hash.update((part.len() as u64).to_be_bytes()); hash.update(part.as_bytes()); }
    let digest = hash.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Uuid::from_bytes(bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

fn same_delivery(expected: &ChannelInboxEntry, actual: &ChannelInboxEntry) -> bool {
    expected.owner_id == actual.owner_id && expected.workspace_id == actual.workspace_id
        && expected.subscription_id == actual.subscription_id && expected.occurrence_id == actual.occurrence_id
        && expected.native_message_id == actual.native_message_id && expected.source_tenant_id == actual.source_tenant_id
        && expected.subscriber_cursor_id == actual.subscriber_cursor_id && expected.delivery_id == actual.delivery_id
        && expected.route_id == actual.route_id && expected.route_revision == actual.route_revision
        && expected.binding_revision == actual.binding_revision && expected.policy_revision == actual.policy_revision
        && expected.original_actor == actual.original_actor && expected.original_principal == actual.original_principal
        && expected.payload_sha256 == actual.payload_sha256 && expected.classification == actual.classification
}

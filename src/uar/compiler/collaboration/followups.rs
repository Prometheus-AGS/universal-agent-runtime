//! Rule decisions and owner inbox delivery share one catalog transaction.
use std::collections::BTreeMap;
use chrono::Utc;
use crate::uar::{domain::{collaboration::{CollaborationCatalogState, RepresentationGrantStatus},
    followups::{FollowupRule, FollowupInboxItem}}, runtime::followups::FollowupError};
use super::service::{CollaborationCatalogService, MAX_CAS_ATTEMPTS};

fn key(owner: &str, workspace: &str, id: &str) -> String {
    format!("{}:{owner}{}:{workspace}{id}", owner.len(), workspace.len())
}

fn current_authority(state: &CollaborationCatalogState, rule: &FollowupRule) -> bool {
    let now = Utc::now();
    [&rule.source_authority, &rule.destination_authority].iter().all(|a| {
        let key = super::bindings::binding_key(&rule.catalog_owner_id, &rule.workspace_id, &a.binding.id);
        state.bindings.get(&key).is_some_and(|binding| {
            binding.revision == a.binding.revision
                && binding.document.get("contentDigest").and_then(serde_json::Value::as_str) == Some(a.binding.digest.as_str())
                && binding.effective_binding_receipt.as_ref().is_some_and(|receipt| {
                    receipt.admitted && receipt.binding_ref == a.binding
                        && binding.document.get("policyRevision").and_then(serde_json::Value::as_str) == Some(receipt.policy_revision.as_str())
                        && super::grants::validate_binding_grants(&rule.catalog_owner_id, &rule.workspace_id, &binding.document, state)
                            .is_ok_and(|grants| grants == receipt.representation_grants)
                })
        })
            && a.representation_grants.iter().all(|reference| {
                let key = format!("{}\u{1f}{}\u{1f}{}", rule.catalog_owner_id, rule.workspace_id, reference.grant_id);
                state.representation_grants.get(&key).is_some_and(|grant| grant.revision == reference.revision
                    && grant.constraint_digest == reference.constraint_digest && grant.status == RepresentationGrantStatus::Active
                    && grant.revocation.is_none() && grant.not_before <= now && grant.expires_at > now)
            })
    })
}

impl CollaborationCatalogService {
    pub(crate) fn require_followup_store(&self) -> Result<(), FollowupError> {
        if self.storage_descriptor().backend == "memory" { return Err(FollowupError::Unsupported); }
        Ok(())
    }
    pub(crate) async fn create_followup_rule(&self, rule: FollowupRule) -> Result<FollowupRule, FollowupError> {
        self.require_followup_store()?;
        let key = key(&rule.owner_key, &rule.workspace_id, &rule.id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let mut next = self.load_state().await.map_err(|_| FollowupError::Storage)?;
            if !current_authority(&next, &rule) { return Err(FollowupError::Denied); }
            if next.followup_rules.contains_key(&key) { return Err(FollowupError::Conflict("rule already exists")); }
            let generation = next.generation; next.generation += 1;
            next.followup_rules.insert(key.clone(), rule.clone());
            if self.cas(generation, &next).await.map_err(|_| FollowupError::Storage)? { return Ok(rule); }
        }
        Err(FollowupError::Conflict("catalog changed during rule creation"))
    }
    pub(crate) async fn followup_rules(&self) -> Result<Vec<FollowupRule>, FollowupError> {
        self.require_followup_store()?;
        Ok(self.load_state().await.map_err(|_| FollowupError::Storage)?.followup_rules.into_values().collect())
    }
    pub(crate) async fn followup_rule(&self, owner: &str, workspace: &str, id: &str) -> Result<FollowupRule, FollowupError> {
        self.require_followup_store()?;
        self.load_state().await.map_err(|_| FollowupError::Storage)?.followup_rules.get(&key(owner, workspace, id)).cloned().ok_or(FollowupError::NotFound)
    }
    pub(crate) async fn followup_items(&self, owner: &str, workspace: &str, conversation: &str) -> Result<Vec<FollowupInboxItem>, FollowupError> {
        self.require_followup_store()?;
        Ok(self.load_state().await.map_err(|_| FollowupError::Storage)?.followup_inbox.into_values()
            .filter(|i| i.owner_key == owner && i.workspace_id == workspace && i.conversation_id == conversation).collect())
    }
    pub(crate) async fn change_followup<F>(&self, owner: &str, workspace: &str, id: &str,
        mut change: F) -> Result<FollowupRule, FollowupError>
    where F: FnMut(&mut FollowupRule, &mut BTreeMap<String, FollowupInboxItem>, bool) -> Result<bool, FollowupError> {
        self.require_followup_store()?;
        let key = key(owner, workspace, id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let mut next = self.load_state().await.map_err(|_| FollowupError::Storage)?;
            let rule = next.followup_rules.get(&key).ok_or(FollowupError::NotFound)?;
            let authority = current_authority(&next, rule);
            let previous_revision = rule.revision;
            let rule = next.followup_rules.get_mut(&key).ok_or(FollowupError::NotFound)?;
            if !change(rule, &mut next.followup_inbox, authority)? { return Ok(rule.clone()); }
            rule.updated_at = Utc::now();
            let result = rule.clone(); let generation = next.generation; next.generation += 1;
            if result.revision != previous_revision && result.enabled && !result.revoked && !current_authority(&next, &result) {
                return Err(FollowupError::Denied);
            }
            if self.cas(generation, &next).await.map_err(|_| FollowupError::Storage)? { return Ok(result); }
        }
        Err(FollowupError::Conflict("catalog changed during follow-up transaction"))
    }
}

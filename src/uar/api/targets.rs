//! One target vocabulary for discovery and execution. Private catalog bindings
//! are always selected under the authenticated owner and explicit workspace.
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use crate::{llm::ProviderRegistry, uar::{
    compiler::collaboration::{BoundAgentRun, CollaborationCatalogService},
    domain::policy::ModelRoute,
    security::{claims::UserContext, credentials::ProviderService},
    settings::manager::SettingsManager,
}};

pub const CATALOG_CAPABILITY: &str = "uar_target_catalog_v1";
pub const OVERRIDE_CAPABILITY: &str = "uar_target_model_override_v1";
const AGENT_PREFIX: &str = "uar:agent:";

#[derive(Clone, Copy)]
pub(crate) struct ModelSources<'a> {
    pub registry: Option<&'a ProviderRegistry>,
    pub settings: Option<&'a SettingsManager>,
    pub credentials: Option<&'a ProviderService>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TargetReceipt {
    pub version: u32,
    pub id: String,
    pub kind: String,
    pub agent_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub verified_at: chrono::DateTime<chrono::Utc>,
    pub configuration_digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<crate::uar::domain::collaboration::ImmutableDefinitionRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding: Option<crate::uar::domain::collaboration::PrivateRevisionRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
}

pub fn agent_target_id(binding_id: &str) -> String {
    let encoded = binding_id.bytes().map(|byte| {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            char::from(byte).to_string()
        } else { format!("%{byte:02X}") }
    }).collect::<String>();
    format!("{AGENT_PREFIX}{encoded}")
}

pub fn target_binding_id(target: &str) -> Result<Option<String>, String> {
    let Some(encoded) = target.strip_prefix(AGENT_PREFIX) else { return Ok(None); };
    let mut bytes = Vec::new(); let raw = encoded.as_bytes(); let mut index = 0;
    while index < raw.len() {
        if raw[index] == b'%' {
            let part = encoded.get(index + 1..index + 3).ok_or("Invalid encoded agent target")?;
            bytes.push(u8::from_str_radix(part, 16).map_err(|_| "Invalid encoded agent target")?);
            index += 3;
        } else { bytes.push(raw[index]); index += 1; }
    }
    let id = String::from_utf8(bytes).map_err(|_| "Invalid encoded agent target")?;
    if id.is_empty() || agent_target_id(&id) != target { return Err("Noncanonical agent target".into()); }
    Ok(Some(id))
}

pub(crate) fn workspace(headers: &HeaderMap) -> Option<&str> {
    headers.get("x-uar-workspace-id").and_then(|v| v.to_str().ok()).filter(|s| !s.trim().is_empty())
}

pub(crate) async fn resolve_bound(catalog: &CollaborationCatalogService, user: &UserContext, workspace_id: &str, binding_id: &str) -> Result<BoundAgentRun, String> {
    let owner = super::user_settings::principal_storage_key(user).ok_or("Verified principal required")?;
    catalog.resolve_bound_agent_run(&owner, workspace_id, binding_id).await.map_err(|error| error.to_string())
}

impl ModelSources<'_> {
    pub(crate) async fn verified_model(&self, selector: &str) -> Result<(ModelRoute, super::model_verification::ModelVerification), String> {
        let registry = self.registry.ok_or("Model configuration unavailable")?;
        let (provider_id, model_id) = selector.split_once('/').ok_or("Use an exact configured provider/model identity")?;
        let provider = registry.get(provider_id).await.ok_or("Provider is not configured")?;
        let proof = super::model_verification::current(&provider, model_id, self.settings, self.credentials).await
            .ok_or("Selected model is disabled, unverified, or its configuration changed; verify it in UAR settings")?;
        Ok((ModelRoute { provider_id:provider_id.to_owned(), model_id:model_id.to_owned() }, proof))
    }

    pub(crate) async fn allowed_overrides(&self, bound: &BoundAgentRun) -> Vec<String> {
        let mut allowed = Vec::new();
        for model in &bound.effective_binding_receipt.resolved_models {
            if let (Some(provider),Some(id)) = (model.get("providerId").and_then(Value::as_str),model.get("modelId").and_then(Value::as_str)) {
                let selector = format!("{provider}/{id}");
                if !allowed.contains(&selector) && self.verified_model(&selector).await.is_ok() { allowed.push(selector); }
            }
        }
        allowed
    }

    pub(crate) async fn admit_bound(&self, bound: &BoundAgentRun, model_override: Option<&str>, workspace_id: &str) -> Result<TargetReceipt, String> {
        let primary = bound.effective_binding_receipt.resolved_models.iter()
            .find(|model| model.get("role").and_then(Value::as_str) == Some("primary"))
            .or_else(|| bound.effective_binding_receipt.resolved_models.first());
        let default = primary.and_then(|model| Some(format!("{}/{}", model.get("providerId")?.as_str()?, model.get("modelId")?.as_str()?)))
            .unwrap_or_else(|| format!("{}/{}", bound.artifact.policy.provider.default.provider, bound.artifact.policy.provider.default.model));
        if let Some(selected) = model_override {
            let admitted = bound.effective_binding_receipt.resolved_models.iter().any(|model| {
                model.get("providerId").and_then(Value::as_str).zip(model.get("modelId").and_then(Value::as_str))
                    .is_some_and(|(provider,id)|format!("{provider}/{id}")==selected)
            });
            if !admitted { return Err("Model override is outside the admitted agent binding; update its model bindings in UAR settings".into()); }
        }
        let (route, proof) = self.verified_model(model_override.unwrap_or(&default)).await?;
        // A model override changes inference only. Existing binding revalidation
        // retains all resource grants, budgets and effect authority.
        Ok(TargetReceipt {
            version:1, id:agent_target_id(&bound.binding.id), kind:"agent".into(), agent_id:bound.artifact.id.clone(),
            provider_id:route.provider_id, model_id:route.model_id, verified_at:proof.verified_at,
            configuration_digest:proof.configuration_digest, definition:Some(bound.binding.package.clone()),
            binding:Some(bound.effective_binding_receipt.binding_ref.clone()), workspace_id:Some(workspace_id.to_owned()),
            instance_id:bound.effective_binding_receipt.service_binding.as_ref().map(|binding|binding.instance_id.clone()),
        })
    }
}

pub(crate) async fn catalog(state: &crate::AppState, user: &UserContext, headers: &HeaderMap) -> Result<Vec<Value>, String> {
    let sources = state.run_manager.target_sources();
    let mut rows = Vec::new(); let mut default_bindings = Vec::new();
    if let Some(workspace_id) = workspace(headers) {
        let owner = super::user_settings::principal_storage_key(user).ok_or("Verified principal required")?;
        let bindings = state.collaboration_catalog.list_bindings(&owner, workspace_id).await.map_err(|e|e.to_string())?;
        for binding in bindings {
            let bound = resolve_bound(&state.collaboration_catalog, user, workspace_id, &binding.id).await;
            let (agent_id, readiness) = match &bound {
                Ok(bound) => {
                    if bound.artifact.id == "default-agent" { default_bindings.push(bound.clone()); }
                    (Some(bound.artifact.id.clone()), sources.admit_bound(bound, None, workspace_id).await)
                },
                Err(error) => (None, Err(error.clone())),
            };
            // Team bindings belong to the team API, not ordinary agent chat.
            if bound.as_ref().err().is_some_and(|error| error.contains("ordinary") || error.contains("AgentDefinition")) { continue; }
            let mut metadata = json!({"version":1,"kind":"agent","display_name":bound.as_ref().map(|bound|bound.artifact.metadata.title.as_str()).unwrap_or(binding.id.as_str()),
                "agent_id":agent_id,"deployment_binding_id":binding.id,"definition":binding.package,
                "binding":binding.effective_binding_receipt.as_ref().map(|receipt| &receipt.binding_ref),
                "available":readiness.is_ok(),"delegation_available":readiness.is_ok()});
            if let Ok(bound) = &bound { metadata["allowed_model_overrides"] = json!(sources.allowed_overrides(bound).await); }
            match readiness {
                Ok(receipt) => { metadata["provider_id"]=json!(receipt.provider_id);metadata["model_id"]=json!(receipt.model_id); metadata["verified_at"]=json!(receipt.verified_at); },
                Err(reason) => metadata["reason"]=json!(reason),
            }
            rows.push(json!({"id":agent_target_id(&binding.id),"object":"model","created":binding.updated_at.timestamp(),"owned_by":"uar","uar":metadata}));
        }
    }
    let default_bound = if default_bindings.len() == 1 { default_bindings.pop() } else { None };
    for provider in state.provider_registry.list().await {
        for model in &provider.models {
            let Some(proof) = super::model_verification::current(&provider, &model.id, sources.settings, sources.credentials).await else { continue; };
            let mut metadata = json!({"version":1,"kind":"model","display_name":model.display_name.as_deref().unwrap_or(&model.id),
                "provider_id":provider.id,"model_id":model.id,"agent_id":"default-agent","available":true,
                "verified_at":proof.verified_at,"delegation_available":default_bound.is_some()});
            if let Some(bound) = &default_bound {
                if let Err(reason) = sources.admit_bound(bound,Some(&format!("{}/{}",provider.id,model.id)),workspace(headers).unwrap_or_default()).await {
                    metadata["delegation_available"]=json!(false);metadata["reason"]=json!(reason);
                }
                metadata["deployment_binding_id"]=json!(bound.binding.id); metadata["definition"]=json!(bound.binding.package);
                metadata["binding"]=json!(bound.effective_binding_receipt.binding_ref);
            } else { metadata["reason"]=json!("default_agent_not_installed_or_ambiguous"); }
            rows.push(json!({"id":format!("{}/{}",provider.id,model.id),"object":"model","created":proof.verified_at.timestamp(),"owned_by":provider.id,"uar":metadata,
                "capabilities":{"tool_call":model.supports_tools,"streaming":model.supports_streaming,"reasoning":model.supports_reasoning,"structured_output":model.supports_structured_output}}));
        }
    }
    Ok(rows)
}

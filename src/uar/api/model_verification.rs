//! Explicit model-operation receipts. Discovery never performs inference.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::llm::registry::ProviderConfig;
use crate::uar::{settings::manager::SettingsManager, security::credentials::{CredentialScope, ProviderService}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVerification {
    pub verified_at: DateTime<Utc>,
    pub configuration_digest: String,
}

pub fn key(provider: &str, model: &str) -> String {
    format!("internal.model-verification.{}", Sha256::digest(format!("{provider}\0{model}").as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect::<String>())
}

pub async fn fingerprint(provider: &ProviderConfig, service: Option<&ProviderService>) -> Option<String> {
    let mut config = serde_json::to_value(provider).ok()?;
    // Credential contents are hashed only in memory, never emitted or persisted.
    // A protected store's revision also invalidates proofs after rotation.
    let revision = if let Some(service) = service {
        service.store().get(CredentialScope::System, "system", &provider.id).await.ok()?
            .map(|record| record.updated_at.to_rfc3339())
    } else { None };
    if revision.is_some() { config["api_key"] = serde_json::Value::Null; }
    config["credential_revision"] = serde_json::json!(revision);
    Some(Sha256::digest(serde_json::to_vec(&config).ok()?).iter().map(|byte| format!("{byte:02x}")).collect())
}

pub async fn current(provider: &ProviderConfig, model: &str, manager: Option<&SettingsManager>, service: Option<&ProviderService>) -> Option<ModelVerification> {
    if !provider.enabled || !provider.models.iter().any(|item| item.enabled && item.id == model) { return None; }
    let proof: ModelVerification = manager?.get_typed(&key(&provider.id, model)).await.ok()??;
    (Some(proof.configuration_digest.as_str()) == fingerprint(provider, service).await.as_deref()).then_some(proof)
}

pub fn permits_anonymous_loopback(base_url: &str) -> bool {
    url::Url::parse(base_url).ok().is_some_and(|url| {
        url.scheme() == "http" && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]" | "::1"))
    })
}

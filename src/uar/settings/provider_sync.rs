//! Hydrate [`ProviderRegistry`](crate::llm::registry::ProviderRegistry) from persisted settings.

use crate::llm::registry::ProviderRegistry;
use crate::uar::settings::manager::SettingsManager;
use anyhow::Result;

/// Apply all `provider.{id}` rows from the database to the registry, then set the
/// default provider from `llm.default_provider` when present.
///
/// Call after [`SettingsManager::initialize`](super::SettingsManager::initialize) so
/// settings types and rows exist. Intended to run once at startup when persistence is enabled.
pub async fn hydrate_provider_registry_from_settings(
    registry: &ProviderRegistry,
    settings: &SettingsManager,
) -> Result<()> {
    let configs = settings.load_provider_configs_from_db().await?;
    for cfg in configs {
        registry.register_custom_provider(cfg).await?;
    }

    if let Some(id) = settings.get_default_provider_id().await {
        if !id.is_empty() {
            let _ = registry.set_default(&id).await;
        }
    }

    Ok(())
}

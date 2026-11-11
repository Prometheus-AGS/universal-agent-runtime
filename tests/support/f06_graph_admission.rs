//! The controlled F06 graphs enter through the same matching and routing gates as production.

use std::sync::Arc;
use universal_agent_runtime::{
    config::LlmConfig,
    llm::{ProviderRegistry, registry::split_model_string_pub},
    uar::{persistence::PersistenceLayer, runtime::skills::SkillService},
};

pub async fn configure(
    store: &Arc<dyn PersistenceLayer>,
    config: &LlmConfig,
) -> (Arc<SkillService>, Arc<ProviderRegistry>) {
    // A new Skill Service uses the production keyword default.
    let skills = Arc::new(SkillService::new(Some(Arc::clone(store)), None));
    let providers = Arc::new(ProviderRegistry::new());
    providers.set_llm_defaults(config).await;
    providers.seed_from_llm_config(config).await;

    let (provider_id, model_id) = split_model_string_pub(&config.model);
    let mut provider = providers
        .get(&provider_id)
        .await
        .expect("controlled provider is registered");
    // Preserve the selected origin, including native drivers with no override.
    provider.base_url = config.base_url.clone().unwrap_or_default();
    let model = provider
        .models
        .iter_mut()
        .find(|model| model.id == model_id)
        .expect("controlled model is registered");
    model.supports_tools = true;
    // The synthetic peer has a bounded fixture window even when its model
    // label has no catalog entry. Admission still computes the output reserve.
    model.context_window.get_or_insert(32_768);
    providers
        .register(provider)
        .await
        .expect("register the controlled peer's tool capability");
    (skills, providers)
}

//! Built-in native skill implementations.
//!
//! These skills are embedded in the runtime binary and available
//! by default without any external tooling or MCP servers.

pub mod echo;
pub mod system_info;

use std::sync::Arc;

use super::native_skill::NativeSkillRegistry;
use crate::uar::compiler::conversational::CompilerSessionStore;
use crate::uar::compiler::signing::{KeyProvider, LocalKeyProvider};

/// Register all built-in native skills into the given registry.
pub async fn register_builtins(registry: &NativeSkillRegistry) {
    // Core skills
    registry.register(echo::EchoSkill).await;
    registry.register(system_info::SystemInfoSkill).await;

    // Compiler skills — single-shot + conversational session tools
    let key_provider: Arc<dyn KeyProvider> = Arc::new(LocalKeyProvider::ephemeral());
    let session_store = CompilerSessionStore::new();

    registry
        .register(crate::uar::compiler::CompilerAgentSkill::new(Arc::clone(
            &key_provider,
        )))
        .await;
    registry
        .register(crate::uar::compiler::UpdateSectionTool::new(
            session_store.clone(),
        ))
        .await;
    registry
        .register(crate::uar::compiler::CheckCompletenessTool::new(
            session_store.clone(),
        ))
        .await;
    registry
        .register(crate::uar::compiler::CompileSessionTool::new(
            session_store,
            key_provider,
        ))
        .await;

    tracing::info!("Registered all built-in native skills (including compiler)");
}

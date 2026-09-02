//! Built-in native skill implementations.
//!
//! Embedded in the runtime binary and available without any external tooling.
//! New tool categories are feature-gated by their config section:
//!
//! - File tools   → `native_tools.file_tools_enabled`
//! - Web fetch    → `native_tools.web_fetch_enabled`
//! - Terminal     → `native_tools.terminal_exec_enabled`
//! - Session search → `native_tools.session_search_enabled`

pub mod a2ui_render;
pub mod echo;
pub mod system_info;

use std::sync::Arc;

use super::native_skill::NativeSkillRegistry;
use crate::config::NativeToolsConfig;
use crate::uar::compiler::conversational::CompilerSessionStore;
use crate::uar::compiler::signing::{KeyProvider, LocalKeyProvider};
use crate::uar::persistence::PersistenceLayer;
use crate::uar::tools::descriptor::ToolAssemblyError;

/// Register all built-in native skills into the given registry.
///
/// `native_cfg` controls which optional tool categories are activated.
/// `persistence` is required for `session_search`; pass `None` to skip it.
///
/// # Errors
///
/// Returns the first invalid schema or provider-name collision encountered
/// while assembling a built-in descriptor.
pub async fn register_builtins(
    registry: &NativeSkillRegistry,
    native_cfg: &NativeToolsConfig,
    persistence: Option<Arc<dyn PersistenceLayer>>,
) -> Result<(), ToolAssemblyError> {
    // ── Core always-on skills ─────────────────────────────────────────────
    registry.register(echo::EchoSkill).await?;
    registry.register(system_info::SystemInfoSkill).await?;
    registry.register(a2ui_render::A2uiRenderSkill).await?;

    // ── Compiler skills ───────────────────────────────────────────────────
    let key_provider: Arc<dyn KeyProvider> = Arc::new(LocalKeyProvider::ephemeral());
    let session_store = CompilerSessionStore::new();
    registry
        .register(crate::uar::compiler::CompilerAgentSkill::new(Arc::clone(
            &key_provider,
        )))
        .await?;
    registry
        .register(crate::uar::compiler::UpdateSectionTool::new(
            session_store.clone(),
        ))
        .await?;
    registry
        .register(crate::uar::compiler::CheckCompletenessTool::new(
            session_store.clone(),
        ))
        .await?;
    registry
        .register(crate::uar::compiler::CompileSessionTool::new(
            session_store,
            key_provider,
        ))
        .await?;

    // ── File system tools ─────────────────────────────────────────────────
    if native_cfg.file_tools_enabled {
        use crate::uar::tools::file_patch::FilePatchTool;
        use crate::uar::tools::file_tools::{FileReadTool, FileWriteTool};
        registry
            .register(FileReadTool {
                allowed_paths: native_cfg.file_allowed_paths.clone(),
                max_size_kb: native_cfg.file_max_size_kb,
            })
            .await?;
        registry
            .register(FileWriteTool {
                allowed_paths: native_cfg.file_allowed_paths.clone(),
                max_size_kb: native_cfg.file_write_max_kb,
            })
            .await?;
        registry
            .register(FilePatchTool {
                allowed_paths: native_cfg.file_allowed_paths.clone(),
                max_size_kb: native_cfg.file_max_size_kb,
            })
            .await?;
        tracing::info!(
            allowed_paths = ?native_cfg.file_allowed_paths,
            "Registered file native tools (file_read, file_write, file_patch)"
        );
    }

    // ── Web fetch tool ────────────────────────────────────────────────────
    if native_cfg.web_fetch_enabled {
        use crate::uar::tools::web_fetch::WebFetchTool;
        registry
            .register(WebFetchTool {
                timeout_secs: native_cfg.web_fetch_timeout_secs,
                max_size_kb: native_cfg.web_fetch_max_size_kb,
                allowed_domains: native_cfg.web_fetch_allowed_domains.clone(),
            })
            .await?;
        tracing::info!("Registered web_fetch native tool");
    }

    // ── Terminal exec tool ────────────────────────────────────────────────
    if native_cfg.terminal_exec_enabled {
        use crate::uar::tools::terminal_exec::TerminalExecTool;
        registry
            .register(TerminalExecTool {
                shell: native_cfg.terminal_shell.clone(),
                timeout_secs: native_cfg.terminal_timeout_secs,
                use_sandbox: native_cfg.terminal_use_sandbox,
            })
            .await?;
        tracing::info!(
            shell = %native_cfg.terminal_shell,
            use_sandbox = native_cfg.terminal_use_sandbox,
            "Registered terminal_exec native tool"
        );
    }

    // ── Session search tool ───────────────────────────────────────────────
    if native_cfg.session_search_enabled {
        if let Some(db) = persistence {
            use crate::uar::tools::session_search::SessionSearchTool;
            registry
                .register(SessionSearchTool {
                    persistence: db,
                    max_results: native_cfg.session_search_max_results,
                })
                .await?;
            tracing::info!("Registered session_search native tool");
        } else {
            tracing::debug!(
                "session_search enabled but no persistence layer configured — skipping"
            );
        }
    }

    tracing::info!("Built-in native skills registration complete");
    Ok(())
}

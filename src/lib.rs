//! Axum + HTMX + Web Components
//!
//! An agentic streaming LLM application that supports tool-first interaction,
//! streams rich typed model output, and remains HTML-first and inspectable.
//!
//! # Architecture
//!
//! - **Server**: Axum-based HTTP server with SSE streaming
//! - **LLM Orchestration**: Protocol-agnostic driver for Chat Completions and Responses APIs
//! - **MCP Client**: Dynamic tool discovery and execution via Model Context Protocol
//! - **UI**: Static HTML + HTMX + Web Components + Alpine.js
//!
//! # Modules
//!
//! - [`llm`]: LLM driver traits and implementations
//! - [`mcp`]: MCP client configuration and registry
//! - [`normalized`]: Unified streaming event model
//! - [`session`]: Conversation and session management

// Allow pedantic clippy warnings that don't add value for this codebase
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_fields_in_debug)]
#![recursion_limit = "512"]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::unused_async)]

pub mod config;
pub mod llm;
pub mod mcp;
pub mod normalized;
pub mod sandbox;
pub mod server;
pub mod session;
pub mod uar;

use crate::config::AppConfig;
use crate::uar::security::rate_limit::AppRateLimiter;

use llm::orchestrator::Orchestrator;
use mcp::registry::McpRegistry;
use session::SessionStore;
use std::collections::HashMap;
use std::sync::Arc;
use uar::api::a2a::AgentRegistry;
use uar::compiler::CompilerService;
use uar::governance::engine::GovernanceEngine;
use uar::memory::service::MemoryService;
use uar::persistence::PersistenceLayer;
use uar::prompt_cache::PromptCacheProvider;
use uar::rag::ingest::IngestService;
use uar::runtime::actor::system::ActorCollaboration;
use uar::runtime::manager::RunManager;
use uar::runtime::matching::VectorMatcher;
use uar::runtime::native_skill::NativeSkillRegistry;
use uar::runtime::skills::service::SkillService;
use uar::runtime::user_settings_store::UserSettingsStore;
use uar::security::api_keys::ApiKeyService;
use uar::settings::manager::SettingsManager;

/// Application state shared across all handlers.
#[derive(Clone, Debug)]
pub struct AppState {
    /// MCP server registry for tool discovery and execution.
    #[allow(dead_code)]
    pub mcp: Arc<McpRegistry>,
    /// LLM orchestrator for chat interactions.
    pub orchestrator: Arc<Orchestrator>,
    /// Session store for conversation management.
    pub sessions: SessionStore,
    /// Run Manager
    pub run_manager: Arc<RunManager>,
    /// Ingest Service
    pub ingest_service: Option<Arc<IngestService>>,
    /// Vector Matcher (for embeddings)
    pub vector_matcher: Arc<VectorMatcher>,
    /// Persistence Layer
    pub persistence: Option<Arc<dyn PersistenceLayer>>,
    /// Global Rate Limiter
    pub rate_limiter: Arc<AppRateLimiter>,
    /// Global Configuration
    pub config: Arc<AppConfig>,
    /// Skill Service
    pub skill_service: Arc<SkillService>,
    /// Provider Registry for multi-provider LLM management
    pub provider_registry: Arc<llm::ProviderRegistry>,
    /// Native Skill Registry for in-process high-performance tools
    pub native_skill_registry: Arc<NativeSkillRegistry>,
    /// Federated A2A registry (if enabled)
    pub federated_agent_registry: Arc<dyn AgentRegistry>,
    /// Actor collaboration system for multi-agent coordination
    pub actor_system: Arc<ActorCollaboration>,
    /// Governance policy engine for declarative authorization
    pub governance_engine: Arc<GovernanceEngine>,
    /// API key service for PAT-based authentication
    pub api_key_service: Option<Arc<ApiKeyService>>,
    /// Memory service backed by surreal-memory + SurrealDB/RocksDB (None if memory.enabled=false).
    pub memory_service: Option<Arc<MemoryService>>,
    /// Compiler service for spec management and pipeline execution
    pub compiler_service: Option<Arc<CompilerService>>,
    /// Settings manager — runtime configuration administration + plugin extension point
    pub settings_manager: Option<Arc<SettingsManager>>,
    /// Prompt cache provider used by Anthropic-compatible API endpoints.
    pub prompt_cache_provider: Arc<dyn PromptCacheProvider>,
    /// Per-user prompt-caching preferences store.
    pub user_settings_store: Arc<UserSettingsStore>,
    /// A2UI schema registry — resolves artifact schema IDs declared in UAR-AGENT-MD §06.
    pub a2ui_registry: Arc<uar::a2ui::registry::A2uiRegistry>,
    /// Model router — selects optimal model based on capability requirements from the catalog.
    pub model_router: Arc<llm::ModelRouter>,
    /// Per-session agent configuration overrides (in-memory).
    pub agent_sessions:
        Arc<tokio::sync::RwLock<HashMap<String, uar::api::discovery::AgentSessionConfig>>>,
    /// Wasm sandbox runtime for executing Wasm agents (feature-gated)
    #[cfg(feature = "wasm-runtime")]
    pub wasm_sandbox: Option<Arc<uar::runtime::wasm::sandbox::WasmSandbox>>,
}

use crate::uar::domain::artifact::{
    AgentArtifact, AgentMemoryConfig, AgentMetadata, AgentPolicy, AgentPrompt, AgentRuntimeConfig,
    AgentSchemas, AgentToolConfig, AgentUiConfig, ArtifactsConfig, ConversationMemory, FeatureFlag,
    KbMemory, ProviderPolicy, ProviderSelection, SkillPolicy, ToolPolicy,
};
use std::collections::HashMap;

/// Returns the default agent artifact used when no specific agent is requested.
pub fn default_agent() -> AgentArtifact {
    AgentArtifact {
        version: "1.0.0".to_string(),
        kind: "agent".to_string(),
        id: "default-agent".to_string(),
        metadata: AgentMetadata {
            title: "Default Assistant".to_string(),
            description: "A helpful generic AI assistant.".to_string(),
            tags: vec!["default".to_string(), "general".to_string()],
            author: Some("System".to_string()),
            icon: None,
        },
        runtime: AgentRuntimeConfig {
            entry: "default".to_string(),
            protocols: HashMap::new(),
        },
        policy: AgentPolicy {
            provider: ProviderPolicy {
                // Empty provider/model defers to the system-wide registry
                // default (see `ProviderRegistry::default_id`/`default_model`
                // and `resolve_requested_model`'s empty-model fallback)
                // instead of pinning a specific model here. A hardcoded model
                // name goes stale the moment a provider's catalog changes
                // (e.g. renamed/deprecated), and `seed_builtin_agents` re-seeds
                // this value on every server restart, silently undoing any
                // runtime override — so pin nothing rather than pin
                // something wrong.
                default: ProviderSelection {
                    provider: String::new(),
                    model: String::new(),
                },
                fallbacks: vec![],
            },
            tools: ToolPolicy {
                allow: vec!["*".to_string()],
                deny: vec![],
                max_concurrent: 1,
                execution_mode: crate::uar::domain::artifact::ToolExecutionMode::Direct,
            },
            skills: SkillPolicy {
                prefer: vec![],
                max_active: 3,
            },
        },
        schemas: AgentSchemas {
            inputs: None,
            outputs: None,
            state: None,
        },
        prompt: AgentPrompt {
            system: "You are a helpful, intelligent, and capable AI assistant. \
            You can answer questions, perform tasks, and use provided tools. \
            Always provide clear, concise, and accurate information."
                .to_string(),
            instructions: vec![],
        },
        memory: AgentMemoryConfig {
            conversation: ConversationMemory { enabled: true },
            // Enable the default knowledge base so freshly-uploaded documents are
            // discoverable in the chat KB panel without manual wiring. An empty
            // `knowledge_bases` list resolves to the system-default KB.
            kb: KbMemory {
                enabled: true,
                knowledge_bases: vec![],
                citation_required: false,
            },
        },
        tools: AgentToolConfig { bundles: vec![] },
        ui: AgentUiConfig {
            forms: FeatureFlag::default(),
            artifacts: ArtifactsConfig::default(),
        },
        extensions: HashMap::new(),
    }
}

pub fn orchestrator_agent() -> AgentArtifact {
    let mut agent = default_agent();
    agent.id = "orchestrator-agent".to_string();
    agent.metadata.title = "Orchestrator".to_string();
    agent.metadata.description =
        "Routes complex tasks to a general-purpose or Rust-review sub-agent and returns the delegated contribution."
            .to_string();
    agent.metadata.tags = vec![
        "orchestration".to_string(),
        "delegation".to_string(),
        "multi-agent".to_string(),
    ];
    agent.runtime.entry = "orchestrator".to_string();
    agent.prompt.system = "You coordinate specialist sub-agents. Route each request to the best available specialist and return that specialist's evidence-backed contribution."
        .to_string();
    agent.prompt.instructions = vec![
        "Use rust-reviewer for Rust implementation and safety review; use general-purpose for other work."
            .to_string(),
    ];
    agent
}

pub(crate) fn orchestrator_graph() -> crate::uar::runtime::graph::AgentGraph {
    use crate::uar::runtime::graph::{AgentGraph, AgentNode, GraphState, RouterNode};

    AgentGraph::builder("router")
        .add_node(RouterNode::new(
            "router",
            "Route Rust implementation, correctness, or safety questions to rust-reviewer. Route all other questions to general-purpose.",
            vec!["general-purpose".to_string(), "rust-reviewer".to_string()],
        ))
        .add_node(AgentNode::new("general-purpose", "general-purpose"))
        .add_node(AgentNode::new("rust-reviewer", "rust-reviewer"))
        .add_conditional_edge("router", |state: &GraphState| {
            state
                .get::<String>("_route")
                .unwrap_or_else(|| "general-purpose".to_string())
        })
        .build()
}

/// Seeds the two built-in agents into the persistence layer at startup.
///
/// This is an idempotent upsert: if a row with the same `id` already exists,
/// the definition is overwritten so system updates (e.g. prompt improvements)
/// are always applied. Because the agents are now persisted, the realtime
/// entity bus will re-emit them after any `Agent` ChangeSet, making them
/// reliably visible in the admin list and chat selector without relying on the
/// `ensure_builtin_agent` shim.
///
/// Call this alongside [`ensure_default_knowledge_base`] in server startup.
pub async fn seed_builtin_agents(
    persistence: &dyn crate::uar::persistence::PersistenceLayer,
) -> anyhow::Result<()> {
    for agent in [default_agent(), orchestrator_agent()] {
        let id = agent.id.clone();
        match persistence.save_agent(&agent).await {
            Ok(()) => {
                tracing::info!(agent_id = %id, "Built-in agent seeded/refreshed");
            }
            Err(e) => {
                tracing::warn!(agent_id = %id, error = ?e, "Failed to seed built-in agent — continuing");
            }
        }
    }
    Ok(())
}

/// Creates the default knowledge base if it doesn't exist.
/// This should be called on application startup.
pub async fn ensure_default_knowledge_base(
    persistence: &dyn crate::uar::persistence::PersistenceLayer,
    config: Option<&crate::config::KnowledgeBaseConfig>,
) -> anyhow::Result<crate::uar::domain::knowledge::KnowledgeBase> {
    use crate::uar::domain::knowledge::{KbConfig, KnowledgeBase};
    use crate::uar::rag::chunking::ChunkingStrategy;

    const DEFAULT_KB_NAME: &str = "default";

    // Check if default KB already exists
    if let Some(existing) = persistence
        .get_knowledge_base_by_name(
            crate::uar::domain::knowledge::ANONYMOUS_KNOWLEDGE_OWNER,
            DEFAULT_KB_NAME,
        )
        .await?
    {
        tracing::debug!("Default knowledge base already exists: {}", existing.id);
        return Ok(existing);
    }

    // Build config from provided config or use hardcoded defaults
    let kb_config = if let Some(cfg) = config {
        // Convert ChunkingConfig to ChunkingStrategy
        let chunk_strategy = match cfg.chunking.strategy.as_str() {
            "fixed" => ChunkingStrategy::FixedSize {
                size: cfg.chunking.chunk_size,
            },
            "recursive" => ChunkingStrategy::Recursive {
                size: cfg.chunking.chunk_size,
            },
            "token" => ChunkingStrategy::Token {
                tokens: cfg.chunking.chunk_size,
            },
            "sentence" => ChunkingStrategy::Sentence,
            "document" => ChunkingStrategy::Document,
            "semantic" => ChunkingStrategy::Semantic {
                threshold: cfg.chunking.semantic_threshold.unwrap_or(0.7),
            },
            _ => ChunkingStrategy::Recursive { size: 512 },
        };

        KbConfig {
            embedding_provider: cfg.embedding_provider.clone(),
            embedding_model: cfg.embedding_model.clone(),
            vector_dimensions: cfg.vector_dimensions,
            file_processor: cfg.file_processor.clone(),
            chunk_strategy,
        }
    } else {
        KbConfig::default()
    };

    let now = chrono::Utc::now().to_rfc3339();
    let kb = KnowledgeBase {
        id: uuid::Uuid::new_v4().to_string(),
        owner_id: crate::uar::domain::knowledge::ANONYMOUS_KNOWLEDGE_OWNER.to_string(),
        name: DEFAULT_KB_NAME.to_string(),
        description: Some("Default knowledge base for general documents".to_string()),
        config: kb_config,
        created_at: now.clone(),
        updated_at: now,
    };

    persistence.save_knowledge_base(&kb).await?;
    tracing::info!("Created default knowledge base: {} ({})", kb.name, kb.id);

    Ok(kb)
}

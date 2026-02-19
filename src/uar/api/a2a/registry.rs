use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::{Surreal, engine::any::Any};

/// Information about a registered agent in the federation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Unique Agent ID.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of the agent's purpose.
    pub description: String,
    /// Base URL where the agent can be reached.
    pub base_url: String,
    /// List of capabilities/skills this agent exposes.
    pub capabilities: Vec<String>,
    /// When the agent was registered.
    pub created_at: DateTime<Utc>,
    /// When the agent information was last updated.
    pub updated_at: DateTime<Utc>,
}

/// A skill exposed by a federated agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSkill {
    /// The agent that provides this skill.
    pub agent_id: String,
    /// Agent name (for display).
    pub agent_name: String,
    /// Skill identifier (e.g. "uar.compile").
    pub skill_id: String,
    /// Base URL of the providing agent.
    pub agent_base_url: String,
}

/// Registry for discovering and managing federated agents.
#[async_trait]
pub trait AgentRegistry: Send + Sync + std::fmt::Debug {
    /// Register or update an agent.
    async fn register_agent(&self, agent: AgentInfo) -> Result<String>;

    /// Retrieve an agent by ID.
    async fn get_agent(&self, id: &str) -> Result<Option<AgentInfo>>;

    /// List all registered agents.
    async fn list_agents(&self) -> Result<Vec<AgentInfo>>;

    /// List all skills across all registered agents.
    async fn list_skills(&self) -> Result<Vec<ExternalSkill>> {
        let agents = self.list_agents().await?;
        let skills = agents
            .into_iter()
            .flat_map(|a| {
                let caps = a.capabilities.clone();
                caps.into_iter().map(move |cap| ExternalSkill {
                    agent_id: a.id.clone(),
                    agent_name: a.name.clone(),
                    skill_id: cap,
                    agent_base_url: a.base_url.clone(),
                })
            })
            .collect();
        Ok(skills)
    }
}

/// SurrealDB-backed implementation of AgentRegistry.
#[derive(Debug, Clone)]
pub struct SurrealAgentRegistry {
    db: Surreal<Any>,
}

impl SurrealAgentRegistry {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Helper to convert a DB value to AgentInfo.
    fn from_db_value(value: serde_json::Value) -> Result<AgentInfo> {
        serde_json::from_value(value).context("failed to deserialize AgentInfo from DB")
    }

    /// Helper to convert a DB list to Vec<AgentInfo>.
    fn from_db_vec(values: Vec<serde_json::Value>) -> Result<Vec<AgentInfo>> {
        values.into_iter().map(Self::from_db_value).collect()
    }

    /// Helper to convert AgentInfo to DB value.
    fn to_db_value(agent: &AgentInfo) -> Result<serde_json::Value> {
        serde_json::to_value(agent).context("failed to serialize AgentInfo to DB value")
    }
}

#[async_trait]
impl AgentRegistry for SurrealAgentRegistry {
    async fn register_agent(&self, mut agent: AgentInfo) -> Result<String> {
        agent.updated_at = Utc::now();
        let id = agent.id.clone();

        // Ensure table exists or just upsert
        let _: Option<serde_json::Value> = self
            .db
            .upsert(("uar_agents", id.as_str()))
            .content(Self::to_db_value(&agent)?)
            .await?;

        Ok(id)
    }

    async fn get_agent(&self, id: &str) -> Result<Option<AgentInfo>> {
        let result: Option<serde_json::Value> = self.db.select(("uar_agents", id)).await?;
        match result {
            Some(val) => Ok(Some(Self::from_db_value(val)?)),
            None => Ok(None),
        }
    }

    async fn list_agents(&self) -> Result<Vec<AgentInfo>> {
        let records: Vec<serde_json::Value> = self.db.select("uar_agents").await?;
        Self::from_db_vec(records)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// In-memory fallback (for non-SurrealDB deployments / tests)
// ─────────────────────────────────────────────────────────────────────────────

use std::collections::HashMap;
use std::sync::RwLock;

/// Simple in-memory agent registry — useful for tests and non-persistent deployments.
#[derive(Debug, Default)]
pub struct InMemoryAgentRegistry {
    agents: RwLock<HashMap<String, AgentInfo>>,
}

impl InMemoryAgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl AgentRegistry for InMemoryAgentRegistry {
    async fn register_agent(&self, mut agent: AgentInfo) -> Result<String> {
        agent.updated_at = chrono::Utc::now();
        let id = agent.id.clone();
        self.agents
            .write()
            .map_err(|e| anyhow::anyhow!("lock poisoned: {e}"))?
            .insert(id.clone(), agent);
        Ok(id)
    }

    async fn get_agent(&self, id: &str) -> Result<Option<AgentInfo>> {
        let guard = self
            .agents
            .read()
            .map_err(|e| anyhow::anyhow!("lock poisoned: {e}"))?;
        Ok(guard.get(id).cloned())
    }

    async fn list_agents(&self) -> Result<Vec<AgentInfo>> {
        let guard = self
            .agents
            .read()
            .map_err(|e| anyhow::anyhow!("lock poisoned: {e}"))?;
        Ok(guard.values().cloned().collect())
    }
}

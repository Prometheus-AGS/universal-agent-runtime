//! Conversational compiler tools — [`NativeSkill`] implementations that power
//! the multi-turn compiler agent.
//!
//! Three tools are registered for conversational compilation:
//!
//! - `uar.session.update_section` — Parse a Markdown snippet and merge into the session's partial IR
//! - `uar.session.check_completeness` — Return which sections are present/missing
//! - `uar.session.compile` — Trigger compilation when all sections are complete
//!
//! These tools are designed to be called by the LLM orchestrator during a
//! conversational compilation session. The LLM drives the conversation, asking
//! the user for information, then calling these tools to update the session.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};
use tokio::sync::RwLock;

use crate::uar::compiler::completeness::CompletenessAnalyzer;
use crate::uar::compiler::parser;
use crate::uar::compiler::pipeline;
use crate::uar::compiler::registries::{InMemoryEndpointRegistry, InMemorySchemaRegistry};
use crate::uar::compiler::session::{CompilerSession, SessionStatus, TurnRole};
use crate::uar::compiler::signing::KeyProvider;
use crate::uar::runtime::native_skill::NativeSkill;
use crate::uar::tools::descriptor::{ToolEffect, ToolSource};

/// Thread-safe session store for conversational compilation.
#[derive(Debug, Clone)]
pub struct CompilerSessionStore {
    sessions: Arc<RwLock<HashMap<String, CompilerSession>>>,
}

impl Default for CompilerSessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerSessionStore {
    /// Create an empty session store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session and return its ID.
    pub async fn create(&self) -> String {
        let session = CompilerSession::new();
        let id = session.id.clone();
        self.sessions.write().await.insert(id.clone(), session);
        id
    }

    /// Get a clone of a session by ID.
    pub async fn get(&self, id: &str) -> Option<CompilerSession> {
        self.sessions.read().await.get(id).cloned()
    }

    /// Update a session in the store.
    pub async fn update(&self, session: CompilerSession) {
        self.sessions
            .write()
            .await
            .insert(session.id.clone(), session);
    }

    /// Remove a session from the store.
    pub async fn remove(&self, id: &str) -> Option<CompilerSession> {
        self.sessions.write().await.remove(id)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool: uar.session.update_section
// ─────────────────────────────────────────────────────────────────────────────

/// Tool that parses a partial Markdown snippet and merges extracted sections
/// into the session's partial IR.
#[derive(Debug)]
pub struct UpdateSectionTool {
    store: CompilerSessionStore,
}

impl UpdateSectionTool {
    /// Create a new update-section tool.
    pub fn new(store: CompilerSessionStore) -> Self {
        Self { store }
    }
}

#[async_trait::async_trait]
impl NativeSkill for UpdateSectionTool {
    fn name(&self) -> &str {
        "uar.session.update_section"
    }

    fn description(&self) -> &str {
        "Parse a Markdown snippet containing one or more UAR-AGENT-MD sections and merge them \
         into the current compilation session's partial IR. Creates a new session if session_id \
         is not provided."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["markdown"],
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Session ID. Omit to create a new session."
                },
                "markdown": {
                    "type": "string",
                    "description": "Markdown snippet containing one or more ## Section blocks with YAML"
                }
            }
        })
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::ExternalMutation
    }

    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }

    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        let markdown = args
            .get("markdown")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required 'markdown' parameter"))?;

        // Get or create session
        let session_id = if let Some(id) = args.get("session_id").and_then(|v| v.as_str()) {
            id.to_string()
        } else {
            self.store.create().await
        };

        let mut session = self
            .store
            .get(&session_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("session '{session_id}' not found"))?;

        // Parse the snippet as a partial document
        let new_partial =
            parser::parse_partial(markdown).map_err(|e| anyhow::anyhow!("parse error: {e}"))?;

        // Merge fields from new_partial into session.partial_ir
        if new_partial.agent_name.is_some() {
            session.partial_ir.agent_name = new_partial.agent_name;
        }
        if new_partial.metadata.is_some() {
            session.partial_ir.metadata = new_partial.metadata;
        }
        if new_partial.identity.is_some() {
            session.partial_ir.identity = new_partial.identity;
        }
        if new_partial.ui.is_some() {
            session.partial_ir.ui = new_partial.ui;
        }
        if new_partial.capabilities.is_some() {
            session.partial_ir.capabilities = new_partial.capabilities;
        }
        if new_partial.skills.is_some() {
            session.partial_ir.skills = new_partial.skills;
        }
        if new_partial.tools.is_some() {
            session.partial_ir.tools = new_partial.tools;
        }
        if new_partial.mcp_servers.is_some() {
            session.partial_ir.mcp_servers = new_partial.mcp_servers;
        }
        if new_partial.knowledge.is_some() {
            session.partial_ir.knowledge = new_partial.knowledge;
        }
        if new_partial.memory.is_some() {
            session.partial_ir.memory = new_partial.memory;
        }
        if new_partial.a2a.is_some() {
            session.partial_ir.a2a = new_partial.a2a;
        }
        if new_partial.governance.is_some() {
            session.partial_ir.governance = new_partial.governance;
        }
        if new_partial.budgets.is_some() {
            session.partial_ir.budgets = new_partial.budgets;
        }
        if new_partial.execution.is_some() {
            session.partial_ir.execution = new_partial.execution;
        }
        if new_partial.observability.is_some() {
            session.partial_ir.observability = new_partial.observability;
        }
        if new_partial.deployment.is_some() {
            session.partial_ir.deployment = new_partial.deployment;
        }

        // Record the update
        session.add_turn(TurnRole::System, format!("Updated sections from snippet"));

        // Analyze completeness
        let report = CompletenessAnalyzer::analyze(&session.partial_ir);

        if report.is_ready {
            session.status = SessionStatus::Ready;
        }

        self.store.update(session).await;

        Ok(json!({
            "session_id": session_id,
            "sections_filled": report.present.len(),
            "sections_total": report.present.len() + report.missing.len(),
            "completeness_pct": report.completeness_pct,
            "is_ready": report.is_ready,
            "missing": report.missing,
            "next_questions": report.next_questions
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool: uar.session.check_completeness
// ─────────────────────────────────────────────────────────────────────────────

/// Tool that checks the completeness of a compilation session.
#[derive(Debug)]
pub struct CheckCompletenessTool {
    store: CompilerSessionStore,
}

impl CheckCompletenessTool {
    /// Create a new check-completeness tool.
    pub fn new(store: CompilerSessionStore) -> Self {
        Self { store }
    }
}

#[async_trait::async_trait]
impl NativeSkill for CheckCompletenessTool {
    fn name(&self) -> &str {
        "uar.session.check_completeness"
    }

    fn description(&self) -> &str {
        "Check how complete a compilation session is. Returns which sections are present, \
         which are missing, the completeness percentage, and suggested next questions."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["session_id"],
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "The session ID to check"
                }
            }
        })
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::ReadOnly
    }

    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }

    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        let session_id = args
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required 'session_id' parameter"))?;

        let session = self
            .store
            .get(session_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("session '{session_id}' not found"))?;

        let report = CompletenessAnalyzer::analyze(&session.partial_ir);

        Ok(json!({
            "session_id": session_id,
            "status": format!("{:?}", session.status),
            "present": report.present,
            "missing": report.missing,
            "completeness_pct": report.completeness_pct,
            "is_ready": report.is_ready,
            "summary": report.summary,
            "next_questions": report.next_questions,
            "filled_count": session.filled_count(),
            "total_sections": CompilerSession::total_sections()
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool: uar.session.compile
// ─────────────────────────────────────────────────────────────────────────────

/// Tool that triggers compilation of a complete session.
#[derive(Debug)]
pub struct CompileSessionTool {
    store: CompilerSessionStore,
    key_provider: Arc<dyn KeyProvider>,
}

impl CompileSessionTool {
    /// Create a new compile-session tool.
    pub fn new(store: CompilerSessionStore, key_provider: Arc<dyn KeyProvider>) -> Self {
        Self {
            store,
            key_provider,
        }
    }
}

#[async_trait::async_trait]
impl NativeSkill for CompileSessionTool {
    fn name(&self) -> &str {
        "uar.session.compile"
    }

    fn description(&self) -> &str {
        "Trigger compilation of a session that has all required sections filled. \
         The session must be in 'Ready' status. Returns the signed agent descriptor."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["session_id"],
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "The session ID to compile"
                }
            }
        })
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::ExternalMutation
    }

    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }

    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        let session_id = args
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required 'session_id' parameter"))?;

        let mut session = self
            .store
            .get(session_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("session '{session_id}' not found"))?;

        // Verify readiness
        let report = CompletenessAnalyzer::analyze(&session.partial_ir);
        if !report.is_ready {
            return Ok(json!({
                "error": "session is not ready for compilation",
                "missing": report.missing,
                "completeness_pct": report.completeness_pct,
                "next_questions": report.next_questions
            }));
        }

        // Mark as compiling
        session.status = SessionStatus::Compiling;
        self.store.update(session.clone()).await;

        // Promote partial IR to complete IR (clone since session is used in error handling below)
        let ir = session
            .partial_ir
            .clone()
            .try_into_complete()
            .ok_or_else(|| {
                anyhow::anyhow!("failed to promote partial IR despite readiness check")
            })?;

        // Create registries and run pipeline
        let schema_registry = Arc::new(InMemorySchemaRegistry::new());
        let endpoint_registry = Arc::new(InMemoryEndpointRegistry::new());

        match pipeline::compile(
            ir,
            schema_registry,
            endpoint_registry,
            Arc::clone(&self.key_provider),
        )
        .await
        {
            Ok(output) => {
                // Update session status
                let mut session = self.store.get(session_id).await.unwrap_or(session);
                session.status = SessionStatus::Completed;
                session.add_turn(
                    TurnRole::System,
                    format!(
                        "Compilation successful: {} v{}",
                        output.descriptor.agent_id, output.descriptor.version
                    ),
                );
                self.store.update(session).await;

                let result = serde_json::to_value(&output)?;
                Ok(result)
            }
            Err(e) => {
                let mut session = self.store.get(session_id).await.unwrap_or(session);
                session.status = SessionStatus::Failed;
                session.add_turn(TurnRole::System, format!("Compilation failed: {e}"));
                self.store.update(session).await;

                Err(anyhow::anyhow!("compilation failed: {e}"))
            }
        }
    }
}

/// The system prompt for the conversational compiler agent.
/// This is used to initialize the LLM with knowledge of the UAR-AGENT-MD format.
pub const COMPILER_AGENT_SYSTEM_PROMPT: &str = r#"You are the UAR-AGENT-MD Compiler Agent. Your job is to help the user define an agent specification by gathering information about each required section of the UAR-AGENT-MD format.

You have access to three tools:
1. `uar.session.update_section` — Parse Markdown containing section definitions and merge into the session
2. `uar.session.check_completeness` — Check which sections are filled and what's still needed
3. `uar.session.compile` — Compile the session when all sections are complete

The UAR-AGENT-MD format has 16 required components:
1. Agent Name (H1 heading: # Agent: <name>)
2. Metadata — version, description, tags, license
3. Identity — name, role, persona, system prompt
4. UI (A2UI) — forms, artifacts, actions for user interaction
5. Capabilities — streaming, file upload, code execution, web browsing
6. Skills — native and external skill declarations
7. Tools — tool access with allow/deny lists
8. MCP Servers — Model Context Protocol server connections
9. Knowledge Base — data sources for RAG
10. Memory Model — conversation and persistent memory configuration
11. A2A Contracts — agent-to-agent endpoints and dependencies
12. Governance — Cedar policies, audit configuration
13. Budgets & Constraints — token limits, cost caps, rate limits
14. Execution Model — execution mode, iteration limits
15. Observability — tracing, metrics, logging
16. Deployment Profiles — LLM provider, model, resource constraints

Guide the user through providing information for each section. When they provide details, construct the appropriate YAML and use `uar.session.update_section` to save it. Use `uar.session.check_completeness` to track progress. When all sections are filled, confirm with the user and use `uar.session.compile` to produce the signed descriptor.

Be conversational and helpful. Ask about 2-3 sections at a time to avoid overwhelming the user. Start with the most important sections: Agent Name, Identity, and Capabilities.
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::compiler::signing::LocalKeyProvider;

    #[tokio::test]
    async fn test_session_lifecycle() {
        let store = CompilerSessionStore::new();
        let kp = Arc::new(LocalKeyProvider::ephemeral());

        let update_tool = UpdateSectionTool::new(store.clone());
        let check_tool = CheckCompletenessTool::new(store.clone());
        let compile_tool = CompileSessionTool::new(store.clone(), kp);

        // Create a session via update_section (no session_id → creates one)
        let result = update_tool
            .execute(json!({
                "markdown": "# Agent: ConvoBot\n\n## Metadata\n```yaml\nversion: \"0.1.0\"\n```\n"
            }))
            .await
            .unwrap();

        let session_id = result["session_id"].as_str().unwrap().to_string();
        assert_eq!(result["sections_filled"].as_u64().unwrap(), 2); // agent_name + metadata
        assert!(!result["is_ready"].as_bool().unwrap());

        // Check completeness
        let report = check_tool
            .execute(json!({ "session_id": session_id }))
            .await
            .unwrap();
        assert!(!report["is_ready"].as_bool().unwrap());
        assert!(report["missing"].as_array().unwrap().len() > 0);

        // Try compile before ready — should fail gracefully
        let compile_result = compile_tool
            .execute(json!({ "session_id": session_id }))
            .await
            .unwrap();
        assert!(compile_result.get("error").is_some());
    }

    #[tokio::test]
    async fn test_full_conversational_compile() {
        let store = CompilerSessionStore::new();
        let kp = Arc::new(LocalKeyProvider::ephemeral());

        let update_tool = UpdateSectionTool::new(store.clone());
        let compile_tool = CompileSessionTool::new(store.clone(), kp);

        // Build up a complete document via multiple updates
        let doc = r#"# Agent: FullBot

## Metadata
```yaml
version: "1.0.0"
```

## Identity
```yaml
name: FullBot
role: helper
persona: A complete test bot
```

## UI
```yaml
forms: []
artifacts: []
actions: []
```

## Capabilities
```yaml
streaming: false
file_upload: false
image_generation: false
code_execution: false
web_browsing: false
```

## Skills
```yaml
skills: []
```

## Tools
```yaml
tools: []
allow: []
deny: []
```

## MCP Servers
```yaml
servers: []
```

## Knowledge Base
```yaml
sources: []
```

## Memory Model
```yaml
conversation:
  max_turns: 50
```

## A2A Contracts
```yaml
endpoints: []
dependencies: []
```

## Governance
```yaml
cedar_policies: []
audit: {}
```

## Budgets
```yaml
{}
```

## Execution Model
```yaml
{}
```

## Observability
```yaml
tracing: {}
metrics: {}
logging: {}
```

## Deployment Profiles
```yaml
profiles: []
```
"#;

        let result = update_tool
            .execute(json!({ "markdown": doc }))
            .await
            .unwrap();

        let session_id = result["session_id"].as_str().unwrap().to_string();
        assert!(result["is_ready"].as_bool().unwrap());

        // Compile
        let output = compile_tool
            .execute(json!({ "session_id": session_id }))
            .await
            .unwrap();

        assert!(output.get("descriptor").is_some());
        assert!(output.get("signature").is_some());
        assert_eq!(output["descriptor"]["agent_id"], "FullBot");
    }
}

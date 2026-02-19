//! Compiler service — orchestrates spec storage, pipeline execution, and sessions.
//!
//! [`CompilerService`] is the primary entry point for all compiler operations:
//! - Storing and managing UAR-AGENT-MD spec documents.
//! - Running the 8-stage compilation pipeline (single-shot mode).
//! - Managing conversational compiler sessions.

use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::{info, warn};

use super::completeness::CompletenessAnalyzer;
use super::ir::AgentDescriptorIR;
use super::parser;
use super::pipeline::{self, CompileOutput};
use super::registries::{InMemoryEndpointRegistry, InMemorySchemaRegistry};
use super::session::persistence::{InMemorySessionStorage, SessionStorage};
use super::session::{CompilerSession, SessionStatus};
use super::signing::LocalKeyProvider;
use super::storage::{InMemorySpecStorage, ReportRecord, SpecRecord, SpecStorage};

// ─────────────────────────────────────────────────────────────────────────────
// CompilerService
// ─────────────────────────────────────────────────────────────────────────────

/// Orchestrates spec storage, pipeline execution, and conversational sessions.
#[derive(Debug)]
pub struct CompilerService {
    /// Spec + report storage backend.
    pub storage: Arc<dyn SpecStorage>,
    /// Active conversational compiler sessions.
    pub session_storage: Arc<dyn SessionStorage>,
}

impl CompilerService {
    /// Create a new `CompilerService` with the given storage backend.
    pub fn new(storage: Arc<dyn SpecStorage>, session_storage: Arc<dyn SessionStorage>) -> Self {
        Self {
            storage,
            session_storage,
        }
    }

    /// Create a `CompilerService` backed by in-memory storage (dev/test default).
    #[must_use]
    pub fn in_memory() -> Self {
        Self::new(
            Arc::new(InMemorySpecStorage::new()),
            Arc::new(InMemorySessionStorage::new()),
        )
    }

    // ── Spec management ───────────────────────────────────────────────────────

    /// Store a new spec document. Extracts the agent name from the Markdown if possible.
    pub async fn store_spec(&self, content: String) -> Result<SpecRecord> {
        let name = extract_agent_name(&content).unwrap_or_else(|| "unnamed-agent".to_string());
        let record = SpecRecord::new(name, content);
        self.storage.create_spec(record).await
    }

    /// Retrieve a spec by ID.
    pub async fn get_spec(&self, id: &str) -> Result<Option<SpecRecord>> {
        self.storage.get_spec(id).await
    }

    /// List all stored specs.
    pub async fn list_specs(&self) -> Result<Vec<SpecRecord>> {
        self.storage.list_specs().await
    }

    /// Update the content of an existing spec.
    pub async fn update_spec(&self, id: &str, content: String) -> Result<Option<SpecRecord>> {
        self.storage.update_spec(id, content).await
    }

    /// Delete a spec and its associated reports.
    pub async fn delete_spec(&self, id: &str) -> Result<bool> {
        self.storage.delete_spec(id).await
    }

    // ── Compilation ───────────────────────────────────────────────────────────

    /// Compile a stored spec by ID. Stores the resulting report.
    ///
    /// Returns the full [`CompileOutput`] on success.
    pub async fn compile_spec(&self, spec_id: &str) -> Result<CompileOutput> {
        let spec = self
            .storage
            .get_spec(spec_id)
            .await?
            .with_context(|| format!("spec '{spec_id}' not found"))?;

        let output = self.compile_content(&spec.content).await?;

        // Persist the report
        let report_record = ReportRecord::new(spec_id, output.clone());
        self.storage.save_report(report_record).await?;

        info!(
            spec_id,
            report_id = %output.report.id,
            outcome = ?output.report.overall,
            "spec compiled and report stored"
        );

        Ok(output)
    }

    /// Compile raw Markdown content inline (no storage).
    pub async fn compile_content(&self, content: &str) -> Result<CompileOutput> {
        let ir = parser::parse(content).map_err(|e| anyhow::anyhow!("parse error: {e}"))?;

        self.run_pipeline(ir).await
    }

    /// Run the 8-stage pipeline with default in-memory registries and a local key provider.
    async fn run_pipeline(&self, ir: AgentDescriptorIR) -> Result<CompileOutput> {
        let schema_registry = Arc::new(InMemorySchemaRegistry::new());
        let endpoint_registry = Arc::new(InMemoryEndpointRegistry::new());
        let key_provider = Arc::new(LocalKeyProvider::ephemeral());

        pipeline::compile(ir, schema_registry, endpoint_registry, key_provider)
            .await
            .map_err(|e| anyhow::anyhow!("compilation failed: {e}"))
    }

    // ── Report retrieval ──────────────────────────────────────────────────────

    /// Retrieve a compile report by ID.
    pub async fn get_report(&self, report_id: &str) -> Result<Option<ReportRecord>> {
        self.storage.get_report(report_id).await
    }

    /// List all compile reports for a given spec.
    pub async fn list_reports(&self, spec_id: &str) -> Result<Vec<ReportRecord>> {
        self.storage.list_reports_for_spec(spec_id).await
    }

    // ── Conversational sessions ───────────────────────────────────────────────

    /// Create a new conversational compiler session.
    pub async fn create_session(&self) -> CompilerSession {
        let session = CompilerSession::new();
        match self.session_storage.save_session(session.clone()).await {
            Ok(_) => {
                info!(session_id = %session.id, "compiler session created");
                session
            }
            Err(e) => {
                warn!(error = %e, "failed to save new session");
                // Return session anyway, though it might not be persisted effectively if DB is down.
                // In a real app we might return Result<CompilerSession>.
                // For now, to keep signature, we return it.
                session
            }
        }
    }

    /// Get a session by ID.
    pub async fn get_session(&self, session_id: &str) -> Option<CompilerSession> {
        self.session_storage
            .get_session(session_id)
            .await
            .unwrap_or_else(|e| {
                warn!(session_id, error = %e, "failed to get session");
                None
            })
    }

    /// List all active sessions.
    pub async fn list_sessions(&self) -> Vec<CompilerSession> {
        self.session_storage
            .list_sessions()
            .await
            .unwrap_or_else(|e| {
                warn!(error = %e, "failed to list sessions");
                Vec::new()
            })
    }

    /// Cancel a session.
    pub async fn cancel_session(&self, session_id: &str) -> bool {
        if let Some(mut session) = self.get_session(session_id).await {
            session.status = SessionStatus::Cancelled;
            if let Err(e) = self.session_storage.save_session(session).await {
                warn!(session_id, error = %e, "failed to save cancelled session");
                return false;
            }
            true
        } else {
            false
        }
    }

    /// Compile a session's partial IR (force-compile even if incomplete).
    ///
    /// Promotes the `PartialAgentDescriptorIR` to a full IR and runs the pipeline.
    /// Returns an error if required fields are missing.
    pub async fn compile_session(&self, session_id: &str) -> Result<CompileOutput> {
        let session = self
            .get_session(session_id)
            .await
            .with_context(|| format!("session '{session_id}' not found"))?;

        // Check completeness
        let state = CompletenessAnalyzer::analyze(&session.partial_ir);
        if !state.is_ready {
            return Err(anyhow::anyhow!(
                "session is not ready to compile: {}/{} required sections filled",
                state.present.len(),
                state.present.len() + state.missing.len()
            ));
        }

        // Mark as compiling
        let mut session = session;
        session.status = SessionStatus::Compiling;
        self.session_storage.save_session(session.clone()).await?;

        // Promote partial IR → full IR
        let ir = session
            .partial_ir
            .clone()
            .try_into_complete()
            .ok_or_else(|| anyhow::anyhow!("IR promotion failed: required sections are missing"))?;

        let output = match self.run_pipeline(ir).await {
            Ok(out) => {
                // Mark as completed
                session.status = SessionStatus::Completed;
                if let Err(e) = self.session_storage.save_session(session).await {
                    warn!(session_id, error = %e, "failed to save completed session status");
                }
                out
            }
            Err(e) => {
                // Mark as failed
                session.status = SessionStatus::Failed;
                if let Err(save_err) = self.session_storage.save_session(session).await {
                    warn!(session_id, error = %save_err, "failed to save failed session status");
                }
                warn!(session_id, error = %e, "session compilation failed");
                return Err(e);
            }
        };

        Ok(output)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Try to extract the agent name from the Markdown frontmatter or first heading.
fn extract_agent_name(content: &str) -> Option<String> {
    // Priority 1: `# Agent: <name>` heading (UAR-AGENT-MD convention)
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("# Agent:") {
            let name = rest.trim().to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
        // Also accept bare `# <name>` headings
        if let Some(rest) = line.strip_prefix("# ") {
            let name = rest.trim().to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    // Priority 2: YAML frontmatter `agent_name:` or `name:` (only inside --- blocks)
    let mut in_frontmatter = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if in_frontmatter {
            if let Some(rest) = trimmed.strip_prefix("agent_name:") {
                let name = rest.trim().trim_matches('"').trim_matches('\'').to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
            if let Some(rest) = trimmed.strip_prefix("name:") {
                let name = rest.trim().trim_matches('"').trim_matches('\'').to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_agent_name_from_frontmatter() {
        let content = "---\nagent_name: my-agent\nversion: 1.0.0\n---\n## Identity\n";
        assert_eq!(extract_agent_name(content), Some("my-agent".to_string()));
    }

    #[test]
    fn test_extract_agent_name_from_heading() {
        let content = "# Customer Support Agent\n\n## Identity\n";
        assert_eq!(
            extract_agent_name(content),
            Some("Customer Support Agent".to_string())
        );
    }

    #[tokio::test]
    async fn test_store_and_retrieve_spec() {
        let svc = CompilerService::in_memory();
        let content = "# Test Agent\n\n## Identity\nname: test";
        let spec = svc.store_spec(content.to_string()).await.unwrap();
        assert_eq!(spec.name, "Test Agent");

        let fetched = svc.get_spec(&spec.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().content, content);
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        let svc = CompilerService::in_memory();

        let session = svc.create_session().await;
        assert_eq!(session.status, SessionStatus::Active);

        let fetched = svc.get_session(&session.id).await;
        assert!(fetched.is_some());

        let cancelled = svc.cancel_session(&session.id).await;
        assert!(cancelled);

        let after = svc.get_session(&session.id).await.unwrap();
        assert_eq!(after.status, SessionStatus::Cancelled);
    }
}

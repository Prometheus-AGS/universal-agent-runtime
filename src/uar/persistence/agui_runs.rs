//! AG-UI wire-run correlation. Host runs and approval records remain authoritative.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AguiRunBinding {
    pub owner: String,
    pub agent_id: String,
    pub thread_id: String,
    pub wire_run_id: String,
    pub native_run_id: String,
    pub session_id: String,
    pub workspace_id: Option<String>,
    pub input_digest: String,
    pub ordinal: u64,
    pub revision: u64,
    pub start_cursor: u64,
    pub previous_wire_run_id: Option<String>,
    pub resumed_tool_ids: Vec<String>,
    pub terminal: Option<AguiTerminal>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AguiTerminal {
    pub source_cursor: u64,
    pub event: serde_json::Value,
    pub interrupt_ids: Vec<String>,
}

impl AguiRunBinding {
    pub fn key(owner: &str, agent: &str, thread: &str, run: &str) -> String {
        Sha256::digest(format!("{}:{owner}{}:{agent}{}:{thread}{}:{run}",
            owner.len(), agent.len(), thread.len(), run.len()).as_bytes())
            .iter().map(|byte| format!("{byte:02x}")).collect::<String>()
    }
    pub fn storage_key(&self) -> String { Self::key(&self.owner, &self.agent_id, &self.thread_id, &self.wire_run_id) }
    pub fn context_key(&self) -> String { Self::key(&self.owner, &self.agent_id, &self.thread_id, "") }
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.ordinal > 0 && self.revision > 0, "invalid_agui_revision");
        for id in [&self.owner, &self.agent_id, &self.thread_id, &self.wire_run_id, &self.native_run_id, &self.session_id] {
            anyhow::ensure!(!id.trim().is_empty() && id.trim() == id.as_str() && !id.chars().any(char::is_control), "invalid_agui_identity");
        }
        Ok(())
    }
    pub fn validate_create(&self, previous: Option<&Self>) -> anyhow::Result<()> {
        self.validate()?;
        anyhow::ensure!(self.revision == 1 && self.terminal.is_none(), "invalid_agui_initial_state");
        match previous {
            Some(previous) => anyhow::ensure!(previous.owner == self.owner && previous.agent_id == self.agent_id
                && previous.thread_id == self.thread_id && previous.session_id == self.session_id
                && previous.workspace_id == self.workspace_id && previous.terminal.is_some()
                && self.previous_wire_run_id.as_deref() == Some(previous.wire_run_id.as_str())
                && previous.ordinal.checked_add(1) == Some(self.ordinal), "agui_context_conflict"),
            None => anyhow::ensure!(self.ordinal == 1 && self.previous_wire_run_id.is_none(), "agui_context_conflict"),
        }
        Ok(())
    }
    pub fn validate_transition(&self, next: &Self) -> anyhow::Result<()> {
        self.validate()?;
        next.validate()?;
        let mut identity = next.clone();
        identity.revision = self.revision;
        identity.terminal = self.terminal.clone();
        anyhow::ensure!(&identity == self && self.terminal.is_none() && next.terminal.is_some()
            && self.revision.checked_add(1) == Some(next.revision), "agui_projection_conflict");
        Ok(())
    }
}

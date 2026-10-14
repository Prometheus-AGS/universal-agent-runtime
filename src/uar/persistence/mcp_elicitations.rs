//! Durable MCP interaction history. These records confer no tool authority.
use chrono::{DateTime, Utc};
use rmcp::model::{ElicitRequestParams, ElicitResult, ElicitationAction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElicitationState { Pending, Answered, Cancelled, Disconnected, Expired }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpElicitationRecord {
    pub id: String,
    pub owner_key: String,
    pub run_id: String,
    pub session_id: String,
    pub workspace_id: Option<String>,
    pub channel_id: String,
    pub server_name: String,
    pub connection_id: String,
    pub request_id: String,
    pub request: ElicitRequestParams,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub state: ElicitationState,
    /// Persisted user decision; transport receipt is deliberately not asserted.
    pub response: Option<ElicitResult>,
}

impl McpElicitationRecord {
    pub fn validate(&self) -> anyhow::Result<()> {
        for id in [&self.id, &self.owner_key, &self.run_id, &self.session_id,
            &self.channel_id, &self.server_name, &self.connection_id, &self.request_id] {
            anyhow::ensure!(!id.is_empty() && id.trim() == id && !id.chars().any(char::is_control), "invalid_elicitation_identity");
        }
        anyhow::ensure!(self.expires_at > self.created_at, "invalid_elicitation_expiration");
        anyhow::ensure!((self.state == ElicitationState::Answered) == self.response.is_some(), "invalid_elicitation_disposition");
        if let Some(response) = &self.response { self.validate_response(response)?; }
        Ok(())
    }

    pub fn validate_response(&self, response: &ElicitResult) -> anyhow::Result<()> {
        anyhow::ensure!(response.meta.is_none(), "elicitation_response_metadata_not_supported");
        match response.action {
            ElicitationAction::Accept => match &self.request {
                ElicitRequestParams::FormElicitationParams { requested_schema, .. } => {
                    let schema = serde_json::to_value(requested_schema)?;
                    let content = response.content.as_ref().ok_or_else(|| anyhow::anyhow!("elicitation_content_required"))?;
                    let validator = jsonschema::options().should_validate_formats(true).build(&schema)
                        .map_err(|_| anyhow::anyhow!("invalid_elicitation_schema"))?;
                    anyhow::ensure!(validator.is_valid(content), "invalid_elicitation_content");
                }
                ElicitRequestParams::UrlElicitationParams { .. } => anyhow::ensure!(response.content.is_none(), "url_elicitation_has_no_content"),
                _ => anyhow::bail!("unsupported_elicitation_mode"),
            },
            ElicitationAction::Decline | ElicitationAction::Cancel => anyhow::ensure!(response.content.is_none(), "nonaccept_elicitation_has_no_content"),
            _ => anyhow::bail!("unsupported_elicitation_action"),
        }
        Ok(())
    }

    pub fn finish(&self, state: ElicitationState) -> Self {
        let mut next = self.clone(); next.state = state; next
    }

    pub fn answer(&self, response: ElicitResult) -> anyhow::Result<Self> {
        self.validate_response(&response)?;
        let mut next = self.clone(); next.state = ElicitationState::Answered; next.response = Some(response);
        self.validate_transition(&next)?; Ok(next)
    }

    pub fn validate_transition(&self, next: &Self) -> anyhow::Result<()> {
        self.validate()?; next.validate()?;
        let mut identity = next.clone(); identity.state = self.state.clone(); identity.response = self.response.clone();
        anyhow::ensure!(&identity == self && self.state == ElicitationState::Pending
            && next.state != ElicitationState::Pending, "elicitation_transition_conflict");
        Ok(())
    }
}

/// Public state projection. A cold pending request remains history, not a replay grant.
#[derive(Debug, Clone, Serialize)]
pub struct McpElicitationView { pub record: McpElicitationRecord, pub resolvable: bool }

//! Optional host-mediated MailCraft pilot. Only the fixed reviewed recipe consumes staged headers.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

mod provider;
pub(crate) use provider::run_job;

/// Definitive provider denials and uncertain execution outcomes stay distinguishable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum JobError {
    #[error("executor_unavailable")]
    ExecutorUnavailable,
    #[error("executor_busy")]
    ExecutorBusy,
    #[error("executor_pin_mismatch")]
    ExecutorPinMismatch,
    #[error("cleanup_unknown")]
    CleanupUnknown,
    #[error("outcome_unknown")]
    OutcomeUnknown,
    #[error("invalid_job_receipt")]
    InvalidJobReceipt,
    #[error("recipe_pin_mismatch")]
    RecipePinMismatch,
    #[error("job_rejected")]
    JobRejected,
    #[error("intent_scope_changed")]
    IntentScopeChanged,
    #[error("intent_unavailable")]
    IntentUnavailable,
    #[error("invalid_job_scope")]
    InvalidJobScope,
    #[error("invalid_header_projection")]
    InvalidHeaderProjection,
    #[error("invalid_job_configuration")]
    InvalidJobConfiguration,
    #[error("job_cancelled")]
    JobCancelled,
    #[error("job_failed")]
    JobFailed,
    #[error("invalid_job_result")]
    InvalidJobResult,
}

pub(crate) const RECIPE: &str = "14a6a558997f83ab816365d29fa6d25031a05c4e9d456a8d2797ad7fc1f5dc91";
pub(crate) const TOOL: &str = "unanswered_demo_report";
pub(crate) const MAX_PAGES: usize = 20;
const MAX_ROWS: usize = 2000;
const MAX_BYTES: usize = 2 * 1024 * 1024;

/// Trusted startup settings. Omitting the entire setting selects ordinary direct MCP.
#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DemoConfig {
    pub issuer_kind: String,
    pub issuer: String,
    pub subject: String,
    pub tenant: Option<String>,
    pub mailcraft_tool: String,
    pub mailcraft_command: String,
    pub mailbox_file: String,
    pub header_key_file: Option<String>,
    pub account: u64,
    pub folders: Vec<u64>,
    pub cutoff: i64,
    pub bunbox_command: PathBuf,
    pub bunbox_sha256: String,
    pub state_dir: PathBuf,
    pub msb_command: PathBuf,
    pub msb_home: PathBuf,
}

impl DemoConfig {
    pub(crate) fn matches_owner(
        &self,
        owner: &crate::uar::security::claims::CredentialPrincipal,
    ) -> bool {
        owner.issuer_kind() == self.issuer_kind
            && owner.issuer() == self.issuer
            && owner.subject() == self.subject
            && owner.tenant_id() == self.tenant.as_deref()
    }

    /// A declaration reserves a candidate identity before the ordinary policy intersection.
    /// Executing it still requires the separately captured, owner-bound live pager.
    pub(crate) fn matches_declaration(&self, server: &super::catalog::ServerDefinition) -> bool {
        let crate::mcp::config::McpServerEntry::Stdio {
            command, args, env, ..
        } = server.configuration()
        else {
            return false;
        };
        let ids = self
            .folders
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let mut expected = vec![
            "mcp".into(),
            "--file".into(),
            self.mailbox_file.clone(),
            "--header-scope".into(),
            format!("{}:{ids}", self.account),
            "--header-cutoff".into(),
            self.cutoff.to_string(),
        ];
        if let Some(key) = &self.header_key_file {
            expected.extend(["--header-key-file".into(), key.clone()]);
        }
        self.names_connector(server.name())
            && env.is_empty()
            && std::path::Path::new(command).is_absolute()
            && std::path::Path::new(&self.mailbox_file).is_absolute()
            && self
                .header_key_file
                .as_ref()
                .is_none_or(|p| std::path::Path::new(p).is_absolute())
            && command == &self.mailcraft_command
            && args == &expected
            && (1..=2).contains(&self.folders.len())
            && self.folders.iter().collect::<BTreeSet<_>>().len() == self.folders.len()
    }
    fn names_connector(&self, server: &str) -> bool {
        super::registry::McpRegistry::sanitize_tool_name(&format!("{server}__page_message_headers"))
            == self.mailcraft_tool
    }

    pub(crate) fn hides_raw(
        &self,
        descriptor: &crate::uar::tools::descriptor::ToolDescriptor,
    ) -> bool {
        descriptor.source == crate::uar::tools::descriptor::ToolSource::Mcp
            && (descriptor.provider_name == self.mailcraft_tool
                || descriptor
                    .server
                    .as_deref()
                    .is_some_and(|server| self.names_connector(server)))
    }

    pub(crate) async fn authorize(
        &self,
        owner: Option<&crate::uar::runtime::actor::messages::ActorOwner>,
        policy: &crate::uar::domain::policy::EffectiveRunPolicy,
        mode: &crate::uar::domain::artifact::ToolExecutionMode,
        preflight: Option<&super::preflight::McpPreflight>,
    ) -> Result<(), &'static str> {
        use crate::uar::domain::{artifact::ToolExecutionMode, policy::SelectionMode};
        use crate::uar::tools::descriptor::{Exposure, ToolEffect};
        let preflight = preflight.ok_or("connector_unavailable")?;
        if owner != Some(preflight.owner())
            || !owner
                .and_then(|o| o.credential_principal())
                .is_some_and(|p| self.matches_owner(p))
        {
            return Err("denied_owner");
        }
        if matches!(mode, ToolExecutionMode::Sandboxed)
            || policy.tools.mode == SelectionMode::None
            || !policy.tools.ids.iter().any(|name| name == TOOL)
        {
            return Err("report_not_granted");
        }
        let pager = preflight
            .projection()
            .tools()
            .get(&self.mailcraft_tool)
            .ok_or("connector_revoked")?;
        let descriptor = pager.descriptor();
        if !self.matches_declaration(pager.server())
            || descriptor.id != format!("{}::page_message_headers", pager.server().name())
            || descriptor.provider_name != self.mailcraft_tool
            || descriptor.effect != ToolEffect::ReadOnly
            || descriptor.sandbox_required
            || policy.mcp_servers.mode == SelectionMode::None
            || (policy.mcp_servers.mode == SelectionMode::Selected
                && !policy
                    .mcp_servers
                    .ids
                    .iter()
                    .any(|id| id == pager.server().name()))
            || matches!(descriptor.exposure, Exposure::Hidden | Exposure::ModelOnly)
            || (policy.tools.mode == SelectionMode::Selected
                && !policy
                    .tools
                    .ids
                    .iter()
                    .any(|id| id == &descriptor.provider_name || id == &descriptor.id))
        {
            return Err("connector_scope_changed");
        }
        let binding = preflight
            .servers()
            .get(pager.server().name())
            .ok_or("connector_unavailable")?
            .wait_until_ready()
            .await
            .map_err(|_| "connector_unavailable")?;
        binding.registry().map_err(|_| "connector_revoked")?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Coverage {
    pub inbox: bool,
    pub sent: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Header {
    source_ref: String,
    thread_ref: Option<String>,
    timestamp: i64,
    direction: String,
    subject: String,
    #[serde(skip_serializing)]
    subject_truncated: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Page {
    rows: Vec<Header>,
    next_cursor: Option<String>,
    coverage: Coverage,
    cutoff: i64,
}

pub(crate) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn is_ref(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Default)]
pub(crate) struct Scan {
    rows: Vec<Header>,
    // Only inbound rows may appear as request evidence in the final report.
    refs: BTreeMap<String, (bool, Option<String>)>,
    cursors: BTreeSet<String>,
    coverage: Option<Coverage>,
    bytes: usize,
}

impl Scan {
    pub(crate) fn add(
        &mut self,
        value: &Value,
        cutoff: i64,
    ) -> Result<Option<String>, &'static str> {
        let content = value
            .get("content")
            .and_then(Value::as_array)
            .filter(|v| v.len() == 1)
            .and_then(|v| v.first())
            .filter(|v| v["type"] == "text")
            .and_then(|v| v.get("text"))
            .and_then(Value::as_str)
            .ok_or("invalid_header_page")?;
        if content.len() > 256 * 1024 {
            return Err("page_size_exceeded");
        }
        let page: Page = serde_json::from_str(content).map_err(|_| "invalid_header_projection")?;
        if page.cutoff != cutoff
            || self.coverage.is_some_and(|c| c != page.coverage)
            || page.rows.len() > 100
        {
            return Err("changed_scope_or_coverage");
        }
        if page
            .next_cursor
            .as_ref()
            .is_some_and(|c| !is_ref(c) || page.rows.is_empty() || !self.cursors.insert(c.clone()))
        {
            return Err("invalid_or_repeated_cursor");
        }
        if self.rows.len() + page.rows.len() > MAX_ROWS {
            return Err("row_budget_exceeded");
        }
        self.coverage = Some(page.coverage);
        for row in page.rows {
            if !is_ref(&row.source_ref)
                || row.thread_ref.as_deref().is_some_and(|id| !is_ref(id))
                || row.timestamp > cutoff
                || row.subject.len() > 256
                || row.subject_truncated
                || !matches!(
                    row.direction.as_str(),
                    "inbound" | "outbound" | "auto" | "draft" | "unknown"
                )
                || self
                    .refs
                    .insert(
                        row.source_ref.clone(),
                        (row.direction == "inbound", row.thread_ref.clone()),
                    )
                    .is_some()
            {
                return Err("invalid_or_incomplete_header");
            }
            self.bytes += serde_json::to_vec(&row)
                .map_err(|_| "invalid_header_projection")?
                .len()
                + 1;
            if self.bytes > MAX_BYTES {
                return Err("input_budget_exceeded");
            }
            self.rows.push(row);
        }
        Ok(page.next_cursor)
    }
    pub(crate) fn take_input(&mut self, cutoff: i64) -> Result<Value, &'static str> {
        let input =
            json!({"rows":std::mem::take(&mut self.rows),"coverage":self.coverage,"cutoff":cutoff});
        if serde_json::to_vec(&input)
            .map_err(|_| "invalid_header_projection")?
            .len()
            > MAX_BYTES
        {
            return Err("input_budget_exceeded");
        }
        Ok(input)
    }
    pub(crate) fn checked_result(&self, raw: Value) -> Result<Value, &'static str> {
        #[derive(Deserialize, Serialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Finding {
            thread_ref: String,
            source_ref: String,
        }
        #[derive(Deserialize, Serialize)]
        #[serde(deny_unknown_fields)]
        struct Report {
            unanswered: Vec<Finding>,
            unassigned: Vec<String>,
            coverage: String,
            truncated: bool,
        }
        let report: Report = serde_json::from_value(raw).map_err(|_| "invalid_job_result")?;
        let complete = self.coverage
            == Some(Coverage {
                inbox: true,
                sent: true,
            });
        let mut threads = BTreeSet::new();
        let mut sources = BTreeSet::new();
        if report.unanswered.len() > 20
            || report.unassigned.len() > 20
            || report.coverage != if complete { "complete" } else { "partial" }
            || (!complete && !report.unanswered.is_empty())
            || report.unanswered.iter().any(|f| {
                !matches!(self.refs.get(&f.source_ref), Some((true, Some(thread))) if thread == &f.thread_ref)
                    || !threads.insert(&f.thread_ref)
                    || !sources.insert(&f.source_ref)
            })
            || report
                .unassigned
                .iter()
                .any(|id| !matches!(self.refs.get(id), Some((true, _))) || !sources.insert(id))
        {
            return Err("invalid_job_result");
        }
        serde_json::to_value(report).map_err(|_| "invalid_job_result")
    }
}

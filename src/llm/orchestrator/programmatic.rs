//! Host-side staging for one reviewed operation. The guest never gets a connector.
use super::*;
use crate::mcp::programmatic::{self as demo, DemoConfig, Scan};
use crate::uar::runtime::native_skill::{NativeExecutionContext, NativeSkill};
use crate::uar::runtime::tool_admission::AdmittedToolInvocation;
use serde_json::{Value, json};

#[derive(Debug, thiserror::Error)]
pub(super) enum StagedError {
    #[error("{0}")]
    Input(&'static str),
    #[error(transparent)]
    Job(#[from] demo::JobError),
}

impl StagedError {
    pub(super) fn outcome(&self) -> ToolOutcome {
        match self {
            Self::Job(error) => ToolOutcome::from(*error),
            Self::Input(_) => ToolOutcome::ApplicationFailed,
        }
    }
}

impl Orchestrator {
    pub(crate) fn with_demo_config(mut self, config: Option<DemoConfig>) -> Self {
        self.demo_config = config.map(Arc::new);
        self
    }
    pub(super) fn demo_scope(&self) -> Option<&Arc<DemoConfig>> {
        let config = self.demo_config.as_ref()?;
        let principal = self
            .resolved_turn
            .as_ref()?
            .verified_owner()?
            .credential_principal()?;
        (self.thread_policy.is_none() && config.matches_owner(principal)).then_some(config)
    }
    pub(crate) fn demo_descriptor(
        registry: &NativeSkillRegistry,
    ) -> Result<Arc<ToolDescriptor>, crate::uar::tools::descriptor::ToolAssemblyError> {
        registry.compile_descriptor(&StagedReport(None))
    }
    pub(super) async fn demo_authorized(&self, config: &DemoConfig) -> bool {
        let Some(turn) = &self.resolved_turn else {
            return false;
        };
        let preflight = if let Some(activation) = &self.skill_activation {
            activation.context.lock().await.mcp_preflight().cloned()
        } else {
            self.mcp_preflight.clone()
        };
        !self.tool_admission.is_cancelled()
            && config
                .authorize(
                    turn.verified_owner(),
                    turn.policy(),
                    &self.tool_execution_mode,
                    preflight.as_deref(),
                )
                .await
                .is_ok()
    }
    pub(super) async fn register_demo(&self) -> anyhow::Result<bool> {
        let Some(config) = self.demo_scope() else {
            return Ok(false);
        };
        if !self.demo_authorized(config).await {
            return Ok(false);
        }
        anyhow::ensure!(
            !self.assembled_descriptors().await?.contains_key(demo::TOOL),
            "Code Mode descriptor collision"
        );
        let mut host = self.clone();
        // Holding the handler's eventual registry here would keep the stream alive.
        // Inner calls are always host MCP operations with an exact preflight binding.
        host.native_skills = Arc::new(NativeSkillRegistry::new());
        self.native_skills
            .register(StagedReport(Some(DemoTool {
                host,
                config: Arc::clone(config),
            })))
            .await?;
        Ok(true)
    }
}

struct DemoTool {
    host: Orchestrator,
    config: Arc<DemoConfig>,
}

impl DemoTool {
    async fn binding(&self) -> Result<Arc<crate::mcp::preflight::McpPreflight>, StagedError> {
        if self.host.tool_admission.is_cancelled() {
            return Err(StagedError::Input("run_cancelled"));
        }
        let turn = self
            .host
            .resolved_turn
            .as_ref()
            .ok_or(StagedError::Input("missing_turn"))?;
        let preflight = if let Some(activation) = &self.host.skill_activation {
            activation.context.lock().await.mcp_preflight().cloned()
        } else {
            self.host.mcp_preflight.clone()
        }
        .ok_or(StagedError::Input("connector_unavailable"))?;
        self.config
            .authorize(
                turn.verified_owner(),
                turn.policy(),
                &self.host.tool_execution_mode,
                Some(&preflight),
            )
            .await
            .map_err(StagedError::Input)?;
        Ok(preflight)
    }

    async fn read_page(&self, index: usize, cursor: Option<&str>) -> Result<Value, StagedError> {
        let preflight = self.binding().await?;
        let tool = &self.config.mailcraft_tool;
        let descriptor = preflight
            .projection()
            .tools()
            .get(tool)
            .ok_or(StagedError::Input("connector_revoked"))?
            .descriptor();
        let mut args = json!({"account":self.config.account,"folders":self.config.folders,"cutoff":self.config.cutoff,
            "fields":["subject"],"limit":100});
        if let Some(c) = cursor {
            args["cursor"] = Value::String(c.to_string());
        }
        let validated = validate::validate(&descriptor.validator, &args.to_string())
            .map_err(|_| StagedError::Input("invalid_connector_schema"))?;
        if self.host.tool_approval_gate.is_none() {
            return Err(StagedError::Input("missing_approval_channel"));
        }
        let admitted: AdmittedToolInvocation = self
            .host
            .prepare_and_admit_tool(Uuid::new_v4().to_string(), descriptor, validated, index)
            .await
            .map_err(|_| StagedError::Input("read_not_admitted"))?;
        self.host
            .ensure_tool_dispatch(&admitted, ToolExecutionKind::HostMcp)
            .await
            .map_err(|_| StagedError::Input("read_revoked"))?;
        let read = preflight
            .call_tool(
                tool,
                admitted.prepared.validated_arguments.clone(),
                admitted.host_receipt.mcp_request_meta(),
            )
            .await;
        let outcome = ToolOutcome::from_mcp_result(&read);
        self.host
            .tool_admission
            .finish_outcome(&admitted, outcome.into())
            .await
            .map_err(|_| StagedError::Input("read_receipt_unavailable"))?;
        // Connector bodies do not pass through canonical preservation or model-history emitters.
        read.map_err(|_| StagedError::Input("read_failed"))
    }

    async fn run(&self, ctx: &NativeExecutionContext) -> Result<Value, StagedError> {
        let turn = self
            .host
            .resolved_turn
            .as_ref()
            .ok_or(StagedError::Input("missing_turn"))?;
        if ctx.verified_owner.as_ref() != turn.verified_owner() || ctx.thread_policy.is_some() {
            return Err(StagedError::Input("denied_owner"));
        }
        let principal = turn
            .verified_owner()
            .and_then(|o| o.credential_principal())
            .ok_or(StagedError::Input("denied_owner"))?;
        if !self.config.matches_owner(principal) {
            return Err(StagedError::Input("denied_owner"));
        }
        let owner = demo::digest(principal.storage_key().as_bytes());
        let run = &turn.environment().run_id;
        let mut scan = Scan::default();
        let mut cursor = None;
        for index in 0..demo::MAX_PAGES {
            let page = self.read_page(index, cursor.as_deref()).await?;
            cursor = scan
                .add(&page, self.config.cutoff)
                .map_err(StagedError::Input)?;
            if cursor.is_none() {
                break;
            }
        }
        if cursor.is_some() {
            return Err(StagedError::Input("page_budget_exceeded"));
        }
        let input = scan
            .take_input(self.config.cutoff)
            .map_err(StagedError::Input)?;
        self.binding().await?;
        let receipt = demo::run_job(
            &self.config,
            &owner,
            run,
            input,
            &self.host.tool_admission.cancellation_token(),
        )
        .await?;
        self.binding().await?;
        let result = scan
            .checked_result(receipt.result)
            .map_err(StagedError::Input)?;
        Ok(
            json!({"status":"succeeded","recipeSha256":demo::RECIPE,"jobId":receipt.id,"cutoff":self.config.cutoff,
            "evidenceDigest":demo::digest(result.to_string().as_bytes()),"result":result}),
        )
    }
}

// The manifest needs only the descriptor. An executable instance also holds its
// run-owned host; schema, effect and approval classification come from one implementation.
struct StagedReport(Option<DemoTool>);

#[async_trait::async_trait]
impl NativeSkill for StagedReport {
    fn name(&self) -> &str {
        demo::TOOL
    }
    fn description(&self) -> &str {
        "Find unanswered demo requests within the operator-approved mailbox scope. Returns scoped evidence handles and coverage. The job cannot send mail."
    }
    fn parameters_schema(&self) -> Value {
        json!({"type":"object","properties":{},"additionalProperties":false})
    }
    fn effect(&self) -> ToolEffect {
        ToolEffect::ExternalMutation
    }
    fn exposure(&self) -> Exposure {
        Exposure::ModelOnly
    }
    fn source(&self) -> crate::uar::tools::descriptor::ToolSource {
        crate::uar::tools::descriptor::ToolSource::BuiltIn
    }
    async fn execute(&self, _: Value) -> anyhow::Result<Value> {
        Err(StagedError::Input("missing_run_context").into())
    }
    async fn execute_with_context(
        &self,
        _: Value,
        ctx: &NativeExecutionContext,
    ) -> anyhow::Result<Value> {
        self.0
            .as_ref()
            .ok_or(StagedError::Input("missing_run_context"))?
            .run(ctx)
            .await
            .map_err(Into::into)
    }
}

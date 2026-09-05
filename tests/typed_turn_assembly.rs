//! Integration coverage for the owned turn request and staged assembler.

use std::sync::Arc;

use tokio::sync::RwLock;
use universal_agent_runtime::config::LlmConfig;
use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
use universal_agent_runtime::mcp::registry::McpRegistry;
use universal_agent_runtime::normalized::NormalizedEvent as DriverEvent;
use universal_agent_runtime::session::SessionStore;
use universal_agent_runtime::uar::defaults::default_agent;
use universal_agent_runtime::uar::domain::events::{MemoryItem, NormalizedEvent as RunEvent};
use universal_agent_runtime::uar::domain::policy::{PolicyResolutionInput, resolve_run_policy};
use universal_agent_runtime::uar::rag::embeddings::{
    EmbeddingBackend, UnavailableEmbeddingBackend,
};
use universal_agent_runtime::uar::runtime::manager::RunManager;
use universal_agent_runtime::uar::runtime::matching::VectorMatcher;
use universal_agent_runtime::uar::runtime::prompt::PromptBudgets;
use universal_agent_runtime::uar::runtime::skills::SkillRegistry;
use universal_agent_runtime::uar::runtime::turn::{
    RunExecutionRequest,
    contributors::{
        AssemblyError, AssemblyInputs, AssemblyState, Contribution, ContributorRegistry,
        McpToolsContributor,
    },
};
use universal_agent_runtime::uar::tools::{
    descriptor::{ApprovalClass, Exposure, ToolDescriptor, ToolEffect, ToolSource},
    validate::ValidatorCompiler,
};

async fn manager(driver: Arc<MockLlmDriver>) -> Arc<RunManager> {
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are outside this integration boundary",
    ));
    Arc::new(
        RunManager::new(
            LlmConfig {
                model: "openai/gpt-4o".to_string(),
                api_key: Some("typed-turn-fixture-key".to_string()),
                base_url: Some("http://typed-turn.invalid/v1".to_string()),
                ..LlmConfig::default()
            },
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            Arc::new(RwLock::new(SkillRegistry::default())),
            Arc::new(VectorMatcher::new(embeddings, 0.75)),
            None,
        )
        .await
        .with_llm_driver(driver),
    )
}

async fn wait_for_completion(manager: &RunManager, run_id: &str) {
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            let complete = manager
                .history_since(run_id, None)
                .await
                .expect("started run keeps event history")
                .iter()
                .any(|event| matches!(event.event, RunEvent::RunDone { .. }));
            if complete {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("run completes through the mock driver");
}

struct UnauthorizedToolContributor(Arc<ToolDescriptor>);

#[async_trait::async_trait]
impl McpToolsContributor for UnauthorizedToolContributor {
    fn name(&self) -> &str {
        "unauthorized_tool_fixture"
    }

    async fn contribute(
        &self,
        _: &AssemblyInputs,
        _: &AssemblyState,
    ) -> Result<Contribution, AssemblyError> {
        Ok(Contribution {
            tools: vec![Arc::clone(&self.0)],
            ..Contribution::default()
        })
    }
}

fn unauthorized_tool() -> Arc<ToolDescriptor> {
    let input_schema = serde_json::json!({
        "type": "object",
        "additionalProperties": false,
    });
    let validator = ValidatorCompiler::default()
        .compile("unauthorized_tool", &input_schema)
        .expect("fixture schema compiles");
    Arc::new(ToolDescriptor {
        id: "unauthorized_tool".to_string(),
        provider_name: "unauthorized_tool".to_string(),
        description: "must remain outside the effective policy".to_string(),
        source: ToolSource::BuiltIn,
        server: None,
        input_schema,
        validator,
        effect: ToolEffect::ReadOnly,
        approval_class: ApprovalClass::NotRequired,
        sandbox_required: false,
        concurrency_key: None,
        exposure: Exposure::Eager,
        output_limit: None,
    })
}

#[tokio::test]
async fn legacy_arguments_and_owned_request_resolve_to_the_same_dispatched_turn() {
    let driver = Arc::new(MockLlmDriver::new(vec![vec![DriverEvent::Done]]));
    let manager = manager(Arc::clone(&driver)).await;
    let artifact = default_agent();
    let input = "compare the turn adapters".to_string();
    let owner = Some("typed-turn-owner".to_string());

    let legacy_run = manager
        .start_run_with_policy_and_history(
            artifact.clone(),
            input.clone(),
            Some("legacy-adapter-session".to_string()),
            owner.clone(),
            Vec::new(),
            None,
            Vec::new(),
        )
        .await;
    wait_for_completion(&manager, &legacy_run).await;

    let mut request = RunExecutionRequest::new(artifact, input);
    request.session_id = Some("owned-request-session".to_string());
    request.user_id = owner;
    let owned_run = manager.execute_request(request).await;
    wait_for_completion(&manager, &owned_run).await;

    let requests = driver.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].messages, requests[1].messages);
    assert_eq!(requests[0].tools, requests[1].tools);
    assert_eq!(requests[0].anthropic_system, requests[1].anthropic_system);
    assert_eq!(requests[0].extra_params, requests[1].extra_params);

    let legacy = manager
        .get_run(&legacy_run)
        .await
        .expect("legacy-adapter run remains inspectable");
    let owned = manager
        .get_run(&owned_run)
        .await
        .expect("owned-request run remains inspectable");
    assert_eq!(
        legacy.context["effective_run_policy"],
        owned.context["effective_run_policy"]
    );
    assert_eq!(
        legacy.context["turn_manifest"], owned.context["turn_manifest"],
        "fragment identities, hashes, ordering, and budgets must match"
    );
}

#[tokio::test]
async fn contributor_cannot_add_a_tool_outside_the_effective_policy() {
    let registry = ContributorRegistry {
        mcp_tools: vec![Arc::new(UnauthorizedToolContributor(unauthorized_tool()))],
        ..ContributorRegistry::default()
    };
    let inputs = AssemblyInputs {
        artifact: default_agent(),
        policy: resolve_run_policy(PolicyResolutionInput::default()),
        memory_hits: Vec::new(),
        prepared_fragments: Vec::new(),
        history: Vec::new(),
        prepared_history: None,
        authorized_tools: Default::default(),
        active_skills: Vec::new(),
        budgets: PromptBudgets::default(),
    };

    let error = registry
        .assemble(&inputs)
        .await
        .expect_err("an unadmitted tool must fail assembly");

    assert!(matches!(
        error,
        AssemblyError::OutsidePolicy { resource, id }
            if resource == "tools" && id == "unauthorized_tool"
    ));
}

#[tokio::test]
async fn direct_start_run_contributes_memory_to_the_model_turn() {
    let driver = Arc::new(MockLlmDriver::new(vec![vec![DriverEvent::Done]]));
    let manager = manager(Arc::clone(&driver)).await;
    let memory_value = "typed memory contribution sentinel";
    let run_id = manager
        .start_run(
            default_agent(),
            "use recalled context".to_string(),
            Some("direct-memory-session".to_string()),
            Some("typed-turn-owner".to_string()),
            vec![MemoryItem {
                key: "memory-1".to_string(),
                value: memory_value.to_string(),
                source: "memory_context".to_string(),
                scope: Some("session".to_string()),
                memory_type: Some("semantic".to_string()),
                importance: Some(0.9),
            }],
        )
        .await;
    wait_for_completion(&manager, &run_id).await;

    let requests = driver.requests();
    assert_eq!(requests.len(), 1);
    let messages = serde_json::to_string(&requests[0].messages).expect("request serializes");
    assert!(messages.contains("[MEMORY CONTEXT]"));
    assert!(messages.contains(memory_value));

    let run = manager
        .get_run(&run_id)
        .await
        .expect("completed direct run remains inspectable");
    assert!(
        run.context["turn_manifest"]["fragments"]
            .as_array()
            .expect("manifest fragments are an array")
            .iter()
            .any(|fragment| fragment["id"] == "retrieved.memory")
    );
}

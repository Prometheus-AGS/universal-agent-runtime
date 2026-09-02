//! Integration coverage for fail-closed tool-call assembly and execution.
//!
//! These tests exercise the public orchestrator and registry surfaces. They
//! deliberately use scripted provider turns so the assertions cover the same
//! model-visible tool result that is sent back on the next request.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use rmcp::model::ToolAnnotations;
use serde_json::{Value, json};
use tokio::sync::RwLock;
use universal_agent_runtime::session::SessionStore;
use universal_agent_runtime::uar::runtime::context::truncate::TruncationPolicy;
use universal_agent_runtime::{
    config::LlmConfig,
    llm::{Message, MessageRole, Orchestrator, mock_driver::MockLlmDriver},
    mcp::registry::McpRegistry,
    normalized::NormalizedEvent,
    uar::{
        defaults,
        domain::{events::NormalizedEvent as RunEvent, policy::ToolApprovalPolicy},
        rag::embeddings::{EmbeddingBackend, UnavailableEmbeddingBackend},
        runtime::native_skill::{NativeSkill, NativeSkillRegistry},
        runtime::{manager::RunManager, skills::SkillRegistry},
        tools::{
            descriptor::{ApprovalClass, ToolAssemblyError, ToolCollision, ToolEffect},
            validate::ValidatorCompiler,
        },
    },
};

struct CountingPathSkill {
    executions: Arc<AtomicUsize>,
}

#[async_trait]
impl NativeSkill for CountingPathSkill {
    fn name(&self) -> &str {
        "path_tool"
    }

    fn description(&self) -> &str {
        "Reads one path"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": { "type": "string" }
            },
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: Value) -> anyhow::Result<Value> {
        self.executions.fetch_add(1, Ordering::SeqCst);
        Ok(arguments)
    }
}

fn tool_turn(id: &str, name: &str, arguments: &str) -> Vec<NormalizedEvent> {
    tool_batch(&[(id, name, arguments)])
}

fn tool_batch(calls: &[(&str, &str, &str)]) -> Vec<NormalizedEvent> {
    let mut events = Vec::with_capacity(calls.len() * 2 + 1);
    for (call_index, (id, name, arguments)) in calls.iter().enumerate() {
        events.push(NormalizedEvent::ToolCallDelta {
            call_index,
            id: Some((*id).to_string()),
            name: Some((*name).to_string()),
            arguments_delta: Some((*arguments).to_string()),
        });
        events.push(NormalizedEvent::ToolCallComplete {
            call_index,
            id: (*id).to_string(),
            name: (*name).to_string(),
            arguments_json: (*arguments).to_string(),
        });
    }
    events.push(NormalizedEvent::Done);
    events
}

fn final_turn() -> Vec<NormalizedEvent> {
    vec![
        NormalizedEvent::MessageDelta {
            text: "done".to_string(),
        },
        NormalizedEvent::Done,
    ]
}

fn model_visible_tool_result(events: &[NormalizedEvent], id: &str) -> (Value, bool) {
    events
        .iter()
        .find_map(|event| match event {
            NormalizedEvent::ToolResult {
                id: result_id,
                content,
                success,
                ..
            } if result_id == id => Some((
                serde_json::from_str(content).expect("tool result is structured JSON"),
                *success,
            )),
            _ => None,
        })
        .expect("tool result event is emitted")
}

fn history_tool_result(request: &universal_agent_runtime::llm::LlmRequest, id: &str) -> Value {
    request
        .messages
        .iter()
        .cloned()
        .map(serde_json::from_value::<Message>)
        .collect::<Result<Vec<_>, _>>()
        .expect("provider history remains typed")
        .into_iter()
        .find(|message| {
            message.role == MessageRole::Tool && message.tool_call_id.as_deref() == Some(id)
        })
        .and_then(|message| message.content.as_text().map(str::to_owned))
        .map(|content| serde_json::from_str(&content).expect("history result is structured JSON"))
        .expect("failed tool result reaches the next provider request")
}

#[tokio::test]
async fn malformed_json_is_model_visible_and_never_executes_the_tool() {
    let executions = Arc::new(AtomicUsize::new(0));
    let native = Arc::new(NativeSkillRegistry::new());
    native
        .register(CountingPathSkill {
            executions: Arc::clone(&executions),
        })
        .await
        .expect("valid descriptor registers");

    let driver = Arc::new(MockLlmDriver::new(vec![
        tool_turn("malformed", "path_tool", r#"{"path": "#),
        final_turn(),
    ]));
    let orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::new_empty()),
        native,
        driver.clone(),
    );

    let events = orchestrator
        .chat("read a path")
        .await
        .expect("descriptor assembly succeeds")
        .collect::<Vec<_>>()
        .await;

    let (result, success) = model_visible_tool_result(&events, "malformed");
    assert!(!success);
    assert_eq!(result["type"], "invalid_arguments");
    assert!(
        result["message"]
            .as_str()
            .is_some_and(|message| { message.contains("EOF") || message.contains("end of input") }),
        "parse error is preserved: {result}"
    );
    assert_eq!(executions.load(Ordering::SeqCst), 0);

    let requests = driver.requests();
    assert_eq!(requests.len(), 2, "the model gets one retry turn");
    assert_eq!(history_tool_result(&requests[1], "malformed"), result);
}

#[tokio::test]
async fn schema_invalid_json_is_model_visible_and_never_executes_the_tool() {
    let executions = Arc::new(AtomicUsize::new(0));
    let native = Arc::new(NativeSkillRegistry::new());
    native
        .register(CountingPathSkill {
            executions: Arc::clone(&executions),
        })
        .await
        .expect("valid descriptor registers");

    let driver = Arc::new(MockLlmDriver::new(vec![
        tool_turn("schema-invalid", "path_tool", r#"{"path":42}"#),
        final_turn(),
    ]));
    let orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::new_empty()),
        native,
        driver.clone(),
    );

    let events = orchestrator
        .chat("read a path")
        .await
        .expect("descriptor assembly succeeds")
        .collect::<Vec<_>>()
        .await;

    let (result, success) = model_visible_tool_result(&events, "schema-invalid");
    assert!(!success);
    assert_eq!(result["type"], "invalid_arguments");
    assert!(
        result["message"]
            .as_str()
            .is_some_and(|message| message.contains("string")),
        "validator message is preserved: {result}"
    );
    assert_eq!(executions.load(Ordering::SeqCst), 0);

    let requests = driver.requests();
    assert_eq!(requests.len(), 2, "the model gets one retry turn");
    assert_eq!(history_tool_result(&requests[1], "schema-invalid"), result);
}

#[derive(Default)]
struct SchedulingProbe {
    active: AtomicUsize,
    max_active: AtomicUsize,
    timeline: Mutex<Vec<String>>,
}

struct ProbeSkill {
    name: String,
    effect: ToolEffect,
    approval_class: ApprovalClass,
    concurrency_key: Option<String>,
    probe: Arc<SchedulingProbe>,
}

#[async_trait]
impl NativeSkill for ProbeSkill {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Records scheduler overlap"
    }

    fn parameters_schema(&self) -> Value {
        json!({"type": "object", "additionalProperties": false})
    }

    fn effect(&self) -> ToolEffect {
        self.effect
    }

    fn approval_class(&self) -> ApprovalClass {
        self.approval_class
    }

    fn concurrency_key(&self) -> Option<&str> {
        self.concurrency_key.as_deref()
    }

    async fn execute(&self, _arguments: Value) -> anyhow::Result<Value> {
        self.probe
            .timeline
            .lock()
            .expect("timeline lock")
            .push(format!("start:{}", self.name));
        let active = self.probe.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.probe.max_active.fetch_max(active, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(40)).await;
        self.probe.active.fetch_sub(1, Ordering::SeqCst);
        self.probe
            .timeline
            .lock()
            .expect("timeline lock")
            .push(format!("end:{}", self.name));
        Ok(json!({"name": self.name}))
    }
}

async fn run_probe_pair(
    first_effect: ToolEffect,
    first_key: Option<&str>,
    second_effect: ToolEffect,
    second_key: Option<&str>,
) -> Arc<SchedulingProbe> {
    let probe = Arc::new(SchedulingProbe::default());
    let native = Arc::new(NativeSkillRegistry::new());
    for (name, effect, concurrency_key) in [
        ("probe_first", first_effect, first_key),
        ("probe_second", second_effect, second_key),
    ] {
        let approval_class = match effect {
            ToolEffect::ReadOnly => ApprovalClass::NotRequired,
            ToolEffect::ExternalMutation | ToolEffect::CodeExecution | ToolEffect::Unknown => {
                ApprovalClass::Required
            }
        };
        native
            .register(ProbeSkill {
                name: name.to_string(),
                effect,
                approval_class,
                concurrency_key: concurrency_key.map(str::to_owned),
                probe: Arc::clone(&probe),
            })
            .await
            .expect("probe descriptor registers");
    }

    let driver = Arc::new(MockLlmDriver::new(vec![
        tool_batch(&[
            ("probe-1", "probe_first", "{}"),
            ("probe-2", "probe_second", "{}"),
        ]),
        final_turn(),
    ]));
    let orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            parallel_tool_calls: Some(true),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::new_empty()),
        native,
        driver.clone(),
    );

    let events = orchestrator
        .chat("run both probes")
        .await
        .expect("descriptor assembly succeeds")
        .collect::<Vec<_>>()
        .await;
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, NormalizedEvent::ToolResult { .. }))
            .count(),
        2
    );
    assert_eq!(driver.requests().len(), 2);
    probe
}

fn assert_call_order(probe: &SchedulingProbe) {
    assert_eq!(
        *probe.timeline.lock().expect("timeline lock"),
        [
            "start:probe_first",
            "end:probe_first",
            "start:probe_second",
            "end:probe_second",
        ]
    );
}

#[tokio::test]
async fn descriptor_effect_and_keys_govern_parallel_execution() {
    let distinct = run_probe_pair(
        ToolEffect::ReadOnly,
        Some("alpha"),
        ToolEffect::ReadOnly,
        Some("beta"),
    )
    .await;
    assert_eq!(distinct.max_active.load(Ordering::SeqCst), 2);

    let absent = run_probe_pair(
        ToolEffect::ReadOnly,
        Some("alpha"),
        ToolEffect::ReadOnly,
        None,
    )
    .await;
    assert_eq!(absent.max_active.load(Ordering::SeqCst), 2);

    let same = run_probe_pair(
        ToolEffect::ReadOnly,
        Some("shared"),
        ToolEffect::ReadOnly,
        Some("shared"),
    )
    .await;
    assert_eq!(same.max_active.load(Ordering::SeqCst), 1);
    assert_call_order(&same);

    let unknown = run_probe_pair(ToolEffect::ReadOnly, None, ToolEffect::Unknown, None).await;
    assert_eq!(unknown.max_active.load(Ordering::SeqCst), 1);
    assert_call_order(&unknown);
}

#[tokio::test]
async fn repeated_no_key_calls_overlap_but_approval_required_calls_do_not() {
    let repeated_probe = Arc::new(SchedulingProbe::default());
    let repeated_native = Arc::new(NativeSkillRegistry::new());
    repeated_native
        .register(ProbeSkill {
            name: "probe_repeat".to_string(),
            effect: ToolEffect::ReadOnly,
            approval_class: ApprovalClass::NotRequired,
            concurrency_key: None,
            probe: Arc::clone(&repeated_probe),
        })
        .await
        .expect("repeated probe descriptor registers");
    let repeated_driver = Arc::new(MockLlmDriver::new(vec![
        tool_batch(&[
            ("repeat-1", "probe_repeat", "{}"),
            ("repeat-2", "probe_repeat", "{}"),
        ]),
        final_turn(),
    ]));
    let repeated_orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            parallel_tool_calls: Some(true),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::new_empty()),
        repeated_native,
        repeated_driver,
    );
    repeated_orchestrator
        .chat("run the same probe twice")
        .await
        .expect("descriptor assembly succeeds")
        .collect::<Vec<_>>()
        .await;
    assert_eq!(repeated_probe.max_active.load(Ordering::SeqCst), 2);

    let approval_probe = Arc::new(SchedulingProbe::default());
    let approval_native = Arc::new(NativeSkillRegistry::new());
    for name in ["probe_first", "probe_second"] {
        approval_native
            .register(ProbeSkill {
                name: name.to_string(),
                effect: ToolEffect::ReadOnly,
                approval_class: ApprovalClass::Required,
                concurrency_key: None,
                probe: Arc::clone(&approval_probe),
            })
            .await
            .expect("approval-required probe descriptor registers");
    }
    let approval_driver = Arc::new(MockLlmDriver::new(vec![
        tool_batch(&[
            ("approval-1", "probe_first", "{}"),
            ("approval-2", "probe_second", "{}"),
        ]),
        final_turn(),
    ]));
    let approval_orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            parallel_tool_calls: Some(true),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::new_empty()),
        approval_native,
        approval_driver,
    );
    approval_orchestrator
        .chat("run approval-required probes")
        .await
        .expect("descriptor assembly succeeds")
        .collect::<Vec<_>>()
        .await;
    assert_eq!(approval_probe.max_active.load(Ordering::SeqCst), 1);
    assert_call_order(&approval_probe);
}

struct BoundedOutputSkill {
    name: String,
}

#[async_trait]
impl NativeSkill for BoundedOutputSkill {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Returns output larger than its descriptor limit"
    }

    fn parameters_schema(&self) -> Value {
        json!({"type": "object", "additionalProperties": false})
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::ReadOnly
    }

    fn output_limit(&self) -> Option<TruncationPolicy> {
        Some(TruncationPolicy::Bytes(128))
    }

    async fn execute(&self, _arguments: Value) -> anyhow::Result<Value> {
        Ok(json!({"content": "x".repeat(2_000)}))
    }
}

fn assert_descriptor_output_bound(events: &[NormalizedEvent], expected_results: usize) {
    let results = events
        .iter()
        .filter_map(|event| match event {
            NormalizedEvent::ToolResult {
                content,
                success: true,
                ..
            } => Some(content),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(results.len(), expected_results);
    for content in results {
        assert!(content.len() <= 128, "descriptor bound was exceeded");
        assert!(content.starts_with("Warning: truncated output"));
    }
}

#[tokio::test]
async fn descriptor_output_limit_applies_to_sequential_and_parallel_paths() {
    let sequential_native = Arc::new(NativeSkillRegistry::new());
    sequential_native
        .register(BoundedOutputSkill {
            name: "bounded_single".to_string(),
        })
        .await
        .expect("bounded descriptor registers");
    let sequential = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            parallel_tool_calls: Some(true),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::new_empty()),
        sequential_native,
        Arc::new(MockLlmDriver::new(vec![
            tool_turn("bounded-1", "bounded_single", "{}"),
            final_turn(),
        ])),
    )
    .chat("run one bounded tool")
    .await
    .expect("descriptor assembly succeeds")
    .collect::<Vec<_>>()
    .await;
    assert_descriptor_output_bound(&sequential, 1);

    let parallel_native = Arc::new(NativeSkillRegistry::new());
    for name in ["bounded_first", "bounded_second"] {
        parallel_native
            .register(BoundedOutputSkill {
                name: name.to_string(),
            })
            .await
            .expect("bounded descriptor registers");
    }
    let parallel = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            parallel_tool_calls: Some(true),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::new_empty()),
        parallel_native,
        Arc::new(MockLlmDriver::new(vec![
            tool_batch(&[
                ("bounded-1", "bounded_first", "{}"),
                ("bounded-2", "bounded_second", "{}"),
            ]),
            final_turn(),
        ])),
    )
    .chat("run bounded tools in parallel")
    .await
    .expect("descriptor assembly succeeds")
    .collect::<Vec<_>>()
    .await;
    assert_descriptor_output_bound(&parallel, 2);
}

fn unavailable_embedding_backend() -> Arc<dyn EmbeddingBackend> {
    Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by this test",
    ))
}

#[tokio::test]
async fn readonly_mcp_hint_does_not_bypass_ask_approval() {
    let mcp = Arc::new(
        McpRegistry::new_with_test_tool_annotations(
            "readonly_probe",
            "Returns its input after approval",
            ToolAnnotations::new().read_only(true),
        )
        .expect("read-only MCP test descriptor assembles"),
    );
    let driver = Arc::new(MockLlmDriver::new(vec![
        tool_turn(
            "readonly-call",
            "test__readonly_probe",
            r#"{"mirror":"approval required"}"#,
        ),
        final_turn(),
    ]));
    let manager = Arc::new(
        RunManager::new(
            LlmConfig {
                model: "openai/gpt-4o".to_string(),
                api_key: Some("test-key".to_string()),
                ..LlmConfig::default()
            },
            mcp,
            SessionStore::new(),
            Arc::new(RwLock::new(SkillRegistry::new(None, None))),
            Arc::new(
                universal_agent_runtime::uar::runtime::matching::VectorMatcher::new(
                    unavailable_embedding_backend(),
                    0.75,
                ),
            ),
            None,
        )
        .await
        .with_llm_driver(driver.clone()),
    );
    let mut artifact = defaults::default_agent();
    artifact.extensions.insert(
        "uar.run_policy".to_string(),
        json!({
            "version": 1,
            "tool_approval": ToolApprovalPolicy::Ask,
        }),
    );

    let run_id = manager
        .start_run(
            artifact,
            "run the read-only probe".to_string(),
            None,
            None,
            vec![],
        )
        .await;

    let parked_events = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let events = manager
                .history_since(&run_id, None)
                .await
                .expect("run history exists");
            if events.iter().any(|event| {
                matches!(
                    &event.event,
                    RunEvent::ToolCallApprovalRequired { name, .. }
                        if name == "test__readonly_probe"
                )
            }) {
                break events;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("read-only MCP call pauses for approval");

    assert!(parked_events.iter().any(|event| {
        matches!(
            &event.event,
            RunEvent::ToolCallApprovalRequired {
                tool_call_id,
                name,
                ..
            } if tool_call_id == "readonly-call" && name == "test__readonly_probe"
        )
    }));
    assert!(
        parked_events
            .iter()
            .all(|event| !matches!(event.event, RunEvent::ToolEnd { .. })),
        "the MCP tool must not execute before approval"
    );
    assert_eq!(
        driver.requests().len(),
        1,
        "the run remains on its tool turn"
    );

    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if manager.resolve_approval(&run_id, true).await {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("pending approval becomes resolvable");

    let completed_events = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let events = manager
                .history_since(&run_id, None)
                .await
                .expect("run history exists");
            if events
                .iter()
                .any(|event| matches!(event.event, RunEvent::RunDone { .. }))
            {
                break events;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("approved run completes");

    assert!(completed_events.iter().any(|event| {
        matches!(
            &event.event,
            RunEvent::ToolEnd { tool, ok, .. }
                if tool == "test__readonly_probe" && *ok
        )
    }));
    assert_eq!(driver.requests().len(), 2);
}

struct SchemaSkill {
    schema: Value,
}

#[async_trait]
impl NativeSkill for SchemaSkill {
    fn name(&self) -> &str {
        "duplicate_native"
    }

    fn description(&self) -> &str {
        "Native collision probe"
    }

    fn parameters_schema(&self) -> Value {
        self.schema.clone()
    }

    async fn execute(&self, arguments: Value) -> anyhow::Result<Value> {
        Ok(arguments)
    }
}

#[tokio::test]
async fn assembly_preserves_mcp_namespaces_and_rejects_native_collisions() {
    let first_mcp = McpRegistry::new_with_test_tool_for_server(
        "alpha",
        "lookup",
        "Alpha lookup",
        ToolAnnotations::new().read_only(true),
    )
    .expect("alpha descriptor assembles");
    let second_mcp = McpRegistry::new_with_test_tool_for_server(
        "beta",
        "lookup",
        "Beta lookup",
        ToolAnnotations::new().read_only(true),
    )
    .expect("beta descriptor assembles");
    let merged = first_mcp
        .merge(&second_mcp)
        .expect("namespaced MCP descriptors do not collide");
    let descriptor_names = merged
        .descriptors()
        .into_iter()
        .map(|descriptor| descriptor.provider_name.clone())
        .collect::<Vec<_>>();
    assert_eq!(descriptor_names, ["alpha__lookup", "beta__lookup"]);

    let native = NativeSkillRegistry::new();
    native
        .register(SchemaSkill {
            schema: json!({
                "type": "object",
                "properties": { "path": { "type": "string" } },
            }),
        })
        .await
        .expect("first native descriptor registers");
    let error = native
        .register(SchemaSkill {
            schema: json!({
                "type": "object",
                "properties": { "path": { "type": "integer" } },
            }),
        })
        .await
        .expect_err("different schemas with the same provider name collide");
    assert!(matches!(
        error,
        ToolAssemblyError::Collision(ToolCollision { provider_name })
            if provider_name == "duplicate_native"
    ));
}

#[tokio::test]
async fn descriptor_compiles_its_validator_once_for_ten_invocations() {
    let compiler = Arc::new(ValidatorCompiler::default());
    let executions = Arc::new(AtomicUsize::new(0));
    let native = Arc::new(NativeSkillRegistry::with_validator_compiler(Arc::clone(
        &compiler,
    )));
    native
        .register(CountingPathSkill {
            executions: Arc::clone(&executions),
        })
        .await
        .expect("path descriptor registers");
    assert_eq!(compiler.compile_count(), 1);

    let calls = [
        ("path-0", "path_tool", r#"{"path":"0"}"#),
        ("path-1", "path_tool", r#"{"path":"1"}"#),
        ("path-2", "path_tool", r#"{"path":"2"}"#),
        ("path-3", "path_tool", r#"{"path":"3"}"#),
        ("path-4", "path_tool", r#"{"path":"4"}"#),
        ("path-5", "path_tool", r#"{"path":"5"}"#),
        ("path-6", "path_tool", r#"{"path":"6"}"#),
        ("path-7", "path_tool", r#"{"path":"7"}"#),
        ("path-8", "path_tool", r#"{"path":"8"}"#),
        ("path-9", "path_tool", r#"{"path":"9"}"#),
    ];
    let driver = Arc::new(MockLlmDriver::new(vec![tool_batch(&calls), final_turn()]));
    let orchestrator = Orchestrator::from_driver(
        LlmConfig {
            model: "openai/gpt-4o".to_string(),
            parallel_tool_calls: Some(false),
            ..LlmConfig::default()
        },
        Arc::new(McpRegistry::new_empty()),
        native,
        driver,
    );

    let events = orchestrator
        .chat("read ten paths")
        .await
        .expect("descriptor assembly succeeds")
        .collect::<Vec<_>>()
        .await;

    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, NormalizedEvent::ToolResult { success: true, .. }))
            .count(),
        10
    );
    assert_eq!(executions.load(Ordering::SeqCst), 10);
    assert_eq!(
        compiler.compile_count(),
        1,
        "invocations reuse the descriptor's compiled validator"
    );
}

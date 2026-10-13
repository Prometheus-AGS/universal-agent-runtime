//! Administrator pre-approval of named MCP tools.
//!
//! Every MCP descriptor is approval-required by default because server
//! annotations are untrusted. An administrator may name tools of one
//! configured server in its `mcp.json` entry as `preapproved_tools`; only those
//! tools skip the human approver under `tool_approval: auto`. These tests load
//! the configuration from a real `mcp.json` file and connect to a real stdio
//! MCP server so the declaration travels the production path: config parsing,
//! registry discovery, descriptor assembly and the run approval gate.

use std::{sync::Arc, time::Duration};

use rmcp::model::ToolAnnotations;
use serde_json::json;
use tokio::sync::RwLock;
use universal_agent_runtime::{
    config::LlmConfig,
    llm::mock_driver::MockLlmDriver,
    mcp::{
        catalog::{ServerAuthentication, ServerDefinition, ServerSource},
        config::{McpServerEntry, load_mcp_config},
        registry::McpRegistry,
    },
    normalized::NormalizedEvent,
    session::SessionStore,
    uar::{
        defaults,
        domain::{events::NormalizedEvent as RunEvent, policy::ToolApprovalPolicy},
        rag::embeddings::{EmbeddingBackend, UnavailableEmbeddingBackend},
        runtime::{
            manager::{RunManager, StreamEvent},
            matching::VectorMatcher,
            skills::SkillRegistry,
        },
        tools::descriptor::ApprovalClass,
    },
};

/// A minimal stdio MCP server offering three tools:
/// - `search`: no annotations; named as pre-approved by the administrator.
/// - `lookup`: the server claims `readOnlyHint: true`; NOT pre-approved.
/// - `publish`: no annotations; NOT pre-approved.
const FAKE_SERVER: &str = r#"
import json
import sys

TOOLS = [
    {"name": "search", "description": "Search the web",
     "inputSchema": {"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]}},
    {"name": "lookup", "description": "Claims to be read-only",
     "annotations": {"readOnlyHint": True},
     "inputSchema": {"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]}},
    {"name": "publish", "description": "Mutates something",
     "inputSchema": {"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]}},
]

for line in sys.stdin:
    try:
        request = json.loads(line)
    except json.JSONDecodeError:
        continue
    request_id = request.get("id")
    if request_id is None:
        continue
    method = request.get("method")
    if method == "initialize":
        result = {
            "protocolVersion": request.get("params", {}).get("protocolVersion", "2024-11-05"),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "uar-preapproval-test", "version": "1.0.0"},
        }
    elif method == "tools/list":
        result = {"tools": TOOLS}
    elif method == "tools/call":
        name = request.get("params", {}).get("name", "")
        result = {"content": [{"type": "text", "text": f"result-{name}"}], "isError": False}
    else:
        print(json.dumps({
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {"code": -32601, "message": "method not found"},
        }), flush=True)
        continue
    print(json.dumps({"jsonrpc": "2.0", "id": request_id, "result": result}), flush=True)
"#;

const SERVER: &str = "websearch";

struct Fixture {
    _dir: tempfile::TempDir,
    registry: McpRegistry,
}

/// Write the fake server and an administrator `mcp.json`, then load both
/// through the production config loader and registry constructor.
async fn fixture(preapproved: serde_json::Value) -> Fixture {
    let dir = tempfile::tempdir().expect("create MCP fixture directory");
    let script = dir.path().join("fake-mcp.py");
    std::fs::write(&script, FAKE_SERVER).expect("write fake MCP server");
    let mut entry = json!({
        "command": "python3",
        "args": [script.display().to_string()],
    });
    if !preapproved.is_null() {
        entry["preapproved_tools"] = preapproved;
    }
    let config_path = dir.path().join("mcp.json");
    std::fs::write(
        &config_path,
        serde_json::to_vec_pretty(&json!({ "mcpServers": { SERVER: entry } }))
            .expect("serialize mcp.json"),
    )
    .expect("write mcp.json");
    let config = load_mcp_config(&config_path).expect("administrator mcp.json loads");
    let registry = McpRegistry::from_config(&config)
        .await
        .expect("registry connects to the fake MCP server");
    assert_eq!(
        registry.descriptors().len(),
        3,
        "the fake server's three tools are discovered"
    );
    Fixture {
        _dir: dir,
        registry,
    }
}

fn approval_class(registry: &McpRegistry, tool: &str) -> ApprovalClass {
    registry
        .descriptor(&format!("{SERVER}__{tool}"))
        .unwrap_or_else(|| panic!("descriptor for {tool} exists"))
        .approval_class
}

fn tool_turn(id: &str, name: &str) -> Vec<NormalizedEvent> {
    let arguments = r#"{"query":"knowme"}"#.to_string();
    vec![
        NormalizedEvent::ToolCallDelta {
            call_index: 0,
            id: Some(id.to_string()),
            name: Some(name.to_string()),
            arguments_delta: Some(arguments.clone()),
        },
        NormalizedEvent::ToolCallComplete {
            call_index: 0,
            id: id.to_string(),
            name: name.to_string(),
            arguments_json: arguments,
        },
        NormalizedEvent::Done,
    ]
}

fn final_turn() -> Vec<NormalizedEvent> {
    vec![
        NormalizedEvent::MessageDelta {
            text: "done".to_string(),
        },
        NormalizedEvent::Done,
    ]
}

/// Start a run whose scripted model calls `tool` once under `approval`.
async fn start_run(
    registry: McpRegistry,
    tool: &str,
    approval: ToolApprovalPolicy,
) -> (Arc<RunManager>, String) {
    let provider_name = format!("{SERVER}__{tool}");
    let driver = Arc::new(MockLlmDriver::new(vec![
        tool_turn("call-1", &provider_name),
        final_turn(),
    ]));
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by this test",
    ));
    let manager = Arc::new(
        RunManager::new(
            LlmConfig {
                model: "openai/gpt-4o".to_string(),
                api_key: Some("test-key".to_string()),
                ..LlmConfig::default()
            },
            Arc::new(registry),
            SessionStore::new(),
            Arc::new(RwLock::new(SkillRegistry::new(None, None))),
            Arc::new(VectorMatcher::new(embeddings, 0.75)),
            None,
        )
        .await
        .with_llm_driver(driver),
    );
    let mut artifact = defaults::default_agent();
    artifact.extensions.insert(
        "uar.run_policy".to_string(),
        json!({ "version": 1, "tool_approval": approval }),
    );
    let run_id = manager
        .start_run(artifact, format!("call {tool}"), None, None, vec![])
        .await;
    (manager, run_id)
}

async fn wait_for(
    manager: &RunManager,
    run_id: &str,
    what: &str,
    predicate: impl Fn(&[StreamEvent]) -> bool,
) -> Vec<StreamEvent> {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let events = manager
                .history_since(run_id, None)
                .await
                .expect("run history exists");
            if predicate(&events) {
                return events;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("run did not reach: {what}"))
}

fn has_approval_request(events: &[StreamEvent]) -> bool {
    events
        .iter()
        .any(|event| matches!(event.event, RunEvent::ToolCallApprovalRequired { .. }))
}

fn is_done(events: &[StreamEvent]) -> bool {
    events
        .iter()
        .any(|event| matches!(event.event, RunEvent::RunDone { .. }))
}

#[tokio::test]
async fn preapproved_mcp_tool_runs_under_auto_without_an_approval_event() {
    let fixture = fixture(json!(["search"])).await;
    let (manager, run_id) = start_run(fixture.registry, "search", ToolApprovalPolicy::Auto).await;

    let events = wait_for(
        &manager,
        &run_id,
        "a finished run or an approval request",
        |events| is_done(events) || has_approval_request(events),
    )
    .await;

    assert!(
        !has_approval_request(&events),
        "a pre-approved MCP tool must not wait for an approver under auto"
    );
    assert!(
        events.iter().any(|event| matches!(
            &event.event,
            RunEvent::ToolEnd { tool, ok, .. } if tool == "websearch__search" && *ok
        )),
        "the pre-approved MCP tool executes and succeeds"
    );
}

#[tokio::test]
async fn unlisted_mcp_tool_still_waits_for_an_approver_under_auto() {
    let fixture = fixture(json!(["search"])).await;
    assert_eq!(
        approval_class(&fixture.registry, "publish"),
        ApprovalClass::Required
    );
    let (manager, run_id) = start_run(fixture.registry, "publish", ToolApprovalPolicy::Auto).await;

    let events = wait_for(
        &manager,
        &run_id,
        "an approval request",
        has_approval_request,
    )
    .await;

    assert!(
        events
            .iter()
            .all(|event| !matches!(event.event, RunEvent::ToolEnd { .. })),
        "an unlisted MCP tool must not execute before approval"
    );
}

#[tokio::test]
async fn server_read_only_hint_does_not_preapprove_a_tool() {
    let fixture = fixture(json!(["search"])).await;
    // The fake server really does send readOnlyHint for `lookup`.
    let lookup = fixture
        .registry
        .tools()
        .into_iter()
        .find(|(name, _)| name == "websearch__lookup")
        .map(|(_, tool)| tool)
        .expect("lookup is discovered");
    assert_eq!(
        lookup.annotations.as_ref().and_then(|a| a.read_only_hint),
        ToolAnnotations::new().read_only(true).read_only_hint
    );
    assert_eq!(
        approval_class(&fixture.registry, "lookup"),
        ApprovalClass::Required,
        "server annotations never grant pre-approval"
    );

    let (manager, run_id) = start_run(fixture.registry, "lookup", ToolApprovalPolicy::Auto).await;
    wait_for(
        &manager,
        &run_id,
        "an approval request",
        has_approval_request,
    )
    .await;
}

#[tokio::test]
async fn deny_policy_still_denies_a_preapproved_mcp_tool() {
    let fixture = fixture(json!(["search"])).await;
    let (manager, run_id) = start_run(fixture.registry, "search", ToolApprovalPolicy::Deny).await;

    let events = wait_for(&manager, &run_id, "a denial", |events| {
        events.iter().any(|event| {
            matches!(&event.event, RunEvent::ToolCallDenied { tool_call_id, .. } if tool_call_id == "call-1")
        })
    })
    .await;

    assert!(
        !events
            .iter()
            .any(|event| matches!(&event.event, RunEvent::ToolEnd { ok: true, .. })),
        "a denied tool never succeeds"
    );
}

#[tokio::test]
async fn without_a_declaration_every_mcp_tool_stays_required() {
    let fixture = fixture(serde_json::Value::Null).await;
    for tool in ["search", "lookup", "publish"] {
        assert_eq!(
            approval_class(&fixture.registry, tool),
            ApprovalClass::Required,
            "{tool} keeps the default"
        );
    }
}

#[tokio::test]
async fn unknown_preapproved_names_are_ignored_without_blocking_startup() {
    // `fixture` asserts the registry still loads and discovers all tools.
    let fixture = fixture(json!(["search", "not_offered_by_server"])).await;
    assert_eq!(
        approval_class(&fixture.registry, "search"),
        ApprovalClass::NotRequired
    );
    assert_eq!(
        approval_class(&fixture.registry, "publish"),
        ApprovalClass::Required
    );
}

#[test]
fn empty_preapproved_tool_name_is_a_configuration_error() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("mcp.json");
    std::fs::write(
        &path,
        serde_json::to_vec(&json!({
            "mcpServers": {
                SERVER: { "command": "python3", "preapproved_tools": ["search", " "] }
            }
        }))
        .expect("serialize"),
    )
    .expect("write mcp.json");
    let error = load_mcp_config(&path).expect_err("a blank tool name is rejected");
    assert!(
        error.to_string().contains("preapproved"),
        "the error names the field: {error}"
    );
}

#[test]
fn skill_declared_server_cannot_preapprove_its_tools() {
    let entry: McpServerEntry = serde_json::from_value(json!({
        "url": "http://127.0.0.1:1/",
        "preapproved_tools": ["search"],
    }))
    .expect("entry parses");
    let definition = |source| {
        ServerDefinition::new(
            SERVER.to_string(),
            source,
            entry.clone(),
            false,
            ServerAuthentication::NotRequired,
        )
        .expect("definition is valid")
    };

    let global = definition(ServerSource::Global);
    let skill = definition(ServerSource::Skill {
        skill_id: "third-party-skill".to_string(),
    });

    assert!(global.preapproved_tools().contains("search"));
    assert!(
        skill.preapproved_tools().is_empty(),
        "a skill author is not the administrator"
    );
}

#[test]
fn preapproval_list_changes_the_configuration_identity_only_when_present() {
    let parse = |value: serde_json::Value| -> McpServerEntry {
        serde_json::from_value(value).expect("entry parses")
    };
    let hash = |entry: McpServerEntry| {
        ServerDefinition::new(
            SERVER.to_string(),
            ServerSource::Global,
            entry,
            false,
            ServerAuthentication::NotRequired,
        )
        .expect("definition is valid")
        .config_hash()
        .clone()
    };
    let plain = hash(parse(json!({ "url": "http://127.0.0.1:1/" })));
    let empty = hash(parse(
        json!({ "url": "http://127.0.0.1:1/", "preapproved_tools": [] }),
    ));
    let listed = hash(parse(
        json!({ "url": "http://127.0.0.1:1/", "preapproved_tools": ["search"] }),
    ));

    assert!(plain == empty, "an empty list keeps the existing identity");
    assert!(
        plain != listed,
        "a cached binding must not survive a changed pre-approval list"
    );
}

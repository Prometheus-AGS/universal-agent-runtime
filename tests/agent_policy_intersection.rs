//! Integration contract for fail-closed child policy intersection.

use std::collections::{BTreeMap, BTreeSet};

use universal_agent_runtime::uar::{
    context::ContextStrategy,
    defaults::default_agent,
    domain::{
        artifact::{AgentArtifact, ToolExecutionMode},
        policy::{
            ChatMode, ModelRoute, PolicyResolutionInput, PolicyUniverse, RunPolicy,
            ToolApprovalPolicy, policy_from_agent_artifact, resolve_run_policy,
        },
    },
    runtime::thread::{
        AgentThread,
        policy_intersection::{
            CredentialGrant, CredentialTarget, PolicyIntersectionError, SandboxPermissions,
            ThreadBudgets, ThreadPermissions, ThreadPolicy, ThreadToolBinding,
        },
    },
};

fn artifact(id: &str, allowed_tools: &[&str]) -> AgentArtifact {
    let mut artifact = default_agent();
    artifact.id = id.to_owned();
    artifact.metadata.title = id.to_owned();
    artifact.policy.provider.default.provider = "test-provider".to_owned();
    artifact.policy.provider.default.model = "test-model".to_owned();
    artifact.policy.tools.allow = allowed_tools
        .iter()
        .map(|tool| (*tool).to_owned())
        .collect();
    artifact.policy.tools.deny.clear();
    artifact
}

fn root_policy() -> ThreadPolicy {
    root_policy_with_approval(ToolApprovalPolicy::Auto)
}

fn root_policy_with_approval(tool_approval: ToolApprovalPolicy) -> ThreadPolicy {
    let mut root_artifact = artifact("root-agent", &["read"]);
    root_artifact.policy.tools.deny = vec!["write".to_owned()];
    let effective = resolve_run_policy(PolicyResolutionInput {
        global: Some(RunPolicy {
            tool_approval,
            ..RunPolicy::default()
        }),
        agent: Some(policy_from_agent_artifact(&root_artifact)),
        universe: PolicyUniverse {
            tools: ["read".to_owned(), "write".to_owned()].into(),
            ..PolicyUniverse::default()
        },
        default_chat_mode: ChatMode::Agent,
        default_context_strategy: ContextStrategy::Auto,
        default_agent_id: Some(root_artifact.id.clone()),
        default_model: Some(ModelRoute {
            provider_id: "test-provider".to_owned(),
            model_id: "test-model".to_owned(),
        }),
        ..PolicyResolutionInput::default()
    });
    assert_eq!(effective.tools.ids, vec!["read"]);

    let root = AgentThread::root(
        "verified-owner".to_owned(),
        root_artifact.id.clone(),
        "root-run".to_owned(),
    )
    .expect("root policy thread must be valid");
    ThreadPolicy::for_root(
        &root,
        &effective,
        &root_artifact,
        ThreadPermissions {
            credentials: BTreeSet::from([CredentialGrant {
                target: CredentialTarget::Provider("test-provider".to_owned()),
                binding_id: "root-provider-binding".to_owned(),
            }]),
            tool_bindings: BTreeMap::from([("read".to_owned(), ThreadToolBinding::Native)]),
            sandbox: SandboxPermissions {
                execution_mode: ToolExecutionMode::Direct,
                network_enabled: false,
                filesystem: BTreeMap::new(),
                environment: BTreeSet::new(),
            },
            budgets: ThreadBudgets::default(),
            max_active_skills: 3,
            max_concurrent_tools: 1,
        },
    )
    .expect("root policy fixture must resolve")
}

#[test]
fn child_cannot_regrant_a_tool_denied_by_the_parent() {
    let parent = root_policy();
    let child_artifact = artifact("child-agent", &["read", "write"]);

    let child = parent
        .intersect(&child_artifact)
        .expect("a valid narrower child policy must resolve");

    assert_eq!(child.effective().tools.ids, vec!["read"]);
    assert_eq!(child.artifact().policy.tools.allow, vec!["read"]);
    assert_eq!(
        child.permissions().tool_bindings.keys().collect::<Vec<_>>(),
        vec!["read"]
    );
    assert!(
        !child
            .effective()
            .tools
            .ids
            .iter()
            .any(|tool| tool == "write")
    );
}

#[test]
fn child_without_model_route_inherits_the_captured_root_route() {
    let parent = root_policy();
    let mut child_artifact = artifact("child-agent", &["read"]);
    child_artifact.policy.provider.default.provider.clear();
    child_artifact.policy.provider.default.model.clear();

    let child = parent
        .intersect(&child_artifact)
        .expect("an omitted child model route must retain the captured root route");

    assert_eq!(
        child.effective().model,
        Some(ModelRoute {
            provider_id: "test-provider".to_owned(),
            model_id: "test-model".to_owned(),
        })
    );
}

#[test]
fn unsupported_child_policy_shape_fails_closed() {
    let parent = root_policy();
    let mut child_artifact = artifact("child-agent", &["read"]);
    child_artifact.extensions.insert(
        "uar.run_policy".to_owned(),
        serde_json::json!({
            "version": 1,
            "tools": {
                "mode": "selected",
                "ids": ["read"],
                "denied_ids": [],
                "unsupported_widening_rule": true
            }
        }),
    );

    let error = parent
        .intersect(&child_artifact)
        .expect_err("an unknown nested policy restriction must not be ignored");

    assert_eq!(
        error,
        PolicyIntersectionError::UnsupportedShape {
            section: "uar.run_policy"
        }
    );
}

#[test]
fn child_cannot_grant_approval_and_retains_the_root_approval_lane() {
    let parent = root_policy_with_approval(ToolApprovalPolicy::Ask);
    let root_approval_run = parent.approval_root_run_id().to_owned();
    let mut child_artifact = artifact("child-agent", &["read"]);
    child_artifact.extensions.insert(
        "uar.run_policy".to_owned(),
        serde_json::json!({
            "version": 1,
            "tool_approval": "auto"
        }),
    );

    let child = parent
        .intersect(&child_artifact)
        .expect("a child approval request must remain within the root policy");

    assert_eq!(child.effective().tool_approval, ToolApprovalPolicy::Ask);
    assert_eq!(child.approval_root_run_id(), root_approval_run);
    assert_eq!(child.approval_root_run_id(), "root-run");
}

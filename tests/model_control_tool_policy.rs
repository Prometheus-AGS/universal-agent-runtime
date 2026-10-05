//! Built-in model-control tools follow the policy that restricts them.
//!
//! `activate_skill`, the team tools and the agent controls were exempt from
//! tool policy, so `activate_skill` was offered on every run, even for an
//! agent that allows no skills, and the other controls survived an allowlist
//! that did not name them. `search_tools` stays available because it only
//! searches tools the policy has already admitted. With no restriction,
//! nothing changes.

use serde_json::json;
use universal_agent_runtime::uar::domain::policy::{
    ACTIVATE_SKILL_TOOL, EffectiveRunPolicy, PolicyResolutionInput, PolicyUniverse, RunPolicy,
    SEARCH_TOOLS_TOOL, resolve_run_policy,
};
use universal_agent_runtime::uar::runtime::native_skills::search_tools::SEARCH_TOOLS_NAME;

const TEAM_ROSTER: &str = "team_roster";
const TEAM_SEND: &str = "team_send";
const SEARCH_TOOLS: &str = SEARCH_TOOLS_TOOL;

fn universe() -> PolicyUniverse {
    PolicyUniverse {
        skills: ["rust".to_string(), "docs".to_string()].into(),
        tools: [TEAM_ROSTER, TEAM_SEND, SEARCH_TOOLS, ACTIVATE_SKILL_TOOL]
            .map(String::from)
            .into(),
        ..PolicyUniverse::default()
    }
}

/// The policy an agent artifact resolves to for the given `skills` and
/// `tools` selections (JSON as the agent artifact spells them).
fn resolve(skills: serde_json::Value, tools: serde_json::Value) -> EffectiveRunPolicy {
    let agent: RunPolicy = serde_json::from_value(json!({"skills": skills, "tools": tools}))
        .expect("a valid agent run policy");
    resolve_run_policy(PolicyResolutionInput {
        agent: Some(agent),
        universe: universe(),
        ..PolicyResolutionInput::default()
    })
}

fn allowed(policy: &EffectiveRunPolicy, name: &str) -> bool {
    policy.allows_model_control_tool(name, name)
}

#[test]
fn nothing_is_restricted_when_the_policy_is_unset() {
    let policy = resolve(json!({}), json!({}));

    for name in [ACTIVATE_SKILL_TOOL, TEAM_ROSTER, TEAM_SEND, SEARCH_TOOLS] {
        assert!(allowed(&policy, name), "{name} must stay available");
    }
}

#[test]
fn activate_skill_is_dropped_when_the_agent_allows_no_skills() {
    let policy = resolve(json!({"mode": "none"}), json!({}));

    assert!(!allowed(&policy, ACTIVATE_SKILL_TOOL));
    // Skill policy does not reach the other model-control tools.
    assert!(allowed(&policy, TEAM_ROSTER));
}

#[test]
fn activate_skill_stays_when_skills_are_selected() {
    let policy = resolve(json!({"mode": "selected", "ids": ["rust"]}), json!({}));

    assert!(allowed(&policy, ACTIVATE_SKILL_TOOL));
}

#[test]
fn activate_skill_is_dropped_when_selected_skills_resolve_to_nothing() {
    let policy = resolve(
        json!({"mode": "selected", "ids": ["not-installed"]}),
        json!({}),
    );

    assert!(!allowed(&policy, ACTIVATE_SKILL_TOOL));
}

#[test]
fn a_tool_allowlist_keeps_only_the_model_control_tools_it_names() {
    let policy = resolve(json!({}), json!({"mode": "selected", "ids": [TEAM_ROSTER]}));

    assert!(allowed(&policy, TEAM_ROSTER));
    assert!(!allowed(&policy, TEAM_SEND));
}

#[test]
fn a_tool_allowlist_does_not_decide_activate_skill() {
    // `activate_skill` follows the skill policy, not the tool allowlist.
    let policy = resolve(json!({}), json!({"mode": "selected", "ids": [TEAM_ROSTER]}));

    assert!(allowed(&policy, ACTIVATE_SKILL_TOOL));
}

#[test]
fn no_tools_drops_the_team_controls_but_not_activate_skill_or_search_tools() {
    let policy = resolve(json!({}), json!({"mode": "none"}));

    assert!(!allowed(&policy, TEAM_ROSTER));
    assert!(!allowed(&policy, TEAM_SEND));
    assert!(allowed(&policy, ACTIVATE_SKILL_TOOL));
    assert!(allowed(&policy, SEARCH_TOOLS));
}

#[test]
fn a_tool_is_matched_by_name_or_by_id() {
    let policy = resolve(json!({}), json!({"mode": "selected", "ids": [TEAM_ROSTER]}));

    assert!(policy.allows_model_control_tool("provider_name_differs", TEAM_ROSTER));
    assert!(!policy.allows_model_control_tool("provider_name_differs", "other_id"));
}

#[test]
fn search_tools_is_never_refused() {
    // Refusing it would reject the whole turn once deferred MCP tools exist
    // (`ResolvedStep::new` validates the projection it is part of), and it
    // adds no restriction: it searches only tools the policy already admits.
    for tools in [
        json!({}),
        json!({"mode": "none"}),
        json!({"mode": "selected", "ids": [TEAM_ROSTER]}),
    ] {
        let policy = resolve(json!({"mode": "none"}), tools);
        assert!(allowed(&policy, SEARCH_TOOLS));
    }
}

#[test]
fn the_domain_constant_matches_the_runtime_tool_name() {
    assert_eq!(SEARCH_TOOLS_TOOL, SEARCH_TOOLS_NAME);
}

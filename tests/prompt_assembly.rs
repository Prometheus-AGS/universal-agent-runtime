use std::sync::Arc;

use tokio::sync::RwLock;
use universal_agent_runtime::config::LlmConfig;
use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
use universal_agent_runtime::mcp::registry::McpRegistry;
use universal_agent_runtime::normalized::NormalizedEvent as DriverEvent;
use universal_agent_runtime::session::SessionStore;
use universal_agent_runtime::uar::defaults::default_agent;
use universal_agent_runtime::uar::domain::events::NormalizedEvent as RunEvent;
use universal_agent_runtime::uar::domain::skills::Skill;
use universal_agent_runtime::uar::rag::embeddings::{
    EmbeddingBackend, UnavailableEmbeddingBackend,
};
use universal_agent_runtime::uar::runtime::manager::RunManager;
use universal_agent_runtime::uar::runtime::matching::VectorMatcher;
use universal_agent_runtime::uar::runtime::prompt::{
    Authority, PromptBudgets, PromptFragment, PromptRole, PromptSection, Retention, TurnManifest,
    render,
};
use universal_agent_runtime::uar::runtime::skills::SkillRegistry;
use universal_agent_runtime::uar::runtime::turn::builtin::artifact_fragments;

async fn test_manager(driver: Arc<MockLlmDriver>) -> Arc<RunManager> {
    let embedding_backend: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by this test",
    ));
    Arc::new(
        RunManager::new(
            LlmConfig {
                model: "openai/gpt-4o".to_string(),
                api_key: Some("test-key".to_string()),
                ..LlmConfig::default()
            },
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            Arc::new(RwLock::new(SkillRegistry::default())),
            Arc::new(VectorMatcher::new(embedding_backend, 0.75)),
            None,
        )
        .await
        .with_llm_driver(driver),
    )
}

async fn completed_events(manager: &RunManager, run_id: &str) -> Vec<RunEvent> {
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            let events = manager
                .history_since(run_id, None)
                .await
                .expect("run history exists");
            if events
                .iter()
                .any(|event| matches!(event.event, RunEvent::RunDone { .. }))
            {
                return events.into_iter().map(|event| event.event).collect();
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("run completes")
}

fn skill(id: &str) -> Skill {
    Skill {
        skill_id: id.to_string(),
        title: id.to_string(),
        prompt_overlay: format!("Instructions for {id}"),
        enabled: true,
        ..Skill::default()
    }
}

fn assemble(registration_order: &[&str]) -> (String, TurnManifest) {
    let artifact = default_agent();
    let mut registry = SkillRegistry::default();
    for id in registration_order {
        registry.register_loaded(skill(id));
    }

    let mut fragments = artifact_fragments(&artifact);
    fragments.extend(registry.list().into_iter().map(|skill| {
        PromptFragment::new(
            format!("skill.{}", skill.skill_id),
            PromptSection::ActiveSkills,
            format!("skill:{}", skill.skill_id),
            Authority::Skill,
            PromptRole::System,
            Retention::Reclaimable,
            skill.prompt_overlay,
        )
    }));
    fragments.extend([
        PromptFragment::new(
            "retrieved.rag.0001",
            PromptSection::MemoryAndRetrieval,
            "knowledge-base:test",
            Authority::Retrieved,
            PromptRole::System,
            Retention::Turn,
            "First retrieved fact",
        ),
        PromptFragment::new(
            "retrieved.rag.0002",
            PromptSection::MemoryAndRetrieval,
            "knowledge-base:test",
            Authority::Retrieved,
            PromptRole::System,
            Retention::Turn,
            "Second retrieved fact",
        ),
    ]);

    let rendered = render(&fragments);
    let manifest = TurnManifest::from_fragments(
        &fragments,
        PromptBudgets::for_rendered(&rendered),
        registration_order.iter().map(|id| (*id).to_string()),
        Vec::<String>::new(),
        Vec::<String>::new(),
    );
    (rendered, manifest)
}

#[test]
fn registry_order_does_not_change_prompt_or_manifest_identity() {
    let (first_prompt, first_manifest) = assemble(&["charlie", "alpha", "bravo"]);
    let (second_prompt, second_manifest) = assemble(&["bravo", "charlie", "alpha"]);

    assert_eq!(first_prompt, second_prompt);
    assert_eq!(first_manifest.manifest_hash, second_manifest.manifest_hash);
    assert_eq!(first_manifest, second_manifest);
}

#[test]
fn retrieved_and_skill_content_keep_typed_authority_and_markers() {
    let retrieved = PromptFragment::new(
        "retrieved.rag.0001",
        PromptSection::MemoryAndRetrieval,
        "knowledge-base:test",
        Authority::Retrieved,
        PromptRole::System,
        Retention::Turn,
        "Retrieved text",
    );
    let skill = PromptFragment::new(
        "skill.test",
        PromptSection::ActiveSkills,
        "skill:test",
        Authority::Skill,
        PromptRole::System,
        Retention::Reclaimable,
        "Skill text",
    );

    let rendered = render(&[retrieved.clone(), skill.clone()]);

    assert_eq!(retrieved.authority, Authority::Retrieved);
    assert_eq!(skill.authority, Authority::Skill);
    assert!(rendered.contains("<uar-retrieved-content>\nRetrieved text\n</uar-retrieved-content>"));
    assert!(rendered.contains("<uar-skill-content>\nSkill text\n</uar-skill-content>"));
}

#[test]
fn manifest_serialization_is_complete_metadata_without_prompt_bodies() {
    let secret_body = "Retrieved account token credential-top-secret";
    let fragment = PromptFragment::new(
        "retrieved.secret",
        PromptSection::MemoryAndRetrieval,
        "knowledge-base:test",
        Authority::Retrieved,
        PromptRole::System,
        Retention::Turn,
        secret_body,
    );
    let expected_hash = fragment.content_hash.clone();
    let rendered = render(std::slice::from_ref(&fragment));
    let manifest = TurnManifest::from_fragments(
        &[fragment],
        PromptBudgets::for_rendered(&rendered),
        Vec::<String>::new(),
        Vec::<String>::new(),
        Vec::<String>::new(),
    );
    let serialized = serde_json::to_string(&manifest).expect("manifest serializes");

    assert_eq!(manifest.fragments[0].id, "retrieved.secret");
    assert_eq!(manifest.fragments[0].content_hash, expected_hash);
    assert_eq!(manifest.counts.total, 1);
    assert_eq!(manifest.counts.by_authority["retrieved"], 1);
    assert_eq!(manifest.budgets.rendered_bytes, rendered.len());
    assert_eq!(
        manifest.budgets.rendered_characters,
        rendered.chars().count()
    );
    assert!(!serialized.contains(secret_body));
    assert!(!serialized.contains("credential-top-secret"));
    assert!(!serialized.contains("\"content\""));
}

#[tokio::test]
async fn manager_stores_manifest_and_emits_both_manifest_and_policy_artifacts() {
    let driver = Arc::new(MockLlmDriver::new(vec![vec![
        DriverEvent::MessageDelta {
            text: "done".to_string(),
        },
        DriverEvent::Done,
    ]]));
    let manager = test_manager(driver).await;
    let run_id = manager
        .start_run(default_agent(), "hello".to_string(), None, None, vec![])
        .await;
    let events = completed_events(&manager, &run_id).await;

    let artifact_types = events
        .iter()
        .filter_map(|event| match event {
            RunEvent::Artifact { artifact, .. } => Some(artifact.artifact_type.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(artifact_types.contains(&"turn_manifest"));
    assert!(artifact_types.contains(&"effective_run_policy"));

    let run = manager
        .get_run(&run_id)
        .await
        .expect("completed run remains readable");
    let stored = run
        .context
        .get("turn_manifest")
        .cloned()
        .expect("turn manifest is stored in run context");
    let manifest: TurnManifest =
        serde_json::from_value(stored).expect("stored turn manifest remains typed");
    assert!(!manifest.manifest_hash.is_empty());
    assert!(manifest.counts.total > 0);
}

#[tokio::test]
async fn successive_turns_snapshot_the_rendered_prompt_prefix_diff() {
    let driver = Arc::new(MockLlmDriver::new(vec![
        vec![
            DriverEvent::MessageDelta {
                text: "first answer".to_string(),
            },
            DriverEvent::Done,
        ],
        vec![
            DriverEvent::MessageDelta {
                text: "second answer".to_string(),
            },
            DriverEvent::Done,
        ],
    ]));
    let manager = test_manager(Arc::clone(&driver)).await;
    let session_id = "prompt-prefix-stability";

    let first_run = manager
        .start_run(
            default_agent(),
            "first turn".to_string(),
            Some(session_id.to_string()),
            None,
            vec![],
        )
        .await;
    completed_events(&manager, &first_run).await;
    let second_run = manager
        .start_run(
            default_agent(),
            "second turn".to_string(),
            Some(session_id.to_string()),
            None,
            vec![],
        )
        .await;
    completed_events(&manager, &second_run).await;

    let requests = driver.requests();
    assert_eq!(requests.len(), 2);
    let rendered_system_prompt = |request_index: usize| {
        requests[request_index]
            .messages
            .iter()
            .find(|message| message["role"] == "system")
            .and_then(|message| message["content"].as_str())
            .expect("each model request contains the rendered system prompt")
    };
    let first = rendered_system_prompt(0);
    let second = rendered_system_prompt(1);
    let diff = if first == second {
        "--- first rendered prompt\n+++ second rendered prompt\n(no changes)".to_string()
    } else {
        format!("--- first rendered prompt\n{first}\n+++ second rendered prompt\n{second}")
    };

    insta::assert_snapshot!(diff, @r"
    --- first rendered prompt
    +++ second rendered prompt
    (no changes)
    ");
}

#[test]
fn artifact_instructions_render_as_host_between_policy_and_skill_catalog() {
    let mut artifact = default_agent();
    artifact.prompt.instructions = vec!["Follow the project convention".to_string()];
    let mut fragments = artifact_fragments(&artifact);
    fragments.extend([
        PromptFragment::new(
            "policy.effective",
            PromptSection::EnforcedPolicy,
            "effective_run_policy",
            Authority::Policy,
            PromptRole::System,
            Retention::Turn,
            "Policy text",
        ),
        PromptFragment::new(
            "skill.catalog",
            PromptSection::SkillCatalog,
            "skill_registry",
            Authority::Skill,
            PromptRole::System,
            Retention::Turn,
            "Skill catalog text",
        ),
    ]);

    let instruction = fragments
        .iter()
        .find(|fragment| fragment.id == "host.instruction.0000")
        .expect("artifact instruction becomes a fragment");
    assert_eq!(instruction.authority, Authority::Host);

    let rendered = render(&fragments);
    let policy_index = rendered.find("Policy text").expect("policy renders");
    let instruction_index = rendered
        .find("Follow the project convention")
        .expect("host instruction renders");
    let catalog_index = rendered
        .find("Skill catalog text")
        .expect("skill catalog renders");
    assert!(policy_index < instruction_index);
    assert!(instruction_index < catalog_index);
    assert!(
        rendered.contains("<uar-host-content>\nFollow the project convention\n</uar-host-content>")
    );
}

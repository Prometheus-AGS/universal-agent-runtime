use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use tokio::sync::Semaphore;
use universal_agent_runtime::{
    embedded::EmbeddedRuntime,
    llm::{
        LlmDriver, LlmRequest,
        registry::{ModelConfig, ProtocolSetting, ProviderConfig},
    },
    normalized::NormalizedEvent as ProviderEvent,
    uar::{
        defaults::default_agent,
        domain::{
            events::NormalizedEvent,
            skills::{Skill, SkillOrigin, SkillScope, SkillTriggers},
        },
        persistence::{PersistenceLayer, providers::surreal::SurrealDbProvider},
        runtime::{
            manager::StreamEvent,
            skills::{
                service::SkillService,
                storage::{
                    database::DatabaseStorageProvider, filesystem::FilesystemStorageProvider,
                },
            },
        },
    },
};

#[derive(Debug)]
struct GatedDriver {
    started: Arc<Semaphore>,
    release: Arc<Semaphore>,
    requests: Mutex<Vec<LlmRequest>>,
}

impl Default for GatedDriver {
    fn default() -> Self {
        Self {
            started: Arc::new(Semaphore::new(0)),
            release: Arc::new(Semaphore::new(0)),
            requests: Mutex::new(Vec::new()),
        }
    }
}

impl GatedDriver {
    async fn wait_until_started(&self) {
        self.started
            .acquire()
            .await
            .expect("driver start semaphore remains open")
            .forget();
    }

    fn release_one(&self) {
        self.release.add_permits(1);
    }

    fn requests(&self) -> Vec<LlmRequest> {
        self.requests.lock().expect("requests lock").clone()
    }
}

#[async_trait]
impl LlmDriver for GatedDriver {
    async fn stream(
        &self,
        request: LlmRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ProviderEvent>> + Send>>> {
        self.requests.lock().expect("requests lock").push(request);
        self.started.add_permits(1);
        let release = Arc::clone(&self.release);
        Ok(Box::pin(async_stream::stream! {
            release
                .acquire()
                .await
                .expect("driver release semaphore remains open")
                .forget();
            yield Ok(ProviderEvent::MessageDelta {
                text: "scoped run complete".to_string(),
            });
            yield Ok(ProviderEvent::Done);
        }))
    }
}

fn local_provider() -> ProviderConfig {
    ProviderConfig {
        id: "b4-local".to_string(),
        display_name: "B4 local test provider".to_string(),
        base_url: String::new(),
        api_key: None,
        protocol: ProtocolSetting::Auto,
        default_model: Some("b4-test-model".to_string()),
        models: vec![ModelConfig {
            id: "b4-test-model".to_string(),
            display_name: Some("B4 test model".to_string()),
            context_window: Some(8_192),
            supports_vision: false,
            supports_tools: true,
            supports_reasoning: false,
            supports_structured_output: false,
            supports_streaming: true,
            max_output_tokens: Some(1_024),
            enabled: true,
        }],
        enabled: true,
    }
}

async fn wait_for_done(runtime: &EmbeddedRuntime, run_id: &str) -> Vec<StreamEvent> {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let history = runtime
                .run_manager()
                .history_since(run_id, None)
                .await
                .expect("run history exists");
            if history
                .iter()
                .any(|event| matches!(event.event, NormalizedEvent::RunDone { .. }))
            {
                return history;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("run completes before timeout")
}

fn builtin(skill_id: &str) -> Skill {
    Skill {
        skill_id: skill_id.to_string(),
        version: "1.0.0".to_string(),
        title: skill_id.to_string(),
        description: "Matches the cold-restart-proof phrase".to_string(),
        triggers: SkillTriggers {
            keywords: vec!["cold-restart-proof".to_string()],
            semantic: None,
        },
        enabled: true,
        origin: SkillOrigin::Builtin,
        provider_id: "builtin".to_string(),
        ..Skill::default()
    }
}

#[tokio::test]
async fn scoped_state_and_user_deletion_survive_cold_restart() {
    const CHILD_MODE: &str = "UAR_B4_SCOPED_CHILD_MODE";
    const CHILD_ENDPOINT: &str = "UAR_B4_SCOPED_CHILD_ENDPOINT";
    const CHILD_SKILLS_DIR: &str = "UAR_B4_SCOPED_CHILD_SKILLS_DIR";

    if let Ok(mode) = std::env::var(CHILD_MODE) {
        let endpoint = std::env::var(CHILD_ENDPOINT).expect("child SurrealKV endpoint");
        let skills_dir = std::path::PathBuf::from(
            std::env::var(CHILD_SKILLS_DIR).expect("child filesystem skills directory"),
        );
        let persistence: Arc<dyn PersistenceLayer> = Arc::new(
            SurrealDbProvider::new(&endpoint, None, None, Some("b4"), Some("scoped"))
                .await
                .expect("open embedded SurrealKV database in child process"),
        );
        let mut service = SkillService::new(Some(Arc::clone(&persistence)), None);
        service.add_provider(Arc::new(DatabaseStorageProvider::new(
            "test-db",
            "Test database",
            Arc::clone(&persistence),
        )));
        service.add_provider(Arc::new(FilesystemStorageProvider::new(
            "fs-skills",
            "Filesystem skills",
            &skills_dir,
        )));

        let global_builtin = builtin("global-builtin");
        let agent_builtin = builtin("agent-builtin");
        match mode.as_str() {
            "seed" => {
                service
                    .register_builtins(vec![global_builtin, agent_builtin])
                    .await;
                assert!(
                    service
                        .set_scoped_enabled("global-builtin", SkillScope::Global, false)
                        .await
                );
                assert!(
                    service
                        .set_scoped_enabled(
                            "agent-builtin",
                            SkillScope::Agent("agent-a".to_string()),
                            false,
                        )
                        .await
                );
                let user = Skill {
                    skill_id: "user-delete-proof".to_string(),
                    version: "1.0.0".to_string(),
                    title: "user-delete-proof".to_string(),
                    description: "User skill deleted across restart".to_string(),
                    enabled: true,
                    ..Skill::default()
                };
                service.create_skill(user).await.expect("create user skill");
                assert!(
                    skills_dir
                        .join("dynamic/user-delete-proof/SKILL.md")
                        .is_file(),
                    "API-created skill reaches the filesystem provider"
                );
            }
            "reopen-delete" => {
                service
                    .register_builtins(vec![global_builtin, agent_builtin])
                    .await;
                service.initialize().await.expect("load reopened providers");
                assert!(
                    service
                        .match_skills_scoped(
                            "cold-restart-proof",
                            Some("agent-a"),
                            Some("conversation-b"),
                        )
                        .await
                        .is_empty(),
                    "global and per-agent disables survive a cold reopen"
                );
                assert_eq!(
                    service
                        .match_skills_scoped(
                            "cold-restart-proof",
                            Some("agent-b"),
                            Some("conversation-b"),
                        )
                        .await
                        .len(),
                    1,
                    "the per-agent disable does not affect another agent"
                );
                assert!(
                    service
                        .get_skills()
                        .await
                        .iter()
                        .any(|skill| skill.skill_id == "user-delete-proof"),
                    "user skill loads after the seed process exits"
                );
                let error = service
                    .delete_skill_permanent("global-builtin")
                    .await
                    .expect_err("builtin deletion remains refused");
                assert!(error.to_string().contains("system_skill_immutable"));
                assert!(
                    service
                        .delete_skill_permanent("user-delete-proof")
                        .await
                        .expect("user deletion succeeds")
                );
                assert!(
                    persistence
                        .list_skills()
                        .await
                        .expect("list rows after deletion")
                        .iter()
                        .all(|skill| skill.skill_id != "user-delete-proof"),
                    "user deletion removes the durable database row"
                );
                assert!(
                    !skills_dir.join("dynamic/user-delete-proof").exists(),
                    "user deletion removes the filesystem copy"
                );
            }
            "verify-deleted" => {
                service
                    .register_builtins(vec![global_builtin, agent_builtin])
                    .await;
                service
                    .initialize()
                    .await
                    .expect("load providers after deletion");
                let skills = service.get_skills().await;
                assert!(
                    skills
                        .iter()
                        .all(|skill| skill.skill_id != "user-delete-proof"),
                    "deleted user skill stays absent after another cold reopen"
                );
                assert!(
                    skills
                        .iter()
                        .any(|skill| skill.skill_id == "global-builtin"),
                    "refused builtin deletion leaves the builtin present"
                );
                assert!(
                    service
                        .match_skills_scoped(
                            "cold-restart-proof",
                            Some("agent-a"),
                            Some("conversation-b"),
                        )
                        .await
                        .is_empty(),
                    "scoped disables remain durable after the deletion boot"
                );
            }
            _ => panic!("unknown B4 child mode: {mode}"),
        }
        return;
    }

    let directory = tempfile::tempdir().expect("temporary B4 restart directory");
    let endpoint = format!(
        "surrealkv://{}",
        directory.path().join("scoped-governance.db").display()
    );
    let skills_dir = directory.path().join("skills");
    for mode in ["seed", "reopen-delete", "verify-deleted"] {
        let output = std::process::Command::new(
            std::env::current_exe().expect("current integration-test executable"),
        )
        .args([
            "--exact",
            "scoped_state_and_user_deletion_survive_cold_restart",
            "--test-threads=1",
        ])
        .env(CHILD_MODE, mode)
        .env(CHILD_ENDPOINT, &endpoint)
        .env(CHILD_SKILLS_DIR, &skills_dir)
        .output()
        .expect("run B4 cold-restart child process");
        assert!(
            output.status.success(),
            "B4 {mode} child failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[tokio::test]
async fn conversation_enable_widens_global_disable_and_in_flight_binding_is_stable() {
    let directory = tempfile::tempdir().expect("temporary database directory");
    let endpoint = format!(
        "surrealkv://{}",
        directory.path().join("scoped-governance.db").display()
    );
    let persistence: Arc<dyn PersistenceLayer> = Arc::new(
        SurrealDbProvider::new(&endpoint, None, None, Some("b4-run"), Some("b4-run"))
            .await
            .expect("embedded SurrealKV starts"),
    );
    let driver = Arc::new(GatedDriver::default());
    let runtime = EmbeddedRuntime::builder()
        .local_provider(driver.clone(), local_provider())
        .persistence(persistence)
        .seed_defaults(false)
        .build()
        .await
        .expect("embedded runtime builds");

    let skill = Skill {
        skill_id: "scoped-run-proof".to_string(),
        version: "1.0.0".to_string(),
        title: "Scoped run proof".to_string(),
        description: "Matches the scoped-run-proof phrase".to_string(),
        triggers: SkillTriggers {
            keywords: vec!["scoped-run-proof".to_string()],
            semantic: None,
        },
        prompt_overlay: "B4_IN_FLIGHT_BINDING_MARKER".to_string(),
        enabled: true,
        ..Skill::default()
    };
    runtime
        .skill_service()
        .create_skill(skill)
        .await
        .expect("skill is created");
    assert!(
        runtime
            .skill_service()
            .set_scoped_enabled("scoped-run-proof", SkillScope::Global, false)
            .await
    );
    assert!(
        runtime
            .skill_service()
            .set_scoped_enabled(
                "scoped-run-proof",
                SkillScope::Conversation("conversation-a".to_string()),
                true,
            )
            .await
    );

    let mut agent = default_agent();
    agent.id = "agent-a".to_string();
    agent.memory.kb.enabled = false;
    let first_run = runtime
        .run_manager()
        .start_run(
            agent.clone(),
            "please use scoped-run-proof".to_string(),
            Some("conversation-a".to_string()),
            None,
            Vec::new(),
        )
        .await;
    driver.wait_until_started().await;

    assert!(
        runtime
            .skill_service()
            .set_scoped_enabled(
                "scoped-run-proof",
                SkillScope::Conversation("conversation-a".to_string()),
                false,
            )
            .await
    );
    driver.release_one();
    let first_history = wait_for_done(&runtime, &first_run).await;
    assert!(first_history.iter().any(|event| matches!(
        &event.event,
        NormalizedEvent::SkillActivated { skill_id, .. } if skill_id == "scoped-run-proof"
    )));
    assert!(
        driver.requests()[0]
            .messages
            .iter()
            .any(|message| message.to_string().contains("B4_IN_FLIGHT_BINDING_MARKER"))
    );

    let second_run = runtime
        .run_manager()
        .start_run(
            agent,
            "please use scoped-run-proof".to_string(),
            Some("conversation-a".to_string()),
            None,
            Vec::new(),
        )
        .await;
    driver.wait_until_started().await;
    driver.release_one();
    let second_history = wait_for_done(&runtime, &second_run).await;
    assert!(second_history.iter().all(|event| !matches!(
        &event.event,
        NormalizedEvent::SkillActivated { skill_id, .. } if skill_id == "scoped-run-proof"
    )));
    assert!(
        driver.requests()[1]
            .messages
            .iter()
            .all(|message| !message.to_string().contains("B4_IN_FLIGHT_BINDING_MARKER"))
    );
}

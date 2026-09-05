//! Cross-provider contract tests for persisted agent-thread lineage.

use std::sync::Arc;

use universal_agent_runtime::{
    llm::{Message, MessageContent, MessageRole, ToolCall, ToolCallFunction},
    uar::{
        api::{adapters::to_agui_spec_event, sse::to_agui_event},
        domain::events::{AgentLifecycleOutcome, AgentLifecycleStatus, NormalizedEvent},
        persistence::{PersistenceLayer, agent_threads::PersistedAgentThread},
        runtime::thread::{
            AgentEdge, AgentThread, AgentThreadResult,
            limits::{AgentLimitError, AgentTreeAdmission, AgentTreeLimits},
            messages::InterAgentMessage,
            spawn::{AgentSpawnRequest, HistoryForkMode},
        },
    },
};

#[derive(Debug)]
struct ThreadFixture {
    owner_id: String,
    root: AgentThread,
    alpha: AgentThread,
    beta: AgentThread,
    alpha_edge: AgentEdge,
    beta_edge: AgentEdge,
}

impl ThreadFixture {
    fn new() -> Self {
        let suffix = uuid::Uuid::new_v4();
        let owner_id = format!("agent-thread-owner-{suffix}");
        let root = AgentThread::root(
            owner_id.clone(),
            "root-agent".to_owned(),
            format!("root-run-{suffix}"),
        )
        .expect("root thread fixture must be valid");
        let beta = AgentThread::child(&root, "beta-agent".to_owned(), Some("beta"))
            .expect("beta child fixture must be valid");
        let alpha = AgentThread::child(&root, "alpha-agent".to_owned(), Some("alpha"))
            .expect("alpha child fixture must be valid");
        let beta_edge = AgentEdge::between(&root, &beta).expect("beta edge fixture must be valid");
        let alpha_edge =
            AgentEdge::between(&root, &alpha).expect("alpha edge fixture must be valid");
        Self {
            owner_id,
            root,
            alpha,
            beta,
            alpha_edge,
            beta_edge,
        }
    }

    fn expected(&self) -> ProviderSnapshot {
        ProviderSnapshot {
            threads: vec![
                ThreadProjection::from(&self.root),
                ThreadProjection::from(&self.alpha),
                ThreadProjection::from(&self.beta),
            ],
            edges: vec![
                EdgeProjection::from(&self.alpha_edge),
                EdgeProjection::from(&self.beta_edge),
            ],
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ThreadProjection {
    thread_id: String,
    parent_thread_id: Option<String>,
    canonical_path: String,
    artifact_id: String,
}

impl From<&AgentThread> for ThreadProjection {
    fn from(thread: &AgentThread) -> Self {
        Self {
            thread_id: thread.thread_id.clone(),
            parent_thread_id: thread.parent_thread_id.clone(),
            canonical_path: thread.canonical_path.clone(),
            artifact_id: thread.artifact_id.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct EdgeProjection {
    parent_thread_id: String,
    child_thread_id: String,
    canonical_path: String,
    artifact_id: String,
}

impl From<&AgentEdge> for EdgeProjection {
    fn from(edge: &AgentEdge) -> Self {
        Self {
            parent_thread_id: edge.parent_thread_id.clone(),
            child_thread_id: edge.child_thread_id.clone(),
            canonical_path: edge.canonical_path.clone(),
            artifact_id: edge.artifact_id.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ProviderSnapshot {
    threads: Vec<ThreadProjection>,
    edges: Vec<EdgeProjection>,
}

async fn persist_out_of_order(
    provider: Arc<dyn PersistenceLayer>,
    fixture: &ThreadFixture,
) -> ProviderSnapshot {
    let root = provider
        .create_agent_root(&fixture.owner_id, &fixture.root)
        .await
        .expect("provider must persist the root thread");
    assert_eq!(root.thread, fixture.root);

    let beta = provider
        .create_agent_child(&fixture.owner_id, &fixture.beta, &fixture.beta_edge)
        .await
        .expect("provider must atomically persist the beta child and edge");
    assert_eq!(beta.thread, fixture.beta);
    let alpha = provider
        .create_agent_child(&fixture.owner_id, &fixture.alpha, &fixture.alpha_edge)
        .await
        .expect("provider must atomically persist the alpha child and edge");
    assert_eq!(alpha.thread, fixture.alpha);

    let threads = provider
        .list_agent_threads(&fixture.owner_id, &fixture.root.root_run_id)
        .await
        .expect("provider must list the complete thread tree")
        .iter()
        .map(|record| ThreadProjection::from(&record.thread))
        .collect();
    let edges = provider
        .list_agent_edges(&fixture.owner_id, &fixture.root.root_run_id)
        .await
        .expect("provider must list the complete edge set")
        .iter()
        .map(EdgeProjection::from)
        .collect();

    ProviderSnapshot { threads, edges }
}

fn message(role: MessageRole, content: &str) -> Message {
    Message {
        role,
        content: MessageContent::text(content),
        tool_call_id: None,
        tool_calls: None,
    }
}

fn spawn_request(history_fork: HistoryForkMode) -> AgentSpawnRequest {
    AgentSpawnRequest {
        artifact_id: "child-agent".to_owned(),
        delegated_prompt: "perform the delegated work".to_owned(),
        task_name: Some("delegated-child".to_owned()),
        history_fork,
    }
}

fn admission_root() -> AgentThread {
    AgentThread::root(
        "limit-test-owner".to_owned(),
        "root-agent".to_owned(),
        format!("limit-root-{}", uuid::Uuid::new_v4()),
    )
    .expect("limit-test root must be valid")
}

fn commit_admission_child(admission: &AgentTreeAdmission, child: &AgentThread) {
    let mut reservation = admission
        .reserve_child(child)
        .expect("child below every tree limit must be reserved");
    reservation.begin_persistence();
    drop(
        reservation
            .commit()
            .expect("persisted child must commit its admission reservation"),
    );
}

#[test]
fn tree_limits_refuse_the_fifth_concurrent_child() {
    let root = admission_root();
    let admission = AgentTreeAdmission::new(root.clone(), AgentTreeLimits::default())
        .expect("default tree limits must accept a root");
    let children = (1..=5)
        .map(|index| {
            AgentThread::child(
                &root,
                format!("child-agent-{index}"),
                Some(&format!("concurrent-{index}")),
            )
            .expect("concurrent child fixture must be valid")
        })
        .collect::<Vec<_>>();
    let reservations = children[..4]
        .iter()
        .map(|child| {
            admission
                .reserve_child(child)
                .expect("the first four concurrent children must be admitted")
        })
        .collect::<Vec<_>>();

    assert_eq!(
        admission
            .reserve_child(&children[4])
            .expect_err("the fifth concurrent child must be refused"),
        AgentLimitError::ConcurrentChildren { limit: 4 }
    );
    assert_eq!(admission.counts().unwrap().concurrent_children, 4);
    drop(reservations);
}

#[test]
fn tree_limits_refuse_the_fourth_child_nesting_level() {
    let root = admission_root();
    let admission = AgentTreeAdmission::new(root.clone(), AgentTreeLimits::default())
        .expect("default tree limits must accept a root");
    let depth_one = AgentThread::child(&root, "depth-one-agent".to_owned(), Some("one"))
        .expect("depth-one child must be valid");
    commit_admission_child(&admission, &depth_one);
    let depth_two = AgentThread::child(&depth_one, "depth-two-agent".to_owned(), Some("two"))
        .expect("depth-two child must be valid");
    commit_admission_child(&admission, &depth_two);
    let depth_three = AgentThread::child(&depth_two, "depth-three-agent".to_owned(), Some("three"))
        .expect("depth-three child must be valid");
    commit_admission_child(&admission, &depth_three);
    let depth_four = AgentThread::child(&depth_three, "depth-four-agent".to_owned(), Some("four"))
        .expect("depth-four child record must be structurally valid");

    assert_eq!(
        admission
            .reserve_child(&depth_four)
            .expect_err("the fourth child nesting level must be refused"),
        AgentLimitError::Depth { limit: 3 }
    );
}

#[test]
fn tree_limits_refuse_the_seventeenth_lifetime_child() {
    let root = admission_root();
    let admission = AgentTreeAdmission::new(root.clone(), AgentTreeLimits::default())
        .expect("default tree limits must accept a root");
    for index in 1..=16 {
        let child = AgentThread::child(
            &root,
            format!("child-agent-{index}"),
            Some(&format!("lifetime-{index:02}")),
        )
        .expect("lifetime child fixture must be valid");
        commit_admission_child(&admission, &child);
    }
    let seventeenth = AgentThread::child(&root, "child-agent-17".to_owned(), Some("lifetime-17"))
        .expect("seventeenth child record must be structurally valid");

    assert_eq!(
        admission
            .reserve_child(&seventeenth)
            .expect_err("the seventeenth lifetime child must be refused"),
        AgentLimitError::TotalChildren { limit: 16 }
    );
    assert_eq!(admission.counts().unwrap().total_children, 16);
}

#[test]
fn child_history_fork_none_contains_only_the_delegated_prompt() {
    let parent_history = vec![
        message(MessageRole::System, "root-only authority"),
        message(MessageRole::User, "parent question"),
        message(MessageRole::Assistant, "parent answer"),
    ];

    let child_messages = spawn_request(HistoryForkMode::None)
        .initial_messages(&parent_history)
        .expect("valid child request must build its initial messages");

    assert_eq!(child_messages.len(), 1);
    assert!(matches!(child_messages[0].role, MessageRole::User));
    assert_eq!(
        child_messages[0].content.as_text(),
        Some("perform the delegated work")
    );
    assert!(child_messages[0].tool_call_id.is_none());
    assert!(child_messages[0].tool_calls.is_none());
}

#[test]
fn child_history_fork_last_two_keeps_dialogue_and_drops_tool_traffic() {
    let parent_history = vec![
        message(MessageRole::System, "root-only authority"),
        message(MessageRole::User, "discarded turn"),
        message(MessageRole::Assistant, "discarded answer"),
        message(MessageRole::User, "retained tool turn"),
        Message {
            role: MessageRole::Assistant,
            content: MessageContent::text("intermediate answer"),
            tool_call_id: None,
            tool_calls: Some(vec![ToolCall {
                id: "call-1".to_owned(),
                call_type: "function".to_owned(),
                function: ToolCallFunction {
                    name: "lookup".to_owned(),
                    arguments: r#"{"query":"private"}"#.to_owned(),
                },
            }]),
        },
        Message {
            role: MessageRole::Tool,
            content: MessageContent::text("private tool output"),
            tool_call_id: Some("call-1".to_owned()),
            tool_calls: None,
        },
        message(MessageRole::Assistant, "retained final answer"),
        message(MessageRole::User, "retained open turn"),
    ];

    let child_messages = spawn_request(HistoryForkMode::LastTurns(2))
        .initial_messages(&parent_history)
        .expect("valid child request must build its initial messages");
    let visible = child_messages
        .iter()
        .map(|entry| (entry.role, entry.content.as_text().unwrap_or_default()))
        .collect::<Vec<_>>();

    assert_eq!(
        visible,
        vec![
            (MessageRole::User, "retained tool turn"),
            (MessageRole::Assistant, "retained final answer"),
            (MessageRole::User, "retained open turn"),
            (MessageRole::User, "perform the delegated work"),
        ]
    );
    assert!(child_messages.iter().all(|entry| {
        entry.tool_call_id.is_none()
            && entry.tool_calls.is_none()
            && entry.content.as_text() != Some("private tool output")
            && entry.content.as_text() != Some("intermediate answer")
    }));
}

#[test]
fn inter_agent_message_keeps_sender_identity_out_of_the_child_user_turn() {
    let fixture = ThreadFixture::new();
    let envelope = InterAgentMessage::between(
        &fixture.alpha,
        &fixture.beta,
        1,
        "continue with the delegated work".to_owned(),
        true,
    )
    .expect("siblings in one root tree may exchange a typed message");

    assert_eq!(envelope.sender_thread_id, fixture.alpha.thread_id);
    assert_eq!(envelope.sender_artifact_id, fixture.alpha.artifact_id);
    assert_eq!(envelope.recipient_thread_id, fixture.beta.thread_id);
    assert_eq!(envelope.root_thread_id, fixture.root.thread_id);
    assert_eq!(envelope.root_run_id, fixture.root.root_run_id);
    assert_eq!(envelope.sequence, 1);
    assert!(envelope.trigger_turn);

    let child_turn = envelope.user_message();
    assert_eq!(child_turn.role, MessageRole::User);
    assert_eq!(
        child_turn.content.as_text(),
        Some("continue with the delegated work")
    );
    assert!(child_turn.tool_call_id.is_none());
    assert!(child_turn.tool_calls.is_none());
    assert!(
        !child_turn
            .content
            .as_text()
            .unwrap()
            .contains(&fixture.alpha.thread_id)
    );
    assert!(
        !child_turn
            .content
            .as_text()
            .unwrap()
            .contains(&fixture.alpha.artifact_id)
    );
}

#[test]
fn lifecycle_events_are_content_free_and_drive_both_agui_mappings() {
    let fixture = ThreadFixture::new();
    let parent = PersistedAgentThread {
        thread: fixture.root.clone(),
        revision: 0,
    };
    let pending = PersistedAgentThread {
        thread: fixture.alpha.clone(),
        revision: 0,
    };
    let mut running_thread = fixture.alpha.clone();
    running_thread
        .begin_turn("child-run".to_owned())
        .expect("child turn starts");
    let running = PersistedAgentThread {
        thread: running_thread.clone(),
        revision: 1,
    };
    let started = running
        .lifecycle_event(&fixture.owner_id, &parent, Some(&pending))
        .expect("confirmed start projects")
        .expect("start changes lifecycle state");
    let NormalizedEvent::AgentThreadStarted { run_id, lifecycle } = &started else {
        panic!("running transition must emit AgentThreadStarted");
    };
    assert_eq!(run_id, &fixture.root.root_run_id);
    assert_eq!(lifecycle.parent_thread_id, fixture.root.thread_id);
    assert_eq!(lifecycle.child_thread_id, fixture.alpha.thread_id);
    assert_eq!(lifecycle.canonical_path, fixture.alpha.canonical_path);
    assert_eq!(lifecycle.status, AgentLifecycleStatus::Running);
    assert_eq!(lifecycle.terminal_outcome, None);
    assert_eq!(lifecycle.child_run_id.as_deref(), Some("child-run"));
    let (started_name, started_payload) =
        to_agui_spec_event(&started).expect("started lifecycle maps to AG-UI");
    assert_eq!(started_name, "SUBAGENT_STARTED");
    assert_eq!(started_payload["subagentRunId"], "child-run");
    assert_eq!(started_payload["name"], fixture.alpha.canonical_path);

    let private_output = "private child output that must not enter lifecycle events";
    let mut completed_thread = running_thread;
    completed_thread
        .finish_turn(AgentThreadResult::Completed {
            output: private_output.to_owned(),
        })
        .expect("child turn completes");
    let completed = PersistedAgentThread {
        thread: completed_thread,
        revision: 2,
    };
    let finished = completed
        .lifecycle_event(&fixture.owner_id, &parent, Some(&running))
        .expect("confirmed completion projects")
        .expect("completion changes lifecycle state");
    let NormalizedEvent::AgentThreadFinished { lifecycle, .. } = &finished else {
        panic!("completed transition must emit AgentThreadFinished");
    };
    assert_eq!(lifecycle.parent_thread_id, fixture.root.thread_id);
    assert_eq!(lifecycle.child_thread_id, fixture.alpha.thread_id);
    assert_eq!(lifecycle.canonical_path, fixture.alpha.canonical_path);
    assert_eq!(lifecycle.status, AgentLifecycleStatus::Completed);
    assert_eq!(
        lifecycle.terminal_outcome,
        Some(AgentLifecycleOutcome::Completed)
    );
    assert!(
        !serde_json::to_string(&finished)
            .expect("lifecycle event serializes")
            .contains(private_output)
    );

    let (finished_name, finished_payload) =
        to_agui_spec_event(&finished).expect("finished lifecycle maps to spec AG-UI");
    assert_eq!(finished_name, "SUBAGENT_FINISHED");
    assert_eq!(
        finished_payload["lifecycle"]["terminal_outcome"],
        "completed"
    );
    let (legacy_name, legacy_payload) =
        to_agui_event(&finished).expect("finished lifecycle maps to legacy AG-UI");
    assert_eq!(legacy_name, "agui.subagent.finished");
    assert_eq!(legacy_payload["kind"], "subagent");
    assert_eq!(legacy_payload["phase"], "finished");
    assert!(!finished_payload.to_string().contains(private_output));
    assert!(!legacy_payload.to_string().contains(private_output));
}

#[cfg(feature = "in-memory-backend")]
#[tokio::test]
async fn in_memory_spawn_persists_identically_ordered_threads_and_edges() {
    use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;

    let fixture = ThreadFixture::new();
    let provider: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());

    assert_eq!(
        persist_out_of_order(provider, &fixture).await,
        fixture.expected()
    );
}

#[cfg(feature = "surreal-backend")]
#[tokio::test]
async fn surreal_spawn_persists_identically_ordered_threads_and_edges() {
    use universal_agent_runtime::uar::persistence::providers::surreal::SurrealDbProvider;

    let fixture = ThreadFixture::new();
    let directory = tempfile::tempdir().expect("SurrealKV test directory must be created");
    let endpoint = format!(
        "surrealkv://{}",
        directory.path().join("threads.db").display()
    );
    let provider: Arc<dyn PersistenceLayer> = Arc::new(
        SurrealDbProvider::new(
            &endpoint,
            None,
            None,
            Some("agent-threads"),
            Some("provider-contract"),
        )
        .await
        .expect("embedded SurrealKV provider must start"),
    );

    assert_eq!(
        persist_out_of_order(provider, &fixture).await,
        fixture.expected()
    );
}

#[cfg(feature = "postgres-backend")]
#[tokio::test]
#[serial_test::serial]
async fn postgres_spawn_persists_identically_ordered_threads_and_edges() {
    use universal_agent_runtime::uar::persistence::providers::postgres::PostgresProvider;

    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!(
            "SKIPPED agent_threads postgres contract: DATABASE_URL is unset; \
             PostgreSQL thread and edge ordering remains unverified"
        );
        return;
    };
    let fixture = ThreadFixture::new();
    let provider = Arc::new(
        PostgresProvider::new(&database_url)
            .await
            .expect("PostgreSQL provider must connect and migrate"),
    );
    let snapshot = persist_out_of_order(provider.clone(), &fixture).await;

    sqlx::query("DELETE FROM agent_edges WHERE owner_id = $1 AND root_run_id = $2")
        .bind(&fixture.owner_id)
        .bind(&fixture.root.root_run_id)
        .execute(provider.get_pool())
        .await
        .expect("PostgreSQL edge fixtures must be removed");
    sqlx::query("DELETE FROM agent_threads WHERE owner_id = $1 AND root_run_id = $2")
        .bind(&fixture.owner_id)
        .bind(&fixture.root.root_run_id)
        .execute(provider.get_pool())
        .await
        .expect("PostgreSQL thread fixtures must be removed");

    assert_eq!(snapshot, fixture.expected());
}

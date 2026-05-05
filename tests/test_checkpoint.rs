/// Integration tests for checkpoint save / load / list via SurrealDB in-process.
use std::sync::Arc;

use universal_agent_runtime::uar::{
    persistence::{PersistenceLayer, providers::surreal::SurrealDbProvider},
    runtime::{checkpoint::Checkpoint, graph::GraphState},
};

async fn make_db() -> (Arc<dyn PersistenceLayer>, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let url = format!("surrealkv://{}", dir.path().to_str().unwrap());
    let provider = Arc::new(
        SurrealDbProvider::new(&url, None, None)
            .await
            .expect("SurrealDB init failed"),
    );
    (provider, dir)
}

fn make_state(iter: u32) -> GraphState {
    let mut state = GraphState::default();
    state.iteration = iter;
    state.set("key", format!("value-{iter}"));
    state
        .messages
        .push(serde_json::json!({"role": "user", "content": "hello"}));
    state
}

#[tokio::test]
async fn test_save_and_load_checkpoint() {
    let (db, _dir) = make_db().await;

    let state = make_state(3);
    let cp = Checkpoint::new("run-1", "thread-1", "node-a", &state);
    let cp_id = cp.id.clone();

    db.save_checkpoint(&cp)
        .await
        .expect("save_checkpoint failed");

    let loaded = db
        .load_checkpoint(&cp_id)
        .await
        .expect("load_checkpoint failed")
        .expect("checkpoint should exist");

    assert_eq!(loaded.run_id, "run-1");
    assert_eq!(loaded.thread_id, "thread-1");
    assert_eq!(loaded.node_id, "node-a");
    assert_eq!(loaded.iteration, 3);
    assert_eq!(loaded.messages.len(), 1);

    let restored = loaded.restore_state();
    assert_eq!(restored.iteration, 3);
    assert_eq!(restored.get::<String>("key").as_deref(), Some("value-3"));
}

#[tokio::test]
async fn test_load_nonexistent_checkpoint_returns_none() {
    let (db, _dir) = make_db().await;
    let result = db
        .load_checkpoint("does-not-exist")
        .await
        .expect("load_checkpoint should not error");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_list_checkpoints_for_run() {
    let (db, _dir) = make_db().await;

    // Save 3 checkpoints for the same run and 1 for a different run.
    for i in 0u32..3 {
        let state = make_state(i);
        let cp = Checkpoint::new("run-multi", "thread-1", format!("node-{i}"), &state);
        db.save_checkpoint(&cp).await.expect("save");
    }
    let other = Checkpoint::new("run-other", "thread-2", "node-0", &make_state(0));
    db.save_checkpoint(&other).await.expect("save other");

    let checkpoints = db
        .list_checkpoints("run-multi")
        .await
        .expect("list_checkpoints");

    assert_eq!(
        checkpoints.len(),
        3,
        "should list exactly 3 checkpoints for run-multi"
    );
    for cp in &checkpoints {
        assert_eq!(cp.run_id, "run-multi");
    }
}

#[tokio::test]
async fn test_checkpoint_node_persists_via_graph_context() {
    use universal_agent_runtime::{
        llm::mock_driver::MockLlmDriver,
        mcp::registry::McpRegistry,
        normalized::NormalizedEvent,
        uar::runtime::graph::{AgentGraph, GraphContext, GraphState},
    };

    let (db, _dir) = make_db().await;

    // Build a simple graph with a CheckpointNode
    let graph = AgentGraph::builder("entry")
        .add_node(
            universal_agent_runtime::uar::runtime::graph::CheckpointNode::new("entry", "start"),
        )
        .build();

    let driver = Arc::new(MockLlmDriver::new(vec![vec![
        NormalizedEvent::MessageDelta {
            text: "hi".to_string(),
        },
    ]]));

    let ctx = GraphContext {
        run_id: "run-cp-test".to_string(),
        session_id: Some("session-1".to_string()),
        mcp: Arc::new(McpRegistry::new_empty()),
        llm_config: universal_agent_runtime::config::LlmConfig::default(),
        driver,
        persistence: Some(Arc::clone(&db)),
    };

    let state = GraphState::default();
    graph.execute(state, &ctx).await;

    // Verify a checkpoint was saved
    let checkpoints = db
        .list_checkpoints("run-cp-test")
        .await
        .expect("list_checkpoints");

    assert_eq!(
        checkpoints.len(),
        1,
        "CheckpointNode should have saved one checkpoint"
    );
    assert_eq!(checkpoints[0].run_id, "run-cp-test");
    assert_eq!(checkpoints[0].node_id, "entry");
}

use super::*;
use crate::uar::a2ui::presentation_selection::PresentationNegotiation;
use crate::uar::rag::embeddings::UnavailableEmbeddingBackend;
use crate::uar::runtime::actor::messages::ActorOwner;
use crate::uar::runtime::presentations::RunPresentationSnapshot;
use crate::uar::security::claims::{TenantId, UserClaims, UserContext};
use serde_json::{Value, json};

fn user(tenant: &str) -> UserContext {
    UserContext {
        user_id: "operator".into(),
        tenant_id: Some(TenantId::for_test(tenant)),
        claims: UserClaims {
            sub: "operator".into(),
            name: None,
            roles: None,
            tenant_id: Some(tenant.into()),
            uar_instance_id: None,
            exp: usize::MAX,
        },
    }
}

async fn manager_with_run(captured: bool) -> RunManager {
    let sessions = SessionStore::new();
    let dialogue = RunDialogue(sessions.get_or_create_for_user("conversation", "operator"));
    let manager = RunManager::new(
        LlmConfig::default(),
        Arc::new(McpRegistry::new_empty()),
        sessions,
        Arc::new(RwLock::new(SkillRegistry::default())),
        Arc::new(crate::uar::runtime::matching::VectorMatcher::new(
            Arc::new(UnavailableEmbeddingBackend::new(
                384,
                "this history test performs no embedding inference",
            )),
            0.75,
        )),
        None,
    )
    .await;
    let owner = ActorOwner::from_verified_context(&user("tenant-a")).unwrap();
    let (snapshot, _) = RunPresentationSnapshot::capture(
        None,
        Some(owner.clone()),
        &resolve_run_policy(PolicyResolutionInput::default()),
        PresentationNegotiation::default(),
    )
    .await;
    let (sender, _) = broadcast::channel(1024);
    manager.active_runs.write().await.insert(
        "run".into(),
        RunStreamState {
            run: Run {
                run_id: "run".into(),
                agent_id: "agent".into(),
                conversation_id: None,
                user_id: Some("operator".into()),
                status: RunStatus::Running,
                context: json!({}),
            },
            verified_owner: Some(owner),
            presentations: captured.then(|| Arc::new(snapshot)),
            dialogue,
            sender,
            history: Arc::new(Mutex::new(EventHistory {
                next_id: 1,
                buffer: VecDeque::new(),
                presentation: None,
                latest_presentation: None,
            })),
            completion: None,
            delegation: None,
        },
    );
    manager
}

fn provenance(event: &StreamEvent) -> Option<&Value> {
    if let NormalizedEvent::StatePatch { patch, .. } = &event.event {
        return patch
            .iter()
            .find(|op| op.path == "/presentation")
            .and_then(|op| op.value.as_ref());
    }
    None
}

#[tokio::test]
async fn retained_provenance_keeps_original_sequence_after_ring_eviction() {
    let manager = manager_with_run(true).await;
    manager
        .emit_to_run(
            "run",
            NormalizedEvent::RunStart {
                run_id: "run".into(),
                agent_id: "agent".into(),
            },
        )
        .await;
    let initial = manager.history_since("run", None).await.unwrap();
    let projection = initial
        .iter()
        .find(|event| provenance(event).is_some())
        .unwrap()
        .clone();
    for _ in 0..EVENT_HISTORY_LIMIT + 20 {
        manager
            .emit_to_run(
                "run",
                NormalizedEvent::ChatDelta {
                    run_id: "run".into(),
                    text_delta: "token".into(),
                },
            )
            .await;
    }
    let retained = manager.history_since("run", None).await.unwrap();
    assert_eq!(retained.len(), EVENT_HISTORY_LIMIT + 1);
    assert_eq!(retained[0].id, projection.id);
    assert_eq!(retained[0].event, projection.event);
    assert_eq!(
        retained
            .iter()
            .filter(|event| provenance(event).is_some())
            .count(),
        1
    );
    assert!(retained.windows(2).all(|pair| pair[0].id < pair[1].id));
    let after = manager
        .history_since("run", Some(projection.id))
        .await
        .unwrap();
    assert_eq!(after.len(), EVENT_HISTORY_LIMIT);
    assert!(after.iter().all(|event| provenance(event).is_none()));
}

#[tokio::test]
async fn publication_projection_follows_output_and_terminal_projection_precedes_closure() {
    let manager = manager_with_run(true).await;
    manager
        .emit_to_run(
            "run",
            NormalizedEvent::StatePatch {
                run_id: "run".into(),
                patch: vec![StatePatchOp {
                    op: "add".into(),
                    path: "/a2ui/surfaces/one".into(),
                    value: Some(json!({})),
                }],
            },
        )
        .await;
    let published = manager.history_since("run", None).await.unwrap();
    assert!(provenance(&published[0]).is_none());
    assert_eq!(
        provenance(&published[1]).unwrap()["surface_published"],
        true
    );
    manager
        .emit_to_run(
            "run",
            NormalizedEvent::RunDone {
                run_id: "run".into(),
            },
        )
        .await;
    let finished = manager
        .history_since("run", Some(published[1].id))
        .await
        .unwrap();
    assert_eq!(finished.len(), 2);
    assert_eq!(provenance(&finished[0]).unwrap()["run_outcome"], "finished");
    assert!(matches!(finished[1].event, NormalizedEvent::RunDone { .. }));
}

#[tokio::test]
async fn root_replacement_restores_host_provenance_and_forgery_is_only_diagnostic() {
    let manager = manager_with_run(true).await;
    manager
        .emit_to_run(
            "run",
            NormalizedEvent::RunStart {
                run_id: "run".into(),
                agent_id: "agent".into(),
            },
        )
        .await;
    let admission = manager.history_since("run", None).await.unwrap();
    let previous = admission
        .iter()
        .find(|event| provenance(event).is_some())
        .unwrap();
    for value in [
        json!({"ordinary": true}),
        json!({"presentation": {"client_display": "confirmed"}}),
    ] {
        manager
            .emit_to_run(
                "run",
                NormalizedEvent::StatePatch {
                    run_id: "run".into(),
                    patch: vec![StatePatchOp {
                        op: "replace".into(),
                        path: "".into(),
                        value: Some(value),
                    }],
                },
            )
            .await;
    }
    let events = manager
        .history_since("run", Some(previous.id))
        .await
        .unwrap();
    assert_eq!(events.len(), 3);
    assert!(matches!(
        events[0].event,
        NormalizedEvent::StatePatch { .. }
    ));
    assert_eq!(
        provenance(&events[1]).unwrap()["client_display"],
        "unconfirmed"
    );
    assert_eq!(provenance(&events[1]), provenance(previous));
    assert!(events[1].id > previous.id);
    assert!(matches!(
        events[2].event,
        NormalizedEvent::PresentationDiagnostic { .. }
    ));
    manager
        .emit_to_run(
            "run",
            NormalizedEvent::ChatDelta {
                run_id: "run".into(),
                text_delta: "still readable".into(),
            },
        )
        .await;
    assert!(matches!(
        manager
            .history_since("run", Some(events[2].id))
            .await
            .unwrap()[0]
            .event,
        NormalizedEvent::ChatDelta { .. }
    ));
}

#[tokio::test]
async fn pre_admission_failure_stream_still_requires_exact_subject_and_tenant() {
    for captured in [false, true] {
        let manager = manager_with_run(captured).await;
        assert!(
            manager
                .get_run_for_context(&user("tenant-a"), "run")
                .await
                .is_some()
        );
        assert!(
            manager
                .get_run_for_context(&user("tenant-b"), "run")
                .await
                .is_none()
        );
        let mut inconsistent = user("tenant-a");
        inconsistent.claims.sub = "other".into();
        assert!(
            manager
                .get_run_for_context(&inconsistent, "run")
                .await
                .is_none()
        );
    }
}

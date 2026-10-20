//! Exact broker scenarios, shared by standalone roots and actor descendants.
use super::*;

#[derive(Default)]
struct RecordingSink {
    events: AsyncMutex<Vec<NormalizedEvent>>,
}

#[async_trait::async_trait]
impl RuntimeEventSink for RecordingSink {
    async fn emit(&self, event: NormalizedEvent) {
        self.events.lock().await.push(event);
    }
}

async fn originating_id(sink: &RecordingSink, tool_id: &str) -> String {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(id) = sink
                .events
                .lock()
                .await
                .iter()
                .find_map(|event| match event {
                    NormalizedEvent::ToolCallApprovalRequired {
                        tool_call_id,
                        approval_id,
                        ..
                    } if tool_call_id == tool_id => approval_id.clone(),
                    _ => None,
                })
            {
                return id;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("originating approval event must be published")
}

fn request(
    channel: RootApprovalChannel,
    call: &str,
    cancellation: CancellationToken,
) -> tokio::task::JoinHandle<ApprovalOutcome> {
    let call = call.to_owned();
    tokio::spawn(async move {
        let invocation =
            crate::uar::runtime::tool_admission::PreparedToolInvocation {
                representation_effect: crate::uar::tools::descriptor::ToolEffect::Unknown,
                admission_owner:
                    crate::uar::persistence::tool_admission::AdmissionOwner::PairedHost,
                version: 1,
                execution_kind:
                    crate::uar::runtime::tool_admission::ToolExecutionKind::HostMcp,
                invocation_id: "fixture-invocation".to_owned(),
                model_tool_call_id: call.clone(),
                attempt: 0,
                root_run_id: "root".to_owned(),
                executing_run_id: "root".to_owned(),
                owner_id: "operator".to_owned(),
                principal_id: "operator".to_owned(),
                workspace: "default".to_owned(),
                runtime_epoch: "test".to_owned(),
                host_epoch: "test".to_owned(),
                catalog_revision: "test".to_owned(),
                mounted_server_id: "fixture".to_owned(),
                native_tool_name: "write".to_owned(),
                provider_tool_name: "write".to_owned(),
                run_policy_revision: "test".to_owned(),
                governance_policy_revision: "test".to_owned(),
                tool_policy_revision: "test".to_owned(),
                resource_revision: "test".to_owned(),
                payload_revision: "test".to_owned(),
                grant_revision: "test".to_owned(),
                lease_revision: "test".to_owned(),
                budget_revision: "test".to_owned(),
                lease: crate::uar::runtime::tool_admission::EffectLeaseFacts {
                    lease_id: String::new(),
                    task: String::new(),
                    attempt: 0,
                    epoch: String::new(),
                    holder: String::new(),
                    expires_at: 0,
                    active: false,
                },
                budget_reservation:
                    crate::uar::runtime::tool_admission::EffectBudgetReservationFacts {
                        reservation_id: String::new(),
                        budget_id: String::new(),
                        revision: String::new(),
                        amount: 0,
                        unit: String::new(),
                        expires_at: 0,
                        active: false,
                    },
                authority_revision: "test".to_owned(),
                approval_class: crate::uar::tools::descriptor::ApprovalClass::Required,
                call_index: 0,
                validated_arguments: serde_json::json!({"path":"scoped"}),
            };
        channel
            .request_tool(
                crate::uar::runtime::tool_admission::ToolApprovalRequest {
                    invocation: std::sync::Arc::new(invocation),
                    admission_id: "prepared-admission".to_owned(),
                    host_requires_approval: false,
                    action_display: serde_json::json!({"path":"scoped"}),
                    checkpoint_history: None,
                },
                crate::uar::persistence::tool_admission::AdmissionOwner::PairedHost,
                "write effect".to_owned(),
                &cancellation,
            )
            .await
    })
}

#[tokio::test]
async fn roots_and_children_require_exact_ids_without_consuming_other_pending_requests() {
    for child in [false, true] {
        let broker = ApprovalBroker::default();
        let sink = Arc::new(RecordingSink::default());
        let root = broker
            .register(
                "root".to_owned(),
                "owner".to_owned(),
                sink.clone(),
                CancellationToken::new(),
            )
            .unwrap();
        let channel = if child {
            root.for_child()
        } else {
            root.clone()
        };
        let first = request(channel.clone(), "call-one", CancellationToken::new());
        let first_id = originating_id(&sink, "call-one").await;
        assert!(broker.pending("other-owner", "root").is_none());
        for id in [None, Some(""), Some(" "), Some("stale-invocation")] {
            assert!(!broker.resolve("root", id, true).await);
            assert_eq!(
                broker.pending("owner", "root").unwrap().approval_id,
                first_id
            );
            assert!(!first.is_finished());
        }
        assert!(!broker.resolve("other-run", Some(&first_id), true).await);
        assert!(broker.resolve("root", Some(&first_id), true).await);
        assert!(!broker.resolve("root", Some(&first_id), true).await);
        assert_eq!(first.await.unwrap(), ApprovalOutcome::Approved);
        let second = request(channel, "call-two", CancellationToken::new());
        let second_id = originating_id(&sink, "call-two").await;
        assert_ne!(first_id, second_id);
        assert!(!broker.resolve("root", Some(&first_id), true).await);
        assert_eq!(
            broker.pending("owner", "root").unwrap().approval_id,
            second_id
        );
        assert!(broker.resolve("root", Some(&second_id), false).await);
        assert_eq!(second.await.unwrap(), ApprovalOutcome::Rejected);
    }
}

#[tokio::test]
async fn cancellation_and_dropped_waiters_cannot_be_revived() {
    let broker = ApprovalBroker::default();
    let sink = Arc::new(RecordingSink::default());
    let root_cancel = CancellationToken::new();
    let root = broker
        .register(
            "root".to_owned(),
            "owner".to_owned(),
            sink.clone(),
            root_cancel.clone(),
        )
        .unwrap();
    let child_cancel = CancellationToken::new();
    let child = request(root.for_child(), "cancelled-child", child_cancel.clone());
    let child_id = originating_id(&sink, "cancelled-child").await;
    child_cancel.cancel();
    // Resolve immediately, before the request future necessarily polls cancellation.
    assert!(!broker.resolve("root", Some(&child_id), true).await);
    assert!(broker.pending("owner", "root").is_none());
    assert_eq!(child.await.unwrap(), ApprovalOutcome::Cancelled);
    let dropped = request(root.clone(), "dropped", CancellationToken::new());
    let dropped_id = originating_id(&sink, "dropped").await;
    dropped.abort();
    let _ = dropped.await;
    assert!(!broker.resolve("root", Some(&dropped_id), true).await);
    let last = request(root, "cancelled-root", CancellationToken::new());
    let last_id = originating_id(&sink, "cancelled-root").await;
    root_cancel.cancel();
    assert!(!broker.resolve("root", Some(&last_id), true).await);
    assert_eq!(last.await.unwrap(), ApprovalOutcome::Cancelled);
}

#[tokio::test]
async fn concurrent_decisions_deliver_at_most_one_exact_resolution() {
    let broker = ApprovalBroker::default();
    let sink = Arc::new(RecordingSink::default());
    let root = broker
        .register(
            "root".to_owned(),
            "owner".to_owned(),
            sink.clone(),
            CancellationToken::new(),
        )
        .unwrap();
    let waiter = request(root, "one-effect", CancellationToken::new());
    let id = originating_id(&sink, "one-effect").await;
    let other = broker.clone();
    let duplicate = id.clone();
    let first = tokio::spawn(async move { other.resolve("root", Some(&duplicate), true).await });
    let second = broker.resolve("root", Some(&id), true).await;
    assert_ne!(first.await.unwrap(), second);
    assert_eq!(waiter.await.unwrap(), ApprovalOutcome::Approved);
}

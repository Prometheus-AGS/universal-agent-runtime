//! Case33 uses a previously qualified real supervisor exclusion. Never creates
//! registrations, challenges, exclusions or a fabricated supervisor proof.
//! The whole target requires its actual dedicated-host predecessor evidence.
#![cfg(all(feature = "server", feature = "surreal-backend"))]
use serde::Deserialize;
use sha2::{Digest, Sha256};
use universal_agent_runtime::uar::{persistence::{PersistenceLayer,
    providers::surreal::SurrealDbProvider, checkpoint_resume::CheckpointResumeError,
    tool_admission::ToolAdmissionEvidenceState}, runtime::supervisor::SupervisorRegistration};

#[derive(Deserialize)]
struct Case {
    database_url: String, namespace: String, database: String,
    user_env: Option<String>, password_env: Option<String>,
    qualified_observation: String, qualified_observation_sha256: String,
    effect_trace: String, evidence_owner: String, accounting_owner: String, root_run_id: String,
}
fn bytes(path: &str) -> Vec<u8> { std::fs::read(path).expect("actual qualified artifact required") }
fn hash(value: &[u8]) -> String { format!("{:x}", Sha256::digest(value)) }

#[tokio::test]
async fn actual_closed_epoch_rejects_new_admission_without_effect_or_receipt() {
    let path = std::env::var("UAR_DOT_STALE_EPOCH_CASE").expect("qualified supervisor case required");
    let case: Case = serde_json::from_slice(&bytes(&path)).unwrap();
    let artifact = bytes(&case.qualified_observation);
    assert_eq!(hash(&artifact), case.qualified_observation_sha256);
    let observation: serde_json::Value = serde_json::from_slice(&artifact).unwrap();
    let data = &observation["data"];
    // Source is the real supervisor runner's native/CRI completion observation.
    let observed = data.get("observation").or_else(|| data.get("after")).expect("actual supervisor observation");
    assert!(!observed["exclusion"].is_null());
    assert_eq!(data["http"]["exclusion_status"], 200);
    let challenge = data["http"]["challenge_id"].as_str().unwrap();
    let registration: SupervisorRegistration = serde_json::from_value(observed["registration"].clone()).unwrap();
    let user = case.user_env.as_ref().map(|name| std::env::var(name).unwrap());
    let password = case.password_env.as_ref().map(|name| std::env::var(name).unwrap());
    let store = SurrealDbProvider::new(&case.database_url, user.as_deref(), password.as_deref(),
        Some(&case.namespace), Some(&case.database)).await.unwrap();
    assert_eq!(store.load_supervisor_domain(&registration.runtime_epoch).await.unwrap(), Some(registration.clone()));
    assert!(store.load_supervisor_challenge(challenge).await.unwrap().is_none(), "qualified challenge not consumed");
    let before = store.list_tool_admission_evidence(&case.evidence_owner).await.unwrap();
    let original = before.iter().find(|row| row.runtime_epoch == registration.runtime_epoch
        && row.root_run_id == case.root_run_id).expect("actual predecessor tool admission required");
    let receipts = serde_json::to_value(store.list_canonical_tool_receipts(&case.evidence_owner, &case.root_run_id).await.unwrap()).unwrap();
    let accounting = serde_json::to_value(store.list_accounting_calls(&case.accounting_owner, &case.root_run_id).await.unwrap()).unwrap();
    let effects = bytes(&case.effect_trace);
    for state in [ToolAdmissionEvidenceState::AwaitingApproval, ToolAdmissionEvidenceState::ClaimIntent] {
        // Copy provenance from a real admitted predecessor; this is an attempted
        // stale storage admission, never an execution grant or forged receipt.
        let mut stale = original.clone();
        stale.evidence_id = uuid::Uuid::new_v4().to_string();
        stale.invocation_id = uuid::Uuid::new_v4().to_string();
        stale.admission_id = uuid::Uuid::new_v4().to_string();
        stale.state = state;
        stale.occurred_at = chrono::Utc::now();
        let error = store.save_tool_admission_evidence(&stale).await.unwrap_err();
        assert!(matches!(error.downcast_ref::<CheckpointResumeError>(), Some(CheckpointResumeError::ExecutorClosed)),
            "rejection must be the actual durable closed epoch, not an unrelated storage failure: {error}");
    }
    assert_eq!(store.list_tool_admission_evidence(&case.evidence_owner).await.unwrap(), before);
    assert_eq!(serde_json::to_value(store.list_canonical_tool_receipts(&case.evidence_owner, &case.root_run_id).await.unwrap()).unwrap(), receipts);
    assert_eq!(serde_json::to_value(store.list_accounting_calls(&case.accounting_owner, &case.root_run_id).await.unwrap()).unwrap(), accounting);
    assert_eq!(bytes(&case.effect_trace), effects);
}

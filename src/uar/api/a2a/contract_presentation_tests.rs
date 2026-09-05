use super::*;
use crate::uar::a2ui::presentation_selection::{PresentationMode, PresentationNegotiation};
use crate::uar::domain::policy::ResourceSelection;
use serde_json::json;

// Deliberately fixed bytes for the pre-Presentation, twelve-field policy wire
// contract. This is a local compatibility oracle, not a live old-peer receipt.
const LEGACY_WIRE: &str = r#"{"version":2,"source_instance_id":"source","target_instance_id":"target","owner_id":"owner","root_run_id":"run","parent_thread_id":"parent","child_thread_id":"child","target_agent_id":"agent","policy":{"version":1,"chat_mode":"agent","agent_id":"agent","model":{"provider_id":"provider","model_id":"model"},"skills":{"mode":"none","ids":[],"denied_ids":[]},"tools":{"mode":"none","ids":[],"denied_ids":[]},"mcp_servers":{"mode":"none","ids":[],"denied_ids":[]},"knowledge_bases":{"mode":"none","ids":[],"denied_ids":[]},"memory_enabled":false,"prompt_caching_enabled":false,"context_strategy":{"type":"auto"},"tool_approval":"auto"},"budgets":{"max_tokens_per_turn":null,"max_tokens_per_session":null,"max_tool_calls_per_turn":null,"max_cost_per_session_usd":null,"timeout_seconds":null,"rate_limit":null},"usage_grant":{"max_total_tokens":null,"max_total_cost_usd":null,"max_total_model_requests":null,"max_total_tool_calls":null,"expires_after_seconds":null},"sandbox":{"execution_mode":"direct","network_enabled":true,"filesystem":{},"environment":[]}}"#;

fn legacy() -> UarDelegationContract {
    serde_json::from_str(LEGACY_WIRE).unwrap()
}

#[test]
fn omitted_ceiling_preserves_wire_bytes_digest_and_acknowledgement() {
    let contract = legacy();
    contract.validate().unwrap();
    assert_eq!(serde_json::to_string(&contract).unwrap(), LEGACY_WIRE);
    let expected_digest: String = Sha256::digest(LEGACY_WIRE.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(contract.digest().unwrap(), expected_digest);
    let ack = UarDelegationAcknowledgement {
        version: 2,
        target_instance_id: "target".into(),
        child_thread_id: "child".into(),
        contract_digest: expected_digest,
        remote_thread_id: None,
    };
    ack.validate_for(&contract).unwrap();
    let execution = contract.execution_policy();
    assert_eq!(execution.presentations.mode, SelectionMode::None);
    assert!(execution.presentations.ids.is_empty());
    assert_eq!(serde_json::to_string(&contract).unwrap(), LEGACY_WIRE);
    assert_eq!(contract.policy.presentations.mode, SelectionMode::Inherit);
    ack.validate_for(&contract).unwrap();
}

#[test]
fn legacy_outbound_round_trip_does_not_create_new_authority() {
    let mut contract = legacy();
    contract.policy = UarDelegationPolicy::for_peer(contract.execution_policy(), &None);
    assert_eq!(serde_json::to_string(&contract).unwrap(), LEGACY_WIRE);
    assert_eq!(
        serde_json::from_str::<UarDelegationContract>(LEGACY_WIRE).unwrap(),
        contract
    );
    assert_eq!(
        contract.execution_policy().presentations.mode,
        SelectionMode::None
    );
}

#[test]
fn negotiated_restrictions_and_selected_templates_never_use_legacy_wire() {
    for negotiated in [false, true] {
        let mut contract = legacy();
        let mut policy = contract.execution_policy();
        if negotiated {
            contract.presentation_negotiation = Some(PresentationNegotiation {
                presentation_mode: Some(PresentationMode::Text),
                client_rendering: None,
            });
        } else {
            policy.presentations = ResourceSelection::selected(["template".into()]);
        }
        contract.policy = UarDelegationPolicy::for_peer(policy, &contract.presentation_negotiation);
        contract.validate().unwrap();
        let wire = serde_json::to_value(&contract).unwrap();
        assert!(wire["policy"].get("presentations").is_some());
        let decoded: UarDelegationContract = serde_json::from_value(wire).unwrap();
        assert_eq!(decoded, contract);
        assert_ne!(contract.digest().unwrap(), legacy().digest().unwrap());
        let old_ack = UarDelegationAcknowledgement::for_contract(&legacy(), None).unwrap();
        assert!(old_ack.validate_for(&contract).is_err());
    }
}

#[test]
fn absent_ceiling_cannot_carry_negotiation_and_explicit_null_is_not_omission() {
    let mut wire: serde_json::Value = serde_json::from_str(LEGACY_WIRE).unwrap();
    wire["presentation_negotiation"] = json!({"presentation_mode": "text"});
    assert!(
        serde_json::from_value::<UarDelegationContract>(wire.clone())
            .unwrap()
            .validate()
            .is_err()
    );
    wire["presentation_negotiation"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<UarDelegationContract>(wire).is_err());
}

#[test]
fn explicit_ceiling_rejects_ambiguous_or_nonconcrete_resource_modes() {
    for selection in [
        json!({"mode": "inherit"}),
        json!({"mode": "auto"}),
        json!({"mode": "all"}),
        json!({"mode": "selected", "ids": []}),
        json!({"mode": "none", "ids": ["template"]}),
        json!({"mode": "selected", "ids": ["template"], "denied_ids": ["template"]}),
    ] {
        let mut wire: serde_json::Value = serde_json::from_str(LEGACY_WIRE).unwrap();
        wire["policy"]["presentations"] = selection;
        assert!(
            serde_json::from_value::<UarDelegationContract>(wire)
                .unwrap()
                .validate()
                .is_err()
        );
    }
}

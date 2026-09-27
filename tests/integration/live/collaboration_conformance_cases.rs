//! C03 completed-path fixture. Source only until OpenSpec task 7.2 runs the gate.

use reqwest::StatusCode;
use serde_json::{Value, json};
use serial_test::serial;

use super::backend::{BACKEND_ENV_VAR, resolve};
use super::harness::{ServiceNeeds, boot_test_server_process, mint_harness_peer_token};
use super::stub_llm::{FixtureResponse, FixtureSet, RequestFingerprint};

#[path = "../../fixtures/collaboration/harness_support.rs"]
mod support;
use support::*;

macro_rules! get {
    ($api:expr, $label:literal, $path:expr) => {
        $api.get($label, $path).await
    };
}

macro_rules! post {
    ($api:expr, $label:literal, $path:expr, $body:expr, $status:expr) => {
        $api.post($label, $path, $body, $status).await
    };
}

const MODEL: &str = "gpt-5.4-mini";
const PROFILE: &str = "urn:prometheus:uar:collaboration:0.1.0-draft.2";
const SOURCE_REVISION: &str = "fba2b34a6449c501b0ad9de29936eb63f5726843";
const WORKSPACE: &str = "workspace:c03";
const PACKAGE_ID: &str = "urn:uar:c03:package";
const BINDING_ID: &str = "urn:uar:c03:binding";
const GRANT_ID: &str = "urn:uar:c03:grant";
const SKILL_ROOT: &str = "tests/fixtures/collaboration/builtin-skills";
const SKILL_PATH: &str = "tests/fixtures/collaboration/builtin-skills/sample-skill/SKILL.md";
const SKILL_LOCATION: &str =
    "file://tests/fixtures/collaboration/builtin-skills/sample-skill/SKILL.md";
const LEGACY_V1: &str = include_str!("../../fixtures/collaboration/legacy-v1.1.md");
const LEGACY_V2_EXPLICIT: &str =
    include_str!("../../fixtures/collaboration/legacy-v2-explicit-defaults.md");
const LEGACY_V2_OMITTED: &str =
    include_str!("../../fixtures/collaboration/legacy-v2-omitted-defaults.md");
const AGENT: &str = include_str!("../../fixtures/collaboration/agent-definition.json");
const TEAM: &str = include_str!("../../fixtures/collaboration/team-definition.json");
const WORKFLOW: &str = include_str!("../../fixtures/collaboration/workflow-definition.json");
const MANIFEST: &str = include_str!("../../fixtures/collaboration/package-manifest.json");
const PRIVATE_AGENT: &str =
    include_str!("../../fixtures/collaboration/private-authority-agent.json");
const BLOCKED_AGENT: &str =
    include_str!("../../fixtures/collaboration/required-unsupported-agent.json");
const GRANT: &str = include_str!("../../fixtures/collaboration/representation-grant-v1.json");
const BINDING: &str = include_str!("../../fixtures/collaboration/deployment-binding.json");

/// OpenSpec 7.2 invokes this source serially in recorded and live modes after freeze.
#[tokio::test]
#[serial]
async fn c03_completed_path_emits_one_acceptance_receipt() {
    let original_skill_root = std::env::var_os("UAR_BUILTIN_SKILLS_DIR");
    // SAFETY: this case is serial and restores the process-global fixture root below.
    unsafe { std::env::set_var("UAR_BUILTIN_SKILLS_DIR", SKILL_ROOT) };
    let skill_digest = sha256(&std::fs::read(SKILL_PATH).expect("deterministic skill fixture"));
    assert_eq!(
        parse_fixture(AGENT)["skills"][0]["digest"].as_str(),
        Some(skill_digest.as_str())
    );
    let recorded = std::env::var(BACKEND_ENV_VAR).as_deref() != Ok("live");
    let backend = resolve(FixtureSet::new().with(
        RequestFingerprint {
            model: MODEL.to_owned(),
            last_user_message: "exercise the C03 ordinary run".to_owned(),
            has_tools: true,
            has_tool_result: false,
        },
        FixtureResponse::Content("C03_BOUND_RUN_OK".to_owned()),
    ))
    .await;
    let scratch = tempfile::tempdir().expect("C03 SurrealKV scratch");
    let persistence_path = scratch.path().join("surrealkv");
    let first = boot_test_server_process(
        &backend.base_url,
        &backend.model,
        ServiceNeeds::default(),
        &persistence_path,
    )
    .await;
    let token = mint_harness_peer_token();
    let client = reqwest::Client::new();
    let api = Api {
        client: &client,
        base_url: &first.base_url,
        token: &token,
    };

    let legacy = compile_legacy(&api, LEGACY_V1).await;
    let explicit = compile_legacy(&api, LEGACY_V2_EXPLICIT).await;
    let omitted = compile_legacy(&api, LEGACY_V2_OMITTED).await;
    let explicit_fields = explicit["descriptor"]["payload"]["source"]["authoredFields"]
        .as_array()
        .expect("explicit authored fields");
    let omitted_fields = omitted["descriptor"]["payload"]["source"]["authoredFields"]
        .as_array()
        .expect("omitted authored fields");
    assert!(
        explicit_fields
            .iter()
            .any(|field| field == "/model_requirements")
    );
    assert!(
        !omitted_fields
            .iter()
            .any(|field| field == "/model_requirements")
    );
    assert_eq!(
        legacy["descriptor"]["payload"]["source"]["profile"],
        "urn:prometheus:uar:agent-md:1.1"
    );

    let capabilities = get!(
        api,
        "collaboration capabilities",
        "/api/v1/collaboration/capabilities"
    );
    let owner = capabilities["bindingOwnerId"]
        .as_str()
        .expect("opaque binding owner");
    assert_eq!(parse_fixture(BINDING)["ownerId"].as_str(), Some(owner));

    let installed = post!(
        api,
        "canonical package install",
        "/api/v1/collaboration/packages:install",
        main_package("c03-install-main"),
        StatusCode::CREATED
    );
    assert_eq!(installed["preflight"]["activationSupported"], false);
    assert_eq!(installed["receipt"]["catalogRevision"], 1);

    let mut changed = parse_fixture(AGENT);
    changed["instructions"] = Value::String("Conflicting immutable bytes.".to_owned());
    finalize(&mut changed);
    post!(
        api,
        "immutable package conflict",
        "/api/v1/collaboration/packages:install",
        single_agent_package("c03-conflict", PACKAGE_ID, &changed),
        StatusCode::CONFLICT
    );
    let private_refusal = post!(
        api,
        "portable private-authority exclusion",
        "/api/v1/collaboration/packages:preflight",
        single_agent_package(
            "c03-private",
            "urn:uar:c03:private-package",
            &parse_fixture(PRIVATE_AGENT),
        ),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let private_diagnostic = private_refusal.to_string();
    for secret in [
        "protected-credential://must-not-export",
        "example_secret_value_source_descriptor",
        "example_secret_value_legacy_section",
        "example_secret_value_skill_config",
        "example_secret_value_contract",
        "example_secret_value_free_text",
    ] {
        assert!(
            !private_diagnostic.contains(secret),
            "diagnostic leaked {secret}"
        );
    }

    let blocked_package = post!(
        api,
        "required semantics package",
        "/api/v1/collaboration/packages:install",
        single_agent_package(
            "c03-blocked-package",
            "urn:uar:c03:blocked-package",
            &parse_fixture(BLOCKED_AGENT),
        ),
        StatusCode::CREATED
    );
    assert_eq!(blocked_package["preflight"]["activationSupported"], false);

    let grant_v1 = parse_fixture(GRANT);
    post!(
        api,
        "grant revision one",
        "/api/v1/collaboration/representation-grants",
        json!({"commandId":"c03-grant-1","expectedRevision":0,"grant":grant_v1}),
        StatusCode::CREATED
    );
    let binding = parse_fixture(BINDING);
    let installed_binding = post!(
        api,
        "effective binding install",
        "/api/v1/collaboration/deployment-bindings",
        json!({"commandId":"c03-binding-1","expectedRevision":0,"binding":binding}),
        StatusCode::CREATED
    );
    assert_eq!(installed_binding["preflight"]["activationSupported"], true);

    let mut blocked_binding = parse_fixture(BINDING);
    blocked_binding["id"] = Value::String("urn:uar:c03:blocked-binding".to_owned());
    blocked_binding["package"] = blocked_package["preflight"]["package"].clone();
    let blocked_skill = parse_fixture(BLOCKED_AGENT)["skills"][0].clone();
    blocked_binding["skillBindings"][0] = json!({
        "id": blocked_skill["id"], "version": blocked_skill["version"],
        "digest": blocked_skill["digest"], "required": blocked_skill["required"],
        "config": blocked_skill["config"], "entrypoint": blocked_skill["entrypoint"],
        "requiredTools": blocked_skill["requiredTools"],
        "installedLocation": "file://tests/fixtures/collaboration/missing-mirror/SKILL.md"
    });
    blocked_binding["representationGrantRefs"] = json!([]);
    finalize(&mut blocked_binding);
    let blocked = post!(
        api,
        "required skill and v2 refusal",
        "/api/v1/collaboration/deployment-bindings:preflight",
        json!({"commandId":"c03-blocked-binding","expectedRevision":0,"binding":blocked_binding}),
        StatusCode::OK
    );
    assert_eq!(blocked["activationSupported"], false);
    assert!(blocked["diagnostics"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item["disposition"] == "required-unsupported")
    }));

    let original_barrier = first.shutdown_to_pre_exit_barrier("TERM").await;
    let restarted = boot_test_server_process(
        &backend.base_url,
        &backend.model,
        ServiceNeeds::default(),
        &persistence_path,
    )
    .await;
    original_barrier.allow_exit().await;
    let api = Api {
        client: &client,
        base_url: &restarted.base_url,
        token: &token,
    };

    let package = get!(
        api,
        "cold package readback",
        &format!("/api/v1/collaboration/packages/{PACKAGE_ID}/versions/1.0.0")
    );
    let receipt = get!(
        api,
        "cold effective receipt readback",
        &format!("/api/v1/collaboration/deployment-bindings/{BINDING_ID}/effective-receipt")
    );
    let grant_current = get!(
        api,
        "cold current grant readback",
        &format!("/api/v1/collaboration/representation-grants/{GRANT_ID}")
    );
    let grant_history = get!(
        api,
        "cold grant history readback",
        &format!("/api/v1/collaboration/representation-grants/{GRANT_ID}/history")
    );
    assert_eq!(
        receipt["resolvedSkills"][0]["digest"].as_str(),
        Some(skill_digest.as_str())
    );
    assert_eq!(
        receipt["resolvedSkills"][0]["requiredTools"],
        json!(["native_echo"])
    );
    assert_eq!(
        receipt["resolvedSkills"][0]["installedLocation"].as_str(),
        Some(SKILL_LOCATION)
    );
    assert_eq!(grant_current["revision"], 1);
    assert_eq!(grant_history.as_array().map(Vec::len), Some(1));

    let started = post!(
        api,
        "ordinary REST run by deployment binding",
        "/api/uar/runs",
        json!({"deployment_binding_id":BINDING_ID,"input":"exercise the C03 ordinary run"}),
        StatusCode::OK
    );
    let run_id = started["run_id"].as_str().expect("bound run id");
    let run = wait_for_run(&api, run_id).await;
    assert_eq!(run["status"], "done", "bound run failed: {run}");
    let replay = replay_run(&api, run_id).await;
    let observed_response = response_text(&replay);
    assert!(
        !observed_response.trim().is_empty(),
        "bound run emitted no assistant content: {replay}"
    );
    if recorded {
        assert_eq!(observed_response, "C03_BOUND_RUN_OK");
    }

    let exported = post!(
        api,
        "canonical package export",
        "/api/v1/collaboration/packages:export",
        json!({"package":package["identity"],"target":{"kind":"canonicalDraft2"}}),
        StatusCode::OK
    );
    assert_eq!(exported["status"], "exported");
    assert!(
        exported["export"]["files"]
            .to_string()
            .contains("byte-exact")
    );
    post!(
        api,
        "canonical package reimport",
        "/api/v1/collaboration/packages:install",
        json!({"commandId":"c03-reimport","manifest":exported["export"]["manifest"],"files":exported["export"]["files"]}),
        StatusCode::CREATED
    );
    let template = post!(
        api,
        "sanitized binding template",
        &format!("/api/v1/collaboration/deployment-bindings/{BINDING_ID}/template:export"),
        json!({}),
        StatusCode::OK
    );
    let template_text = template.to_string();
    assert_eq!(template["template"]["status"], "needs-private-binding");
    for private in [
        "ownerId",
        "workspaceId",
        "credentialRef",
        "connectionRef",
        "representationGrantRefs",
    ] {
        assert!(
            !template_text.contains(private),
            "template retained {private}"
        );
    }
    let downgrade = post!(
        api,
        "compatibility downgrade refusal",
        "/api/v1/collaboration/packages:export",
        json!({"package":package["identity"],"target":{"kind":"compatibility","profile":"urn:prometheus:uar:collaboration:0.1.0-draft.1","harness":null,"supportedSemantics":[]}}),
        StatusCode::OK
    );
    assert_eq!(downgrade["status"], "refused");

    let mut stale_policy = parse_fixture(BINDING);
    stale_policy["revision"] = json!(2);
    stale_policy["policyRevision"] = Value::String("policy:c03:2".to_owned());
    finalize(&mut stale_policy);
    post!(
        api,
        "stale policy revision refusal",
        "/api/v1/collaboration/deployment-bindings",
        json!({"commandId":"c03-stale-policy","expectedRevision":0,"binding":stale_policy}),
        StatusCode::CONFLICT
    );
    let mut revoked = parse_fixture(GRANT);
    revoked["revision"] = json!(2);
    revoked["status"] = Value::String("revoked".to_owned());
    revoked["revocation"] =
        json!({"revision":2,"revokedAt":"2026-09-27T00:00:00Z","reason":"C03 stale grant fixture"});
    post!(
        api,
        "grant revocation revision",
        "/api/v1/collaboration/representation-grants",
        json!({"commandId":"c03-grant-2","expectedRevision":1,"grant":revoked}),
        StatusCode::CREATED
    );
    post!(
        api,
        "stale grant blocks bound run",
        "/api/uar/runs",
        json!({"deployment_binding_id":BINDING_ID,"input":"must not dispatch"}),
        StatusCode::CONFLICT
    );
    let revoked_current = get!(
        api,
        "revoked grant is current",
        &format!("/api/v1/collaboration/representation-grants/{GRANT_ID}")
    );
    let complete_grant_history = get!(
        api,
        "complete grant history",
        &format!("/api/v1/collaboration/representation-grants/{GRANT_ID}/history")
    );
    assert_eq!(revoked_current["revision"], 2);
    assert_eq!(complete_grant_history.as_array().map(Vec::len), Some(2));

    restarted.shutdown().await;
    // SAFETY: paired restoration for the serial fixture root override above.
    unsafe {
        match original_skill_root {
            Some(root) => std::env::set_var("UAR_BUILTIN_SKILLS_DIR", root),
            None => std::env::remove_var("UAR_BUILTIN_SKILLS_DIR"),
        }
    }
    let main_manifest = parse_fixture(MANIFEST);
    let acceptance = json!({
        "sourceRevision": SOURCE_REVISION,
        "profile": PROFILE,
        "schemaValidity": {"legacyIngress":3,"documentKinds":["AgentDefinition","TeamDefinition","WorkflowDefinition"],"requiredUnsupportedObserved":true},
        "catalogPersistence": {"coldRestart":true,"catalogRevision":1,"package":package["identity"],"definitionDigests":main_manifest["files"].as_array().expect("manifest files").iter().map(|file| file["definition"]["digest"].clone()).collect::<Vec<_>>()},
        "effectiveRuntimeSemantics": {"bindingRevision":receipt["revision"],"bindingDigest":receipt["bindingRef"]["digest"],"policyRevision":receipt["policyRevision"],"resolvedSkills":receipt["resolvedSkills"],"immutableConflict":true,"downgradeRefused":true},
        "privateAuthorityExclusion": {
            "portablePrivateAuthorityRejected":true,
            "grantId":GRANT_ID,
            "constraintDigest":grant_v1["constraintDigest"],
            "observedGrantRevisions":complete_grant_history.as_array().expect("grant history").iter().map(|grant| grant["revision"].clone()).collect::<Vec<_>>(),
            "currentGrantRevision":revoked_current["revision"],
            "sanitizedTemplate":true,
            "staleGrantRefused":true,
            "stalePolicyRefused":true
        },
        "ordinaryExecutionEvidence": {
            "selector":"deployment_binding_id",
            "agentId":run["agent_id"],
            "status":run["status"],
            "backendMode":if recorded { "recorded" } else { "live" },
            "provider":receipt["resolvedModels"][0]["providerId"],
            "model":receipt["resolvedModels"][0]["modelId"],
            "nonemptyResponse":true,
            "recordedProtocolBoundary":if recorded { Value::String("C03_BOUND_RUN_OK".to_owned()) } else { Value::Null },
            "inferenceCertified":!recorded
        }
    });
    println!(
        "C03_ACCEPTANCE_RECEIPT={}",
        serde_json::to_string(&acceptance).expect("acceptance receipt JSON")
    );
}

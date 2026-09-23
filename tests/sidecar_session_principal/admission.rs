//! Where `X-UAR-Principal` is accepted and rejected (tasks 1.1, 1.2, 1.20).

use reqwest::Method;
use serde_json::Value;

use crate::principal_host::{P1, P2, PRINCIPAL_HEADER, boot, reply};
use crate::sidecar_process::{
    ConfigOptions, JWT_SECRET, LaunchOptions, Workspace, contains_bytes, launch_standalone,
    raw_request, render_config, test_agent,
};
use crate::stub_llm::{FixtureSet, start_stub_llm};

const CAPABILITIES: &str = "/api/uar/capabilities";

fn mint_jwt() -> String {
    let _ = jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER.install_default();
    let claims = universal_agent_runtime::uar::security::claims::UserClaims {
        sub: "standalone-user".to_owned(),
        name: None,
        roles: Some(vec!["user".to_owned()]),
        tenant_id: None,
        uar_instance_id: None,
        exp: usize::MAX,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .expect("mint JWT")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn principal_header_is_accepted_only_after_the_launch_token() {
    let input = "principal admission probe";
    let host = boot(reply(FixtureSet::new(), input, "ok"), "", |options| options).await;

    // Accepted on a launch-token request: the run belongs to the principal.
    let run = host.drive(Some(P1), test_agent(None), input, None).await;
    assert!(
        host.run_exists(Some(P1), &run.run_id).await,
        "the asserting principal cannot address its own run"
    );
    assert!(
        !host.run_exists(None, &run.run_id).await,
        "the anonymous scope can address a principal's run"
    );
    assert!(
        !host.run_exists(Some(P2), &run.run_id).await,
        "another principal can address the run"
    );

    // Rejected values, on a route whose handler has no side effects.
    let port = host.port();
    let authority = format!("127.0.0.1:{port}");
    let authorization = format!("Bearer {}", host.token);
    let too_long = "a".repeat(129);
    for value in ["anonymous", "", too_long.as_str(), "boss session"] {
        let response = raw_request(
            port,
            "GET",
            CAPABILITIES,
            &[
                ("Host", &authority),
                ("Authorization", &authorization),
                (PRINCIPAL_HEADER, value),
            ],
        )
        .await;
        assert_eq!(
            response.status,
            400,
            "value {value:?}: {}",
            response.body_text()
        );
        assert!(
            response.body_text().contains("principal_invalid"),
            "value {value:?}: {}",
            response.body_text()
        );
        if !value.is_empty() {
            assert!(
                !contains_bytes(&response.all_bytes(), value.as_bytes()),
                "the response echoes the rejected value {value:?}"
            );
        }
    }
    let longest = "a".repeat(128);
    let accepted = raw_request(
        port,
        "GET",
        CAPABILITIES,
        &[
            ("Host", &authority),
            ("Authorization", &authorization),
            (PRINCIPAL_HEADER, &longest),
        ],
    )
    .await;
    assert_eq!(accepted.status, 200, "{}", accepted.body_text());

    // Standalone UAR never accepts the header, with or without a JWT.
    let stub = start_stub_llm(FixtureSet::new()).await;
    let workspace = Workspace::new();
    let options = ConfigOptions::new(&stub.base_url);
    let standalone_port = options.port;
    let config = workspace.write_config("config.yaml", &render_config(&workspace, &options));
    let mut standalone = launch_standalone(
        &workspace,
        &LaunchOptions::new(config, "standalone"),
        standalone_port,
    )
    .await;
    let standalone_authority = format!("127.0.0.1:{standalone_port}");
    let jwt = format!("Bearer {}", mint_jwt());
    for headers in [
        vec![("Host", standalone_authority.as_str()), (PRINCIPAL_HEADER, P1)],
        vec![
            ("Host", standalone_authority.as_str()),
            ("Authorization", jwt.as_str()),
            (PRINCIPAL_HEADER, P1),
        ],
    ] {
        let response = raw_request(standalone_port, "GET", CAPABILITIES, &headers).await;
        assert_eq!(response.status, 400, "{}", response.body_text());
        assert!(
            response
                .body_text()
                .contains("principal_header_not_allowed"),
            "{}",
            response.body_text()
        );
        assert!(
            !contains_bytes(&response.all_bytes(), P1.as_bytes()),
            "the response echoes the header value"
        );
    }
    let control = raw_request(
        standalone_port,
        "GET",
        CAPABILITIES,
        &[("Host", standalone_authority.as_str())],
    )
    .await;
    assert_eq!(control.status, 200, "{}", control.body_text());
    let status = standalone.stop_standalone().await;
    assert!(status.success(), "standalone exit: {status}");
}

/// Regression guard: a host that sends no principal keeps the anonymous owner.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_without_header_stays_anonymous() {
    let input = "anonymous host probe";
    let host = boot(reply(FixtureSet::new(), input, "ok"), "", |options| options).await;
    let run = host.drive(None, test_agent(None), input, None).await;
    assert!(
        host.run_exists(None, &run.run_id).await,
        "anonymous run not addressable by the anonymous scope"
    );
    assert!(
        !host.run_exists(Some(P1), &run.run_id).await,
        "a principal can address an anonymous run"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn capabilities_list_session_principal() {
    let host = boot(FixtureSet::new(), "", |options| options).await;
    let (status, body) = host.call(Method::GET, CAPABILITIES, None, None).await;
    assert_eq!(status, 200, "{body}");
    let body: Value = serde_json::from_str(&body).expect("capabilities JSON");
    let names: Vec<&str> = body["capabilities"]
        .as_array()
        .unwrap_or_else(|| panic!("capabilities array: {body}"))
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(names.contains(&"session_principal"), "{names:?}");
}

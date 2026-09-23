//! Per-request token, Host and Origin enforcement (tasks 2.6–2.12).

use crate::sidecar_process::{
    LaunchOptions, RawResponse, Workspace, launch_sidecar, random_token, raw_request,
    route_inventory,
};
use crate::stub_llm::{FixtureSet, start_stub_llm};
use crate::{Sidecar, bearer, boot_sidecar, host, write_config};

const SSE_ROUTES: [&str; 2] = ["/api/uar/runs/dummy-run/stream", "/api/uar/sync/stream"];

fn wrong_authorizations(token: &str) -> Vec<String> {
    vec![
        bearer(&random_token()),
        format!("Basic {token}"),
        format!("bearer {token}"),
        String::new(),
    ]
}

async fn request_every_route(
    port: u16,
    headers: &[(&str, &str)],
) -> Vec<(String, String, RawResponse)> {
    let mut responses = Vec::new();
    for (method, path) in route_inventory() {
        let response = raw_request(port, &method, &path, headers).await;
        responses.push((method, path, response));
    }
    responses
}

fn unexpected_statuses(responses: &[(String, String, RawResponse)], expected: u16) -> Vec<String> {
    responses
        .iter()
        .filter(|(_, _, response)| response.status != expected)
        .map(|(method, path, response)| format!("{method} {path} -> {}", response.status))
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_route_rejects_missing_token() {
    let sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
    let responses = request_every_route(sidecar.port(), &[("Host", &sidecar.host())]).await;
    let failures = unexpected_statuses(&responses, 401);
    assert!(failures.is_empty(), "not 401: {failures:#?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_route_rejects_wrong_token() {
    let sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
    let mut failures = Vec::new();
    for authorization in wrong_authorizations(&sidecar.token) {
        let responses = request_every_route(
            sidecar.port(),
            &[("Host", &sidecar.host()), ("Authorization", &authorization)],
        )
        .await;
        failures.extend(
            unexpected_statuses(&responses, 401)
                .into_iter()
                .map(|failure| format!("[{}] {failure}", redact(&authorization))),
        );
    }
    assert!(failures.is_empty(), "not 401: {failures:#?}");
}

fn redact(authorization: &str) -> &str {
    authorization.split_whitespace().next().unwrap_or("<empty>")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn previous_launch_token_is_rejected() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let workspace = Workspace::new();
    let config = write_config(&workspace, &stub.base_url, "");
    let first_token = random_token();
    let mut first = launch_sidecar(
        &workspace,
        &LaunchOptions::new(config.clone(), "first").with_token(&first_token),
    )
    .await;
    let status = first.stop_sidecar().await;
    assert!(status.success(), "first launch exit: {status}");

    let second_token = random_token();
    let second = launch_sidecar(
        &workspace,
        &LaunchOptions::new(config, "second").with_token(&second_token),
    )
    .await;
    let port = second.port();
    let responses = request_every_route(
        port,
        &[
            ("Host", &host(port)),
            ("Authorization", &bearer(&first_token)),
        ],
    )
    .await;
    let failures = unexpected_statuses(&responses, 401);
    assert!(
        failures.is_empty(),
        "previous token not rejected: {failures:#?}"
    );

    let current = raw_request(
        port,
        "GET",
        "/readyz",
        &[
            ("Host", &host(port)),
            ("Authorization", &bearer(&second_token)),
        ],
    )
    .await;
    assert_ne!(current.status, 401, "current token rejected");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sse_streams_reject_without_token_and_write_no_events() {
    let sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
    let wrong = bearer(&random_token());
    let host = sidecar.host();
    for path in SSE_ROUTES {
        for headers in [
            vec![("Host", host.as_str())],
            vec![("Host", host.as_str()), ("Authorization", wrong.as_str())],
        ] {
            let response = raw_request(sidecar.port(), "GET", path, &headers).await;
            assert_eq!(response.status, 401, "{path}: {}", response.body_text());
            let body = response.body_text();
            assert!(
                !body
                    .lines()
                    .any(|line| line.starts_with("data:") || line.starts_with("event:")),
                "{path} wrote event bytes: {body}"
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn foreign_host_rejected_with_valid_token() {
    let sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
    let port = sidecar.port();
    let authorization = sidecar.bearer();
    let hosts = [
        "attacker.example".to_owned(),
        format!("attacker.example:{port}"),
        "127.0.0.1".to_owned(),
        "localhost".to_owned(),
        "localhost:1".to_owned(),
        format!("[::1]:{port}"),
        format!("127.0.0.2:{port}"),
    ];
    for foreign in &hosts {
        for path in ["/readyz", "/api/openapi.json", SSE_ROUTES[1]] {
            let response = raw_request(
                port,
                "GET",
                path,
                &[("Host", foreign), ("Authorization", &authorization)],
            )
            .await;
            assert_eq!(response.status, 403, "Host {foreign} {path}");
            assert!(
                !response.body_text().contains(foreign.as_str()),
                "Host {foreign} echoed in body"
            );
        }
    }
    let uppercase = raw_request(
        port,
        "GET",
        "/readyz",
        &[
            ("Host", &format!("LOCALHOST:{port}")),
            ("Authorization", &authorization),
        ],
    )
    .await;
    assert_eq!(uppercase.status, 200, "LOCALHOST:{port} must be accepted");
}

fn origin_values(port: u16) -> Vec<String> {
    vec![
        "null".to_owned(),
        format!("http://127.0.0.1:{port}"),
        format!("http://localhost:{port}"),
        "https://example.com".to_owned(),
    ]
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn any_origin_rejected_with_valid_token() {
    let sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
    let port = sidecar.port();
    let host = sidecar.host();
    let authorization = sidecar.bearer();
    for origin in origin_values(port) {
        let response = raw_request(
            port,
            "GET",
            "/readyz",
            &[
                ("Host", &host),
                ("Authorization", &authorization),
                ("Origin", &origin),
            ],
        )
        .await;
        assert_eq!(response.status, 403, "Origin {origin}");
    }
    let preflight = preflight(&sidecar).await;
    assert_eq!(preflight.status, 403, "preflight");
}

async fn preflight(sidecar: &Sidecar) -> RawResponse {
    raw_request(
        sidecar.port(),
        "OPTIONS",
        "/api/uar/runs",
        &[
            ("Host", &sidecar.host()),
            ("Origin", "https://example.com"),
            ("Access-Control-Request-Method", "POST"),
        ],
    )
    .await
}

fn cors_headers(response: &RawResponse) -> Vec<String> {
    response
        .headers
        .iter()
        .filter(|(name, _)| name.to_ascii_lowercase().starts_with("access-control-"))
        .map(|(name, _)| name.clone())
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn no_cors_headers_on_any_response() {
    let sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
    let port = sidecar.port();
    let host = sidecar.host();
    let authorization = sidecar.bearer();
    let mut responses =
        request_every_route(port, &[("Host", &host), ("Authorization", &authorization)]).await;
    responses.extend(request_every_route(port, &[("Host", &host)]).await);
    for wrong in wrong_authorizations(&sidecar.token) {
        responses
            .extend(request_every_route(port, &[("Host", &host), ("Authorization", &wrong)]).await);
    }
    for foreign in ["attacker.example".to_owned(), format!("[::1]:{port}")] {
        let response = raw_request(
            port,
            "GET",
            "/readyz",
            &[("Host", &foreign), ("Authorization", &authorization)],
        )
        .await;
        responses.push(("GET".into(), format!("/readyz Host={foreign}"), response));
    }
    for origin in origin_values(port) {
        let response = raw_request(
            port,
            "GET",
            "/readyz",
            &[
                ("Host", &host),
                ("Authorization", &authorization),
                ("Origin", &origin),
            ],
        )
        .await;
        responses.push(("GET".into(), format!("/readyz Origin={origin}"), response));
    }
    responses.push((
        "OPTIONS".into(),
        "/api/uar/runs".into(),
        preflight(&sidecar).await,
    ));

    let offenders: Vec<String> = responses
        .iter()
        .filter_map(|(method, path, response)| {
            let headers = cors_headers(response);
            (!headers.is_empty()).then(|| format!("{method} {path}: {headers:?}"))
        })
        .collect();
    assert!(offenders.is_empty(), "CORS headers present: {offenders:#?}");
}

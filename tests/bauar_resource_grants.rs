//! Router/transport resource contract; synthetic credentials do not certify an external issuer.
#[path = "support/bauar_resource_peer.rs"]
mod peer;
#[path = "support/sidecar_process.rs"]
mod sidecar_process;
#[path = "integration/live/stub_llm.rs"]
mod stub_llm;

use peer::{Host, OWNER_A, OWNER_B, Peer, SCOPE, grant, now};
use reqwest::Method;
use serde_json::{Value, json};
use std::{collections::BTreeSet, time::Duration};

fn run_id(response: (u16, Value)) -> String {
    assert_eq!(response.0, 200, "run admission: {}", response.1);
    response.1["run_id"].as_str().expect("run id").to_owned()
}

async fn finish(host: &Host, owner: &str, run: &str) {
    let stream = host.open(owner, run).await.finish(host, owner, run).await;
    assert!(
        stream.contains("event: agui.done"),
        "run did not complete: {stream}"
    );
}

async fn resume(host: &Host, owner: &str, source: &str, resource: Value) -> (u16, Value) {
    host.call(
        Method::POST,
        &format!("/api/uar/runs/{source}/resume"),
        owner,
        json!({"input":"renewed","mcp_servers":[resource]}),
    )
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn grants_pin_owner_run_destination_revision_lease_and_action() {
    let peer = Peer::start().await;
    // Independent receiver control: a reachable receiver rejects a bad credential.
    let client = reqwest::Client::new();
    let effect = json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
        "params":{"name":"effect","arguments":{"argument":"receiver-control"}}});
    let rejected = client
        .post(&peer.url)
        .bearer_auth("unknown")
        .json(&effect)
        .send()
        .await
        .unwrap();
    assert_eq!(rejected.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert!(peer.effects().is_empty());
    let accepted = client
        .post(&peer.url)
        .bearer_auth("resource-a1")
        .json(&effect)
        .send()
        .await
        .unwrap();
    assert!(accepted.status().is_success());
    assert_eq!(peer.effects().len(), 1);

    let host = Host::start(&peer).await;
    let mut agent = sidecar_process::test_agent(Some(json!({"tool_approval":"ask"})));
    agent["id"] = json!("resource-agent");
    let (status, body) = host
        .call(Method::POST, "/api/agents", OWNER_A, agent.clone())
        .await;
    assert_eq!(status, 201, "register executable positive control: {body}");
    agent["id"] = json!("resource-denied");
    agent["extensions"]["uar.run_policy"]["tools"] = json!({"mode":"none"});
    let (status, body) = host.call(Method::POST, "/api/agents", OWNER_A, agent).await;
    assert_eq!(status, 201, "register denied-tool artifact: {body}");

    let expiry = now() + 240;
    let (a1, a2, b1) = tokio::join!(
        host.create(
            OWNER_A,
            "owner-a-first",
            grant("a1", "a-v1", expiry),
            "resource-agent"
        ),
        host.create(
            OWNER_A,
            "owner-a-second",
            grant("a2", "a-v2", expiry),
            "resource-agent"
        ),
        host.create(
            OWNER_B,
            "owner-b-first",
            grant("b1", "b-v1", expiry),
            "resource-agent"
        ),
    );
    let (a1, a2, b1) = (run_id(a1), run_id(a2), run_id(b1));
    assert_ne!(a1, a2);
    assert_ne!(a1, b1);
    tokio::join!(
        finish(&host, OWNER_A, &a1),
        finish(&host, OWNER_A, &a2),
        finish(&host, OWNER_B, &b1)
    );
    let effects = peer.effects();
    assert_eq!(effects.len(), 4, "one receiver effect per admitted run");
    let mut sessions = BTreeSet::new();
    for (input, label) in [
        ("owner-a-first", "a1"),
        ("owner-a-second", "a2"),
        ("owner-b-first", "b1"),
    ] {
        let matches: Vec<_> = effects
            .iter()
            .filter(|effect| effect.argument == input)
            .collect();
        assert_eq!(matches.len(), 1, "run call replayed or absent: {input}");
        assert_eq!(matches[0].label, label);
        assert!(!matches[0].session.is_empty());
        assert!(
            sessions.insert(matches[0].session.clone()),
            "run reused another run's transport"
        );
    }

    // Admission checks use registered destination requirements, independent of tool names.
    let (status, body) = host
        .create(
            OWNER_A,
            "expired",
            grant("a1", "expired", now() - 1),
            "resource-agent",
        )
        .await;
    assert_eq!(status, 401, "{body}");
    assert!(
        body.to_string()
            .contains("run_mcp_grant_authentication_required")
    );
    let mut missing_scope = grant("a1", "scope", expiry);
    missing_scope["grant"]["scopes"] = json!([]);
    let (status, body) = host
        .create(OWNER_A, "scope", missing_scope, "resource-agent")
        .await;
    assert_eq!(status, 422, "{body}");
    assert!(body.to_string().contains("run_mcp_grant_forbidden"));
    let mut unknown = grant("a1", "unknown", expiry);
    unknown["name"] = json!("unregistered");
    let (status, _) = host
        .create(OWNER_A, "unknown", unknown, "resource-agent")
        .await;
    assert_eq!(status, 422);
    assert_eq!(peer.effects().len(), 4);

    // Same-owner renewal is a new run; it cannot widen scope or swap destination.
    assert_eq!(
        resume(
            &host,
            OWNER_B,
            &a1,
            grant("renewed", "renewed", expiry + 60)
        )
        .await
        .0,
        404
    );
    let (status, body) = resume(&host, OWNER_A, &a1, grant("a1", "a-v1", expiry + 60)).await;
    assert_eq!(status, 401, "unchanged revision extended lease: {body}");
    let mut wider = grant("renewed", "renewed", expiry + 60);
    wider["grant"]["scopes"] = json!([SCOPE, "fixture:additional"]);
    assert_eq!(resume(&host, OWNER_A, &a1, wider).await.0, 401);
    let mut alternate = grant("renewed", "renewed", expiry + 60);
    alternate["name"] = json!("alternate");
    assert_eq!(resume(&host, OWNER_A, &a1, alternate).await.0, 422);
    assert_eq!(peer.effects().len(), 4);
    let renewed = run_id(
        resume(
            &host,
            OWNER_A,
            &a1,
            grant("renewed", "renewed", expiry + 60),
        )
        .await,
    );
    assert_ne!(renewed, a1);
    finish(&host, OWNER_A, &renewed).await;
    let effects = peer.effects();
    assert_eq!(effects.len(), 5);
    let renewed_effect = effects
        .iter()
        .find(|effect| effect.argument == "renewed")
        .unwrap();
    assert_eq!(renewed_effect.label, "renewed");
    assert!(!sessions.contains(&renewed_effect.session));

    // Hold an exact governed call until the already-admitted immutable lease expires.
    let short_expiry = now() + 8;
    let expires = run_id(
        host.create(
            OWNER_A,
            "expired-at-effect",
            grant("a1", "short", short_expiry),
            "resource-agent",
        )
        .await,
    );
    let mut stream = host.open(OWNER_A, &expires).await;
    let approval = stream.pending(&expires).await;
    while now() < short_expiry {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    host.approve(OWNER_A, &expires, &approval).await;
    stream.answered.insert(approval);
    let text = stream.finish(&host, OWNER_A, &expires).await;
    assert!(
        text.contains("authentication") || text.contains("expired"),
        "lease failure absent: {text}"
    );
    assert_eq!(
        peer.effects().len(),
        5,
        "expired credential reached receiver effect"
    );

    let revoked = run_id(
        host.create(
            OWNER_A,
            "revoked-at-effect",
            grant("a1", "revoked", now() + 120),
            "resource-agent",
        )
        .await,
    );
    let mut stream = host.open(OWNER_A, &revoked).await;
    let approval = stream.pending(&revoked).await;
    let path = format!("/api/uar/runs/{revoked}/mcp-grants/resource/revoke");
    assert_eq!(
        host.call(Method::POST, &path, OWNER_B, json!({})).await.0,
        404
    );
    let (status, body) = host.call(Method::POST, &path, OWNER_A, json!({})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["revoked"], true);
    // Do not release a cancelled approval. Observe cancellation through the existing stream.
    stream.answered.insert(approval);
    let text = stream.finish(&host, OWNER_A, &revoked).await;
    assert!(
        text.contains("agui.cancelled") || text.contains("agui.error"),
        "{text}"
    );
    assert_eq!(peer.effects().len(), 5, "revoked call produced an effect");

    // A valid destination credential grants no action removed by effective tool policy.
    let denied = run_id(
        host.create(
            OWNER_A,
            "denied-tool",
            grant("a1", "denied", now() + 120),
            "resource-denied",
        )
        .await,
    );
    let text = host
        .open(OWNER_A, &denied)
        .await
        .finish(&host, OWNER_A, &denied)
        .await;
    assert!(
        text.contains("agui.done") || text.contains("agui.error"),
        "{text}"
    );
    assert_eq!(
        peer.effects().len(),
        5,
        "policy-excluded tool reached receiver"
    );
    // Configuration is explicit and workspace-local; the distributed preset stays empty.
    let preset: Value = serde_json::from_str(include_str!("../mcp.json")).unwrap();
    assert_eq!(preset, json!({"mcpServers":{}}));
    assert!(host.workspace.work().join("mcp.json").exists());
    assert!(host.model.base_url.starts_with("http://127.0.0.1:"));
}

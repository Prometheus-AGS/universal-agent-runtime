//! Dot-03 acceptance at actual HTTP/gRPC, process and durable database boundaries.
//! Run serialized; shared and Postgres tests require explicitly isolated stores.
#![cfg(all(
    feature = "server",
    feature = "surreal-backend",
    feature = "a2a-transport"
))]

#[path = "support/api_key_process.rs"]
mod process;

use process::{
    Database, ISSUER, Node, OutageProxy, grpc, http, issue_sidecar_grant, issuer_grpc_after_outage,
    signed, token,
};
use reqwest::Method;
use serde_json::json;
use universal_agent_runtime::uar::security::api_keys::{ApiKeyStorage, SurrealApiKeyStorage};

const KEYS: &str = "/api/uar/auth/keys";
const EXCHANGE: &str = "/api/uar/auth/exchange";

async fn create(node: &Node, owner: &str, lifetime: Option<i64>) -> (String, String) {
    let response = http(
        node,
        owner,
        false,
        Method::POST,
        KEYS,
        Some(json!({"name":"durable fixture","roles":["user"],"expires_in_secs":lifetime})),
        201,
    )
    .await;
    let id = response["metadata"]["id"]
        .as_str()
        .expect("created key ID")
        .to_owned();
    let key = response["raw_key"]
        .as_str()
        .expect("one-time key response")
        .to_owned();
    node.redact(&key);
    (id, key)
}

async fn exchange(node: &Node, key: &str) -> String {
    http(node, key, true, Method::POST, EXCHANGE, None, 200).await["token"]
        .as_str()
        .expect("exchange token")
        .to_owned()
}

async fn admitted(node: &Node, key: &str, exchanged: &str) {
    http(node, key, true, Method::GET, KEYS, None, 200).await;
    http(node, exchanged, false, Method::GET, KEYS, None, 200).await;
    // An authenticated nonexistent task reaches task lookup; this proves the
    // real gRPC auth path without generating a model/tool effect.
    grpc(node, exchanged, tonic::Code::NotFound).await;
}

async fn denied(node: &Node, key: &str, exchanged: &str) {
    http(node, key, true, Method::GET, KEYS, None, 401).await;
    http(node, key, true, Method::POST, EXCHANGE, None, 401).await;
    http(node, exchanged, false, Method::GET, KEYS, None, 401).await;
    grpc(node, exchanged, tonic::Code::Unauthenticated).await;
}

async fn revoke(node: &Node, owner: &str, id: &str) {
    http(
        node,
        owner,
        false,
        Method::DELETE,
        &format!("{KEYS}/{id}"),
        None,
        200,
    )
    .await;
}

async fn ownership_and_expiry(node: &Node, issuer: Option<&str>, owner: &str, id: &str) {
    let other_tenant = token(issuer, "tenant-b", "same");
    let other_subject = token(issuer, "tenant-a", "other");
    for foreign in [&other_tenant, &other_subject] {
        // Remote subject 'other' has no workspace assignment and stops at ingress.
        let expected = if issuer.is_some() && foreign == &other_subject {
            403
        } else {
            200
        };
        let response = http(node, foreign, false, Method::GET, KEYS, None, expected).await;
        if expected == 200 {
            assert!(
                response["keys"].as_array().unwrap().is_empty(),
                "foreign owner saw metadata"
            );
            http(
                node,
                foreign,
                false,
                Method::DELETE,
                &format!("{KEYS}/{id}"),
                None,
                404,
            )
            .await;
        }
    }
    http(
        node,
        "invalid-jwt",
        false,
        Method::POST,
        KEYS,
        Some(json!({"name":"unauthorized"})),
        401,
    )
    .await;
    http(
        node,
        owner,
        false,
        Method::POST,
        KEYS,
        Some(json!({"name":"escalated","roles":["admin"]})),
        403,
    )
    .await;
    let legacy_exchange = signed(
        json!({"sub":"same","tenant_id":"tenant-a","roles":["user"],
        "uar_credential_kind":"api_key","exp":chrono::Utc::now().timestamp()+3600}),
        issuer,
    );
    let response = http(node, &legacy_exchange, false, Method::GET, KEYS, None, 401).await;
    assert!(
        response["error"]["code"] == "api_key_reissue_required",
        "legacy exchange lacks explicit reissue outcome"
    );
    grpc(node, &legacy_exchange, tonic::Code::Unauthenticated).await;

    let (_expiry_id, expiring) = create(node, owner, Some(12)).await;
    let expiring_jwt = exchange(node, &expiring).await;
    admitted(node, &expiring, &expiring_jwt).await;
    tokio::time::sleep(std::time::Duration::from_secs(13)).await;
    denied(node, &expiring, &expiring_jwt).await;
}

async fn prove_surreal_hash_and_tombstone(database: &Database, id: &str, raw: &str) {
    let store = SurrealApiKeyStorage::new(database.connect().await);
    let row = store
        .get_by_id(id)
        .await
        .expect("durable authority lookup")
        .expect("durable record exists");
    assert!(
        row.key_hash.starts_with("$argon2id$") && !row.key_hash.contains(raw),
        "stored secret must be Argon2id only"
    );
    assert!(row.revoked, "revocation tombstone missing");
    assert!(
        row.authority_version == 1
            && row.subject == "same"
            && row.tenant_id.as_deref() == Some("tenant-a"),
        "durable verified owner and authority metadata missing"
    );
    let mut restore = row;
    restore.revoked = false;
    assert!(
        store.insert(restore).await.is_err(),
        "duplicate insert revived revoked key"
    );
    assert!(
        store.get_by_id(id).await.unwrap().unwrap().revoked,
        "failed duplicate insert changed tombstone"
    );
}

#[cfg(feature = "in-memory-backend")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn explicit_local_development_memory_remains_volatile() {
    let database = Database::development_memory();
    let owner = token(None, "tenant-a", "same");
    let first = Node::boot(&database, None).await;
    let (_id, key) = create(&first, &owner, None).await;
    let exchanged = exchange(&first, &key).await;
    admitted(&first, &key, &exchanged).await;
    first.stop().await;
    let second = Node::boot(&database, None).await;
    denied(&second, &key, &exchanged).await;
    second.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires isolated durable SurrealDB via UAR_API_KEY_TEST_SURREAL_URL (ws:// non-TLS)"]
async fn sidecar_delegated_key_admission_rechecks_revocation_and_outage() {
    let mut database = Database::shared();
    let proxy = OutageProxy::start(&database.url).await;
    database.url = proxy.url.clone();
    let owner = token(None, "tenant-a", "same");
    let management = Node::boot(&database, None).await;
    let sidecar = Node::sidecar(&database).await;
    let (id, key) = create(&management, &owner, None).await;
    let derived_grant = issue_sidecar_grant(&sidecar, Some(&key)).await;
    let host_grant = issue_sidecar_grant(&sidecar, None).await;
    let capabilities = "/api/uar/full-harness/v1/capabilities";
    http(
        &sidecar,
        &derived_grant,
        false,
        Method::GET,
        capabilities,
        None,
        200,
    )
    .await;
    http(
        &sidecar,
        &host_grant,
        false,
        Method::GET,
        capabilities,
        None,
        200,
    )
    .await;
    revoke(&management, &owner, &id).await;
    http(
        &sidecar,
        &derived_grant,
        false,
        Method::GET,
        capabilities,
        None,
        401,
    )
    .await;
    http(
        &sidecar,
        &host_grant,
        false,
        Method::GET,
        capabilities,
        None,
        200,
    )
    .await;
    let (_live_id, live_key) = create(&management, &owner, None).await;
    let live_grant = issue_sidecar_grant(&sidecar, Some(&live_key)).await;
    http(
        &sidecar,
        &live_grant,
        false,
        Method::GET,
        capabilities,
        None,
        200,
    )
    .await;
    proxy.disconnect();
    http(
        &sidecar,
        &live_grant,
        false,
        Method::GET,
        capabilities,
        None,
        503,
    )
    .await;
    http(
        &sidecar,
        &host_grant,
        false,
        Method::GET,
        capabilities,
        None,
        200,
    )
    .await;
    sidecar.stop().await;
    management.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn local_surreal_kv_keys_exchange_and_revocation_survive_restart() {
    let database = Database::local();
    let owner = token(None, "tenant-a", "same");
    let first = Node::boot(&database, None).await;
    let (id, key) = create(&first, &owner, None).await;
    let exchanged = exchange(&first, &key).await;
    admitted(&first, &key, &exchanged).await;
    ownership_and_expiry(&first, None, &owner, &id).await;
    first.stop().await;
    let second = Node::boot(&database, None).await;
    admitted(&second, &key, &exchanged).await;
    revoke(&second, &owner, &id).await;
    denied(&second, &key, &exchanged).await;
    second.stop().await;
    let third = Node::boot(&database, None).await;
    denied(&third, &key, &exchanged).await;
    let metadata = http(&third, &owner, false, Method::GET, KEYS, None, 200).await;
    assert!(
        metadata["keys"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["id"] == id && row["revoked"] == true),
        "revocation must remain inspectable after restart"
    );
    third.stop().await;
    prove_surreal_hash_and_tombstone(&database, &id, &key).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires isolated durable SurrealDB via UAR_API_KEY_TEST_SURREAL_URL (ws:// non-TLS)"]
async fn shared_surreal_two_processes_current_authority_and_transport_outage() {
    let mut database = Database::shared();
    let proxy = OutageProxy::start(&database.url).await;
    let real_url = std::mem::replace(&mut database.url, proxy.url.clone());
    let owner = token(Some(ISSUER), "tenant-a", "same");
    let first = Node::boot(&database, Some(ISSUER)).await;
    let second = Node::boot(&database, Some(ISSUER)).await;
    let (id, key) = create(&first, &owner, None).await;
    let exchanged = exchange(&first, &key).await;
    admitted(&second, &key, &exchanged).await;
    ownership_and_expiry(&second, Some(ISSUER), &owner, &id).await;
    revoke(&first, &owner, &id).await;
    denied(&second, &key, &exchanged).await;
    first.stop().await;
    second.stop().await;
    let first = Node::boot(&database, Some(ISSUER)).await;
    let second = Node::boot(&database, Some(ISSUER)).await;
    denied(&first, &key, &exchanged).await;
    denied(&second, &key, &exchanged).await;

    // Trusted storage changes its delegated roles. Previously signed authority
    // must not retain those old permissions on either ingress.
    let (changed_id, changed_key) = create(&first, &owner, None).await;
    let changed_jwt = exchange(&first, &changed_key).await;
    admitted(&second, &changed_key, &changed_jwt).await;
    let db = database.connect().await;
    db.query("UPDATE type::record('auth_api_keys', $id) SET data.roles = []")
        .bind(("id", changed_id))
        .await
        .unwrap()
        .check()
        .unwrap();
    http(&second, &changed_jwt, false, Method::GET, KEYS, None, 401).await;
    grpc(&second, &changed_jwt, tonic::Code::Unauthenticated).await;

    // Same subject/tenant under a different verifier cannot claim stored keys.
    let foreign = Node::boot(&database, Some("other-key-issuer")).await;
    let foreign_owner = token(Some("other-key-issuer"), "tenant-a", "same");
    assert!(
        http(
            &foreign,
            &foreign_owner,
            false,
            Method::GET,
            KEYS,
            None,
            200
        )
        .await["keys"]
            .as_array()
            .unwrap()
            .is_empty(),
        "foreign issuer saw owner keys"
    );
    http(
        &foreign,
        &foreign_owner,
        false,
        Method::DELETE,
        &format!("{KEYS}/{id}"),
        None,
        404,
    )
    .await;
    http(&foreign, &changed_key, true, Method::GET, KEYS, None, 401).await;
    foreign.stop().await;
    drop(db);

    let (_live_id, live_key) = create(&first, &owner, None).await;
    let live_jwt = exchange(&first, &live_key).await;
    admitted(&second, &live_key, &live_jwt).await;
    proxy.disconnect();
    http(&second, &live_key, true, Method::GET, KEYS, None, 503).await;
    http(&second, &live_jwt, false, Method::GET, KEYS, None, 503).await;
    http(&first, &live_key, true, Method::POST, EXCHANGE, None, 503).await;
    http(&first, &owner, false, Method::GET, KEYS, None, 503).await;
    http(
        &first,
        &owner,
        false,
        Method::POST,
        KEYS,
        Some(json!({"name":"outage"})),
        503,
    )
    .await;
    http(
        &first,
        &owner,
        false,
        Method::DELETE,
        &format!("{KEYS}/{id}"),
        None,
        503,
    )
    .await;
    grpc(&second, &live_jwt, tonic::Code::Unavailable).await;
    // Issuer JWT auth itself does not acquire API-key authority.
    issuer_grpc_after_outage(&second, &owner).await;
    first.stop().await;
    second.stop().await;
    database.url = real_url;
    prove_surreal_hash_and_tombstone(&database, &id, &key).await;
}

#[cfg(feature = "postgres-backend")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires isolated migrated pgvector Postgres DB via UAR_API_KEY_TEST_POSTGRES_URL"]
async fn postgres_modified_key_paths_restart_revocation_and_outage() {
    use universal_agent_runtime::uar::security::api_keys::PostgresApiKeyStorage;
    let mut database = Database::postgres();
    let proxy = OutageProxy::start(&database.url).await;
    let real_url = std::mem::replace(&mut database.url, proxy.url.clone());
    let owner = token(Some(ISSUER), "tenant-a", "same");
    let first = Node::boot(&database, Some(ISSUER)).await;
    let second = Node::boot(&database, Some(ISSUER)).await;
    let (id, key) = create(&first, &owner, None).await;
    let exchanged = exchange(&first, &key).await;
    admitted(&second, &key, &exchanged).await;
    ownership_and_expiry(&second, Some(ISSUER), &owner, &id).await;
    revoke(&first, &owner, &id).await;
    denied(&second, &key, &exchanged).await;
    first.stop().await;
    second.stop().await;
    let first = Node::boot(&database, Some(ISSUER)).await;
    let second = Node::boot(&database, Some(ISSUER)).await;
    denied(&first, &key, &exchanged).await;
    denied(&second, &key, &exchanged).await;
    let authority_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&real_url)
        .await
        .expect("fixture current-authority connection");
    let (changed_id, changed_key) = create(&first, &owner, None).await;
    let changed_jwt = exchange(&first, &changed_key).await;
    admitted(&second, &changed_key, &changed_jwt).await;
    sqlx::query(
        "UPDATE auth_api_keys SET data = jsonb_set(data, '{roles}', '[]'::jsonb) WHERE key_id = $1",
    )
    .bind(changed_id)
    .execute(&authority_pool)
    .await
    .expect("trusted fixture authority reduction");
    http(&second, &changed_jwt, false, Method::GET, KEYS, None, 401).await;
    grpc(&second, &changed_jwt, tonic::Code::Unauthenticated).await;
    authority_pool.close().await;
    let (_live_id, live_key) = create(&first, &owner, None).await;
    let live_jwt = exchange(&first, &live_key).await;
    admitted(&second, &live_key, &live_jwt).await;
    proxy.disconnect();
    http(&second, &live_key, true, Method::GET, KEYS, None, 503).await;
    http(&second, &live_jwt, false, Method::GET, KEYS, None, 503).await;
    http(&first, &live_key, true, Method::POST, EXCHANGE, None, 503).await;
    http(&first, &owner, false, Method::GET, KEYS, None, 503).await;
    http(
        &first,
        &owner,
        false,
        Method::POST,
        KEYS,
        Some(json!({"name":"outage"})),
        503,
    )
    .await;
    http(
        &first,
        &owner,
        false,
        Method::DELETE,
        &format!("{KEYS}/{id}"),
        None,
        503,
    )
    .await;
    grpc(&second, &live_jwt, tonic::Code::Unavailable).await;
    issuer_grpc_after_outage(&second, &owner).await;
    first.stop().await;
    second.stop().await;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&real_url)
        .await
        .expect("fixture DB authority");
    let store = PostgresApiKeyStorage::new(pool.clone());
    let row = store.get_by_id(&id).await.unwrap().unwrap();
    assert!(
        row.revoked && row.key_hash.starts_with("$argon2id$") && !row.key_hash.contains(&key),
        "durable hashed revocation missing"
    );
    assert!(
        row.issuer.as_deref() == Some(ISSUER)
            && row.tenant_id.as_deref() == Some("tenant-a")
            && row.subject == "same",
        "Postgres owner metadata missing"
    );
    let mut restore = row;
    restore.revoked = false;
    assert!(
        store.insert(restore).await.is_err(),
        "Postgres duplicate insert revived key"
    );
    assert!(
        store.get_by_id(&id).await.unwrap().unwrap().revoked,
        "Postgres tombstone changed"
    );
    pool.close().await;
}

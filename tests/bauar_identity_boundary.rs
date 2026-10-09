//! AUTH source scenarios; execution belongs to the complete delivery gate.
//! Synthetic signed credentials exercise local boundaries; external IdP certification is excluded.
#![cfg(all(feature = "server", feature = "test-probes"))]

#[path = "support/bauar_identity_peer.rs"]
mod peer;
#[path = "support/bauar_resource_peer.rs"]
#[allow(
    dead_code,
    reason = "reuse receiver; sidecar helpers are exercised by the separate resource gate"
)]
mod resource_peer;
#[path = "support/sidecar_process.rs"]
mod sidecar_process;
#[path = "integration/live/stub_llm.rs"]
mod stub_llm;

use peer::{
    AUDIENCE,
    Credential::{Bearer, Key, None as Anonymous},
    Host, ISSUER, JwksPeer, Reply, WORKSPACE, after, claims, exchange_claims, jwk, local_token,
    now, rsa_token, security,
};
use reqwest::Method;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

const KEYS: &str = "/api/uar/auth/keys";
const EXCHANGE: &str = "/api/uar/auth/exchange";

fn admitted(result: (u16, Value)) -> String {
    assert_eq!(result.0, 200, "run admission: {}", result.1);
    result.1["run_id"].as_str().unwrap().to_owned()
}

async fn issue(host: &Host, token: &str, name: &str, lifetime: Option<i64>) -> Value {
    let result = host
        .call(
            Method::POST,
            KEYS,
            Bearer(token),
            Some(WORKSPACE),
            json!({"name":name,"roles":["user"],"expires_in_secs":lifetime}),
        )
        .await;
    assert_eq!(result.0, 201, "key issuance: {}", result.1);
    result.1
}

async fn listing(host: &Host, token: &str) -> Value {
    let result = host
        .call(Method::GET, KEYS, Bearer(token), Some(WORKSPACE), json!({}))
        .await;
    assert_eq!(result.0, 200, "key listing: {}", result.1);
    result.1
}

fn contains_key(list: &Value, id: &str) -> bool {
    list["keys"]
        .as_array()
        .unwrap()
        .iter()
        .any(|key| key["id"] == id)
}

#[path = "bauar_identity_boundary/key_authority.rs"]
mod key_authority;
#[path = "bauar_identity_boundary/local_admission.rs"]
mod local_admission;
#[path = "bauar_identity_boundary/remote_jwks.rs"]
mod remote_jwks;

use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn remote_jwks_router_requires_tenant_and_mapping_for_bearer_and_direct_key() {
    let peer = JwksPeer::start(vec![jwk("a", false)]).await;
    let host = Host::start(security(Some(&peer.url)), None).await;
    let valid = claims("owner-a", "tenant-a");
    let token = rsa_token("a", false, &valid);
    let mut variants = Vec::new();
    for field in ["iss", "aud", "exp", "tenant_id", "sub"] {
        let mut invalid = valid.clone();
        invalid.as_object_mut().unwrap().remove(field);
        variants.push(invalid);
    }
    for field in ["iss", "aud"] {
        let mut invalid = valid.clone();
        invalid[field] = json!("other");
        variants.push(invalid);
    }
    for field in ["sub", "tenant_id"] {
        let mut invalid = valid.clone();
        invalid[field] = json!("");
        variants.push(invalid);
    }
    let mut expired = valid.clone();
    expired["exp"] = json!(now() - 300);
    variants.push(expired);
    let mut future = valid.clone();
    future["nbf"] = json!(now() + 300);
    variants.push(future);
    for invalid in variants {
        assert_eq!(
            host.run(
                Bearer(&rsa_token("a", false, &invalid)),
                Some(WORKSPACE),
                None
            )
            .await
            .0,
            401
        );
    }
    assert_eq!(
        host.run(Bearer(&local_token(&valid)), Some(WORKSPACE), None)
            .await
            .0,
        401
    );
    assert_eq!(
        host.run(Bearer(&token), Some("workspace-unmapped"), None)
            .await
            .0,
        403
    );
    assert_eq!(
        host.run(
            Bearer(&rsa_token("a", false, &claims("unmapped", "tenant-a"))),
            None,
            None
        )
        .await
        .0,
        403
    );
    assert_eq!(host.model_calls().await, 0);
    let run = admitted(host.run(Bearer(&token), Some(WORKSPACE), None).await);
    host.finish(Bearer(&token), Some(WORKSPACE), &run).await;
    let key = issue(&host, &token, "remote-key", None).await;
    let raw = key["raw_key"].as_str().unwrap();
    assert_eq!(key["metadata"]["issuer"], ISSUER);
    assert_eq!(key["metadata"]["tenant_id"], "tenant-a");
    assert_eq!(
        host.run(Key(raw), Some("workspace-unmapped"), None).await.0,
        403
    );
    let run = admitted(host.run(Key(raw), Some(WORKSPACE), None).await);
    host.finish(Key(raw), Some(WORKSPACE), &run).await;
    assert_eq!(host.exchange_key(raw).await.0, 501);
    assert_eq!(
        peer.starts().len(),
        1,
        "known signing key fetched repeatedly"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn real_jwks_rotation_cooldown_failed_refresh_hard_age_and_recovery() {
    let peer = JwksPeer::start(vec![jwk("a", false), jwk("b", true)]).await;
    let host = Host::start(security(Some(&peer.url)), None).await;
    let user = claims("owner-a", "tenant-a");
    let a = rsa_token("a", false, &user);
    let b = rsa_token("b", true, &user);
    assert_eq!(
        host.call(Method::GET, KEYS, Bearer(&a), Some(WORKSPACE), json!({}))
            .await
            .0,
        200
    );
    assert_eq!(
        host.call(Method::GET, KEYS, Bearer(&b), Some(WORKSPACE), json!({}))
            .await
            .0,
        200
    );
    let unknown = rsa_token("unknown-kid-sentinel", false, &user);
    assert_eq!(
        host.call(
            Method::GET,
            KEYS,
            Bearer(&unknown),
            Some(WORKSPACE),
            json!({})
        )
        .await
        .0,
        401
    );
    assert_eq!(peer.starts().len(), 1);
    after(peer.starts()[0], Duration::from_secs(6)).await;
    let mut rotated = Reply::keys(vec![jwk("a", true), jwk("b", true)]);
    rotated.header_delay = Duration::from_millis(500);
    peer.set(rotated);
    let mut wave = tokio::task::JoinSet::new();
    for index in 0..8 {
        let request = host.request(
            Method::GET,
            KEYS,
            Bearer(&rsa_token(&format!("unknown-{index}"), false, &user)),
            Some(WORKSPACE),
        );
        wave.spawn(async move { request.send().await.unwrap().status().as_u16() });
    }
    while let Some(status) = wave.join_next().await {
        assert_eq!(status.unwrap(), 401);
    }
    assert_eq!(peer.starts().len(), 2);
    assert_eq!(peer.maximum(), 1);
    let rotated_at = peer.completed_at();
    let new_a = rsa_token("a", true, &user);
    assert_eq!(
        host.call(
            Method::GET,
            KEYS,
            Bearer(&new_a),
            Some(WORKSPACE),
            json!({})
        )
        .await
        .0,
        200
    );
    assert_eq!(
        host.call(Method::GET, KEYS, Bearer(&a), Some(WORKSPACE), json!({}))
            .await
            .0,
        401
    );
    assert_eq!(
        peer.starts().len(),
        2,
        "fresh/cooldown requests fetched repeatedly"
    );
    // A known key must initiate target-age refresh and apply successful removal.
    after(rotated_at, Duration::from_secs(61)).await;
    peer.set(Reply::keys(vec![jwk("b", true)]));
    assert_eq!(
        host.call(Method::GET, KEYS, Bearer(&b), Some(WORKSPACE), json!({}))
            .await
            .0,
        200
    );
    let success = peer.completed_at();
    assert_eq!(
        host.call(
            Method::GET,
            KEYS,
            Bearer(&new_a),
            Some(WORKSPACE),
            json!({})
        )
        .await
        .0,
        401
    );
    assert_eq!(peer.starts().len(), 3);
    after(success, Duration::from_secs(61)).await;
    peer.set(Reply::failure());
    assert_eq!(
        host.call(Method::GET, KEYS, Bearer(&b), Some(WORKSPACE), json!({}))
            .await
            .0,
        200
    );
    for body in [
        "malformed-key-material-sentinel".to_owned(),
        json!({"keys":[jwk("b", true),
        {"kty":"RSA","kid":"invalid-material-sentinel","n":"!","e":"AQAB"}]})
        .to_string(),
    ] {
        after(*peer.starts().last().unwrap(), Duration::from_secs(6)).await;
        let mut invalid = Reply::keys(Vec::new());
        invalid.body = body;
        peer.set(invalid);
        assert_eq!(
            host.call(Method::GET, KEYS, Bearer(&b), Some(WORKSPACE), json!({}))
                .await
                .0,
            200
        );
    }
    peer.set(Reply::failure());
    // Peer completion is slightly before server publication; 301s crosses hard age
    // without a private clock hook. Exact 300s equality is an unexecuted component scenario.
    after(success, Duration::from_secs(301)).await;
    assert_eq!(
        host.call(Method::GET, KEYS, Bearer(&b), Some(WORKSPACE), json!({}))
            .await
            .0,
        401
    );
    after(*peer.starts().last().unwrap(), Duration::from_secs(6)).await;
    peer.set(Reply::keys(vec![jwk("b", true)]));
    assert_eq!(
        host.call(Method::GET, KEYS, Bearer(&b), Some(WORKSPACE), json!({}))
            .await
            .0,
        200
    );
    assert_eq!(
        host.call(
            Method::GET,
            KEYS,
            Bearer(&new_a),
            Some(WORKSPACE),
            json!({})
        )
        .await
        .0,
        401
    );
    assert_eq!(peer.maximum(), 1);
    let starts = peer.starts();
    assert!(
        starts
            .windows(2)
            .all(|pair| pair[1].duration_since(pair[0]) >= Duration::from_secs(5))
    );
    let logs = format!("{}{}", host.process.stdout(), host.process.stderr());
    assert!(logs.contains("JWKS refresh failed"));
    for sentinel in [
        "fixture-endpoint-sentinel",
        "unknown-kid-sentinel",
        "malformed-key-material-sentinel",
        "invalid-material-sentinel",
    ] {
        assert!(
            !logs.contains(sentinel),
            "JWKS diagnostic exposed {sentinel}"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn remote_jwks_header_and_body_delay_share_the_total_verifier_budget() {
    let peer = JwksPeer::start(vec![jwk("a", false)]).await;
    let host = Host::start(security(Some(&peer.url)), None).await;
    let token = rsa_token("a", false, &claims("owner-a", "tenant-a"));
    for body_delay in [false, true] {
        let mut reply = Reply::keys(vec![jwk("a", false)]);
        if body_delay {
            reply.body_delay = Duration::from_secs(6);
        } else {
            reply.header_delay = Duration::from_secs(6);
        }
        peer.set(reply);
        let started = Instant::now();
        assert_eq!(host.run(Bearer(&token), Some(WORKSPACE), None).await.0, 401);
        assert!((Duration::from_secs(4)..Duration::from_secs(8)).contains(&started.elapsed()));
        after(started, Duration::from_secs(7)).await;
    }
    assert_eq!(host.model_calls().await, 0);
    assert_eq!(peer.starts().len(), 2);
    assert_eq!(peer.maximum(), 1);
    peer.set(Reply::keys(vec![jwk("a", false)]));
    let run = admitted(host.run(Bearer(&token), Some(WORKSPACE), None).await);
    host.finish(Bearer(&token), Some(WORKSPACE), &run).await;
}

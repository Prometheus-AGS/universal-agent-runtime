use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn http_key_issue_direct_exchange_revoke_preserve_identity_roles_and_bounded_ttl() {
    let host = Host::start(security(None), None).await;
    let mut owner = claims("owner-a", "tenant-a");
    owner["roles"] = json!([
        "user",
        "fixture:invoke",
        "not-delegable",
        "admin",
        "host-session"
    ]);
    let token = local_token(&owner);
    for roles in [
        json!(["admin"]),
        json!(["host-session"]),
        json!(["unheld"]),
        json!(["not-delegable"]),
    ] {
        assert_eq!(
            host.call(
                Method::POST,
                KEYS,
                Bearer(&token),
                None,
                json!({"name":"must-not-insert","roles":roles})
            )
            .await
            .0,
            403
        );
    }
    let mut no_default = owner.clone();
    no_default["roles"] = json!(["fixture:invoke"]);
    assert_eq!(
        host.call(
            Method::POST,
            KEYS,
            Bearer(&local_token(&no_default)),
            None,
            json!({"name":"missing-default-role"})
        )
        .await
        .0,
        403
    );
    assert_eq!(listing(&host, &token).await["keys"], json!([]));
    for lifetime in [0, -1, i64::MAX] {
        assert_eq!(
            host.call(
                Method::POST,
                KEYS,
                Bearer(&token),
                None,
                json!({"name":"invalid-life","expires_in_secs":lifetime})
            )
            .await
            .0,
            400
        );
    }
    let created = issue(&host, &token, "retained-identity", None).await;
    let raw = created["raw_key"].as_str().unwrap();
    let id = created["metadata"]["id"].as_str().unwrap();
    assert_eq!(created["metadata"]["authority_version"], 1);
    assert_eq!(created["metadata"]["issuer"], ISSUER);
    assert_eq!(created["metadata"]["tenant_id"], "tenant-a");
    assert_eq!(created["metadata"]["roles"], json!(["user"]));
    let list = listing(&host, &token).await;
    assert!(contains_key(&list, id));
    assert!(!list.to_string().contains(raw));
    assert!(!list.to_string().contains("key_hash"));
    let direct = admitted(host.run(Key(raw), None, None).await);
    host.finish(Key(raw), None, &direct).await;
    let issued = host
        .call(
            Method::POST,
            EXCHANGE,
            Anonymous,
            None,
            json!({"api_key":raw}),
        )
        .await;
    assert_eq!(issued.0, 200, "body exchange: {}", issued.1);
    assert_eq!(issued.1["expires_in"], 3600);
    let exchanged = issued.1["token"].as_str().unwrap();
    let retained = exchange_claims(exchanged);
    assert_eq!(retained["iss"], ISSUER);
    assert_eq!(retained["aud"], AUDIENCE);
    assert_eq!(retained["sub"], "owner-a");
    assert_eq!(retained["tenant_id"], "tenant-a");
    assert_eq!(retained["roles"], json!(["user"]));
    assert_eq!(retained["uar_credential_kind"], "api_key");
    assert!(retained["uar_instance_id"].is_null());
    let run = admitted(host.run(Bearer(exchanged), None, None).await);
    host.finish(Bearer(exchanged), None, &run).await;
    // Bearer precedence must not recover an invalid bearer through a valid key.
    let response = host
        .request(Method::GET, KEYS, Key(raw), None)
        .bearer_auth("invalid")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 401);

    let short = issue(&host, &token, "bounded-life", Some(15)).await;
    let short_raw = short["raw_key"].as_str().unwrap();
    let before = now();
    let issued_short = host.exchange_key(short_raw).await;
    let after_exchange = now();
    assert_eq!(issued_short.0, 200);
    let short_jwt = issued_short.1["token"].as_str().unwrap();
    let expiry = exchange_claims(short_jwt)["exp"].as_u64().unwrap();
    assert_eq!(expiry, short["metadata"]["expires_at"].as_u64().unwrap());
    let ttl = issued_short.1["expires_in"].as_u64().unwrap();
    assert!((1..=15).contains(&ttl));
    assert!((before..=after_exchange).contains(&(expiry - ttl)));
    while now() < expiry {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(host.run(Key(short_raw), None, None).await.0, 401);
    assert_eq!(host.exchange_key(short_raw).await.0, 401);
    // The existing JWT expiry leeway is explicit; no retroactive JWT revocation claim.
    assert_eq!(
        host.call(Method::GET, KEYS, Bearer(short_jwt), None, json!({}))
            .await
            .0,
        200
    );
    after(Instant::now(), Duration::from_secs(62)).await;
    assert_eq!(host.run(Bearer(short_jwt), None, None).await.0, 401);

    assert_eq!(
        host.call(
            Method::DELETE,
            &format!("{KEYS}/{id}"),
            Bearer(&token),
            None,
            json!({})
        )
        .await
        .0,
        200
    );
    assert_eq!(host.run(Key(raw), None, None).await.0, 401);
    assert_eq!(host.exchange_key(raw).await.0, 401);
    assert_eq!(
        host.call(Method::GET, KEYS, Bearer(exchanged), None, json!({}))
            .await
            .0,
        200
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn key_metadata_and_mutation_require_owner_tenant_and_exact_issuer_admin_kind() {
    let host = Host::start(security(None), None).await;
    let owner = local_token(&claims("owner-a", "tenant-a"));
    let created = issue(&host, &owner, "foreign-owner-sentinel", None).await;
    let id = created["metadata"]["id"].as_str().unwrap();
    let other_tenant = local_token(&claims("owner-a", "tenant-b"));
    let tenant_key = issue(&host, &other_tenant, "foreign-tenant-sentinel", None).await;
    let tenant_id = tenant_key["metadata"]["id"].as_str().unwrap();
    let mut role_only = claims("owner-b", "tenant-a");
    role_only["roles"] = json!(["user", "admin"]);
    for denied in [local_token(&role_only), other_tenant.clone()] {
        let list = listing(&host, &denied).await;
        assert!(!contains_key(&list, id));
        assert!(!list.to_string().contains("foreign-owner-sentinel"));
        let actual = host
            .call(
                Method::DELETE,
                &format!("{KEYS}/{id}"),
                Bearer(&denied),
                None,
                json!({}),
            )
            .await;
        let missing = host
            .call(
                Method::DELETE,
                &format!("{KEYS}/missing"),
                Bearer(&denied),
                None,
                json!({}),
            )
            .await;
        assert_eq!(
            (actual.0, &actual.1["error"]),
            (missing.0, &missing.1["error"])
        );
        assert_eq!(actual.0, 404);
    }
    let admin = claims("admin-a", "tenant-a");
    let administrator = local_token(&admin);
    let own = issue(&host, &administrator, "admin-own-key", None).await;
    let own_id = own["metadata"]["id"].as_str().unwrap();
    for marker in [None, Some("unknown"), Some("api_key")] {
        let mut unprivileged = admin.clone();
        match marker {
            Some(kind) => unprivileged["uar_credential_kind"] = json!(kind),
            None => {
                unprivileged
                    .as_object_mut()
                    .unwrap()
                    .remove("uar_credential_kind");
            }
        }
        let unprivileged = local_token(&unprivileged);
        let list = listing(&host, &unprivileged).await;
        assert!(
            contains_key(&list, own_id),
            "ordinary owner metadata must remain accessible"
        );
        assert!(!contains_key(&list, id));
        assert_eq!(
            host.call(
                Method::DELETE,
                &format!("{KEYS}/{id}"),
                Bearer(&unprivileged),
                None,
                json!({})
            )
            .await
            .0,
            404
        );
    }
    let admin_key = own["raw_key"].as_str().unwrap();
    let direct = host
        .call(Method::GET, KEYS, Key(admin_key), None, json!({}))
        .await;
    assert_eq!(direct.0, 200);
    assert!(contains_key(&direct.1, own_id));
    assert!(!contains_key(&direct.1, id));
    let exchange = host.exchange_key(admin_key).await;
    assert_eq!(exchange.0, 200);
    let exchange = exchange.1["token"].as_str().unwrap();
    assert!(!contains_key(&listing(&host, exchange).await, id));
    for credential in [Key(admin_key), Bearer(exchange)] {
        assert_eq!(
            host.call(
                Method::DELETE,
                &format!("{KEYS}/{id}"),
                credential,
                None,
                json!({})
            )
            .await
            .0,
            404
        );
    }
    let list = listing(&host, &administrator).await;
    assert!(contains_key(&list, id));
    assert!(!contains_key(&list, tenant_id));
    let wrong_admin_tenant = local_token(&claims("admin-a", "tenant-b"));
    assert!(!contains_key(
        &listing(&host, &wrong_admin_tenant).await,
        tenant_id
    ));
    assert_eq!(
        host.call(
            Method::DELETE,
            &format!("{KEYS}/{tenant_id}"),
            Bearer(&wrong_admin_tenant),
            None,
            json!({})
        )
        .await
        .0,
        404
    );
    assert_eq!(listing(&host, &owner).await["keys"][0]["revoked"], false);
    let raw = created["raw_key"].as_str().unwrap();
    let run = admitted(host.run(Key(raw), None, None).await);
    host.finish(Key(raw), None, &run).await;
    assert_eq!(
        host.call(
            Method::DELETE,
            &format!("{KEYS}/{id}"),
            Bearer(&administrator),
            None,
            json!({})
        )
        .await
        .0,
        200
    );
    assert_eq!(host.run(Key(raw), None, None).await.0, 401);
    assert_eq!(
        host.call(
            Method::DELETE,
            &format!("{KEYS}/{tenant_id}"),
            Bearer(&administrator),
            None,
            json!({})
        )
        .await
        .0,
        404
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn standalone_host_grant_requires_mapped_issuer_credential_and_retains_scope_attenuation() {
    let receiver = resource_peer::Peer::start().await;
    let response = reqwest::Client::new()
        .post(&receiver.url)
        .bearer_auth("resource-a1")
        .json(&json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"effect","arguments":{"argument":"identity-peer-control"}}}))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    assert_eq!(receiver.effects().len(), 1);
    let host = Host::start(security(None), Some(&receiver.url)).await;
    let host_claims = claims("host-a", "tenant-a");
    let token = local_token(&host_claims);
    let grant = resource_peer::grant("a1", "identity-revision", now() + 180);
    for marker in [None, Some("unknown"), Some("api_key")] {
        let mut denied = host_claims.clone();
        denied["roles"] = json!(["user", "host-session", "uar:mcp:delegate"]);
        denied["uar_instance_id"] = json!("identity-service-host");
        match marker {
            Some(kind) => denied["uar_credential_kind"] = json!(kind),
            None => {
                denied
                    .as_object_mut()
                    .unwrap()
                    .remove("uar_credential_kind");
            }
        }
        assert_eq!(
            host.run(Bearer(&local_token(&denied)), None, Some(grant.clone()))
                .await
                .0,
            422
        );
    }
    for (subject, tenant) in [("owner-a", "tenant-a"), ("host-a", "tenant-b")] {
        assert_eq!(
            host.run(
                Bearer(&local_token(&claims(subject, tenant))),
                None,
                Some(grant.clone())
            )
            .await
            .0,
            422
        );
    }
    let key = issue(&host, &token, "service-owned-key", None).await;
    let raw = key["raw_key"].as_str().unwrap();
    assert_eq!(host.run(Key(raw), None, Some(grant.clone())).await.0, 422);
    let exchange = host.exchange_key(raw).await;
    assert_eq!(exchange.0, 200);
    assert_eq!(
        host.run(
            Bearer(exchange.1["token"].as_str().unwrap()),
            None,
            Some(grant.clone())
        )
        .await
        .0,
        422
    );
    let mut reduced = grant.clone();
    reduced["grant"]["scopes"] = json!([]);
    assert_eq!(host.run(Bearer(&token), None, Some(reduced)).await.0, 422);
    assert_eq!(
        host.model_calls().await,
        0,
        "unprivileged grant reached provider"
    );
    assert_eq!(receiver.effects().len(), 1);
    let run = admitted(host.run(Bearer(&token), None, Some(grant)).await);
    host.finish(Bearer(&token), None, &run).await;
}

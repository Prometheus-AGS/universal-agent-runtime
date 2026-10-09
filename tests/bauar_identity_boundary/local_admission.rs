use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn explicit_local_startup_validates_signed_registered_claims_before_run_admission() {
    let host = Host::start(security(None), None).await;
    assert!(host.workspace.path().join("identity.yaml").exists());
    let valid = claims("owner-a", "tenant-a");
    let mut variants = Vec::new();
    for field in ["iss", "aud", "exp"] {
        let mut invalid = valid.clone();
        invalid.as_object_mut().unwrap().remove(field);
        variants.push(invalid);
    }
    for field in ["iss", "aud"] {
        let mut invalid = valid.clone();
        invalid[field] = json!("untrusted");
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
            host.run(Bearer(&local_token(&invalid)), None, None).await.0,
            401
        );
    }
    assert_eq!(host.run(Anonymous, None, None).await.0, 401);
    assert_eq!(
        host.model_calls().await,
        0,
        "denied credentials reached provider"
    );
    let token = local_token(&valid);
    let run = admitted(host.run(Bearer(&token), None, None).await);
    host.finish(Bearer(&token), None, &run).await;
    assert!(
        host.model_calls().await > 0,
        "positive control did not dispatch"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn incomplete_remote_policy_is_rejected_at_actual_startup() {
    let workspace = sidecar_process::Workspace::new();
    let mut policy = security(Some("http://127.0.0.1:1/jwks"));
    policy["jwt_required"] = json!(false);
    let (path, _) = peer::config(&workspace, "http://127.0.0.1:1/v1", policy);
    let mut process = sidecar_process::spawn(
        sidecar_process::standalone_binary(),
        &workspace,
        &sidecar_process::LaunchOptions::new(path, "invalid-remote-policy"),
    );
    let exit = process
        .wait_exit(Duration::from_secs(20))
        .await
        .expect("invalid policy must stop startup");
    assert!(!exit.success());
    let diagnostics = format!("{}{}", process.stdout(), process.stderr());
    assert!(
        diagnostics.contains("remote identity policy requires JWT"),
        "expected policy failure: {diagnostics}"
    );
    assert!(!diagnostics.contains(sidecar_process::JWT_SECRET));
}

//! Source scenarios for the actual registered-destination host admission boundary.
use super::*;
use crate::config::SecurityConfig;
use crate::uar::security::{
    claims::UserContext,
    jwt,
    sidecar_guard::{self, HostAuthenticated, SidecarGuard, SidecarLaunchToken},
    verifier,
};
use axum::{
    Extension, Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::get,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use tower::ServiceExt;

struct Destination {
    catalog: McpCatalog,
    environment: McpBindingEnvironment,
    calls: Arc<AtomicUsize>,
    task: tokio::task::JoinHandle<()>,
    _directory: tempfile::TempDir,
}

impl Drop for Destination {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Destination {
    async fn start() -> Self {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let app = Router::new().route(
            "/resource",
            get(move |headers: axum::http::HeaderMap| {
                let counter = counter.clone();
                async move {
                    if headers
                        .get(header::AUTHORIZATION)
                        .and_then(|value| value.to_str().ok())
                        != Some("Bearer synthetic-downstream")
                    {
                        return StatusCode::UNAUTHORIZED;
                    }
                    counter.fetch_add(1, Ordering::SeqCst);
                    StatusCode::OK
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/resource", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        // Establish receiver-valid credentials independently of host admission.
        let client = reqwest::Client::new();
        assert_eq!(
            client.get(&url).send().await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            client
                .get(&url)
                .bearer_auth("synthetic-downstream")
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        let definition = ServerDefinition::new(
            "registered".to_owned(),
            ServerSource::Global,
            McpServerEntry::RemoteHttp {
                url,
                env: HashMap::new(),
                headers: HashMap::new(),
                grant_policy: Some(RemoteHttpGrantPolicy {
                    destination_id: "fixture-destination".to_owned(),
                    trusted_hosts: BTreeSet::from([
                        "sidecar-launch-host".to_owned(),
                        "configured-host".to_owned(),
                    ]),
                    required_scopes: BTreeSet::from(["read".to_owned()]),
                    allow_private_http: true,
                }),
            },
            true,
            ServerAuthentication::Required,
        )
        .unwrap();
        let directory = tempfile::tempdir().unwrap();
        Self {
            catalog: McpCatalog::from_definitions(vec![definition]).unwrap(),
            environment: McpBindingEnvironment::new(
                directory.path().to_path_buf(),
                BTreeMap::new(),
            )
            .unwrap(),
            calls,
            task,
            _directory: directory,
        }
    }

    fn input(&self) -> RunMcpServerInput {
        RunMcpServerInput {
            name: "registered".to_owned(),
            url: None,
            headers: BTreeMap::new(),
            grant: Some(RunMcpGrantInput {
                credential_revision: "fixture-revision".to_owned(),
                scopes: BTreeSet::from(["read".to_owned()]),
                expires_at_unix: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + 120,
                headers: BTreeMap::from([(
                    "Authorization".to_owned(),
                    SecretString::from("Bearer synthetic-downstream"),
                )]),
            }),
        }
    }

    fn admit(&self, user: &UserContext) -> Result<RunMcpServers, HostInputError> {
        RunMcpServers::from_inputs(vec![self.input()], user, &self.catalog, &self.environment)
    }

    fn denied(&self, user: &UserContext) {
        assert_eq!(
            self.admit(user).unwrap_err().code,
            "run_mcp_grant_forbidden"
        );
        assert_eq!(
            self.calls.load(Ordering::SeqCst),
            1,
            "denial must not call the receiver"
        );
    }
}

fn config() -> SecurityConfig {
    serde_json::from_value(serde_json::json!({
        "jwt_required": true, "jwt_secret": "synthetic-host-signing-key", "jwt_issuer": "fixture-issuer",
        "jwt_audience": "fixture-runtime", "jwt_algorithm": "HS256", "settings_mutation_auth_required": false,
        "trusted_host_principals": [{"issuer": "fixture-issuer", "subject": "service", "tenant_id": "tenant-a", "host_id": "configured-host"}]
    })).unwrap()
}

async fn identity(
    config: &SecurityConfig,
    subject: &str,
    tenant: &str,
    marker: Option<&str>,
) -> UserContext {
    let mut claims = serde_json::json!({"sub": subject, "tenant_id": tenant,
        "roles": ["user", "host-session", "uar:mcp:delegate"], "uar_instance_id": "configured-host",
        "iss": "fixture-issuer", "aud": "fixture-runtime", "exp": chrono::Utc::now().timestamp() + 300});
    if let Some(marker) = marker {
        claims["uar_credential_kind"] = marker.into();
    }
    let token = jwt::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"synthetic-host-signing-key"),
    )
    .unwrap();
    from_principal(verifier::verify_token(config, &token).await.unwrap())
}

fn from_principal(principal: verifier::Principal) -> UserContext {
    UserContext {
        host_authority: principal.host_authority,
        authority: principal.authority,
        user_id: principal.subject,
        tenant_id: principal.tenant_id,
        claims: principal.claims,
    }
}

#[tokio::test]
async fn roles_and_legacy_tokens_cannot_replace_exact_service_provenance() {
    let destination = Destination::start().await;
    let config = config();
    for marker in [None, Some("unknown"), Some("api_key")] {
        destination.denied(&identity(&config, "service", "tenant-a", marker).await);
    }
    destination.denied(&identity(&config, "ordinary-user", "tenant-a", Some("issuer")).await);
    destination.denied(&identity(&config, "service", "tenant-b", Some("issuer")).await);
    let service = identity(&config, "service", "tenant-a", Some("issuer")).await;
    let admitted = destination.admit(&service).unwrap();
    assert_eq!(admitted.grant_markers()[0].trusted_host, "configured-host");
    let mut tampered = service.clone();
    tampered.user_id = "other-owner".to_owned();
    tampered.claims.sub = tampered.user_id.clone();
    destination.denied(&tampered);
    let mut no_mapping = config.clone();
    no_mapping.trusted_host_principals.clear();
    destination.denied(&identity(&no_mapping, "service", "tenant-a", Some("issuer")).await);
    // A valid typed host still needs the destination's scope and lease policy.
    let mut expired = destination.input();
    expired.grant.as_mut().unwrap().expires_at_unix = 0;
    assert_eq!(
        RunMcpServers::from_inputs(
            vec![expired],
            &service,
            &destination.catalog,
            &destination.environment
        )
        .unwrap_err()
        .code,
        "run_mcp_grant_authentication_required"
    );
    let mut unscoped = destination.input();
    unscoped.grant.as_mut().unwrap().scopes.clear();
    assert_eq!(
        RunMcpServers::from_inputs(
            vec![unscoped],
            &service,
            &destination.catalog,
            &destination.environment
        )
        .unwrap_err()
        .code,
        "run_mcp_grant_forbidden"
    );
}

#[tokio::test]
async fn direct_and_exchanged_keys_do_not_inherit_configured_host_authority() {
    use crate::uar::security::api_keys::{ApiKeyService, CreateKeyRequest, InMemoryApiKeyStorage};
    let destination = Destination::start().await;
    let config = config();
    let issuer = identity(&config, "service", "tenant-a", Some("issuer")).await;
    assert!(destination.admit(&issuer).is_ok());
    let keys = ApiKeyService::new(
        Arc::new(InMemoryApiKeyStorage::new()),
        "synthetic-host-signing-key",
    )
    .with_security_config(&config)
    .unwrap();
    let created = keys
        .create_key(
            &issuer,
            CreateKeyRequest {
                name: "attenuated".to_owned(),
                roles: None,
                expires_in_secs: None,
            },
        )
        .await
        .unwrap();
    let direct = keys.validate_key(&created.raw_key).await.unwrap().unwrap();
    destination.denied(&direct);
    let token = keys
        .exchange_for_jwt(&created.raw_key)
        .await
        .unwrap()
        .unwrap();
    let exchanged = from_principal(verifier::verify_token(&config, &token).await.unwrap());
    destination.denied(&exchanged);
    assert_eq!(
        keys.list_keys(&direct).await.unwrap().len(),
        1,
        "delegated owner access survives"
    );
    assert_eq!(keys.list_keys(&exchanged).await.unwrap().len(), 1);
}

#[tokio::test]
async fn actual_launch_guard_supplies_independent_installation_proof() {
    let destination = Arc::new(Destination::start().await);
    let fixture = destination.clone();
    let app = Router::new()
        .route(
            "/admit",
            get(move |Extension(proof): Extension<HostAuthenticated>| {
                let fixture = fixture.clone();
                async move {
                    let mut user = UserContext {
                        host_authority: None,
                        authority: None,
                        user_id: "local-owner".to_owned(),
                        tenant_id: None,
                        claims: crate::uar::security::claims::UserClaims {
                            sub: "local-owner".to_owned(),
                            name: None,
                            roles: None,
                            tenant_id: None,
                            uar_instance_id: None,
                            exp: usize::MAX,
                        },
                    };
                    proof.bind_context(&mut user);
                    assert!(
                        user.verified_identity().is_none(),
                        "launch must not manufacture tenant identity"
                    );
                    assert_eq!(
                        fixture.admit(&user).unwrap().grant_markers()[0].trusted_host,
                        "sidecar-launch-host"
                    );
                    StatusCode::OK
                }
            }),
        )
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(SidecarGuard::new(
                SidecarLaunchToken::from_line(format!("{}\n", "a".repeat(64)).as_bytes()).unwrap(),
                32199,
            )),
            sidecar_guard::enforce,
        ));
    let request = || {
        Request::builder()
            .uri("/admit")
            .header(header::HOST, "127.0.0.1:32199")
    };
    assert_eq!(
        app.clone()
            .oneshot(request().body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        app.oneshot(
            request()
                .header(header::AUTHORIZATION, format!("Bearer {}", "a".repeat(64)))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap()
        .status(),
        StatusCode::OK
    );
    assert_eq!(destination.calls.load(Ordering::SeqCst), 1);
}

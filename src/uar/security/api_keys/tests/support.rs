use super::super::*;

pub(super) fn local_config() -> crate::config::SecurityConfig {
    serde_json::from_value(
        serde_json::json!({"jwt_required":true, "jwt_secret":"test-secret",
        "settings_mutation_auth_required":false}),
    )
    .unwrap()
}

pub(super) async fn context_for(
    config: &crate::config::SecurityConfig,
    subject: &str,
    tenant: Option<&str>,
    roles: Vec<String>,
) -> UserContext {
    context_with_kind(config, subject, tenant, roles, Some("issuer")).await
}

pub(super) async fn context_with_kind(
    config: &crate::config::SecurityConfig,
    subject: &str,
    tenant: Option<&str>,
    roles: Vec<String>,
    kind: Option<&str>,
) -> UserContext {
    let claims = UserClaims {
        sub: subject.to_owned(),
        name: None,
        roles: Some(roles),
        tenant_id: tenant.map(str::to_owned),
        uar_instance_id: None,
        exp: usize::MAX,
    };
    let mut claims = serde_json::to_value(&claims).unwrap();
    if let Some(issuer) = &config.jwt_issuer {
        claims["iss"] = issuer.clone().into();
    }
    if let Some(audience) = &config.jwt_audience {
        claims["aud"] = audience.clone().into();
    }
    if let Some(kind) = kind {
        claims["uar_credential_kind"] = kind.into();
    }
    let token = jwt::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test-secret"),
    )
    .unwrap();
    let principal = crate::uar::security::verifier::verify_token(config, &token)
        .await
        .unwrap();
    UserContext {
        host_authority: principal.host_authority,
        authority: principal.authority,
        user_id: principal.subject,
        tenant_id: principal.tenant_id,
        claims: principal.claims,
    }
}

pub(super) async fn caller(subject: &str) -> UserContext {
    context_for(&local_config(), subject, None, vec!["user".to_owned()]).await
}

pub(super) fn make_service() -> ApiKeyService {
    let storage: Arc<dyn ApiKeyStorage> = Arc::new(InMemoryApiKeyStorage::new());
    ApiKeyService::new(storage, "test-secret")
}

#[cfg(feature = "server")]
pub(super) fn remote_config() -> crate::config::SecurityConfig {
    serde_json::from_value(serde_json::json!({
        "deployment_profile":"remote", "jwt_required":true, "jwt_algorithm":"HS256",
        "jwt_secret":"test-secret", "jwt_issuer":"key-issuer", "jwt_audience":"key-runtime",
        "jwt_validate_nbf":true, "settings_mutation_auth_required":false,
        "api_key_delegable_roles":["user"], "workspace_authorities":[{
            "issuer":"key-issuer", "subject":"key-owner", "tenant_id":"tenant-a", "workspace_id":"workspace-a"
        }]
    })).expect("remote key scenario configuration")
}

#[cfg(feature = "server")]
pub(super) async fn verified_remote_caller(config: &crate::config::SecurityConfig) -> UserContext {
    context_for(
        config,
        "key-owner",
        Some("tenant-a"),
        vec!["user".to_owned(), "reader".to_owned()],
    )
    .await
}

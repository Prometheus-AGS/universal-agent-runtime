use super::{
    authority::{AdmissionError, authorize_context},
    claims::{UserClaims, UserContext},
    credential_capture::AuthenticatedCredentialCapture,
    verifier::{VerificationError, verify_token},
};
use crate::{AppState, config::SecurityConfig};
use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

pub const SIDECAR_PRINCIPAL_HEADER: &str = "x-uar-principal";

fn principal_error(code: &'static str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        [(header::CONTENT_TYPE, "application/json")],
        format!(r#"{{"error":{{"code":"{code}"}}}}"#),
    )
        .into_response()
}

fn sidecar_principal(value: &str) -> Option<UserContext> {
    let valid = !value.eq_ignore_ascii_case("anonymous")
        && (1..=128).contains(&value.len())
        && value
            .as_bytes()
            .iter()
            .copied()
            .enumerate()
            .all(|(index, byte)| {
                byte.is_ascii_alphanumeric()
                    || (index > 0 && matches!(byte, b'.' | b'_' | b':' | b'-'))
            });
    valid.then(|| UserContext {
        host_authority: None, authority: None,
        user_id: value.to_owned(),
        tenant_id: None,
        claims: UserClaims {
            sub: value.to_owned(),
            name: None,
            roles: Some(vec!["host-session".to_owned()]),
            tenant_id: None,
            uar_instance_id: None,
            exp: usize::MAX,
        },
    })
}

fn anonymous_context() -> UserContext {
    UserContext {
        host_authority: None, authority: None,
        user_id: "anonymous".to_string(),
        tenant_id: None,
        claims: UserClaims {
            sub: "anonymous".to_string(),
            name: Some("Anonymous".to_string()),
            roles: Some(vec!["anonymous".to_string()]),
            tenant_id: None,
            uar_instance_id: None,
            // far-future expiry for internal placeholder context
            exp: usize::MAX,
        },
    }
}

async fn resolve_authenticated_context(
    config: &SecurityConfig,
    auth_header: Option<&str>,
) -> Result<(UserContext, AuthenticatedCredentialCapture), StatusCode> {
    let token = match auth_header {
        Some(header_val) if header_val.starts_with("Bearer ") => {
            &header_val[7..] // Strip "Bearer "
        }
        _ => {
            return if config.jwt_required {
                Err(StatusCode::UNAUTHORIZED)
            } else {
                Ok((anonymous_context(), AuthenticatedCredentialCapture::default()))
            };
        }
    };

    let verification = verify_token(config, token).await;

    match verification {
        Ok(principal) => {
            let claims = principal.claims;
            Ok((UserContext {
                host_authority: principal.host_authority, authority: principal.authority,
                user_id: principal.subject,
                tenant_id: principal.tenant_id,
                claims,
            }, AuthenticatedCredentialCapture::bearer(token)))
        }
        Err(VerificationError::ProviderConflict) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Err(VerificationError::IdentityPolicy(_)) => Err(StatusCode::SERVICE_UNAVAILABLE),
        Err(_) => {
            if config.jwt_required {
                Err(StatusCode::UNAUTHORIZED)
            } else {
                Ok((anonymous_context(), AuthenticatedCredentialCapture::default()))
            }
        }
    }
}

#[cfg(test)]
async fn resolve_user_context_with_config(
    config: &SecurityConfig,
    auth_header: Option<&str>,
) -> Result<UserContext, StatusCode> {
    resolve_authenticated_context(config, auth_header).await.map(|(context, _)| context)
}

#[cfg(test)]
fn resolve_user_context(
    jwt_required: bool,
    jwt_secret: &str,
    auth_header: Option<&str>,
) -> Result<UserContext, StatusCode> {
    let config = SecurityConfig {
        deployment_profile: Default::default(),
        jwt_algorithm: None,
        workspace_authorities: Vec::new(),
        api_key_delegable_roles: SecurityConfig::default_api_key_delegable_roles(),
        api_key_admin_principals: Vec::new(),
        trusted_host_principals: Vec::new(),
        jwt_required,
        jwt_secret: jwt_secret.to_owned().into(),
        jwks_url: None,
        jwt_issuer: None,
        jwt_audience: None,
        jwt_validate_nbf: true,
        settings_mutation_auth_required: true,
        settings_admin_key: Some("test-admin-key".to_owned().into()),
    };
    futures::executor::block_on(resolve_user_context_with_config(&config, auth_header))
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let asserted_principal = request.headers_mut().remove(SIDECAR_PRINCIPAL_HEADER);
    let host_authenticated = request
        .extensions()
        .get::<super::sidecar_guard::HostAuthenticated>()
        .copied();
    if asserted_principal.is_some() && host_authenticated.is_none() {
        return Err(principal_error("principal_header_not_allowed"));
    }
    let asserted_principal = asserted_principal
        .map(|value| {
            value
                .to_str()
                .ok()
                .and_then(sidecar_principal)
                .ok_or_else(|| principal_error("principal_invalid"))
        })
        .transpose()?;

    // Health probe endpoints must always be reachable without credentials
    // so that Kubernetes liveness and readiness probes pass.
    let path = request.uri().path();
    if matches!(path, "/health" | "/healthz" | "/readyz" | "/metrics") {
        return Ok(next.run(request).await);
    }

    if state.config.security.validate_identity_policy().is_err() {
        return Err(StatusCode::SERVICE_UNAVAILABLE.into_response());
    }
    if state.config.security.is_remote() && asserted_principal.is_some() {
        return Err(StatusCode::UNAUTHORIZED.into_response());
    }

    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    // The exchange handler owns credential verification for this exact ingress,
    // including its JSON-body key path. It never admits a run or installs context.
    if request.method() == axum::http::Method::POST
        && matches!(request.uri().path(), "/api/uar/auth/exchange" | "/api/auth/exchange")
    {
        return Ok(next.run(request).await);
    }

    let mut credential_capture = request.extensions()
        .get::<AuthenticatedCredentialCapture>().cloned().unwrap_or_default();
    let context = if let Some(mut principal) = asserted_principal {
        // The launch guard already authenticated and consumed the bearer secret.
        if let Some(proof) = host_authenticated { proof.bind_context(&mut principal); }
        principal
    } else if auth_header.is_none() {
        if let Some(key) = request.headers().get("x-api-key") {
            let key = key.to_str().map_err(|_| StatusCode::UNAUTHORIZED.into_response())?;
            let service = state.api_key_service.as_ref()
                .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;
            let context = service.validate_key(key).await
                .map_err(|error| {
                    if matches!(error.downcast_ref::<super::api_keys::ApiKeyAuthorityError>(),
                        Some(super::api_keys::ApiKeyAuthorityError::ReissueRequired)) {
                        (StatusCode::UNAUTHORIZED, axum::Json(serde_json::json!({
                            "error": {"code": "api_key_reissue_required"}
                        }))).into_response()
                    } else { StatusCode::UNAUTHORIZED.into_response() }
                })?
                .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;
            credential_capture.extend(AuthenticatedCredentialCapture::api_key(key));
            context
        } else {
            resolve_authenticated_context(&state.config.security, None).await
                .map_err(|status| status.into_response())?.0
        }
    } else {
        // A presented invalid bearer never falls through into key authority.
        let (context, capture) = resolve_authenticated_context(&state.config.security, auth_header).await
            .map_err(|status| status.into_response())?;
        credential_capture.extend(capture);
        context
    };

    let workspace = if state.config.security.is_remote() {
        request.headers().get("x-uar-workspace-id")
            .map(|value| value.to_str().map(str::trim))
            .transpose().map_err(|_| StatusCode::BAD_REQUEST.into_response())?
    } else { None };
    authorize_context(&state.config.security, Some(&context), workspace)
        .map_err(|error| match error {
            AdmissionError::Configuration => StatusCode::SERVICE_UNAVAILABLE,
            AdmissionError::Unauthenticated => StatusCode::UNAUTHORIZED,
            AdmissionError::Forbidden => StatusCode::FORBIDDEN,
        }.into_response())?;
    request.extensions_mut().insert(credential_capture);
    request.extensions_mut().insert(context);
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use std::{
        io::{self, Write},
        sync::{Arc, Mutex},
    };

    use super::*;
    use crate::uar::security::{
        jwt,
        verifier::test_support::{TestJwksServer, jwk, signed_token},
    };
    use jsonwebtoken::{EncodingKey, Header};
    use serde::Serialize;
    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Clone, Default)]
    struct LogBuffer(Arc<Mutex<Vec<u8>>>);

    struct LogWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for LogWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.0
                .lock()
                .expect("log buffer mutex must remain available")
                .extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for LogBuffer {
        type Writer = LogWriter;

        fn make_writer(&'a self) -> Self::Writer {
            LogWriter(Arc::clone(&self.0))
        }
    }

    impl LogBuffer {
        fn contents(&self) -> String {
            String::from_utf8(
                self.0
                    .lock()
                    .expect("log buffer mutex must remain available")
                    .clone(),
            )
            .expect("captured tracing output must be UTF-8")
        }
    }

    fn jwks_config(url: String, issuer: &str, audience: &str) -> SecurityConfig {
        SecurityConfig {
            deployment_profile: Default::default(),
            jwt_algorithm: None,
            workspace_authorities: Vec::new(),
            api_key_delegable_roles: SecurityConfig::default_api_key_delegable_roles(),
            api_key_admin_principals: Vec::new(),
            trusted_host_principals: Vec::new(),
            jwt_required: true,
            jwt_secret: "unused-for-jwks".to_owned().into(),
            jwks_url: Some(url),
            jwt_issuer: Some(issuer.to_owned()),
            jwt_audience: Some(audience.to_owned()),
            jwt_validate_nbf: true,
            settings_mutation_auth_required: true,
            settings_admin_key: Some("test-admin-key".to_owned().into()),
        }
    }

    fn shared_secret_config(issuer: Option<&str>, audience: Option<&str>) -> SecurityConfig {
        SecurityConfig {
            deployment_profile: Default::default(),
            jwt_algorithm: None,
            workspace_authorities: Vec::new(),
            api_key_delegable_roles: SecurityConfig::default_api_key_delegable_roles(),
            api_key_admin_principals: Vec::new(),
            trusted_host_principals: Vec::new(),
            jwt_required: true,
            jwt_secret: "claim-test-secret".to_owned().into(),
            jwks_url: None,
            jwt_issuer: issuer.map(str::to_owned),
            jwt_audience: audience.map(str::to_owned),
            jwt_validate_nbf: true,
            settings_mutation_auth_required: true,
            settings_admin_key: Some("test-admin-key".to_owned().into()),
        }
    }

    #[derive(Serialize)]
    struct RegisteredClaims<'a> {
        sub: &'a str,
        exp: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        iss: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        aud: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        nbf: Option<usize>,
    }

    fn shared_secret_token(
        issuer: Option<&str>,
        audience: Option<&str>,
        not_before: Option<usize>,
    ) -> String {
        jwt::encode(
            &Header::default(),
            &RegisteredClaims {
                sub: "claim-user",
                exp: usize::MAX,
                iss: issuer,
                aud: audience,
                nbf: not_before,
            },
            &EncodingKey::from_secret(b"claim-test-secret"),
        )
        .expect("claim test token must encode")
    }

    #[test]
    fn test_resolve_user_context_anonymous_when_jwt_disabled_and_no_header() {
        let ctx = resolve_user_context(false, "secret", None).expect("expected context");
        assert_eq!(ctx.user_id, "anonymous");
    }

    #[test]
    fn test_resolve_user_context_anonymous_when_jwt_disabled_and_invalid_header() {
        let ctx = resolve_user_context(false, "secret", Some("Bearer invalid.token"))
            .expect("expected anonymous fallback context");
        assert_eq!(ctx.user_id, "anonymous");
    }

    #[test]
    fn test_resolve_user_context_unauthorized_when_jwt_required_and_no_header() {
        let err = resolve_user_context(true, "secret", None).expect_err("expected unauthorized");
        assert_eq!(err, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_resolve_user_context_valid_token() {
        let claims = UserClaims {
            sub: "user-123".to_string(),
            name: Some("Test User".to_string()),
            roles: Some(vec!["user".to_string()]),
            tenant_id: None,
            uar_instance_id: None,
            exp: usize::MAX,
        };
        let token = jwt::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("secret".as_bytes()),
        )
        .expect("token encode should succeed");
        let header_value = format!("Bearer {token}");

        let ctx =
            resolve_user_context(true, "secret", Some(&header_value)).expect("expected context");
        assert_eq!(ctx.user_id, "user-123");
    }

    #[test]
    fn test_resolve_user_context_rejects_token_signed_with_wrong_secret() {
        let claims = UserClaims {
            sub: "user-123".to_string(),
            name: Some("Test User".to_string()),
            roles: Some(vec!["user".to_string()]),
            tenant_id: None,
            uar_instance_id: None,
            exp: usize::MAX,
        };
        let token = jwt::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("wrong-secret".as_bytes()),
        )
        .expect("token encode should succeed");
        let header_value = format!("Bearer {token}");

        let err = resolve_user_context(true, "secret", Some(&header_value))
            .expect_err("expected invalid signature to be rejected");
        assert_eq!(err, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn jwks_token_authenticates_through_middleware_resolution() {
        let server = TestJwksServer::start(vec![jwk("middleware-key")]).await;
        let config = jwks_config(server.url.clone(), "issuer", "audience");
        let token = signed_token("middleware-key", "issuer", "audience");

        let context = resolve_user_context_with_config(&config, Some(&format!("Bearer {token}")))
            .await
            .expect("valid JWKS token must authenticate");
        assert_eq!(context.user_id, "user-123");
    }

    #[tokio::test]
    async fn jwks_wrong_audience_maps_to_unauthorized() {
        let server = TestJwksServer::start(vec![jwk("claim-key")]).await;
        let config = jwks_config(server.url.clone(), "issuer", "audience");

        let wrong_audience = signed_token("claim-key", "issuer", "other-audience");
        let audience_error =
            resolve_user_context_with_config(&config, Some(&format!("Bearer {wrong_audience}")))
                .await
                .expect_err("wrong audience must return 401");
        assert_eq!(audience_error, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn jwks_wrong_issuer_maps_to_unauthorized() {
        let server = TestJwksServer::start(vec![jwk("claim-key")]).await;
        let config = jwks_config(server.url.clone(), "issuer", "audience");

        let wrong_issuer = signed_token("claim-key", "other-issuer", "audience");
        let issuer_error =
            resolve_user_context_with_config(&config, Some(&format!("Bearer {wrong_issuer}")))
                .await
                .expect_err("wrong issuer must return 401");
        assert_eq!(issuer_error, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn shared_secret_issuer_and_audience_are_required_when_configured() {
        let config = shared_secret_config(Some("expected-issuer"), Some("expected-audience"));
        let valid = shared_secret_token(Some("expected-issuer"), Some("expected-audience"), None);
        let context = resolve_user_context_with_config(&config, Some(&format!("Bearer {valid}")))
            .await
            .expect("matching issuer and audience must authenticate");
        assert_eq!(context.user_id, "claim-user");

        for token in [
            shared_secret_token(None, Some("expected-audience"), None),
            shared_secret_token(Some("expected-issuer"), None, None),
            shared_secret_token(Some("wrong-issuer"), Some("expected-audience"), None),
            shared_secret_token(Some("expected-issuer"), Some("wrong-audience"), None),
        ] {
            let error = resolve_user_context_with_config(&config, Some(&format!("Bearer {token}")))
                .await
                .expect_err("missing or mismatched registered claims must fail closed");
            assert_eq!(error, StatusCode::UNAUTHORIZED);
        }
    }

    #[tokio::test]
    async fn not_before_is_enforced_when_enabled_and_optional_when_disabled() {
        let future = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock must follow the Unix epoch")
            .as_secs()
            .saturating_add(3_600) as usize;
        let token = shared_secret_token(None, None, Some(future));
        let enabled = shared_secret_config(None, None);
        let error = resolve_user_context_with_config(&enabled, Some(&format!("Bearer {token}")))
            .await
            .expect_err("a future nbf must fail closed when validation is enabled");
        assert_eq!(error, StatusCode::UNAUTHORIZED);

        let mut disabled = enabled;
        disabled.jwt_validate_nbf = false;
        let context = resolve_user_context_with_config(&disabled, Some(&format!("Bearer {token}")))
            .await
            .expect("operators may explicitly disable nbf validation");
        assert_eq!(context.user_id, "claim-user");
    }

    #[tokio::test]
    async fn jwks_unknown_kid_maps_to_unauthorized() {
        let server = TestJwksServer::start(vec![jwk("known-key")]).await;
        let config = jwks_config(server.url.clone(), "issuer", "audience");
        let token = signed_token("missing-key", "issuer", "audience");

        let error = resolve_user_context_with_config(&config, Some(&format!("Bearer {token}")))
            .await
            .expect_err("unknown kid must return 401");
        assert_eq!(error, StatusCode::UNAUTHORIZED);
        assert_eq!(server.request_count(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn unreachable_jwks_with_no_cache_fails_closed() {
        let logs = LogBuffer::default();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::ERROR)
            .without_time()
            .with_ansi(false)
            .with_writer(logs.clone())
            .finish();
        let _subscriber_guard = tracing::subscriber::set_default(subscriber);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("temporary listener must bind");
        let address = listener
            .local_addr()
            .expect("temporary listener must have an address");
        drop(listener);

        let config = jwks_config(format!("http://{address}/jwks"), "issuer", "audience");
        let token = signed_token("unreachable-key", "issuer", "audience");
        let error = resolve_user_context_with_config(&config, Some(&format!("Bearer {token}")))
            .await
            .expect_err("unreachable JWKS must fail closed");
        assert_eq!(error, StatusCode::UNAUTHORIZED);
        let output = logs.contents();
        assert!(
            output.contains("ERROR"),
            "expected error-level log: {output}"
        );
        assert!(
            output.contains("JWKS refresh failed"),
            "expected JWKS failure log: {output}"
        );
    }
}

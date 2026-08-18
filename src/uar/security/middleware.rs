use super::{
    claims::{UserClaims, UserContext},
    verifier::{VerificationError, verify_token},
};
use crate::{AppState, config::SecurityConfig};
use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};

fn anonymous_context() -> UserContext {
    UserContext {
        user_id: "anonymous".to_string(),
        tenant_id: None,
        claims: UserClaims {
            sub: "anonymous".to_string(),
            name: Some("Anonymous".to_string()),
            roles: Some(vec!["anonymous".to_string()]),
            tenant_id: None,
            // far-future expiry for internal placeholder context
            exp: usize::MAX,
        },
    }
}

async fn resolve_user_context_with_config(
    config: &SecurityConfig,
    auth_header: Option<&str>,
) -> Result<UserContext, StatusCode> {
    let token = match auth_header {
        Some(header_val) if header_val.starts_with("Bearer ") => {
            &header_val[7..] // Strip "Bearer "
        }
        _ => {
            return if config.jwt_required {
                Err(StatusCode::UNAUTHORIZED)
            } else {
                Ok(anonymous_context())
            };
        }
    };

    let verification = verify_token(config, token).await;

    match verification {
        Ok(principal) => {
            let claims = principal.claims;
            Ok(UserContext {
                user_id: principal.subject,
                tenant_id: principal.tenant_id,
                claims,
            })
        }
        Err(VerificationError::ProviderConflict) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Err(_) => {
            if config.jwt_required {
                Err(StatusCode::UNAUTHORIZED)
            } else {
                Ok(anonymous_context())
            }
        }
    }
}

#[cfg(test)]
fn resolve_user_context(
    jwt_required: bool,
    jwt_secret: &str,
    auth_header: Option<&str>,
) -> Result<UserContext, StatusCode> {
    let config = SecurityConfig {
        jwt_required,
        jwt_secret: jwt_secret.to_owned().into(),
        jwks_url: None,
        jwt_issuer: None,
        jwt_audience: None,
        settings_mutation_auth_required: true,
    };
    futures::executor::block_on(resolve_user_context_with_config(&config, auth_header))
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Health probe endpoints must always be reachable without credentials
    // so that Kubernetes liveness and readiness probes pass.
    let path = request.uri().path();
    if matches!(path, "/health" | "/healthz" | "/readyz" | "/metrics") {
        return Ok(next.run(request).await);
    }

    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    // Try JWT first
    let mut context = resolve_user_context_with_config(&state.config.security, auth_header).await?;

    // If still anonymous, try X-API-Key header
    if context.user_id == "anonymous" {
        if let Some(api_key) = request
            .headers()
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
        {
            if let Some(api_key_service) = &state.api_key_service {
                match api_key_service.validate_key(api_key).await {
                    Ok(Some(claims)) => {
                        context = UserContext {
                            user_id: claims.sub.clone(),
                            tenant_id: None,
                            claims,
                        };
                    }
                    Ok(None) => {
                        // Invalid key — if JWT is required, reject; otherwise stay anonymous
                        if state.config.security.jwt_required {
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                    }
                    Err(_) => {
                        if state.config.security.jwt_required {
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                    }
                }
            }
        } else if state.config.security.jwt_required {
            // No auth at all and JWT required
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

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
            jwt_required: true,
            jwt_secret: "unused-for-jwks".to_owned().into(),
            jwks_url: Some(url),
            jwt_issuer: Some(issuer.to_owned()),
            jwt_audience: Some(audience.to_owned()),
            settings_mutation_auth_required: true,
        }
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

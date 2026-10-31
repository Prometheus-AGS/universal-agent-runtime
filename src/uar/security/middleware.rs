use super::{
    authority::{AdmissionError, authorize_context},
    claims::{UserClaims, UserContext},
    credential_capture::AuthenticatedCredentialCapture,
    verifier::{VerificationError, verify_token},
};
use crate::{AppState, config::SecurityConfig};
use axum::{
    extract::{ConnectInfo, Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;

pub const SIDECAR_PRINCIPAL_HEADER: &str = "x-uar-principal";

/// Subject of the single owner that credential-less loopback callers act as
/// in a trusted-local installation with optional JWT.
pub const LOCAL_LOOPBACK_SUBJECT: &str = "local-user";

fn principal_error(code: &'static str) -> Response {
    principal_error_with_status(StatusCode::BAD_REQUEST, code)
}

fn principal_error_with_status(status: StatusCode, code: &'static str) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        format!(r#"{{"error":{{"code":"{code}"}}}}"#),
    )
        .into_response()
}

/// Resolve the host-asserted principal for one request.
///
/// Without a launch binding this is the shared-sidecar rule: `x-uar-principal`
/// is honored only after launch-token authentication. A launch-bound sidecar
/// serves exactly its bound subject. An absent header uses that subject, an
/// equal header is accepted, and a different subject is refused with 403
/// `principal_mismatch`. The caller already authenticated with the launch
/// token, so the refusal is an authorization failure, not a malformed or
/// unauthenticated request.
fn resolve_host_principal(
    asserted: Option<&axum::http::HeaderValue>,
    host_authenticated: Option<super::sidecar_guard::HostAuthenticated>,
    bound: Option<&super::sidecar_guard::HostBoundPrincipal>,
) -> Result<Option<UserContext>, Response> {
    if asserted.is_some() && host_authenticated.is_none() {
        return Err(principal_error("principal_header_not_allowed"));
    }
    let asserted = asserted
        .map(|value| {
            value
                .to_str()
                .ok()
                .and_then(sidecar_principal)
                .ok_or_else(|| principal_error("principal_invalid"))
        })
        .transpose()?;
    let Some(bound) = bound.filter(|_| host_authenticated.is_some()) else {
        return Ok(asserted);
    };
    match asserted {
        None => sidecar_principal(bound.subject())
            .map(Some)
            .ok_or_else(|| principal_error("principal_invalid")),
        Some(principal) if principal.user_id == bound.subject() => Ok(Some(principal)),
        Some(_) => Err(principal_error_with_status(StatusCode::FORBIDDEN, "principal_mismatch")),
    }
}

fn sidecar_principal(value: &str) -> Option<UserContext> {
    let valid = super::sidecar_guard::is_valid_host_principal(value);
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

    // A presented bearer never falls back to anonymous, whatever
    // `jwt_required` says: failure returns no context at all.
    let principal = verify_token(config, token).await
        .map_err(|error| verification_failure_status(&error))?;
    let claims = principal.claims;
    Ok((UserContext {
        host_authority: principal.host_authority, authority: principal.authority,
        user_id: principal.subject,
        tenant_id: principal.tenant_id,
        claims,
    }, AuthenticatedCredentialCapture::bearer(token)))
}

/// Deterministic status for a presented bearer that did not verify.
///
/// Credential defects are 401. Verification that could not run is 503: the
/// configured JWKS source failed or timed out, or the identity policy is
/// misconfigured. A process JWT-provider conflict is 500. The response never
/// carries the error text, the token, or key material.
fn verification_failure_status(error: &VerificationError) -> StatusCode {
    match error {
        VerificationError::ProviderConflict => StatusCode::INTERNAL_SERVER_ERROR,
        VerificationError::IdentityPolicy(_) => StatusCode::SERVICE_UNAVAILABLE,
        VerificationError::Jwks(_) if error.is_key_source_unavailable() => {
            StatusCode::SERVICE_UNAVAILABLE
        }
        VerificationError::Token(_)
        | VerificationError::MissingKeyId
        | VerificationError::UnsupportedAlgorithm(_)
        | VerificationError::Jwks(_)
        | VerificationError::MissingRemoteIdentity
        | VerificationError::AlgorithmMismatch { .. }
        | VerificationError::UnsupportedPresentation
        | VerificationError::UnsupportedTokenType => StatusCode::UNAUTHORIZED,
    }
}

/// The trusted-local owner for a request that carries no credential at all,
/// arrives over a loopback TCP connection, and reaches an installation with
/// `local_loopback_owner: true`, `deployment_profile: trusted_local` and
/// `jwt_required: false`. With the flag off (the default) such a request keeps
/// the anonymous context exactly as before.
///
/// The peer address comes only from the accepted socket (`ConnectInfo`), never
/// from `X-Forwarded-For` or any other header. A request without socket
/// information (a non-TCP listener, an in-process call) is treated as
/// non-loopback and keeps the anonymous behavior. IPv4-mapped IPv6 loopback
/// (`::ffff:127.0.0.0/104`) counts as IPv4 loopback. Headers can only refuse:
/// see [`names_loopback_origin`].
fn trusted_local_loopback_context(
    config: &SecurityConfig,
    request: &Request,
) -> Option<UserContext> {
    let loopback = request.extensions().get::<ConnectInfo<SocketAddr>>()
        .is_some_and(|ConnectInfo(peer)| peer.ip().to_canonical().is_loopback());
    let credential_less = !request.headers().contains_key(header::AUTHORIZATION)
        && !request.headers().contains_key("x-api-key");
    let enabled = config.local_loopback_owner && !config.jwt_required && !config.is_remote();
    (enabled && loopback && credential_less && names_loopback_origin(request))
        .then(|| UserContext::trusted_local_loopback(LOCAL_LOOPBACK_SUBJECT))
}

/// Whether a `host[:port]` authority names the loopback interface.
fn is_loopback_authority(authority: &str) -> bool {
    authority.parse::<axum::http::uri::Authority>().is_ok_and(|authority| {
        let host = authority.host();
        host.eq_ignore_ascii_case("localhost")
            || host
                .trim_start_matches('[')
                .trim_end_matches(']')
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.to_canonical().is_loopback())
    })
}

/// The request names a loopback `Host` (or HTTP/2 authority) and, when a
/// browser sent an `Origin`, that origin is itself a loopback page.
///
/// A loopback port is reachable from any web page: a DNS-rebinding page
/// arrives under its own host name, and a cross-origin page carries its own
/// `Origin`. Neither may become the local owner, so either refuses here, as
/// the sidecar launch guard does. Repeated `Host` or `Origin` headers refuse.
fn names_loopback_origin(request: &Request) -> bool {
    let mut hosts = request.headers().get_all(header::HOST).iter();
    let authority = match (hosts.next(), hosts.next()) {
        (Some(host), None) => host.to_str().ok(),
        (None, _) => request.uri().authority().map(axum::http::uri::Authority::as_str),
        (Some(_), Some(_)) => None,
    };
    let mut origins = request.headers().get_all(header::ORIGIN).iter();
    let origin_is_loopback = match (origins.next(), origins.next()) {
        (None, _) => true,
        (Some(origin), None) => origin
            .to_str()
            .ok()
            .and_then(|origin| origin.parse::<axum::http::Uri>().ok())
            .is_some_and(|origin| {
                matches!(origin.scheme_str(), Some("http" | "https"))
                    && origin.authority().is_some_and(|value| is_loopback_authority(value.as_str()))
            }),
        (Some(_), Some(_)) => false,
    };
    authority.is_some_and(is_loopback_authority) && origin_is_loopback
}

/// Verify bearer identity for a trusted non-HTTP host adapter using the same
/// verifier as HTTP ingress. No caller assertions can create the returned proof.
///
/// # Errors
/// Rejects invalid, anonymous and incomplete configured identities.
pub async fn authenticate_bearer_context(config: &SecurityConfig, token: &str) -> Result<UserContext, StatusCode> {
    let authorization = format!("Bearer {token}");
    let (context, _) = resolve_authenticated_context(config, Some(&authorization)).await?;
    if context.verified_identity().is_none() { return Err(StatusCode::UNAUTHORIZED); }
    // This identity-only adapter has no authoritative storage. Production
    // HTTP/gRPC use the service-bound admission paths below/alongside it.
    if context.verified_identity().is_some_and(|identity| identity.is_api_key_credential()) {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    Ok(context)
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
        provider_credentials: Default::default(),
        deployment_profile: Default::default(),
        jwt_algorithm: None,
        workspace_authorities: Vec::new(),
        skill_required_roles: Default::default(),
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
        local_loopback_owner: false,
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
    let asserted_principal = resolve_host_principal(
        asserted_principal.as_ref(),
        host_authenticated,
        request.extensions().get::<super::sidecar_guard::HostBoundPrincipal>(),
    )?;

    // The outer grant guard captured this identity at issuance. A grant never
    // becomes a host assertion, nor may another caller header replace its owner.
    if let Some(delegated) = request
        .extensions()
        .get::<super::delegation_grants::DelegationAuthenticated>()
        .cloned()
    {
        require_current_key_authority(&state, &delegated.0).await?;
        request.extensions_mut().insert(delegated.0);
        return Ok(next.run(request).await);
    }

    // Only discovery GETs are anonymous; task endpoints still pass admission.
    if request.method() == axum::http::Method::GET
        && matches!(request.uri().path(), "/.well-known/agent-card.json" | "/.well-known/agent.json")
    {
        return Ok(next.run(request).await);
    }

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
                    } else if error.downcast_ref::<super::api_keys::ApiKeyStorageError>().is_some() {
                        StatusCode::SERVICE_UNAVAILABLE.into_response()
                    } else { StatusCode::UNAUTHORIZED.into_response() }
                })?
                .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;
            credential_capture.extend(AuthenticatedCredentialCapture::api_key(key));
            context
        } else if let Some(local) =
            trusted_local_loopback_context(&state.config.security, &request)
        {
            local
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

    require_current_key_authority(&state, &context).await?;

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

async fn require_current_key_authority(state: &AppState, context: &UserContext) -> Result<(), Response> {
    if context.verified_identity().is_some_and(|identity| identity.is_api_key_credential()) {
        let service = state.api_key_service.as_ref()
            .ok_or_else(|| StatusCode::SERVICE_UNAVAILABLE.into_response())?;
        let valid = service.validate_admission(context).await.map_err(|error| {
            if error.downcast_ref::<super::api_keys::ApiKeyStorageError>().is_some() {
                StatusCode::SERVICE_UNAVAILABLE.into_response()
            } else {
                (StatusCode::UNAUTHORIZED, axum::Json(serde_json::json!({
                    "error": {"code": "api_key_reissue_required"}
                }))).into_response()
            }
        })?;
        if !valid { return Err(StatusCode::UNAUTHORIZED.into_response()); }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        io::{self, Write},
        sync::{Arc, Mutex},
    };

    use super::*;
    use crate::uar::runtime::actor::messages::ActorOwner;
    use crate::uar::security::{
        claims::CredentialPrincipal,
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
            provider_credentials: Default::default(),
            deployment_profile: Default::default(),
            jwt_algorithm: None,
            workspace_authorities: Vec::new(),
            skill_required_roles: Default::default(),
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
            local_loopback_owner: false,
        }
    }

    fn shared_secret_config(issuer: Option<&str>, audience: Option<&str>) -> SecurityConfig {
        SecurityConfig {
            provider_credentials: Default::default(),
            deployment_profile: Default::default(),
            jwt_algorithm: None,
            workspace_authorities: Vec::new(),
            skill_required_roles: Default::default(),
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
            local_loopback_owner: false,
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

    /// #422 decision A: a presented bearer that fails verification is 401 even
    /// when JWT is optional. It never degrades to the anonymous context.
    #[test]
    fn j10_invalid_bearer_is_unauthorized_when_jwt_optional() {
        let err = resolve_user_context(false, "secret", Some("Bearer invalid.token"))
            .expect_err("an invalid presented bearer must not fall back to anonymous");
        assert_eq!(err, StatusCode::UNAUTHORIZED);
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
        // #422 J07/J09: verification could not run, which is not a bad credential.
        assert_eq!(error, StatusCode::SERVICE_UNAVAILABLE);
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

    // ── #422 status mapping and #392 trusted-local loopback identity ───────

    fn unreachable_jwks_url() -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("temporary listener must bind");
        let address = listener.local_addr().expect("temporary listener must have an address");
        drop(listener);
        format!("http://{address}/jwks")
    }

    fn optional_jwt_config() -> SecurityConfig {
        let mut config = shared_secret_config(None, None);
        config.jwt_required = false;
        config
    }

    /// Optional JWT with the opt-in loopback owner enabled.
    fn local_owner_config() -> SecurityConfig {
        let mut config = optional_jwt_config();
        config.local_loopback_owner = true;
        config
    }

    /// A request with a loopback `Host` unless `headers` supplies its own.
    fn request_from(peer: Option<&str>, headers: &[(&str, &str)]) -> Request {
        let mut builder = axum::http::Request::builder().uri("/api/uar/sessions");
        if !headers.iter().any(|(name, _)| name.eq_ignore_ascii_case("host")) {
            builder = builder.header("host", "127.0.0.1:8088");
        }
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        let mut request = builder
            .body(axum::body::Body::empty())
            .expect("fixture request must build");
        if let Some(peer) = peer {
            let address: std::net::SocketAddr = peer.parse().expect("fixture peer must parse");
            request.extensions_mut().insert(ConnectInfo(address));
        }
        request
    }

    #[tokio::test]
    async fn j10_invalid_bearers_are_unauthorized_never_anonymous_when_jwt_optional() {
        let config = optional_jwt_config();
        let expired = jwt::encode(
            &Header::default(),
            &serde_json::json!({"sub": "claim-user", "exp": 1}),
            &EncodingKey::from_secret(b"claim-test-secret"),
        )
        .expect("expired fixture must encode");
        let wrong_key = jwt::encode(
            &Header::default(),
            &serde_json::json!({"sub": "claim-user", "exp": usize::MAX}),
            &EncodingKey::from_secret(b"another-secret"),
        )
        .expect("wrong-key fixture must encode");
        for token in [
            "invalid.token".to_owned(),
            "eyJhbGciOiJub25lIn0.eyJzdWIiOiJ4In0.".to_owned(),
            expired,
            wrong_key,
            String::new(),
        ] {
            let error = resolve_user_context_with_config(&config, Some(&format!("Bearer {token}")))
                .await
                .expect_err("an invalid presented bearer must be refused");
            assert_eq!(error, StatusCode::UNAUTHORIZED, "{token}");
        }
        let valid = shared_secret_token(None, None, None);
        let context = resolve_user_context_with_config(&config, Some(&format!("Bearer {valid}")))
            .await
            .expect("a valid bearer still authenticates when JWT is optional");
        assert_eq!(context.user_id, "claim-user");
    }

    #[tokio::test]
    async fn j07_unreachable_jwks_is_service_unavailable_when_jwt_optional() {
        let mut config = jwks_config(unreachable_jwks_url(), "issuer", "audience");
        config.jwt_required = false;
        let token = signed_token("unreachable-optional-key", "issuer", "audience");
        for _ in 0..2 {
            let error = resolve_user_context_with_config(&config, Some(&format!("Bearer {token}")))
                .await
                .expect_err("unavailable trust material must never yield a context");
            assert_eq!(error, StatusCode::SERVICE_UNAVAILABLE);
        }
    }

    #[tokio::test]
    async fn j09_misconfigured_identity_policy_is_service_unavailable_regardless_of_jwt_required() {
        let token = shared_secret_token(None, None, None);
        for jwt_required in [true, false] {
            let mut config = shared_secret_config(None, None);
            config.jwt_required = jwt_required;
            // Contradictory: an asymmetric algorithm with no JWKS source.
            config.jwt_algorithm = Some(crate::config::JwtAlgorithm::RS256);
            for _ in 0..2 {
                let error =
                    resolve_user_context_with_config(&config, Some(&format!("Bearer {token}")))
                        .await
                        .expect_err("contradictory trust configuration must fail closed");
                assert_eq!(error, StatusCode::SERVICE_UNAVAILABLE);
            }
        }
    }

    #[tokio::test]
    async fn j09_repeated_equivalent_credential_failures_map_consistently() {
        let server = TestJwksServer::start(vec![jwk("repeat-key")]).await;
        let config = jwks_config(server.url.clone(), "issuer", "audience");
        let wrong_audience = signed_token("repeat-key", "issuer", "other-audience");
        let unknown_kid = signed_token("repeat-missing", "issuer", "audience");
        for token in [wrong_audience, unknown_kid, "x.y.z".to_owned()] {
            for _ in 0..3 {
                let error =
                    resolve_user_context_with_config(&config, Some(&format!("Bearer {token}")))
                        .await
                        .expect_err("an invalid credential must be refused");
                assert_eq!(error, StatusCode::UNAUTHORIZED);
            }
        }
    }

    #[test]
    fn j10_loopback_without_credential_gets_one_local_host_owner_for_all_stores() {
        let config = local_owner_config();
        let sidecar = {
            let mut context = sidecar_principal(LOCAL_LOOPBACK_SUBJECT)
                .expect("the loopback subject is a valid host principal");
            crate::uar::security::sidecar_guard::HostAuthenticated::for_test()
                .bind_context(&mut context);
            context
        };
        let sidecar_key = sidecar.session_owner_key().expect("sidecar owner key");
        let loopback_requests = [
            request_from(Some("127.0.0.1:50000"), &[]),
            request_from(Some("127.8.9.10:50000"), &[("host", "localhost:8088")]),
            request_from(Some("[::1]:50000"), &[("host", "[::1]:8088")]),
            request_from(Some("[::ffff:127.0.0.1]:50000"), &[("origin", "http://localhost:5173")]),
        ];
        for request in &loopback_requests {
            let context = trusted_local_loopback_context(&config, request)
                .expect("a credential-less loopback caller is the local owner");
            let principal = CredentialPrincipal::from_verified_context(&context)
                .expect("the loopback owner is a credential principal");
            assert_eq!(principal.issuer_kind(), "local-host");
            assert_eq!(principal.issuer(), "urn:uar:local:host");
            assert_eq!(principal.tenant_id(), None);
            assert_eq!(principal.subject(), LOCAL_LOOPBACK_SUBJECT);
            // No host-administration authority and no role-derived privilege.
            assert!(context.trusted_host_id().is_none());
            assert!(
                context
                    .verified_identity()
                    .is_some_and(|identity| identity.roles().is_empty())
            );

            let history = context.session_owner_key().expect("history owner");
            let memory = crate::uar::memory::scopes::user_id_for_query(&context)
                .expect("memory owner");
            let tools = ActorOwner::from_verified_context(&context)
                .expect("tool/run owner")
                .session_owner_key()
                .expect("tool/run owner key");
            assert_eq!(history, principal.storage_key());
            assert_eq!(memory, history);
            assert_eq!(tools, history);
            assert_eq!(history, sidecar_key, "loopback and sidecar launch share one owner");
        }
    }

    #[tokio::test]
    async fn j10_non_loopback_credentialed_or_strict_requests_get_no_local_owner() {
        let optional = local_owner_config();
        for peer in [
            "192.168.1.10:50000",
            "10.0.0.1:50000",
            "[2001:db8::1]:50000",
            "0.0.0.0:50000",
        ] {
            let request = request_from(Some(peer), &[]);
            assert!(trusted_local_loopback_context(&optional, &request).is_none(), "{peer}");
        }
        // Without socket information the caller is treated as non-loopback.
        assert!(trusted_local_loopback_context(&optional, &request_from(None, &[])).is_none());
        // Forwarding headers never establish the peer address.
        for header in ["x-forwarded-for", "x-real-ip", "forwarded"] {
            let request = request_from(Some("203.0.113.7:50000"), &[(header, "127.0.0.1")]);
            assert!(trusted_local_loopback_context(&optional, &request).is_none(), "{header}");
        }
        // A DNS-rebinding or cross-origin page reaches the loopback socket too.
        for headers in [
            vec![("host", "rebound.example:8088")],
            vec![("origin", "https://attacker.example")],
            vec![("origin", "null")],
            vec![("origin", "http://localhost:5173"), ("origin", "https://attacker.example")],
            vec![("host", "127.0.0.1:8088"), ("host", "rebound.example:8088")],
        ] {
            let request = request_from(Some("127.0.0.1:50000"), &headers);
            assert!(trusted_local_loopback_context(&optional, &request).is_none(), "{headers:?}");
        }
        // Any presented credential is verified, never replaced by the local owner.
        for header in [
            ("authorization", "Bearer invalid.token"),
            ("authorization", "Basic dXNlcjpwYXNz"),
            ("x-api-key", "not-a-key"),
        ] {
            let request = request_from(Some("127.0.0.1:50000"), &[header]);
            assert!(trusted_local_loopback_context(&optional, &request).is_none(), "{header:?}");
        }
        let mut strict = local_owner_config();
        strict.jwt_required = true;
        let loopback = || request_from(Some("127.0.0.1:50000"), &[]);
        assert!(trusted_local_loopback_context(&strict, &loopback()).is_none());
        let mut remote = local_owner_config();
        remote.deployment_profile = crate::config::DeploymentProfile::Remote;
        assert!(trusted_local_loopback_context(&remote, &loopback()).is_none());

        // The non-loopback, no-credential caller keeps today's anonymous context.
        let anonymous = resolve_user_context_with_config(&optional, None)
            .await
            .expect("optional JWT still admits an anonymous non-loopback caller");
        assert_eq!(anonymous.user_id, "anonymous");
        assert_eq!(anonymous.session_owner_key().expect("anonymous history owner"), "anonymous");
    }

    #[tokio::test]
    async fn j10_loopback_without_credential_stays_anonymous_when_local_loopback_owner_is_off() {
        let off = optional_jwt_config();
        assert!(!off.local_loopback_owner, "the loopback owner must default off");
        let defaulted: SecurityConfig = serde_json::from_value(serde_json::json!({
            "jwt_required": false, "jwt_secret": "claim-test-secret",
            "settings_mutation_auth_required": false
        }))
        .expect("minimal optional-JWT config must deserialize");
        assert!(!defaulted.local_loopback_owner, "an absent key must mean off");
        for config in [&off, &defaulted] {
            for request in [
                request_from(Some("127.0.0.1:50000"), &[]),
                request_from(Some("[::1]:50000"), &[("host", "[::1]:8088")]),
                request_from(Some("127.0.0.1:50000"), &[("origin", "http://localhost:5173")]),
            ] {
                assert!(trusted_local_loopback_context(config, &request).is_none());
            }
            let context = resolve_user_context_with_config(config, None)
                .await
                .expect("optional JWT admits the credential-less caller");
            assert_eq!(context.user_id, "anonymous");
            assert_eq!(context.session_owner_key().expect("anonymous owner"), "anonymous");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn j10_verification_paths_leak_no_token_or_key_material() {
        let logs = LogBuffer::default();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .without_time()
            .with_ansi(false)
            .with_writer(logs.clone())
            .finish();
        let _subscriber_guard = tracing::subscriber::set_default(subscriber);

        let shared = shared_secret_config(None, None);
        let valid = shared_secret_token(None, None, None);
        let (context, capture) =
            resolve_authenticated_context(&shared, Some(&format!("Bearer {valid}")))
                .await
                .expect("valid bearer must authenticate");
        let forged = format!("{valid}tampered");
        let invalid = resolve_authenticated_context(&shared, Some(&format!("Bearer {forged}")))
            .await
            .expect_err("tampered bearer must be refused");
        assert_eq!(invalid, StatusCode::UNAUTHORIZED);
        let unavailable_token = signed_token("redaction-key", "issuer", "audience");
        let unavailable = resolve_authenticated_context(
            &jwks_config(unreachable_jwks_url(), "issuer", "audience"),
            Some(&format!("Bearer {unavailable_token}")),
        )
        .await
        .expect_err("unavailable verifier must be refused");
        assert_eq!(unavailable, StatusCode::SERVICE_UNAVAILABLE);
        let verifier_error = verify_token(&shared, &forged)
            .await
            .err()
            .expect("tampered token must not verify")
            .to_string();

        let scrubbed = capture
            .clone()
            .into_scrubber()
            .scrub(&format!("upstream echoed Authorization: Bearer {valid}"));
        let observed = [
            logs.contents(),
            format!("{context:?}"),
            format!("{capture:?}"),
            verifier_error,
            scrubbed,
        ];
        let signature = |token: &str| token.rsplit('.').next().unwrap_or_default().to_owned();
        for secret in [
            valid.clone(),
            forged.clone(),
            unavailable_token.clone(),
            signature(&valid),
            signature(&unavailable_token),
            "claim-test-secret".to_owned(),
        ] {
            assert!(!secret.is_empty());
            for text in &observed {
                assert!(!text.contains(&secret), "credential material leaked into: {text}");
            }
        }
    }

    mod sidecar_bound {
        use super::super::{UserContext, resolve_host_principal};
        use crate::uar::security::{
            claims::CredentialPrincipal,
            sidecar_guard::{HostAuthenticated, HostBoundPrincipal},
        };
        use axum::{
            http::{HeaderValue, StatusCode},
            response::Response,
        };

        fn header(value: &str) -> HeaderValue {
            HeaderValue::from_str(value).expect("fixture header must be valid")
        }

        async fn error_code(response: Response) -> (StatusCode, String) {
            let status = response.status();
            let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("error body must be readable");
            (status, String::from_utf8(bytes.to_vec()).expect("error body must be UTF-8"))
        }

        /// Apply the same host proof the middleware applies to every outcome.
        fn bound_context(mut context: UserContext) -> UserContext {
            HostAuthenticated::for_test().bind_context(&mut context);
            context
        }

        #[test]
        fn sidecar_bound_absent_header_binds_to_bound_subject() {
            let bound = HostBoundPrincipal::for_test("alice");
            let context = resolve_host_principal(None, Some(HostAuthenticated::for_test()), Some(&bound))
                .expect("bound subject must resolve")
                .expect("bound mode must produce a principal");
            let context = bound_context(context);
            assert_eq!(context.user_id, "alice");
            assert_eq!(context.claims.sub, "alice");
            assert_eq!(context.claims.roles, Some(vec!["host-session".to_owned()]));
            assert_eq!(context.trusted_host_id(), Some("sidecar-launch-host"));
            assert!(context.verified_identity().is_none());
            assert_eq!(
                CredentialPrincipal::from_verified_context(&context)
                    .expect("launch-bound context must be a credential principal")
                    .issuer_kind(),
                "local-host"
            );
        }

        #[test]
        fn sidecar_bound_equal_header_accepted() {
            let bound = HostBoundPrincipal::for_test("alice");
            let alice = header("alice");
            let context = resolve_host_principal(
                Some(&alice),
                Some(HostAuthenticated::for_test()),
                Some(&bound),
            )
            .expect("equal header must be accepted")
            .expect("equal header must produce a principal");
            assert_eq!(bound_context(context).user_id, "alice");
        }

        #[tokio::test]
        async fn sidecar_bound_mismatched_header_is_forbidden() {
            let bound = HostBoundPrincipal::for_test("alice");
            let bob = header("bob");
            let error = resolve_host_principal(
                Some(&bob),
                Some(HostAuthenticated::for_test()),
                Some(&bound),
            )
            .expect_err("a different subject must be refused");
            assert_eq!(
                error_code(error).await,
                (StatusCode::FORBIDDEN, r#"{"error":{"code":"principal_mismatch"}}"#.to_owned())
            );
        }

        #[tokio::test]
        async fn sidecar_bound_invalid_header_stays_principal_invalid() {
            let bound = HostBoundPrincipal::for_test("alice");
            let anonymous = header("anonymous");
            let error = resolve_host_principal(
                Some(&anonymous),
                Some(HostAuthenticated::for_test()),
                Some(&bound),
            )
            .expect_err("an invalid subject must be refused");
            assert_eq!(
                error_code(error).await,
                (StatusCode::BAD_REQUEST, r#"{"error":{"code":"principal_invalid"}}"#.to_owned())
            );
        }

        #[test]
        fn sidecar_bound_shared_mode_uses_header_subject() {
            let bob = header("bob");
            let context = resolve_host_principal(Some(&bob), Some(HostAuthenticated::for_test()), None)
                .expect("shared mode must accept a valid header")
                .expect("shared mode header must produce a principal");
            assert_eq!(bound_context(context).user_id, "bob");
        }

        #[test]
        fn sidecar_bound_shared_mode_absent_header_is_unchanged() {
            let resolved = resolve_host_principal(None, Some(HostAuthenticated::for_test()), None)
                .expect("absent header must not fail in shared mode");
            assert!(resolved.is_none(), "shared mode must fall through to normal resolution");
        }

        #[tokio::test]
        async fn sidecar_bound_header_without_host_proof_is_refused() {
            let bound = HostBoundPrincipal::for_test("alice");
            let alice = header("alice");
            let error = resolve_host_principal(Some(&alice), None, Some(&bound))
                .expect_err("a header without launch authentication must be refused");
            assert_eq!(
                error_code(error).await,
                (
                    StatusCode::BAD_REQUEST,
                    r#"{"error":{"code":"principal_header_not_allowed"}}"#.to_owned()
                )
            );
        }

        #[test]
        fn sidecar_bound_marker_without_host_proof_grants_nothing() {
            let bound = HostBoundPrincipal::for_test("alice");
            let resolved = resolve_host_principal(None, None, Some(&bound))
                .expect("absent header without host proof must not fail");
            assert!(resolved.is_none(), "a bound marker alone must not create identity");
        }
    }
}

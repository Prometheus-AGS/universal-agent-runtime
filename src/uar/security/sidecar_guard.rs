//! Launch-token authentication for the supervised `uar-sidecar` process.
//!
//! The host that launches the sidecar writes a per-launch 256-bit token as the
//! first line of the sidecar's stdin. [`SidecarGuard`] then admits a request
//! only when it carries that token or an explicitly scoped host-issued grant,
//! names the sidecar's own loopback authority, and carries no `Origin` header.
//! A loopback port is not an authentication
//! boundary: any local process, and a web page through DNS rebinding or a
//! cross-origin request, can reach it. See the OpenSpec change
//! `sidecar-launch-security`.

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use secrecy::SecretString;

/// Marker installed only after the outer launch-token guard admits a request.
/// Inner authentication may trust host assertions only when this marker exists.
#[derive(Debug, Clone, Copy)]
pub struct HostAuthenticated { _private: () }

impl HostAuthenticated {
    /// Attach installation proof without manufacturing a tenant/user credential.
    pub(crate) fn bind_context(&self, context: &mut super::claims::UserContext) {
        context.host_authority = Some(super::claims::HostAuthority::for_launch(self, context));
    }
}

/// Request marker for a launch-bound sidecar. It is installed only by
/// [`enforce`] on a launch-token match, never on the delegation-grant path, and
/// it can be constructed only in this module.
#[derive(Debug, Clone)]
pub struct HostBoundPrincipal {
    subject: SidecarBoundPrincipal,
}

impl HostBoundPrincipal {
    /// The single owner this sidecar process was launched for.
    #[must_use]
    pub(crate) fn subject(&self) -> &str {
        self.subject.as_str()
    }

    #[cfg(test)]
    pub(crate) fn for_test(subject: &str) -> Self {
        Self { subject: SidecarBoundPrincipal::parse(subject).expect("test subject must be valid") }
    }
}

#[cfg(test)]
impl HostAuthenticated {
    pub(crate) fn for_test() -> Self {
        Self { _private: () }
    }
}

/// Whether `value` is a subject a launching host may assert for a sidecar
/// request: 1-128 bytes, ASCII alphanumeric first, then alphanumerics or
/// `.`, `_`, `:` and `-`, and never `anonymous` in any case.
///
/// This is the single rule for both the `x-uar-principal` header and the
/// launch-bound principal.
#[must_use]
pub(crate) fn is_valid_host_principal(value: &str) -> bool {
    !value.eq_ignore_ascii_case("anonymous")
        && (1..=128).contains(&value.len())
        && value
            .as_bytes()
            .iter()
            .copied()
            .enumerate()
            .all(|(index, byte)| {
                byte.is_ascii_alphanumeric()
                    || (index > 0 && matches!(byte, b'.' | b'_' | b':' | b'-'))
            })
}

/// The one owner a launch-bound sidecar serves for its whole lifetime.
///
/// Supplied only by the launching host through the sidecar's launch
/// environment, never by configuration or a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidecarBoundPrincipal(String);

/// The launch-bound principal was not a valid host principal.
///
/// Deliberately carries no detail: the rejected value must never be echoed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("sidecar bound principal is malformed")]
pub struct InvalidBoundPrincipal;

impl SidecarBoundPrincipal {
    /// Validate a bound subject with the same rule as `x-uar-principal`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidBoundPrincipal`] for `anonymous`, an empty or overlong
    /// value, or a value outside the permitted character set.
    ///
    /// # Examples
    ///
    /// ```
    /// use universal_agent_runtime::uar::security::sidecar_guard::SidecarBoundPrincipal;
    /// assert!(SidecarBoundPrincipal::parse("alice").is_ok());
    /// assert!(SidecarBoundPrincipal::parse("anonymous").is_err());
    /// ```
    pub fn parse(value: &str) -> Result<Self, InvalidBoundPrincipal> {
        if is_valid_host_principal(value) {
            Ok(Self(value.to_owned()))
        } else {
            Err(InvalidBoundPrincipal)
        }
    }

    /// The validated subject.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Length of the hex-encoded token: 256 bits.
const TOKEN_HEX_LEN: usize = 64;

/// The token the host handed the sidecar over stdin, plus the optional
/// principal the same host bound the process to at launch.
///
/// Held only in memory: it has no `Display` or `Serialize`, its `Debug` output
/// is redacted, and the secret is zeroized on drop.
pub struct SidecarLaunchToken {
    secret: SecretString,
    bound_principal: Option<SidecarBoundPrincipal>,
}

impl std::fmt::Debug for SidecarLaunchToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SidecarLaunchToken(***redacted***)")
    }
}

/// The first stdin line was missing or was not a well-formed launch token.
///
/// Deliberately carries no detail: the rejected bytes must never be echoed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("sidecar launch token is missing or malformed")]
pub struct InvalidLaunchToken;

impl SidecarLaunchToken {
    /// Parse one line: exactly 64 lowercase hexadecimal characters, an
    /// optional `\r`, then `\n`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidLaunchToken`] when the line has no terminating newline
    /// (stdin reached end-of-file first) or is not exactly the token format.
    ///
    /// # Examples
    ///
    /// ```
    /// use universal_agent_runtime::uar::security::sidecar_guard::SidecarLaunchToken;
    /// let line = format!("{}\n", "a".repeat(64));
    /// assert!(SidecarLaunchToken::from_line(line.as_bytes()).is_ok());
    /// assert!(SidecarLaunchToken::from_line(b"not-a-token\n").is_err());
    /// ```
    pub fn from_line(line: &[u8]) -> Result<Self, InvalidLaunchToken> {
        let body = line.strip_suffix(b"\n").ok_or(InvalidLaunchToken)?;
        let body = body.strip_suffix(b"\r").unwrap_or(body);
        if body.len() != TOKEN_HEX_LEN
            || !body
                .iter()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err(InvalidLaunchToken);
        }
        let text = std::str::from_utf8(body).ok().ok_or(InvalidLaunchToken)?;
        Ok(Self { secret: SecretString::from(text.to_owned()), bound_principal: None })
    }

    /// Bind every launch-token-authenticated request to one principal.
    /// `None` keeps the shared sidecar behavior.
    #[must_use]
    pub fn with_bound_principal(mut self, principal: Option<SidecarBoundPrincipal>) -> Self {
        self.bound_principal = principal;
        self
    }
}

/// Per-request admission for the sidecar port.
#[derive(Debug)]
pub struct SidecarGuard {
    token: Option<SecretString>,
    bound_principal: Option<SidecarBoundPrincipal>,
    allowed_authorities: [String; 2],
    delegation_grants: Option<Arc<super::delegation_grants::DelegationGrantAuthority>>,
}

impl SidecarGuard {
    /// Guard the sidecar listening on `127.0.0.1:{port}`. A principal bound to
    /// the token at launch follows every launch-token match.
    #[must_use]
    pub fn new(token: SidecarLaunchToken, port: u16) -> Self {
        Self {
            token: Some(token.secret),
            bound_principal: token.bound_principal,
            allowed_authorities: [format!("127.0.0.1:{port}"), format!("localhost:{port}")],
            delegation_grants: None,
        }
    }

    /// Attach the process-local authority shared with the host-only grant API.
    #[must_use]
    pub fn with_delegation_grants(
        mut self,
        grants: Arc<super::delegation_grants::DelegationGrantAuthority>,
    ) -> Self {
        self.delegation_grants = Some(grants);
        self
    }

    /// The `Host` header, or for HTTP/2 the request-target authority. More
    /// than one `Host` header is never an allowed authority.
    fn authority_allowed(&self, request: &Request) -> bool {
        let mut hosts = request.headers().get_all(header::HOST).iter();
        let authority = match (hosts.next(), hosts.next()) {
            (Some(host), None) => host.to_str().ok(),
            (None, _) => request
                .uri()
                .authority()
                .map(axum::http::uri::Authority::as_str),
            (Some(_), Some(_)) => None,
        };
        authority.is_some_and(|authority| {
            self.allowed_authorities
                .iter()
                .any(|allowed| authority.eq_ignore_ascii_case(allowed))
        })
    }

    /// Exactly one `Authorization: Bearer <token>` matching in constant time.
    fn token_matches(&self, request: &Request) -> bool {
        let Some(supplied) = bearer_token(request) else {
            return false;
        };
        crate::config::secret_value_matches(&self.token, Some(supplied))
    }
}

fn bearer_token(request: &Request) -> Option<&str> {
    let mut values = request.headers().get_all(header::AUTHORIZATION).iter();
    let (Some(value), None) = (values.next(), values.next()) else {
        return None;
    };
    value
        .to_str()
        .ok()
        .and_then(|value| value.strip_prefix("Bearer "))
}

fn reject(status: StatusCode, reason: &'static str) -> Response {
    tracing::debug!(name: "sidecar.request.rejected", reason, "Sidecar request rejected");
    let body = if status == StatusCode::FORBIDDEN {
        r#"{"error":"forbidden"}"#
    } else {
        r#"{"error":"unauthorized"}"#
    };
    (status, [(header::CONTENT_TYPE, "application/json")], body).into_response()
}

/// Outermost middleware of a token-authenticated sidecar.
///
/// Checks authority, then `Origin`, then the token, so a DNS-rebinding page is
/// refused before the token comparison runs. On success the `Authorization`
/// header is removed so no inner layer, handler, trace or log receives it.
pub async fn enforce(
    State(guard): State<Arc<SidecarGuard>>,
    mut request: Request,
    next: Next,
) -> Response {
    if !guard.authority_allowed(&request) {
        return reject(StatusCode::FORBIDDEN, "bad_host");
    }
    if request.headers().contains_key(header::ORIGIN) {
        return reject(StatusCode::FORBIDDEN, "origin_present");
    }
    if guard.token_matches(&request) {
        // Consume launch authority here; only its redaction contribution
        // follows the authenticated request into an admitted run.
        if let Some(token) = bearer_token(&request) {
            let capture = super::credential_capture::AuthenticatedCredentialCapture::bearer(token);
            request.extensions_mut().insert(capture);
        }
        request.headers_mut().remove(header::AUTHORIZATION);
        request.extensions_mut().insert(HostAuthenticated { _private: () });
        // Only the launch token carries the launch binding; grants below never do.
        if let Some(subject) = &guard.bound_principal {
            request.extensions_mut().insert(HostBoundPrincipal { subject: subject.clone() });
        }
        return next.run(request).await;
    }
    let delegated = bearer_token(&request)
        .and_then(|token| guard.delegation_grants.as_ref()?.authenticate(token, &request));
    if let Some(delegated) = delegated {
        if let Some(token) = bearer_token(&request) {
            let capture = super::credential_capture::AuthenticatedCredentialCapture::bearer(token);
            request.extensions_mut().insert(capture);
        }
        request.headers_mut().remove(header::AUTHORIZATION);
        request.extensions_mut().insert(delegated);
        return next.run(request).await;
    }
    let reason = if request.headers().contains_key(header::AUTHORIZATION) {
        "bad_token"
    } else {
        "missing_token"
    };
    reject(StatusCode::UNAUTHORIZED, reason)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Body, routing::get};
    use tower::ServiceExt as _;

    const PORT: u16 = 32_211;

    fn token() -> SidecarLaunchToken {
        SidecarLaunchToken::from_line(format!("{}\n", "b".repeat(64)).as_bytes())
            .expect("fixture token must parse")
    }

    /// Reports the bound subject the guard installed, or `shared`.
    fn app(bound: Option<SidecarBoundPrincipal>) -> Router {
        Router::new()
            .route(
                "/probe",
                get(|request: Request| async move {
                    assert!(request.extensions().get::<HostAuthenticated>().is_some());
                    request
                        .extensions()
                        .get::<HostBoundPrincipal>()
                        .map_or_else(|| "shared".to_owned(), |bound| bound.subject().to_owned())
                }),
            )
            .layer(axum::middleware::from_fn_with_state(
                Arc::new(SidecarGuard::new(token().with_bound_principal(bound), PORT)),
                enforce,
            ))
    }

    fn request() -> axum::http::request::Builder {
        axum::http::Request::builder()
            .uri("/probe")
            .header(header::HOST, format!("127.0.0.1:{PORT}"))
    }

    async fn body_text(response: Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("probe body must be readable");
        String::from_utf8(bytes.to_vec()).expect("probe body must be UTF-8")
    }

    #[tokio::test]
    async fn sidecar_bound_token_match_installs_bound_marker() {
        let bound = SidecarBoundPrincipal::parse("alice").expect("valid subject");
        let response = app(Some(bound))
            .oneshot(
                request()
                    .header(header::AUTHORIZATION, format!("Bearer {}", "b".repeat(64)))
                    .body(Body::empty())
                    .expect("request must build"),
            )
            .await
            .expect("router must respond");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_text(response).await, "alice");
    }

    #[tokio::test]
    async fn sidecar_bound_shared_guard_installs_no_bound_marker() {
        let response = app(None)
            .oneshot(
                request()
                    .header(header::AUTHORIZATION, format!("Bearer {}", "b".repeat(64)))
                    .body(Body::empty())
                    .expect("request must build"),
            )
            .await
            .expect("router must respond");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_text(response).await, "shared");
    }

    #[tokio::test]
    async fn sidecar_bound_mode_keeps_host_origin_and_token_checks() {
        let bound = || Some(SidecarBoundPrincipal::parse("alice").expect("valid subject"));
        let missing = app(bound())
            .oneshot(request().body(Body::empty()).expect("request must build"))
            .await
            .expect("router must respond");
        assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
        let origin = app(bound())
            .oneshot(
                request()
                    .header(header::ORIGIN, "http://evil.example")
                    .header(header::AUTHORIZATION, format!("Bearer {}", "b".repeat(64)))
                    .body(Body::empty())
                    .expect("request must build"),
            )
            .await
            .expect("router must respond");
        assert_eq!(origin.status(), StatusCode::FORBIDDEN);
        let wrong_host = app(bound())
            .oneshot(
                axum::http::Request::builder()
                    .uri("/probe")
                    .header(header::HOST, "127.0.0.1:1")
                    .header(header::AUTHORIZATION, format!("Bearer {}", "b".repeat(64)))
                    .body(Body::empty())
                    .expect("request must build"),
            )
            .await
            .expect("router must respond");
        assert_eq!(wrong_host.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn sidecar_bound_principal_uses_the_host_principal_rule() {
        assert!(SidecarBoundPrincipal::parse("alice").is_ok());
        assert!(SidecarBoundPrincipal::parse("user:alice.dev_1-x").is_ok());
        assert!(SidecarBoundPrincipal::parse(&"a".repeat(128)).is_ok());
        for rejected in [
            String::new(),
            "anonymous".to_owned(),
            "Anonymous".to_owned(),
            "a".repeat(129),
            "-alice".to_owned(),
            "alice bob".to_owned(),
            "alice/bob".to_owned(),
            "élise".to_owned(),
        ] {
            assert_eq!(SidecarBoundPrincipal::parse(&rejected), Err(InvalidBoundPrincipal));
        }
    }
}

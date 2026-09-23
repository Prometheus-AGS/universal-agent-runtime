//! Launch-token authentication for the supervised `uar-sidecar` process.
//!
//! The host that launches the sidecar writes a per-launch 256-bit token as the
//! first line of the sidecar's stdin. [`SidecarGuard`] then admits a request
//! only when it carries that token, names the sidecar's own loopback authority,
//! and carries no `Origin` header. A loopback port is not an authentication
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

/// Length of the hex-encoded token: 256 bits.
const TOKEN_HEX_LEN: usize = 64;

/// The token the host handed the sidecar over stdin.
///
/// Held only in memory: it has no `Display` or `Serialize`, its `Debug` output
/// is redacted, and the secret is zeroized on drop.
pub struct SidecarLaunchToken(SecretString);

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
        Ok(Self(SecretString::from(text.to_owned())))
    }
}

/// Per-request admission for the sidecar port.
#[derive(Debug)]
pub struct SidecarGuard {
    token: Option<SecretString>,
    allowed_authorities: [String; 2],
}

impl SidecarGuard {
    /// Guard the sidecar listening on `127.0.0.1:{port}`.
    #[must_use]
    pub fn new(token: SidecarLaunchToken, port: u16) -> Self {
        Self {
            token: Some(token.0),
            allowed_authorities: [format!("127.0.0.1:{port}"), format!("localhost:{port}")],
        }
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
        let mut values = request.headers().get_all(header::AUTHORIZATION).iter();
        let (Some(value), None) = (values.next(), values.next()) else {
            return false;
        };
        let Some(supplied) = value
            .to_str()
            .ok()
            .and_then(|value| value.strip_prefix("Bearer "))
        else {
            return false;
        };
        crate::config::secret_value_matches(&self.token, Some(supplied))
    }
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
    if !guard.token_matches(&request) {
        let reason = if request.headers().contains_key(header::AUTHORIZATION) {
            "bad_token"
        } else {
            "missing_token"
        };
        return reject(StatusCode::UNAUTHORIZED, reason);
    }
    request.headers_mut().remove(header::AUTHORIZATION);
    next.run(request).await
}

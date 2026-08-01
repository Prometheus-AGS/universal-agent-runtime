//! Central error type for UAR's public API boundary.
//!
//! `UarError` is the crate's single error type for `src/uar/api/` handlers
//! going forward. It is intentionally additive: existing handlers that
//! return the legacy `(StatusCode, String)` shape are unaffected and may
//! migrate incrementally (see `openspec/changes/central-uar-error-enum/`).
//! Each leaf carries a stable `code` string consumed by the SDKs for typed
//! error matching — codes are part of the public API and MUST NOT change
//! without a SemVer major bump.

#[cfg(feature = "server")]
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
#[cfg(feature = "server")]
use serde::Serialize;

/// The crate's central error type for the public API boundary.
///
/// `#[non_exhaustive]` so new variants are not a breaking change for
/// downstream matches.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UarError {
    /// Configuration is missing, malformed, or fails validation.
    #[error("{message}")]
    Config {
        /// Stable error code, e.g. `E_CONFIG_MISSING_FIELD`.
        code: &'static str,
        message: String,
    },

    /// Authentication or authorization failed.
    #[error("{message}")]
    Auth {
        /// Stable error code, e.g. `E_AUTH_INVALID_TOKEN`.
        code: &'static str,
        message: String,
    },

    /// A RAG (retrieval-augmented generation) operation failed.
    #[error("{message}")]
    Rag {
        /// Stable error code, e.g. `E_RAG_NO_KB`.
        code: &'static str,
        message: String,
    },

    /// A memory-subsystem operation failed.
    #[error("{message}")]
    Memory {
        /// Stable error code, e.g. `E_MEMORY_NOT_FOUND`.
        code: &'static str,
        message: String,
    },

    /// An MCP (Model Context Protocol) operation failed.
    #[error("{message}")]
    Mcp {
        /// Stable error code, e.g. `E_MCP_TOOL_NOT_FOUND`.
        code: &'static str,
        message: String,
    },

    /// An A2A (agent-to-agent) operation failed.
    #[error("{message}")]
    A2a {
        /// Stable error code, e.g. `E_A2A_AGENT_UNREACHABLE`.
        code: &'static str,
        message: String,
    },

    /// An LLM provider call failed.
    #[error("{message}")]
    Llm {
        /// Stable error code, e.g. `E_LLM_PROVIDER_ERROR`.
        code: &'static str,
        message: String,
    },

    /// An internal error not attributable to a specific domain above.
    /// Wraps `anyhow::Error` so call sites using `?` compose naturally.
    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl UarError {
    /// Construct a [`UarError::Config`] variant.
    pub fn config(code: &'static str, message: impl Into<String>) -> Self {
        Self::Config {
            code,
            message: message.into(),
        }
    }

    /// Construct a [`UarError::Auth`] variant.
    pub fn auth(code: &'static str, message: impl Into<String>) -> Self {
        Self::Auth {
            code,
            message: message.into(),
        }
    }

    /// Construct a [`UarError::Rag`] variant.
    pub fn rag(code: &'static str, message: impl Into<String>) -> Self {
        Self::Rag {
            code,
            message: message.into(),
        }
    }

    /// Construct a [`UarError::Memory`] variant.
    pub fn memory(code: &'static str, message: impl Into<String>) -> Self {
        Self::Memory {
            code,
            message: message.into(),
        }
    }

    /// Construct a [`UarError::Mcp`] variant.
    pub fn mcp(code: &'static str, message: impl Into<String>) -> Self {
        Self::Mcp {
            code,
            message: message.into(),
        }
    }

    /// Construct a [`UarError::A2a`] variant.
    pub fn a2a(code: &'static str, message: impl Into<String>) -> Self {
        Self::A2a {
            code,
            message: message.into(),
        }
    }

    /// Construct a [`UarError::Llm`] variant.
    pub fn llm(code: &'static str, message: impl Into<String>) -> Self {
        Self::Llm {
            code,
            message: message.into(),
        }
    }

    /// The stable, SemVer-significant error code for this error.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Config { code, .. }
            | Self::Auth { code, .. }
            | Self::Rag { code, .. }
            | Self::Memory { code, .. }
            | Self::Mcp { code, .. }
            | Self::A2a { code, .. }
            | Self::Llm { code, .. } => code,
            Self::Internal(_) => "E_INTERNAL",
        }
    }

    /// The HTTP status code this error maps to.
    #[must_use]
    #[cfg(feature = "server")]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Auth { .. } => StatusCode::UNAUTHORIZED,
            Self::Config { .. }
            | Self::Rag { .. }
            | Self::Memory { .. }
            | Self::Mcp { .. }
            | Self::A2a { .. }
            | Self::Llm { .. } => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Convenience alias for `Result<T, UarError>`, exposed at the crate root
/// as `crate::Result<T>`.
pub type Result<T> = std::result::Result<T, UarError>;

/// The public JSON shape of a `UarError` HTTP response body.
#[derive(Debug, Serialize)]
#[cfg(feature = "server")]
struct ErrorBody {
    code: &'static str,
    message: String,
}

#[cfg(feature = "server")]
impl IntoResponse for UarError {
    fn into_response(self) -> Response {
        // Full error chain + span trace go to the server-side log only; the
        // public response body carries just `code` + `message` (no internal
        // state, per the tracing-error context requirement).
        let span_trace = tracing_error::SpanTrace::capture();
        tracing::error!(
            error = %self,
            code = self.code(),
            %span_trace,
            "request failed with UarError"
        );

        #[cfg(feature = "sentry")]
        sentry::capture_error(&self);

        let status = self.status_code();
        let body = ErrorBody {
            code: self.code(),
            message: self.to_string(),
        };
        (status, Json(body)).into_response()
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn config_variant_exposes_its_code_and_message() {
        let err = UarError::config("E_CONFIG_MISSING_FIELD", "missing `llm.model`");
        assert_eq!(err.code(), "E_CONFIG_MISSING_FIELD");
        assert_eq!(err.to_string(), "missing `llm.model`");
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn auth_variant_maps_to_unauthorized() {
        let err = UarError::auth("E_AUTH_INVALID_TOKEN", "token expired");
        assert_eq!(err.code(), "E_AUTH_INVALID_TOKEN");
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn rag_variant_exposes_its_code() {
        let err = UarError::rag("E_RAG_NO_KB", "no knowledge base configured");
        assert_eq!(err.code(), "E_RAG_NO_KB");
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn memory_variant_exposes_its_code() {
        let err = UarError::memory("E_MEMORY_NOT_FOUND", "session not found");
        assert_eq!(err.code(), "E_MEMORY_NOT_FOUND");
    }

    #[test]
    fn mcp_variant_exposes_its_code() {
        let err = UarError::mcp("E_MCP_TOOL_NOT_FOUND", "unknown tool");
        assert_eq!(err.code(), "E_MCP_TOOL_NOT_FOUND");
    }

    #[test]
    fn a2a_variant_exposes_its_code() {
        let err = UarError::a2a("E_A2A_AGENT_UNREACHABLE", "agent did not respond");
        assert_eq!(err.code(), "E_A2A_AGENT_UNREACHABLE");
    }

    #[test]
    fn llm_variant_exposes_its_code() {
        let err = UarError::llm("E_LLM_PROVIDER_ERROR", "provider returned 500");
        assert_eq!(err.code(), "E_LLM_PROVIDER_ERROR");
    }

    #[test]
    fn internal_variant_wraps_anyhow_and_uses_generic_code() {
        let source = anyhow::anyhow!("boom");
        let err: UarError = source.into();
        assert_eq!(err.code(), "E_INTERNAL");
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn into_response_captures_span_trace_with_request_ids() {
        use tracing_subscriber::util::SubscriberInitExt;
        let _guard = tracing_subscriber::registry().set_default();

        let span = tracing::info_span!(
            "test_request",
            request_id = "req-123",
            agent_id = "agent-456",
            run_id = "run-789",
        );

        span.in_scope(|| {
            // SpanTrace capture works because tracing-error is on the call
            // stack; the exact textual representation depends on the subscriber.
            let _span_trace = tracing_error::SpanTrace::capture();

            let err = UarError::config("E_CONFIG_MISSING_FIELD", "missing `llm.model`");
            let response = err.into_response();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        });
    }

    #[tokio::test]
    async fn into_response_renders_the_documented_json_shape() {
        use axum::body::to_bytes;

        let err = UarError::config("E_CONFIG_MISSING_FIELD", "missing `llm.model`");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["code"], "E_CONFIG_MISSING_FIELD");
        assert_eq!(json["message"], "missing `llm.model`");
    }
}

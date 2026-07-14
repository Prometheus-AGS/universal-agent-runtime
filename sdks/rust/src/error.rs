//! Typed SDK diagnostics.

use miette::Diagnostic;
use serde::Deserialize;
use thiserror::Error;

/// Structured error envelope returned by UAR.
#[derive(Debug, Clone, Deserialize)]
pub struct UarError {
    /// Stable machine-readable code.
    #[serde(default = "default_code")]
    pub code: String,
    /// Human-readable message.
    #[serde(default)]
    pub message: String,
    /// Optional request identifier for support correlation.
    #[serde(default)]
    pub request_id: Option<String>,
}

fn default_code() -> String {
    "uar::unknown".into()
}

/// Failure produced by the SDK.
#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    /// HTTP transport failed.
    #[cfg(feature = "http-client")]
    #[error("HTTP transport failed: {0}")]
    #[diagnostic(
        code(uar_sdk::http),
        help("Check the runtime URL and network connectivity.")
    )]
    Http(#[from] reqwest::Error),

    /// Base URL was invalid.
    #[error("invalid runtime URL: {0}")]
    #[diagnostic(code(uar_sdk::invalid_url))]
    InvalidUrl(#[from] url::ParseError),

    /// JSON encoding or decoding failed.
    #[error("invalid JSON payload: {0}")]
    #[diagnostic(code(uar_sdk::json))]
    Json(#[from] serde_json::Error),

    /// Runtime returned an unsuccessful response.
    #[error("UAR request failed ({status}, {error_code}): {message}")]
    #[diagnostic(
        code(uar_sdk::api),
        help("Use request_id to correlate the failure with runtime logs.")
    )]
    Api {
        /// HTTP status code.
        status: u16,
        /// Runtime error code.
        error_code: String,
        /// Runtime error message.
        message: String,
        /// Optional request identifier.
        request_id: Option<String>,
    },

    /// SSE data was malformed.
    #[error("invalid server-sent event: {message}")]
    #[diagnostic(code(uar_sdk::stream_decode))]
    StreamDecode {
        /// Decode failure details.
        message: String,
    },

    /// Configuration error (embedded mode).
    #[error("configuration error: {0}")]
    #[diagnostic(code(uar_sdk::config))]
    Config(String),

    /// Runtime error (embedded mode).
    #[error("runtime error: {0}")]
    #[diagnostic(code(uar_sdk::runtime))]
    Runtime(String),
}

/// SDK result alias.
pub type Result<T> = std::result::Result<T, Error>;

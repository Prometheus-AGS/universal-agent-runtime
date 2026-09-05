//! Typed provider failures used by retry and failover policy.

use std::time::Duration;

use crate::uar::settings::resilience_policy::ResiliencePolicy;

/// Stable provider failure classes. Retry decisions never inspect messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderErrorKind {
    Authentication,
    InvalidRequest,
    RateLimited,
    Overloaded,
    Timeout,
    Transport,
    Stream,
    BudgetExceeded,
    External,
    Internal,
}

/// Provider failure metadata retained across the driver-erasing interface.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ProviderError {
    pub status: Option<u16>,
    pub kind: ProviderErrorKind,
    pub retry_after: Option<Duration>,
    pub message: String,
}

impl ProviderError {
    #[must_use]
    pub fn new(
        status: Option<u16>,
        kind: ProviderErrorKind,
        retry_after: Option<Duration>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            status,
            kind,
            retry_after,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn from_http(
        status: u16,
        retry_after: Option<Duration>,
        message: impl Into<String>,
    ) -> Self {
        let kind = match status {
            401 | 403 => ProviderErrorKind::Authentication,
            408 => ProviderErrorKind::Timeout,
            425 | 429 => ProviderErrorKind::RateLimited,
            500..=599 => ProviderErrorKind::Overloaded,
            _ => ProviderErrorKind::InvalidRequest,
        };
        Self::new(Some(status), kind, retry_after, message)
    }

    #[must_use]
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(None, ProviderErrorKind::InvalidRequest, None, message)
    }

    #[must_use]
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(None, ProviderErrorKind::Timeout, None, message)
    }

    #[must_use]
    pub fn external(message: impl Into<String>) -> Self {
        Self::new(None, ProviderErrorKind::External, None, message)
    }

    #[must_use]
    pub fn from_reqwest(error: reqwest::Error) -> Self {
        let status = error.status().map(|status| status.as_u16());
        let kind = if error.is_timeout() {
            ProviderErrorKind::Timeout
        } else if error.is_connect() || error.is_request() {
            ProviderErrorKind::Transport
        } else if error.is_body() || error.is_decode() {
            ProviderErrorKind::Stream
        } else {
            ProviderErrorKind::Internal
        };
        Self::new(status, kind, None, error.to_string())
    }

    #[must_use]
    pub fn from_liter(error: liter_llm::LiterLlmError) -> Self {
        use liter_llm::LiterLlmError;

        let status = match error.status_code() {
            0 => None,
            status => Some(status),
        };
        let retry_after = error.retry_after();
        let kind = match &error {
            LiterLlmError::Authentication { .. } => ProviderErrorKind::Authentication,
            LiterLlmError::RateLimited { .. } => ProviderErrorKind::RateLimited,
            LiterLlmError::BadRequest { .. }
            | LiterLlmError::ContextWindowExceeded { .. }
            | LiterLlmError::ContentPolicy { .. }
            | LiterLlmError::NotFound { .. }
            | LiterLlmError::EndpointNotSupported { .. }
            | LiterLlmError::InvalidHeader { .. }
            | LiterLlmError::IdempotencyConflict { .. }
            | LiterLlmError::OutboundForbidden { .. } => ProviderErrorKind::InvalidRequest,
            LiterLlmError::ServerError { .. }
            | LiterLlmError::ServiceUnavailable { .. }
            | LiterLlmError::IdempotencyInFlight { .. } => ProviderErrorKind::Overloaded,
            LiterLlmError::Timeout => ProviderErrorKind::Timeout,
            LiterLlmError::Network(_) => ProviderErrorKind::Transport,
            LiterLlmError::Streaming { .. } => ProviderErrorKind::Stream,
            LiterLlmError::BudgetExceeded { .. } => ProviderErrorKind::BudgetExceeded,
            LiterLlmError::Serialization(_)
            | LiterLlmError::HookRejected { .. }
            | LiterLlmError::InternalError { .. } => ProviderErrorKind::Internal,
            _ => ProviderErrorKind::Internal,
        };
        Self::new(status, kind, retry_after, error.to_string())
    }

    #[must_use]
    pub fn is_retryable(&self, policy: &ResiliencePolicy) -> bool {
        match self.kind {
            ProviderErrorKind::RateLimited | ProviderErrorKind::Overloaded => self
                .status
                .is_none_or(|status| policy.retryable_http_statuses.contains(&status)),
            ProviderErrorKind::Timeout
            | ProviderErrorKind::Transport
            | ProviderErrorKind::Stream => policy.retryable_transport_errors,
            ProviderErrorKind::Authentication
            | ProviderErrorKind::InvalidRequest
            | ProviderErrorKind::BudgetExceeded
            | ProviderErrorKind::External
            | ProviderErrorKind::Internal => false,
        }
    }

    #[must_use]
    pub fn from_anyhow(error: &anyhow::Error) -> Option<&Self> {
        error.downcast_ref::<Self>()
    }
}

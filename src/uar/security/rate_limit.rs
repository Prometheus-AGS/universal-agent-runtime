use crate::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::warn;

/// Wrapper around Governor Rate Limiter to be stored in `AppState`
/// We use a generic non-keyed limiter for global rate limiting as per current design.
/// (Keyed by IP would require extracting IP which is added complexity).
#[derive(Debug, Clone)]
pub struct AppRateLimiter {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl AppRateLimiter {
    pub fn new(requests_per_second: f32, burst_size: u32) -> Self {
        // Convert f32 rate to Quota. Per second.
        // Governor requires non-zero.
        let burst = NonZeroU32::new(burst_size).unwrap_or(NonZeroU32::new(1).unwrap());

        // Convert requests_per_second to duration between requests approx?
        // Governor's Quota::per_second takes a u32.
        // If we want fractional, we might need per_period.
        // Let's assume u32 for now or ceil.
        #[allow(clippy::cast_sign_loss)]
        let rps = NonZeroU32::new(requests_per_second.ceil() as u32)
            .unwrap_or(NonZeroU32::new(1).unwrap());

        let quota = Quota::per_second(rps).allow_burst(burst);

        Self {
            limiter: Arc::new(RateLimiter::direct(quota)),
        }
    }

    pub fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

fn should_rate_limit_path(path: &str) -> bool {
    path == "/api"
        || path.starts_with("/api/")
        || path == "/v1"
        || path.starts_with("/v1/")
        || path == "/mcp"
        || path.starts_with("/mcp/")
        || path == "/a2a"
        || path.starts_with("/a2a/")
        || path == "/.well-known"
        || path.starts_with("/.well-known/")
}

/// Middleware to enforce rate limits
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    if state.config.resilience.rate_limit_enabled
        && should_rate_limit_path(path)
        && !state.rate_limiter.check()
    {
        warn!("Rate limit exceeded");
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_policy_only_limits_api_and_protocol_routes() {
        assert!(should_rate_limit_path("/api/chat/completion"));
        assert!(should_rate_limit_path("/api/uar/providers"));
        assert!(should_rate_limit_path("/v1/chat/completions"));
        assert!(should_rate_limit_path("/mcp/uar"));
        assert!(should_rate_limit_path("/a2a/registry"));

        assert!(!should_rate_limit_path("/apple-touch-icon.png"));
        assert!(!should_rate_limit_path("/manifest.json"));
        assert!(!should_rate_limit_path("/favicon.svg"));
        assert!(!should_rate_limit_path("/assets/index.js"));
        assert!(!should_rate_limit_path("/threads"));
        assert!(!should_rate_limit_path("/about"));
    }
}

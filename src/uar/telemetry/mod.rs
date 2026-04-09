pub mod metrics;

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::LogFormat;

/// Initialize application telemetry (Logging, Tracing, Metrics).
///
/// Configures:
/// - `tracing-subscriber::fmt` for structured logging (JSON, compact, or pretty).
/// - `EnvFilter` for dynamic log levels (`RUST_LOG`).
///
/// The log format is controlled by `UAR_SERVER__LOG_FORMAT` or the `server.log_format`
/// config key. Defaults to JSON for Kubernetes log aggregator compatibility.
pub fn init(log_format: &LogFormat) {
    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,universal_agent_runtime=debug"));

    match log_format {
        LogFormat::Json => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true);

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt_layer)
                .init();
        }
        LogFormat::Compact => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .compact();

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt_layer)
                .init();
        }
        LogFormat::Pretty => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .pretty();

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt_layer)
                .init();
        }
    }
}

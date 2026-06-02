pub mod metrics;

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::LogFormat;

/// Build an OTLP tracer provider when trace export is enabled.
///
/// Opt-in and gated: returns `Some` only when `OTEL_EXPORTER_OTLP_ENDPOINT` is
/// set AND tracing is not explicitly disabled (`UAR_LLM__TRACING` != false/0).
/// Returns `None` otherwise so default/offline runs need no OTLP collector.
/// Env is read directly because telemetry is initialized before full config load
/// (env is the highest-precedence config source, so this is consistent).
fn build_otlp_provider() -> Option<SdkTracerProvider> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok()?;
    let tracing_enabled = std::env::var("UAR_LLM__TRACING")
        .map(|v| !matches!(v.to_lowercase().as_str(), "false" | "0" | "off" | "no"))
        .unwrap_or(true);
    if !tracing_enabled {
        return None;
    }

    // OTLP/HTTP (the crate's default `http-proto` feature). Endpoint example:
    // `http://localhost:4318/v1/traces`.
    let exporter = match opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .build()
    {
        Ok(e) => e,
        Err(e) => {
            eprintln!("OTLP span exporter build failed; tracing disabled: {e}");
            return None;
        }
    };

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_service_name("universal-agent-runtime")
                .build(),
        )
        .build();
    Some(provider)
}

/// Initialize application telemetry (Logging, Tracing, Metrics).
///
/// Configures:
/// - `tracing-subscriber::fmt` for structured logging (JSON, compact, or pretty).
/// - `EnvFilter` for dynamic log levels (`RUST_LOG`).
/// - An optional OTLP trace-export layer (see [`build_otlp_provider`]).
///
/// The log format is controlled by `UAR_SERVER__LOG_FORMAT` or the `server.log_format`
/// config key. Defaults to JSON for Kubernetes log aggregator compatibility.
///
/// Returns the OTLP [`SdkTracerProvider`] when trace export is active, so the
/// caller can `shutdown()` it on exit to flush buffered spans; `None` otherwise.
#[must_use]
pub fn init(log_format: &LogFormat) -> Option<SdkTracerProvider> {
    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,universal_agent_runtime=debug"));

    let otel_provider = build_otlp_provider();
    let otel_layer = otel_provider
        .as_ref()
        .map(|p| tracing_opentelemetry::layer().with_tracer(p.tracer("universal-agent-runtime")));

    match log_format {
        LogFormat::Json => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true);

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(otel_layer)
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
                .with(otel_layer)
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
                .with(otel_layer)
                .with(fmt_layer)
                .init();
        }
    }

    if otel_provider.is_some() {
        tracing::info!(name: "telemetry.otlp", "OTLP trace export enabled");
    }
    otel_provider
}

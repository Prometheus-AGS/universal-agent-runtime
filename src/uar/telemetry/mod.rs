pub mod metrics;

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::sync::Mutex;
use std::time::Duration;
use tracing_subscriber::{EnvFilter, fmt::writer::BoxMakeWriter, layer::SubscriberExt};

use crate::config::LogFormat;

// Install tracing independently of the optional log bridge: try_init() can
// report a host log logger after it has already installed our subscriber.
// A best-effort bridge preserves the existing logging behavior without treating
// a foreign log logger as failure to own the tracing subscriber.
fn install_subscriber<S>(subscriber: S) -> Result<(), tracing::dispatcher::SetGlobalDefaultError>
where
    S: tracing::Subscriber + Send + Sync + 'static,
{
    tracing::subscriber::set_global_default(subscriber)?;
    use tracing_subscriber::util::SubscriberInitExt as _;
    let dispatch = tracing::dispatcher::get_default(Clone::clone);
    drop(dispatch.set_default());
    Ok(())
}

/// Ownership of the process tracing subscriber.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriberStatus {
    /// UAR initialization has not run.
    Uninitialized,
    /// UAR installed the process subscriber.
    Installed,
    /// A host subscriber already owns the process slot.
    HostOwned,
    /// Initialization could not complete.
    Unavailable,
}

impl SubscriberStatus {
    /// Stable ownership diagnostic.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Uninitialized => "uninitialized",
            Self::Installed => "installed",
            Self::HostOwned => "host_owned",
            Self::Unavailable => "unavailable",
        }
    }
}

/// UAR trace export configuration. Enabled does not attest collector delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportStatus {
    /// No endpoint, explicitly disabled, or telemetry not compiled.
    Disabled,
    /// UAR's subscriber has an OTLP layer; delivery is best effort.
    Enabled,
    /// Only the host can arrange trace export.
    HostOwned,
    /// Exporter construction failed.
    Unavailable,
    /// The owned provider has been asked to shut down.
    Shutdown,
}

impl ExportStatus {
    /// Stable export diagnostic without endpoint or credential values.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Enabled => "enabled",
            Self::HostOwned => "host_owned",
            Self::Unavailable => "unavailable",
            Self::Shutdown => "shutdown",
        }
    }
}

/// Last initialization disposition; not a collector health probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelemetryStatus {
    /// Process subscriber owner.
    pub subscriber: SubscriberStatus,
    /// UAR's export configuration.
    pub export: ExportStatus,
}

static STATUS: Mutex<TelemetryStatus> = Mutex::new(TelemetryStatus {
    subscriber: SubscriberStatus::Uninitialized,
    export: ExportStatus::Disabled,
});

/// Snapshot ownership without installing telemetry or contacting an exporter.
pub fn status() -> TelemetryStatus {
    STATUS.lock().map_or(
        TelemetryStatus {
            subscriber: SubscriberStatus::Unavailable,
            export: ExportStatus::Unavailable,
        },
        |status| *status,
    )
}

/// SDK shutdown wait used by the owned helper and an unattached exporter.
/// This bounds the SDK acknowledgement wait, not exporter-thread release.
pub const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

fn shutdown_provider(provider: &SdkTracerProvider) -> anyhow::Result<()> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        provider.shutdown_with_timeout(SHUTDOWN_TIMEOUT)
    })) {
        Ok(result) => result.map_err(|_| anyhow::anyhow!("telemetry shutdown unconfirmed")),
        Err(_) => Err(anyhow::anyhow!(
            "telemetry shutdown panicked; release unconfirmed"
        )),
    }
}

/// Ask the owned provider to flush and shut down using the pinned SDK timeout.
/// On timeout, full delivery and worker release remain unconfirmed. Call outside
/// a Tokio current-thread runtime; see the operation contract.
pub fn shutdown(provider: &SdkTracerProvider) -> anyhow::Result<()> {
    let result = shutdown_provider(provider);
    if let Ok(mut status) = STATUS.lock() {
        status.export = if result.is_ok() {
            ExportStatus::Shutdown
        } else {
            ExportStatus::Unavailable
        };
    }
    result
}

/// Build an OTLP tracer provider when trace export is enabled.
///
/// Opt-in and gated: returns `Some` only when `OTEL_EXPORTER_OTLP_ENDPOINT` is
/// set AND tracing is not explicitly disabled (`UAR_LLM__TRACING` != false/0).
/// Returns `None` otherwise so default/offline runs need no OTLP collector.
/// Env is read directly because telemetry is initialized before full config load
/// (env is the highest-precedence config source, so this is consistent).
fn build_otlp_provider() -> (Option<SdkTracerProvider>, ExportStatus) {
    let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") else {
        return (None, ExportStatus::Disabled);
    };
    let tracing_enabled = std::env::var("UAR_LLM__TRACING")
        .map(|v| !matches!(v.to_lowercase().as_str(), "false" | "0" | "off" | "no"))
        .unwrap_or(true);
    if !tracing_enabled {
        return (None, ExportStatus::Disabled);
    }

    // OTLP/HTTP (the crate's default `http-proto` feature). Endpoint example:
    // `http://localhost:4318/v1/traces`.
    let exporter = match opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .build()
    {
        Ok(e) => e,
        Err(_) => {
            eprintln!("OTLP span exporter unavailable; continuing without trace export");
            return (None, ExportStatus::Unavailable);
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
    (Some(provider), ExportStatus::Enabled)
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
/// `default_directive` is the filter used when `RUST_LOG` is unset; an
/// explicit `RUST_LOG` always wins.
///
/// Returns the OTLP [`SdkTracerProvider`] when trace export is active, so the
/// caller can `shutdown()` it on exit to flush buffered spans; `None` otherwise.
#[must_use]
pub fn init(
    log_format: &LogFormat,
    default_directive: &str,
) -> anyhow::Result<Option<SdkTracerProvider>> {
    let mut disposition = STATUS
        .lock()
        .map_err(|_| anyhow::anyhow!("telemetry initialization unavailable"))?;
    if matches!(
        disposition.subscriber,
        SubscriberStatus::Installed | SubscriberStatus::HostOwned
    ) {
        return Ok(None);
    }
    disposition.subscriber = SubscriberStatus::Unavailable;
    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_directive));

    let writer = if let Some(path) = std::env::var_os("UAR_LOG_FILE") {
        let path = std::path::PathBuf::from(path);
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|error| {
                anyhow::anyhow!(
                    "opening UAR_LOG_FILE '{}' for append: {error}",
                    path.display()
                )
            })?;
        BoxMakeWriter::new(std::sync::Mutex::new(file))
    } else if std::env::var_os("UAR_SIDECAR").as_deref() == Some(std::ffi::OsStr::new("1")) {
        // The sidecar's stdout is its READY:<port> control channel.
        BoxMakeWriter::new(std::io::stderr)
    } else {
        BoxMakeWriter::new(std::io::stdout)
    };

    let (otel_provider, export_status) = build_otlp_provider();
    let otel_layer = otel_provider
        .as_ref()
        .map(|p| tracing_opentelemetry::layer().with_tracer(p.tracer("universal-agent-runtime")));

    let installed = match log_format {
        LogFormat::Json => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .with_writer(writer);

            install_subscriber(
                tracing_subscriber::registry()
                    .with(filter_layer)
                    .with(otel_layer)
                    .with(fmt_layer),
            )
        }
        LogFormat::Compact => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .compact()
                .with_writer(writer);

            install_subscriber(
                tracing_subscriber::registry()
                    .with(filter_layer)
                    .with(otel_layer)
                    .with(fmt_layer),
            )
        }
        LogFormat::Pretty => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .pretty()
                .with_writer(writer);

            install_subscriber(
                tracing_subscriber::registry()
                    .with(filter_layer)
                    .with(otel_layer)
                    .with(fmt_layer),
            )
        }
    };

    if installed.is_err() {
        disposition.subscriber = SubscriberStatus::HostOwned;
        disposition.export = ExportStatus::HostOwned;
        drop(disposition);
        if let Some(provider) = &otel_provider {
            if shutdown_provider(provider).is_err() {
                eprintln!("unattached telemetry exporter cleanup unconfirmed");
            }
        }
        return Ok(None);
    }
    disposition.subscriber = SubscriberStatus::Installed;
    disposition.export = export_status;
    drop(disposition);

    if otel_provider.is_some() {
        tracing::info!(name: "telemetry.otlp", "OTLP trace export enabled");
    }
    Ok(otel_provider)
}

//! Observability facade for builds without the `telemetry` feature.
//!
//! "Telemetry-disabled" means **no OTLP exporters and no metrics**. It does
//! NOT mean no logs: this module still installs a `tracing` subscriber
//! writing to stdout (or `UAR_LOG_FILE`), matching the telemetry build's
//! console behaviour exactly.
//!
//! It used to return `Ok(None)` and install nothing. Because
//! `default = ["minimal"]` does not include `telemetry`, the DEFAULT BUILD
//! therefore emitted no logs at all — every `info!`, `warn!` and `error!` in
//! the runtime went nowhere. The header comment claimed "structured tracing
//! calls remain compiled", which was true and useless: compiled events with
//! no subscriber are discarded. Observed 2026-09-01, when a startup hang and
//! an empty tool list both had to be diagnosed by `eprintln!` and process
//! sampling because the runtime could not say anything about itself.
//!
//! `tracing-subscriber` is an unconditional dependency, so this costs no new
//! feature surface.

use crate::config::LogFormat;
use std::sync::Mutex;
use std::time::Duration;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::layer::SubscriberExt;

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

/// No exporter exists in this profile; shutdown performs no I/O.
pub fn shutdown(provider: &DisabledTelemetryProvider) -> anyhow::Result<()> {
    match provider.shutdown() {
        Ok(()) => Ok(()),
        Err(error) => match error {},
    }
}

/// Placeholder matching the exporter shutdown contract.
#[derive(Debug)]
pub struct DisabledTelemetryProvider;

impl DisabledTelemetryProvider {
    /// No exporter buffers exist to flush.
    pub fn shutdown(&self) -> Result<(), std::convert::Infallible> {
        Ok(())
    }
}

/// Install a console subscriber. No exporters, but logs are emitted.
///
/// Mirrors the telemetry build's writer selection (`UAR_LOG_FILE` else
/// stdout) and default filter, so switching the feature on or off changes
/// what is EXPORTED, never whether the process can be observed at all.
/// `default_directive` applies when `RUST_LOG` is unset.
pub fn init(
    log_format: &LogFormat,
    default_directive: &str,
) -> anyhow::Result<Option<DisabledTelemetryProvider>> {
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
        BoxMakeWriter::new(std::io::stderr)
    } else {
        BoxMakeWriter::new(std::io::stdout)
    };

    // A second `init()` in one process would panic; a server that cannot log
    // is a lesser failure than a server that will not start, so this is
    // tolerant by design (tests and embedders may install their own).
    let installed = match log_format {
        LogFormat::Json => install_subscriber(
            tracing_subscriber::registry().with(filter_layer).with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(true)
                    .with_line_number(true)
                    .with_writer(writer),
            ),
        ),
        LogFormat::Compact => install_subscriber(
            tracing_subscriber::registry().with(filter_layer).with(
                tracing_subscriber::fmt::layer()
                    .compact()
                    .with_target(true)
                    .with_line_number(true)
                    .with_writer(writer),
            ),
        ),
        LogFormat::Pretty => install_subscriber(
            tracing_subscriber::registry().with(filter_layer).with(
                tracing_subscriber::fmt::layer()
                    .pretty()
                    .with_target(true)
                    .with_line_number(true)
                    .with_writer(writer),
            ),
        ),
    };
    disposition.subscriber = if installed.is_ok() {
        SubscriberStatus::Installed
    } else {
        SubscriberStatus::HostOwned
    };
    disposition.export = ExportStatus::Disabled;

    Ok(None)
}

/// No-op metrics facade preserving instrumentation call sites.
pub mod metrics {
    use std::time::{Duration, Instant};

    /// Ownership of the process metrics recorder, not exporter connectivity.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MetricsStatus {
        /// UAR owns the recorder and can serve its scrape.
        Installed,
        /// The host owns metric writes and must arrange their export.
        HostOwned,
        /// Metrics were not compiled into this profile.
        Disabled,
        /// No supported exporter is available.
        Unavailable,
    }

    impl MetricsStatus {
        /// Stable diagnostic value. Never include resource or owner identifiers.
        pub const fn as_str(self) -> &'static str {
            match self {
                Self::Installed => "installed",
                Self::HostOwned => "host_owned",
                Self::Disabled => "disabled",
                Self::Unavailable => "unavailable",
            }
        }
    }

    /// Metrics are absent in this build; no process-global recorder is installed.
    pub fn status() -> MetricsStatus {
        MetricsStatus::Disabled
    }

    pub fn init() {}
    pub fn record_request(_method: &str, _path: &str, _status: u16, _duration: Duration) {}
    #[must_use]
    pub fn request_timer() -> Instant {
        Instant::now()
    }
    pub fn record_llm_call_latency(_provider: &str, _model: &str, _duration_secs: f64) {}
    pub fn record_llm_cost(_provider: &str, _model: &str, _cost_usd: f64) {}
    pub fn record_provider_health(_provider: &str, _healthy: bool) {}
    pub fn record_llm_tokens(
        _provider: &str,
        _model: &str,
        _input_tokens: u64,
        _output_tokens: u64,
    ) {
    }
    pub fn record_cache_tokens(
        _provider: &str,
        _model: &str,
        _write_tokens: u32,
        _read_tokens: u32,
    ) {
    }
    pub fn record_sycophancy_score(_score: f64) {}
    pub fn record_sycophancy_flagged() {}
    pub fn record_guardrail_flagged(_category: &str) {}
    pub fn record_eval_score(_suite: &str, _scorer: &str, _mean: f64) {}
    pub fn record_eval_regression() {}
    pub fn record_tool_call(_tool_name: &str, _success: bool) {}
    pub fn set_active_sessions(_count: f64) {}
    pub fn set_runs_retained(_count: f64) {}
    pub fn set_a2ui_replay_runs(_count: f64) {}
    pub fn record_sandbox_created(_runner_type: &str, _language: &str) {}
    pub fn record_sandbox_execution(_language: &str, _exit_code_class: &str, _duration_secs: f64) {}
    pub fn set_active_sandboxes(_count: f64) {}
    pub fn sandbox_active_inc() {}
    pub fn sandbox_active_dec() {}
    pub fn record_sandbox_error(_error_type: &str) {}
    pub fn set_mcp_server_status(_server_name: &str, _healthy: bool) {}
    pub fn record_skill_activation(_skill_id: &str, _backend: &str, _accepted: bool) {}
    pub fn record_skill_activation_outcome(_skill_id: &str, _success: bool) {}
    pub fn record_skill_shadow_recall(_backend: &str, _hit: bool) {}
    pub fn record_skill_invocation(_skill_id: &str, _invoke_type: &str) {}
    pub fn record_skill_request_usage(_skill_id: &str, _tokens: u64, _cost_usd: Option<f64>) {}
}

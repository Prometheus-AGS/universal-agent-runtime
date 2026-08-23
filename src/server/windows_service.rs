use std::ffi::OsString;
use std::sync::OnceLock;
use std::time::Duration;

use crate::config::{Cli, LogFormat};
use crate::config_manager::ConfigManager;
use crate::uar;
use tokio_util::sync::CancellationToken;
use windows_service::define_windows_service;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_dispatcher;

const SERVICE_NAME: &str = "PrometheusUniversalAgentRuntime";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

#[derive(Clone)]
struct Bootstrap {
    cli: Cli,
    log_format: LogFormat,
}

static BOOTSTRAP: OnceLock<Bootstrap> = OnceLock::new();

pub fn run(cli: Cli, log_format: LogFormat) -> anyhow::Result<()> {
    BOOTSTRAP
        .set(Bootstrap { cli, log_format })
        .map_err(|_| anyhow::anyhow!("Windows service bootstrap already initialized"))?;
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
        .map_err(|error| anyhow::anyhow!("starting Windows service dispatcher: {error}"))
}

define_windows_service!(ffi_service_main, service_main);

fn service_main(_arguments: Vec<OsString>) {
    if let Err(error) = run_service()
        && tracing::dispatcher::has_been_set()
    {
        tracing::error!(%error, "Windows service stopped with an error");
    }
}

fn status(state: ServiceState, accepted: ServiceControlAccept, code: u32) -> ServiceStatus {
    ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: state,
        controls_accepted: accepted,
        exit_code: if code == 0 {
            ServiceExitCode::Win32(0)
        } else {
            ServiceExitCode::ServiceSpecific(code)
        },
        checkpoint: u32::from(matches!(
            state,
            ServiceState::StartPending | ServiceState::StopPending
        )),
        wait_hint: if matches!(
            state,
            ServiceState::StartPending | ServiceState::StopPending
        ) {
            Duration::from_secs(30)
        } else {
            Duration::ZERO
        },
        process_id: None,
    }
}

fn run_service() -> anyhow::Result<()> {
    let bootstrap = BOOTSTRAP
        .get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Windows service bootstrap is unavailable"))?;
    let shutdown = CancellationToken::new();
    let control_shutdown = shutdown.clone();
    let event_handler = move |event| match event {
        ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
        ServiceControl::Stop | ServiceControl::Shutdown => {
            control_shutdown.cancel();
            ServiceControlHandlerResult::NoError
        }
        _ => ServiceControlHandlerResult::NotImplemented,
    };
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)
        .map_err(|error| anyhow::anyhow!("registering Windows service control handler: {error}"))?;
    status_handle.set_service_status(status(
        ServiceState::StartPending,
        ServiceControlAccept::empty(),
        0,
    ))?;

    let otel_provider = match uar::telemetry::init(&bootstrap.log_format) {
        Ok(provider) => provider,
        Err(error) => {
            status_handle.set_service_status(status(
                ServiceState::Stopped,
                ServiceControlAccept::empty(),
                1,
            ))?;
            return Err(error);
        }
    };
    uar::telemetry::metrics::init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| anyhow::anyhow!("creating Windows service runtime: {error}"))?;
    let strict_config = bootstrap.cli.strict_config;
    let config_manager = runtime.block_on(ConfigManager::load(bootstrap.cli))?;
    if strict_config {
        config_manager.set_strict(true);
    }

    status_handle.set_service_status(status(
        ServiceState::Running,
        ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        0,
    ))?;

    let monitor_shutdown = shutdown.clone();
    let result = runtime.block_on(async move {
        let server = super::start_server_with_shutdown(config_manager, shutdown);
        tokio::pin!(server);
        tokio::select! {
            result = &mut server => result,
            () = monitor_shutdown.cancelled() => {
                status_handle.set_service_status(status(
                    ServiceState::StopPending,
                    ServiceControlAccept::empty(),
                    0,
                ))?;
                server.await
            }
        }
    });

    if let Some(provider) = &otel_provider {
        let _ = provider.shutdown();
    }
    let exit_code = u32::from(result.is_err());
    status_handle.set_service_status(status(
        ServiceState::Stopped,
        ServiceControlAccept::empty(),
        exit_code,
    ))?;
    result
}

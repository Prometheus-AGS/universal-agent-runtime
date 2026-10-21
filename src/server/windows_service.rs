use std::ffi::OsString;
use std::sync::OnceLock;
use std::time::Duration;
use futures::FutureExt as _;

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
        // No estimated duration certifies initialization or release.
        wait_hint: Duration::ZERO,
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

    let result = (|| -> anyhow::Result<()> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|error| anyhow::anyhow!("creating Windows service runtime: {error}"))?;
        runtime.block_on(async {
            let otel_provider = uar::telemetry::init(
                &bootstrap.log_format, "info,universal_agent_runtime=debug",
            )?;
            let mut configuration_owner = None;
            let outcome = std::panic::AssertUnwindSafe(async {
                uar::telemetry::metrics::init();
                let strict_config = bootstrap.cli.strict_config;
                let config_manager = ConfigManager::load(bootstrap.cli).await?;
                configuration_owner = Some(std::sync::Arc::clone(&config_manager));
                if strict_config { config_manager.set_strict(true); }
                let (ready_tx, mut ready_rx) = tokio::sync::oneshot::channel();
                let server = super::start_server_with_listener(
                    config_manager, None, None, None, Some(ready_tx), None,
                    Some(shutdown.clone()), false, None, None,
                );
                tokio::pin!(server);
                let mut status_errors = Vec::new();
                let server_result = tokio::select! {
                    result = &mut server => result,
                    ready = &mut ready_rx => {
                        if ready.is_err() {
                            status_errors.push("Windows service readiness was not acknowledged".to_owned());
                            shutdown.cancel();
                        } else if let Err(error) = status_handle.set_service_status(status(
                            ServiceState::Running,
                            ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
                            0,
                        )) {
                            status_errors.push(format!("publishing Windows service readiness: {error}"));
                            shutdown.cancel();
                        }
                        tokio::select! {
                            result = &mut server => result,
                            () = shutdown.cancelled() => {
                                if let Err(error) = status_handle.set_service_status(status(
                                    ServiceState::StopPending, ServiceControlAccept::empty(), 0,
                                )) {
                                    status_errors.push(format!("publishing Windows service shutdown: {error}"));
                                }
                                server.await
                            }
                        }
                    },
                    () = shutdown.cancelled() => {
                        if let Err(error) = status_handle.set_service_status(status(
                            ServiceState::StopPending, ServiceControlAccept::empty(), 0,
                        )) {
                            status_errors.push(format!("publishing Windows service shutdown: {error}"));
                        }
                        server.await
                    }
                };
                if let Err(error) = server_result { status_errors.push(error.to_string()); }
                anyhow::ensure!(status_errors.is_empty(), "{}", status_errors.join("; "));
                Ok::<(), anyhow::Error>(())
            }).catch_unwind().await;
            let mut failures = Vec::new();
            match outcome {
                Ok(Ok(())) => {},
                Ok(Err(error)) => failures.push(error.to_string()),
                Err(_) => failures.push("Windows service startup/runtime panicked".to_owned()),
            }
            if let Some(configuration) = configuration_owner {
                match std::panic::AssertUnwindSafe(configuration.close_watchers()).catch_unwind().await {
                    Ok(Ok(())) => {},
                    Ok(Err(error)) => failures.push(format!("configuration cleanup: {error}")),
                    Err(_) => failures.push("configuration cleanup panicked; release unconfirmed".to_owned()),
                }
            }
            if let Some(provider) = &otel_provider {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| provider.shutdown())) {
                    Ok(Ok(())) => {},
                    Ok(Err(error)) => failures.push(format!("telemetry cleanup: {error}")),
                    Err(_) => failures.push("telemetry cleanup panicked; release unconfirmed".to_owned()),
                }
            }
            anyhow::ensure!(failures.is_empty(), "{}", failures.join("; "));
            Ok(())
        })
    })();
    let exit_code = u32::from(result.is_err());
    let stopped = status_handle.set_service_status(status(
        ServiceState::Stopped,
        ServiceControlAccept::empty(),
        exit_code,
    ));
    match (result, stopped) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error.into()),
        (Err(runtime), Err(status)) => Err(anyhow::anyhow!(
            "{runtime}; publishing Windows service stopped state: {status}"
        )),
    }
}

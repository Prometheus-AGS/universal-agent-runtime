//! OS subscriptions are acquired before database initialization, not on listen.

use crate::uar::runtime::lifecycle::RuntimeSupervisor;
use tokio_util::sync::CancellationToken;

struct ProcessSignals {
    #[cfg(unix)]
    interrupt: tokio::signal::unix::Signal,
    #[cfg(unix)]
    terminate: tokio::signal::unix::Signal,
    #[cfg(windows)]
    interrupt: tokio::signal::windows::CtrlC,
}

impl ProcessSignals {
    fn new() -> std::io::Result<Self> {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            Ok(Self {
                interrupt: signal(SignalKind::interrupt())?,
                terminate: signal(SignalKind::terminate())?,
            })
        }
        #[cfg(windows)]
        {
            Ok(Self { interrupt: tokio::signal::windows::ctrl_c()? })
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported,
                "OS shutdown signal subscriptions are unsupported on this host"))
        }
    }

    async fn receive(&mut self) -> anyhow::Result<()> {
        #[cfg(unix)]
        let received = tokio::select! {
            value = self.interrupt.recv() => value,
            value = self.terminate.recv() => value,
        };
        #[cfg(windows)]
        let received = self.interrupt.recv().await;
        #[cfg(not(any(unix, windows)))]
        let received: Option<()> = None;
        anyhow::ensure!(received.is_some(), "OS shutdown signal source closed unexpectedly");
        Ok(())
    }
}

pub(super) async fn install(
    supervisor: &RuntimeSupervisor,
    shutdown: CancellationToken,
) -> anyhow::Result<()> {
    let scope = supervisor.register("process-signals", "uar-process").await?;
    let mut signals = match ProcessSignals::new() {
        Ok(signals) => signals,
        Err(error) => {
            scope.record_failure(format!("OS shutdown signal subscription failed: {error}"));
            return Err(error.into());
        }
    };
    let stop = scope.cancellation_token();
    scope.spawn("os-shutdown-signals", async move {
        let _shutdown_on_failure_or_signal = shutdown.clone().drop_guard();
        tokio::select! {
            result = signals.receive() => result,
            () = stop.cancelled() => Ok(()),
        }
    })?;
    scope.mark_ready()?;
    Ok(())
}

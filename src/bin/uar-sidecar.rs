//! Electron sidecar entry-point for the Universal Agent Runtime.
//!
//! Differences from `main.rs`:
//! - Binds to `127.0.0.1:0` so the OS assigns a free ephemeral port.
//! - Retains the OS-assigned listener through initialization and emits exactly
//!   one `READY:{port}` line only after the HTTP application is ready to serve.
//!   The Electron main process reads this line from the child's stdout pipe.
//! - Spawns a background task that reads stdin; when stdin reaches EOF
//!   (Electron closes the pipe on quit) the process exits cleanly.  This is
//!   the cross-platform shutdown contract — it works on Windows where SIGTERM
//!   is unreliable.
//! - Forces JSON log format to avoid ANSI noise in the Electron logger.
//! - Defaults `security.jwt_required` to false, because a supervised sidecar is
//!   reachable only on its own loopback port and its parent authenticates
//!   separately. See [`should_disable_sidecar_jwt`].
//!
//! Process environment writes are confined to [`prepare_sidecar_process`],
//! which runs *before* the Tokio runtime exists. `std::env::set_var` is
//! `unsafe` in Rust 2024 precisely because it is a data race once other threads
//! can read the environment, so doing this work inside `#[tokio::main]` — after
//! the runtime has spawned its workers — is unsound. `main` is deliberately
//! synchronous and builds the runtime by hand.

use std::io::Write as _;
use std::net::SocketAddr;
use std::pin::Pin;

use clap::Parser as _;
use dotenvy::dotenv;
use tokio::io::AsyncReadExt as _;
use universal_agent_runtime::config::{Cli, LogFormat};
use universal_agent_runtime::config_manager::ConfigManager;
use universal_agent_runtime::server;
use universal_agent_runtime::uar;

/// Decide whether the sidecar should default JWT enforcement to off.
///
/// A supervised sidecar listens only on an OS-assigned loopback port. The
/// parent process authenticates its own operator API and then talks to this
/// child directly, so requiring an unrelated UAR JWT makes the local process
/// contract unusable — the parent gets 401s from a server it launched itself.
///
/// The default is only applied when the operator has expressed no opinion. Any
/// explicit setting, through either the current `UAR_SECURITY__JWT_REQUIRED` or
/// the legacy `JWT_REQUIRED`, is honoured as written — including an explicit
/// `true`, which is how a hardened deployment keeps enforcement on.
///
/// # Examples
///
/// ```ignore
/// use std::ffi::OsStr;
/// // No operator opinion: the sidecar relaxes the default.
/// assert!(should_disable_sidecar_jwt(None, None));
/// // Explicit settings win, in either variable.
/// assert!(!should_disable_sidecar_jwt(Some(OsStr::new("true")), None));
/// assert!(!should_disable_sidecar_jwt(None, Some(OsStr::new("false"))));
/// ```
fn should_disable_sidecar_jwt(
    uar_jwt_required: Option<&std::ffi::OsStr>,
    legacy_jwt_required: Option<&std::ffi::OsStr>,
) -> bool {
    uar_jwt_required.is_none() && legacy_jwt_required.is_none()
}

/// The listener bound during synchronous bootstrap, handed to the runtime.
struct SidecarBootstrap {
    listener: std::net::TcpListener,
}

/// Synchronous pre-runtime bootstrap: every environment write lives here.
///
/// # Errors
///
/// Returns an error if the loopback listener cannot be bound or its assigned
/// address cannot be read.
fn prepare_sidecar_process() -> anyhow::Result<SidecarBootstrap> {
    let _ = dotenv();

    let configured_uar_jwt = std::env::var_os("UAR_SECURITY__JWT_REQUIRED");
    let configured_legacy_jwt = std::env::var_os("JWT_REQUIRED");
    let disable_sidecar_jwt = should_disable_sidecar_jwt(
        configured_uar_jwt.as_deref(),
        configured_legacy_jwt.as_deref(),
    );

    // Force loopback-only binding and JSON logs before config is loaded so the
    // config layer picks them up via env-var overrides.
    //
    // UAR_SIDECAR=1 is set by UarSidecarService so the server can detect
    // sidecar mode if needed (e.g. for CORS); we set it here as a fallback for
    // direct invocations.
    //
    // SAFETY: this synchronous bootstrap runs before the Tokio runtime or any
    // application worker threads are created, so no other thread can be reading
    // the environment concurrently. All process-environment writes are
    // deliberately confined to this pre-runtime section.
    unsafe {
        if std::env::var("UAR_SIDECAR").is_err() {
            std::env::set_var("UAR_SIDECAR", "1");
        }
        std::env::set_var("UAR_SERVER__HOST", "127.0.0.1");
        std::env::set_var("UAR_SERVER__LOG_FORMAT", "json");
        if disable_sidecar_jwt {
            std::env::set_var("UAR_SECURITY__JWT_REQUIRED", "false");
        }
    }

    // Bind once and retain ownership until Axum begins serving. This removes
    // the port-stealing race that existed when startup dropped and re-bound the
    // OS-assigned listener. Bound with std here (rather than tokio) because the
    // runtime does not exist yet; converted in `run_sidecar`.
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let port = listener
        .local_addr()
        .map_err(|error| anyhow::anyhow!("Failed to read UAR sidecar listener address: {error}"))?
        .port();
    // SAFETY: still in the synchronous pre-runtime bootstrap described above.
    unsafe {
        std::env::set_var("UAR_SERVER__PORT", port.to_string());
    }

    Ok(SidecarBootstrap { listener })
}

/// Await readiness, failing fast if the server stops first.
///
/// # Errors
///
/// Returns an error if the server exits — successfully or not — before it
/// signals readiness.
async fn await_server_readiness<F>(
    mut ready: tokio::sync::oneshot::Receiver<SocketAddr>,
    server: Pin<&mut F>,
) -> anyhow::Result<SocketAddr>
where
    F: std::future::Future<Output = anyhow::Result<()>>,
{
    tokio::select! {
        ready = &mut ready => ready
            .map_err(|_| anyhow::anyhow!("UAR sidecar stopped before reporting readiness")),
        result = server => match result {
            Ok(()) => Err(anyhow::anyhow!("sidecar stopped before reporting readiness")),
            Err(error) => Err(anyhow::anyhow!(
                "UAR sidecar failed during startup: {error:#}"
            )),
        },
    }
}

fn main() {
    let bootstrap = match prepare_sidecar_process() {
        Ok(bootstrap) => bootstrap,
        Err(error) => {
            eprintln!("Failed to prepare UAR sidecar: {error:#}");
            std::process::exit(1);
        }
    };
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create UAR sidecar async runtime");
    runtime.block_on(run_sidecar(bootstrap));
}

async fn run_sidecar(bootstrap: SidecarBootstrap) {
    let log_format = LogFormat::Json;
    let otel_provider = uar::telemetry::init(&log_format);
    uar::telemetry::metrics::init();

    let listener = tokio::net::TcpListener::from_std(bootstrap.listener)
        .expect("Failed to register UAR sidecar listener with async runtime");

    let config_manager = match ConfigManager::load(Cli::parse()).await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to load configuration: {:?}", e);
            std::process::exit(1);
        }
    };

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let server = server::start_server_sidecar(config_manager, listener, ready_tx, None);
    tokio::pin!(server);
    let ready_addr = match await_server_readiness(ready_rx, server.as_mut()).await {
        Ok(addr) => addr,
        Err(error) => {
            tracing::error!(error = %error, "UAR sidecar failed during startup");
            std::process::exit(1);
        }
    };

    // Emit readiness only after the runtime and HTTP application initialize.
    // UarSidecarService's readline handler watches for this exact format.
    let ready_line = format!("READY:{}\n", ready_addr.port());
    std::io::stdout()
        .write_all(ready_line.as_bytes())
        .expect("Failed to write READY signal to stdout");
    std::io::stdout().flush().expect("Failed to flush stdout");

    // Spawn a task that reads stdin until EOF, then exits the process.
    // Electron closes the child's stdin pipe on app quit, triggering this path.
    tokio::spawn(async move {
        let mut stdin = tokio::io::stdin();
        let mut buf = [0u8; 1];
        // Block until stdin is closed (EOF) or errors out.
        loop {
            match stdin.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {} // ignore any bytes written to stdin
            }
        }
        tracing::info!(name = "sidecar.stdin_eof", "stdin closed — exiting");
        std::process::exit(0);
    });

    if let Err(error) = server.await {
        tracing::error!(error = %error, "UAR sidecar error");
        std::process::exit(1);
    }

    if let Some(provider) = &otel_provider {
        let _ = provider.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::should_disable_sidecar_jwt;
    use std::ffi::OsStr;

    #[test]
    fn disables_jwt_when_operator_expressed_no_opinion() {
        assert!(should_disable_sidecar_jwt(None, None));
    }

    #[test]
    fn honours_explicit_uar_setting_including_true() {
        assert!(!should_disable_sidecar_jwt(Some(OsStr::new("true")), None));
        assert!(!should_disable_sidecar_jwt(Some(OsStr::new("false")), None));
    }

    #[test]
    fn honours_legacy_jwt_required_variable() {
        assert!(!should_disable_sidecar_jwt(None, Some(OsStr::new("true"))));
        assert!(!should_disable_sidecar_jwt(
            Some(OsStr::new("false")),
            Some(OsStr::new("true"))
        ));
    }
}

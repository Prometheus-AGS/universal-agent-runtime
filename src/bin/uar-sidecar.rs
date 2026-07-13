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

use std::io::Write as _;
use std::sync::Arc;

use dotenvy::dotenv;
use tokio::io::AsyncReadExt as _;
use universal_agent_runtime::config::{AppConfig, LogFormat};
use universal_agent_runtime::server;
use universal_agent_runtime::uar;

#[tokio::main]
async fn main() {
    let _ = dotenv();

    // Force loopback-only binding and JSON logs before config is loaded so the
    // config layer picks them up via env-var overrides.
    //
    // UAR_SIDECAR=1 is set by UarSidecarService so the server can detect
    // sidecar mode if needed (e.g. for CORS); we set it here as a fallback for
    // direct invocations.
    //
    // SAFETY: single-threaded at this point; no other threads have spawned yet.
    unsafe {
        if std::env::var("UAR_SIDECAR").is_err() {
            std::env::set_var("UAR_SIDECAR", "1");
        }
        std::env::set_var("UAR_SERVER__HOST", "127.0.0.1");
        std::env::set_var("UAR_SERVER__LOG_FORMAT", "json");
    }

    let log_format = LogFormat::Json;
    let otel_provider = uar::telemetry::init(&log_format);
    uar::telemetry::metrics::init();

    // Bind once and retain ownership until Axum begins serving. This removes
    // the port-stealing race that existed when startup dropped and re-bound the
    // OS-assigned listener.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind UAR sidecar listener");
    let port = listener
        .local_addr()
        .expect("Failed to read UAR sidecar listener address")
        .port();
    // SAFETY: still single-threaded; tokio runtime has not yet spawned workers.
    unsafe {
        std::env::set_var("UAR_SERVER__PORT", port.to_string());
    }

    let config = match AppConfig::load() {
        Ok(c) => Arc::new(c),
        Err(e) => {
            tracing::error!("Failed to load configuration: {:?}", e);
            std::process::exit(1);
        }
    };

    let (ready_tx, mut ready_rx) = tokio::sync::oneshot::channel();
    let server = server::start_server_sidecar(config, listener, ready_tx);
    tokio::pin!(server);
    let ready_addr = tokio::select! {
        ready = &mut ready_rx => match ready {
            Ok(addr) => addr,
            Err(_) => {
                tracing::error!("UAR sidecar stopped before reporting readiness");
                std::process::exit(1);
            }
        },
        result = &mut server => {
            let error = match result {
                Ok(()) => anyhow::anyhow!("sidecar stopped before reporting readiness"),
                Err(error) => error,
            };
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

    match server.await {
        Ok(()) => {}
        Err(error) => {
            tracing::error!(error = %error, "UAR sidecar error");
            std::process::exit(1);
        }
    }

    if let Some(provider) = &otel_provider {
        let _ = provider.shutdown();
    }
}

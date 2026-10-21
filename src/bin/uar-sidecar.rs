#![recursion_limit = "256"]

//! Electron sidecar entry-point for the Universal Agent Runtime.
//!
//! Differences from `main.rs`:
//! - Reads a 256-bit launch token (64 lowercase hex characters) as the first
//!   line of stdin before binding anything, and requires it on every request.
//!   A missing or malformed line exits with status 2. The token is held only
//!   in memory; it is never read from the environment, argv or a file.
//! - Binds to IPv4 loopback at the requested port, defaulting to 1906, and
//!   advances one port at a time while a candidate is occupied.
//! - Retains the selected listener through initialization and emits exactly
//!   one `READY:{port}` line only after the HTTP application is ready to serve.
//!   The Electron main process reads this line from the child's stdout pipe.
//! - Owns a cancellable, joined control task that reads stdin; when stdin reaches EOF
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
#[path = "uar-sidecar/inherited_input.rs"]
mod inherited_input;
#[path = "uar-sidecar/control.rs"]
mod control;

use clap::Parser as _;
use dotenvy::dotenv;
use secrecy::zeroize::Zeroize as _;
use futures::FutureExt as _;
use universal_agent_runtime::config::{Cli, LogFormat};
use universal_agent_runtime::config_manager::ConfigManager;
use universal_agent_runtime::server;
use universal_agent_runtime::uar;
use universal_agent_runtime::uar::security::sidecar_guard::{
    InvalidBoundPrincipal, InvalidLaunchToken, SidecarBoundPrincipal, SidecarLaunchToken,
};

/// Longest accepted token line: 64 hex characters, `\r`, `\n`.
const MAX_TOKEN_LINE_BYTES: usize = 66;

/// Exit status when the launch token is missing or malformed.
const INVALID_TOKEN_EXIT_STATUS: u8 = 2;

/// Launch-environment variable naming the one owner of a launch-bound sidecar.
/// Read only from the process environment set by the launching host.
const BOUND_PRINCIPAL_ENV: &str = "UAR_SIDECAR_BOUND_PRINCIPAL";

/// Exit status when the bound principal is present but not a valid subject.
const INVALID_BOUND_PRINCIPAL_EXIT_STATUS: u8 = 3;

#[derive(Debug)]
struct LaunchRefusal(u8);
impl std::fmt::Display for LaunchRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("sidecar launch authorization rejected")
    }
}
impl std::error::Error for LaunchRefusal {}

/// Parse the optional launch-bound principal from its raw environment value.
///
/// Absent means the shared sidecar. A present value must be UTF-8 and satisfy
/// the `x-uar-principal` rule; an empty or invalid value fails closed.
///
/// # Errors
///
/// Returns [`InvalidBoundPrincipal`] for a non-UTF-8 or invalid value.
fn bound_principal_from_env(
    raw: Option<&std::ffi::OsStr>,
) -> Result<Option<SidecarBoundPrincipal>, InvalidBoundPrincipal> {
    raw.map(|value| {
        value
            .to_str()
            .ok_or(InvalidBoundPrincipal)
            .and_then(SidecarBoundPrincipal::parse)
    })
    .transpose()
}

/// Settings forced off because they can mix content across host sessions.
///
/// Skill evolution is always off. Memory is forced off only for the shared
/// sidecar, where one process serves many host sessions and owners. A
/// launch-bound sidecar has exactly one owner, so memory follows configuration.
fn forced_session_mixing_env(launch_bound: bool) -> &'static [(&'static str, &'static str)] {
    if launch_bound {
        &[("UAR_SKILL_EVOLUTION__ENABLED", "false")]
    } else {
        &[("UAR_SKILL_EVOLUTION__ENABLED", "false"), ("UAR_MEMORY__ENABLED", "false")]
    }
}

/// Read the launch token from the first stdin line.
///
/// The read is bounded, so a host that never sends a newline cannot make the
/// sidecar buffer without limit. The raw bytes are zeroized after parsing.
/// Bootstrap reads exactly one byte at a time; later control bytes remain in
/// the owned inherited pipe for the cancellable runtime reader.
///
/// # Errors
///
/// Returns [`InvalidLaunchToken`] on end-of-file, a read error, or a line that
/// is not exactly a launch token.
fn read_launch_token(input: &mut inherited_input::Input) -> Result<SidecarLaunchToken, InvalidLaunchToken> {
    let mut line = input.read_token_line(MAX_TOKEN_LINE_BYTES).map_err(|_| InvalidLaunchToken)?;
    let token = SidecarLaunchToken::from_line(&line);
    line.zeroize();
    token
}

/// Decide whether the sidecar should default JWT enforcement to off.
///
/// A supervised sidecar listens only on its selected loopback port. The
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

/// The launch token and the listener bound during synchronous bootstrap,
/// handed to the runtime.
struct SidecarBootstrap {
    cli: Cli,
    token: SidecarLaunchToken,
    bound_principal: Option<SidecarBoundPrincipal>,
    listener: std::net::TcpListener,
    input: inherited_input::Input,
}

const DEFAULT_SIDECAR_PORT: u16 = 1906;

fn bind_sidecar_listener(preferred_port: u16) -> std::io::Result<std::net::TcpListener> {
    let start = if preferred_port == 0 {
        DEFAULT_SIDECAR_PORT
    } else {
        preferred_port
    };
    for port in start..=u16::MAX {
        match std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)) {
            Ok(listener) => return Ok(listener),
            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AddrInUse,
        format!("no loopback port is available at or above {start}"),
    ))
}

/// Synchronous pre-runtime bootstrap: every environment write lives here.
///
/// Windows launchers must provide inherited named-pipe stdin opened with
/// FILE_FLAG_OVERLAPPED and declare UAR_SIDECAR_STDIN_OVERLAPPED=1. The legacy
/// blocking-stdin channel is rejected before resource acquisition.
/// The launch token is read after the environment writes and before any socket
/// is bound. A missing or malformed token ends the process with status 2 and a
/// fixed message that never echoes the received bytes.
///
/// # Errors
///
/// Returns an error if the loopback listener cannot be bound or its assigned
/// address cannot be read.
fn prepare_sidecar_process() -> anyhow::Result<SidecarBootstrap> {
    let _ = dotenv();
    let mut cli = Cli::parse();

    let configured_uar_jwt = std::env::var_os("UAR_SECURITY__JWT_REQUIRED");
    let configured_legacy_jwt = std::env::var_os("JWT_REQUIRED");
    let configured_service_ownership = std::env::var_os("UAR_SERVICE_INSTANCE__OWNERSHIP");
    let disable_sidecar_jwt = should_disable_sidecar_jwt(
        configured_uar_jwt.as_deref(),
        configured_legacy_jwt.as_deref(),
    );
    let session_retention_unconfigured = std::env::var_os("UAR_SESSIONS__IDLE_TIMEOUT_SECS")
        .is_none()
        && std::env::var_os("UAR_SESSIONS__MAX_RETAINED").is_none();
    // Fail closed before any socket is bound; never echo the rejected value.
    let bound_principal =
        match bound_principal_from_env(std::env::var_os(BOUND_PRINCIPAL_ENV).as_deref()) {
            Ok(principal) => principal,
            Err(_) => {
                eprintln!(
                    "UAR sidecar refused to start: {BOUND_PRINCIPAL_ENV} is not a valid host principal"
                );
                return Err(LaunchRefusal(INVALID_BOUND_PRINCIPAL_EXIT_STATUS).into());
            }
        };

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
        // Ownership is relative to the connecting host; another supervisor can
        // explicitly expose this packaged executable as external to The Boss.
        if configured_service_ownership.is_none() {
            std::env::set_var("UAR_SERVICE_INSTANCE__OWNERSHIP", "managed");
        }
        std::env::set_var("UAR_SERVICE_INSTANCE__WORKSPACE_LOCATION", "local");
        if disable_sidecar_jwt {
            std::env::set_var("UAR_SECURITY__JWT_REQUIRED", "false");
        }
        if session_retention_unconfigured {
            std::env::set_var("UAR_SESSIONS__IDLE_TIMEOUT_SECS", "1800");
            std::env::set_var("UAR_SESSIONS__MAX_RETAINED", "1000");
        }
        // A shared sidecar serves every host session, so features that mix
        // content across sessions stay off regardless of configuration files.
        // Skill evolution is off in every mode. Memory is forced off only when
        // no principal is bound; a launch-bound sidecar serves exactly one
        // owner, so its memory follows configuration.
        for (key, value) in forced_session_mixing_env(bound_principal.is_some()) {
            std::env::set_var(key, value);
        }
    }

    let mut input = inherited_input::Input::open()?;
    let token = match read_launch_token(&mut input) {
        Ok(token) => token,
        Err(error) => {
            eprintln!("UAR sidecar refused to start: {error}");
            return Err(LaunchRefusal(INVALID_TOKEN_EXIT_STATUS).into());
        }
    };

    // Bind once and retain ownership until Axum begins serving. Start at the
    // operator's preferred port and advance through occupied ports without a
    // probe-then-bind race. Bound with std because the runtime does not exist.
    let listener = bind_sidecar_listener(cli.port.unwrap_or(DEFAULT_SIDECAR_PORT))?;
    listener.set_nonblocking(true)?;
    let port = listener
        .local_addr()
        .map_err(|error| anyhow::anyhow!("Failed to read UAR sidecar listener address: {error}"))?
        .port();
    // The CLI value is the preferred starting point during sidecar bootstrap.
    // From this point on, the runtime configuration must describe the listener
    // we actually retained after conflict resolution.
    cli.port = Some(port);
    // SAFETY: still in the synchronous pre-runtime bootstrap described above.
    unsafe {
        std::env::set_var("UAR_SERVER__PORT", port.to_string());
    }

    Ok(SidecarBootstrap {
        cli,
        token,
        bound_principal,
        listener,
        input,
    })
}

fn main() -> std::process::ExitCode {
    #[cfg(feature = "tract-embeddings")]
    ort::set_api(ort_tract::api());

    let bootstrap = match prepare_sidecar_process() {
        Ok(bootstrap) => bootstrap,
        Err(error) => {
            eprintln!("Failed to prepare UAR sidecar: {error:#}");
            return std::process::ExitCode::from(error.downcast_ref::<LaunchRefusal>().map_or(1, |error| error.0));
        }
    };
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(8 * 1024 * 1024).enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("Failed to create UAR sidecar async runtime: {error}");
            return std::process::ExitCode::FAILURE;
        }
    };
    match runtime.block_on(run_sidecar(bootstrap)) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("UAR sidecar failed after cleanup: {error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}

async fn run_sidecar(bootstrap: SidecarBootstrap) -> anyhow::Result<()> {
    let SidecarBootstrap { cli, token, bound_principal, listener, input } = bootstrap;
    let input = input.into_async()?;
    let otel_provider = match std::panic::catch_unwind(|| uar::telemetry::init(&LogFormat::Json, "info")) {
        Ok(Ok(provider)) => provider,
        outcome => {
            // The OS input worker is already acquired. Even telemetry startup
            // failure must acknowledge its cancellation before returning.
            let close = input.close().await;
            let initial = match outcome {
                Ok(Err(error)) => error.to_string(),
                Err(_) => "sidecar telemetry initialization panicked".to_owned(),
                Ok(Ok(_)) => unreachable!(),
            };
            return Err(match close {
                Ok(()) => anyhow::anyhow!(initial),
                Err(error) => anyhow::anyhow!("{initial}; input cleanup failed: {error}"),
            });
        }
    };
    let process_shutdown = tokio_util::sync::CancellationToken::new();
    let reader_stop = tokio_util::sync::CancellationToken::new();
    let host_control = server::ServerLifecycleControl::default();
    // Register the sole input owner before config/server initialization can await.
    // Parent EOF therefore requests cancellation during startup as well as serving.
    let reader = tokio::spawn(control::serve(input, host_control.clone(),
        process_shutdown.clone(), reader_stop.clone()));

    let result = std::panic::AssertUnwindSafe(async {
        uar::telemetry::metrics::init();
        let listener = tokio::net::TcpListener::from_std(listener)?;
        let config_manager = tokio::select! {
            biased;
            _ = process_shutdown.cancelled() => return Ok(()),
            result = ConfigManager::load(cli) => result?,
        };
        let (ready_tx, mut ready_rx) = tokio::sync::oneshot::channel();
        let server = server::start_server_sidecar_with_control(config_manager, listener,
            token, bound_principal, ready_tx, Some(process_shutdown.clone()), host_control);
        tokio::pin!(server);
        let ready = tokio::select! {
            result = server.as_mut() => {
                result?;
                if process_shutdown.is_cancelled() { return Ok(()); }
                return Err(anyhow::anyhow!("sidecar exited before readiness"));
            },
            result = &mut ready_rx => result,
        };
        let ready_addr = match ready {
            Ok(address) => address,
            Err(error) => {
                process_shutdown.cancel();
                let cleanup = server.await;
                cleanup?;
                return Err(anyhow::anyhow!("sidecar readiness channel closed: {error}"));
            }
        };
        // Keep the exact parent-compatible line. A write failure requests cleanup;
        // it never bypasses the supervisor by exiting the process.
        let announce = (|| -> std::io::Result<()> {
            let mut stdout = std::io::stdout().lock();
            writeln!(stdout, "READY:{}", ready_addr.port())?;
            stdout.flush()
        })();
        if let Err(error) = announce {
            process_shutdown.cancel();
            let cleanup = server.await;
            if let Err(cleanup) = cleanup { tracing::error!(error = %cleanup, "sidecar cleanup failed after output error"); }
            return Err(error.into());
        }
        server.await
    }).catch_unwind().await;

    reader_stop.cancel();
    let reader_result = reader.await;
    let telemetry_result = otel_provider.as_ref().map(|provider| provider.shutdown()).transpose();
    // Cleanup obligations settle before propagating the original failure/panic.
    if let Err(error) = &reader_result { tracing::error!(error = %error, "sidecar control task failed"); }
    if let Ok(Err(error)) = &reader_result { tracing::error!(error = %error, "sidecar control cleanup failed"); }
    if let Err(error) = &telemetry_result { tracing::error!(error = %error, "sidecar telemetry shutdown failed"); }
    result.map_err(|_| anyhow::anyhow!("sidecar startup or runtime panicked"))??;
    reader_result??;
    telemetry_result?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        InvalidBoundPrincipal, bound_principal_from_env, forced_session_mixing_env,
        should_disable_sidecar_jwt,
    };
    use std::ffi::OsStr;

    #[test]
    fn bound_principal_absent_keeps_shared_mode() {
        assert_eq!(bound_principal_from_env(None), Ok(None));
    }

    #[test]
    fn bound_principal_accepts_a_valid_subject() {
        let bound = bound_principal_from_env(Some(OsStr::new("alice")))
            .expect("valid subject must parse")
            .expect("present subject must bind");
        assert_eq!(bound.as_str(), "alice");
        let longest = "a".repeat(128);
        assert!(bound_principal_from_env(Some(OsStr::new(&longest))).is_ok());
    }

    #[test]
    fn bound_principal_rejects_anonymous_empty_overlong_and_bad_charset() {
        let overlong = "a".repeat(129);
        for rejected in [
            "anonymous",
            "ANONYMOUS",
            "",
            overlong.as_str(),
            "-alice",
            "alice bob",
            "alice/bob",
            "alice\n",
        ] {
            assert_eq!(
                bound_principal_from_env(Some(OsStr::new(rejected))),
                Err(InvalidBoundPrincipal),
                "a malformed bound principal must fail closed"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn bound_principal_rejects_non_utf8() {
        use std::os::unix::ffi::OsStrExt as _;
        assert_eq!(
            bound_principal_from_env(Some(OsStr::from_bytes(&[0x61, 0xff]))),
            Err(InvalidBoundPrincipal)
        );
    }

    #[test]
    fn bound_principal_mode_lets_memory_follow_configuration() {
        let forced = forced_session_mixing_env(true);
        assert!(!forced.iter().any(|(key, _)| *key == "UAR_MEMORY__ENABLED"));
        assert!(forced.contains(&("UAR_SKILL_EVOLUTION__ENABLED", "false")));
    }

    #[test]
    fn bound_principal_shared_mode_still_forces_memory_off() {
        let forced = forced_session_mixing_env(false);
        assert!(forced.contains(&("UAR_MEMORY__ENABLED", "false")));
        assert!(forced.contains(&("UAR_SKILL_EVOLUTION__ENABLED", "false")));
    }

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

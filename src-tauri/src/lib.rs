use clap::Parser as _;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tauri::Manager;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use universal_agent_runtime::config::Cli;
use universal_agent_runtime::config_manager::ConfigManager;
use universal_agent_runtime::{config, server};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            if let Ok(resource_dir) = app.path().resource_dir() {
                // SAFETY: still single-threaded at this point in `setup` — no
                // other threads have spawned yet that could race on env var
                // access.
                unsafe {
                    std::env::set_var("MCP_CONFIG_DIR", &resource_dir);
                    let mcp_config_path = resource_dir.join("mcp.json");
                    if mcp_config_path.exists() {
                        std::env::set_var("MCP_CONFIG_PATH", &mcp_config_path);
                    }
                    let mcp_server_dir = resource_dir.join("mcp-servers");
                    std::env::set_var("MCP_SERVER_DIR", &mcp_server_dir);
                }
            }

            let default_config = config::AppConfig::load().expect("Failed to load configuration");
            let host = "127.0.0.1".to_string();
            let app_config_dir = app.path().app_config_dir().ok();
            let port =
                resolve_localhost_port(default_config.server.port, app_config_dir.as_deref());
            let server_url = format!("http://{host}:{port}");

            let app_handle = app.handle();

            // Initialize the Axum server on its own dedicated OS thread with its
            // own current-thread Tokio runtime, rather than via
            // `tauri::async_runtime::spawn`. `server::start_server`'s future
            // captures tracing/log values that are not `Send` (they're never
            // spawned onto a multi-threaded executor by any other caller), so
            // `tauri::async_runtime::spawn`'s `Send + 'static` bound cannot be
            // satisfied; running it via `Runtime::block_on` on a dedicated
            // thread avoids that bound entirely. `ConfigManager` is the current
            // config entry point (`AppConfig::load()` above is only used to
            // read the default port before the resolved port is known);
            // resilience is forced off per the desktop/mobile requirement via
            // the same `Cli` overrides main.rs/uar-sidecar.rs use, and
            // host/port are set via the `UAR_SERVER__*` env vars
            // `ConfigManager` already reads.
            std::thread::spawn({
                let server_url = server_url.clone();
                move || {
                    log::info!("Starting embedded Axum server on {server_url}");

                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(rt) => rt,
                        Err(e) => {
                            log::error!("Failed to create Tokio runtime for embedded server: {e}");
                            return;
                        }
                    };

                    rt.block_on(async move {
                        // SAFETY: this thread is the only one that touches process
                        // env vars at this point in startup.
                        unsafe {
                            std::env::set_var("UAR_SERVER__HOST", "127.0.0.1");
                        }

                        let mut cli = Cli::parse_from(["uar-desktop"]);
                        cli.port = Some(port);
                        // FORCE DISABLE RESILIENCE FEATURES FOR DESKTOP/MOBILE APP
                        // User requirement: "turned off when run in tauri"
                        cli.rate_limit_enabled = Some(false);
                        cli.timeout_disabled = Some(true);

                        let config_manager = match ConfigManager::load(cli).await {
                            Ok(m) => m,
                            Err(e) => {
                                log::error!("Failed to load configuration: {e:?}");
                                return;
                            }
                        };

                        if let Err(e) = server::start_server(config_manager).await {
                            log::error!("Axum server failed: {}", e);
                        }
                    });
                }
            });

            // Navigate the webview to the localhost server once it's ready
            tauri::async_runtime::spawn({
                let app_handle = app_handle.clone();
                let host = host.clone();
                async move {
                    if wait_for_ready(&host, port, Duration::from_secs(20)).await {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ =
                                window.eval(&format!("window.location.replace('{}')", server_url));
                        }
                    } else {
                        log::error!("Timed out waiting for local server to start");
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// File name (under the Tauri app-config directory) that persists the
/// resolved fallback port across launches when the configured default port
/// is unavailable. Plain ASCII digits, no other content.
const PERSISTED_PORT_FILE: &str = "desktop-port.txt";

fn persisted_port_path(app_config_dir: &Path) -> PathBuf {
    app_config_dir.join(PERSISTED_PORT_FILE)
}

fn read_persisted_port(app_config_dir: &Path) -> Option<u16> {
    std::fs::read_to_string(persisted_port_path(app_config_dir))
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// Best-effort write; a persistence failure must never abort startup, so
/// errors are logged and swallowed.
fn write_persisted_port(app_config_dir: &Path, port: u16) {
    if let Err(e) = std::fs::create_dir_all(app_config_dir) {
        log::warn!("Failed to create app config dir for port persistence: {e}");
        return;
    }
    if let Err(e) = std::fs::write(persisted_port_path(app_config_dir), port.to_string()) {
        log::warn!("Failed to persist resolved desktop port: {e}");
    }
}

/// Resolves the local port the embedded server binds to, and the webview is
/// navigated to. Priority order (highest first), so the resulting origin is
/// stable across launches wherever possible:
///
/// 1. `TAURI_LOCALHOST_PORT` env var, if set to a valid port — explicit
///    override, always wins.
/// 2. `fallback` (the configured `server.port`) — the normal, expected case;
///    returns immediately if it's free.
/// 3. A previously-persisted fallback port (see [`read_persisted_port`]), if
///    the configured port is taken but the persisted one still binds — keeps
///    the origin stable even across a real port conflict.
/// 4. A fresh OS-assigned ephemeral port, persisted via
///    [`write_persisted_port`] so step 3 can reuse it next launch.
fn resolve_localhost_port(fallback: u16, app_config_dir: Option<&Path>) -> u16 {
    if let Ok(port) = std::env::var("TAURI_LOCALHOST_PORT") {
        if let Ok(parsed) = port.parse::<u16>() {
            return parsed;
        }
    }

    if TcpListener::bind(("127.0.0.1", fallback)).is_ok() {
        return fallback;
    }

    // Configured port is unavailable — reuse a previously-persisted fallback
    // if it still binds, so the origin stays stable across this specific
    // conflict rather than re-randomizing on every launch.
    if let Some(dir) = app_config_dir {
        if let Some(persisted) = read_persisted_port(dir) {
            if TcpListener::bind(("127.0.0.1", persisted)).is_ok() {
                return persisted;
            }
        }
    }

    if let Ok(listener) = TcpListener::bind("127.0.0.1:0") {
        if let Ok(addr) = listener.local_addr() {
            let resolved = addr.port();
            if let Some(dir) = app_config_dir {
                write_persisted_port(dir, resolved);
            }
            return resolved;
        }
    }

    fallback
}

async fn wait_for_ready(host: &str, port: u16, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let address = format!("{host}:{port}");

    while Instant::now() < deadline {
        if let Ok(mut stream) = TcpStream::connect(&address).await {
            let request =
                format!("GET /readyz HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
            if stream.write_all(request.as_bytes()).await.is_ok() {
                let mut buf = [0_u8; 128];
                if let Ok(read) = stream.read(&mut buf).await {
                    let response = std::str::from_utf8(&buf[..read]).unwrap_or("");
                    if response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200")
                    {
                        return true;
                    }
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(150)).await;
    }

    false
}

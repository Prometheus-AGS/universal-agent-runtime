//! Listener surface in sidecar and standalone mode (tasks 2.13–2.15).

use std::path::{Path, PathBuf};

use crate::sidecar_process::{
    ConfigOptions, LaunchOptions, Workspace, launch_standalone, raw_request, render_config,
};
use crate::stub_llm::{FixtureSet, start_stub_llm};
use crate::{boot_sidecar, host};

const SPA_MARKER: &str = "<!doctype html><html><body>uar-operator-spa</body></html>";

/// A `static/` directory under `parent` that looks like the built operator SPA.
fn spa_fixture(parent: &Path) -> PathBuf {
    let root = parent.join("static");
    std::fs::create_dir_all(root.join("assets")).expect("create SPA fixture");
    std::fs::write(root.join("index.html"), SPA_MARKER).expect("write index.html");
    std::fs::write(root.join("assets").join("index.js"), "console.log('spa');")
        .expect("write index.js");
    std::fs::write(root.join("favicon.svg"), "<svg></svg>").expect("write favicon");
    root
}

/// Listening TCP sockets owned by `pid`, as `address:port` strings.
fn listening_sockets(pid: u32) -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("lsof")
            .args(["-nP", "-a", "-p", &pid.to_string(), "-iTCP", "-sTCP:LISTEN"])
            .output()
            .expect("run lsof");
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .skip(1)
            .filter_map(|line| {
                line.split_whitespace()
                    .rev()
                    .find(|field| field.contains(':') && !field.starts_with('('))
                    .map(str::to_owned)
            })
            .collect()
    }
    #[cfg(target_os = "linux")]
    {
        linux_listening_sockets(pid)
    }
    #[cfg(windows)]
    {
        let script = format!(
            "Get-NetTCPConnection -State Listen -OwningProcess {pid} | \
             ForEach-Object {{ \"$($_.LocalAddress):$($_.LocalPort)\" }}"
        );
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .expect("run powershell");
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect()
    }
}

#[cfg(target_os = "linux")]
fn linux_listening_sockets(pid: u32) -> Vec<String> {
    let inodes: std::collections::BTreeSet<String> = std::fs::read_dir(format!("/proc/{pid}/fd"))
        .expect("read process fds")
        .flatten()
        .filter_map(|entry| std::fs::read_link(entry.path()).ok())
        .filter_map(|target| {
            target
                .to_string_lossy()
                .strip_prefix("socket:[")
                .and_then(|rest| rest.strip_suffix(']'))
                .map(str::to_owned)
        })
        .collect();
    let mut sockets = Vec::new();
    for (table, ipv6) in [("/proc/net/tcp", false), ("/proc/net/tcp6", true)] {
        let Ok(contents) = std::fs::read_to_string(table) else {
            continue;
        };
        for line in contents.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 10 || fields[3] != "0A" || !inodes.contains(fields[9]) {
                continue;
            }
            sockets.push(decode_proc_address(fields[1], ipv6));
        }
    }
    sockets
}

#[cfg(target_os = "linux")]
fn decode_proc_address(field: &str, ipv6: bool) -> String {
    let (address, port) = field.split_once(':').expect("proc address");
    let port = u16::from_str_radix(port, 16).expect("proc port");
    let bytes: Vec<u8> = (0..address.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&address[index..index + 2], 16).expect("hex"))
        .collect();
    if ipv6 {
        let mut octets = [0_u8; 16];
        for (word, chunk) in bytes.chunks(4).enumerate() {
            for (offset, byte) in chunk.iter().rev().enumerate() {
                octets[word * 4 + offset] = *byte;
            }
        }
        format!("[{}]:{port}", std::net::Ipv6Addr::from(octets))
    } else {
        let octets = [bytes[3], bytes[2], bytes[1], bytes[0]];
        format!("{}:{port}", std::net::Ipv4Addr::from(octets))
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_owns_exactly_one_listening_socket() {
    let sidecar = boot_sidecar(FixtureSet::new(), "", |options| options).await;
    let sockets = listening_sockets(sidecar.process.pid());
    assert_eq!(
        sockets,
        vec![format!("127.0.0.1:{}", sidecar.port())],
        "listening sockets"
    );
}

fn html_like(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    lower.contains("<html") || lower.contains("<!doctype")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn operator_ui_not_served_in_sidecar_mode() {
    let mut static_dir = PathBuf::new();
    let sidecar = boot_sidecar(FixtureSet::new(), "", |options| {
        static_dir = spa_fixture(options.config_path.parent().expect("config has a parent"));
        options.env("UAR_STATIC_DIR", static_dir.to_string_lossy().into_owned())
    })
    .await;
    assert!(static_dir.join("index.html").exists(), "fixture present");
    for path in [
        "/",
        "/threads",
        "/admin",
        "/admin/x",
        "/assets/index.js",
        "/favicon.svg",
    ] {
        let response = raw_request(
            sidecar.port(),
            "GET",
            path,
            &[
                ("Host", &sidecar.host()),
                ("Authorization", &sidecar.bearer()),
            ],
        )
        .await;
        assert_eq!(response.status, 404, "{path}");
        assert!(
            !html_like(&response.body_text()),
            "{path} served SPA content"
        );
    }
}

/// Regression guard: the standalone server keeps its companion listener,
/// A2A gRPC listener and SPA.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn standalone_server_keeps_companion_grpc_and_spa() {
    let stub = start_stub_llm(FixtureSet::new()).await;
    let workspace = Workspace::new();
    let static_dir = spa_fixture(workspace.path());
    let options = ConfigOptions::new(&stub.base_url);
    let (port, grpc_port) = (options.port, options.grpc_port);
    let config = workspace.write_config("config.yaml", &render_config(&workspace, &options));
    let launch = LaunchOptions::new(config, "standalone")
        .env("UAR_STATIC_DIR", static_dir.to_string_lossy().into_owned());
    let mut standalone = launch_standalone(&workspace, &launch, port).await;

    let ipv6_loopback = std::net::TcpListener::bind("[::1]:0").is_ok();
    if ipv6_loopback {
        assert!(
            std::net::TcpStream::connect(("::1", port)).is_ok(),
            "companion [::1]:{port} not listening"
        );
    }
    assert!(
        std::net::TcpStream::connect(("127.0.0.1", grpc_port)).is_ok(),
        "A2A gRPC listener 127.0.0.1:{grpc_port} not listening"
    );
    let index = raw_request(port, "GET", "/", &[("Host", &host(port))]).await;
    assert_eq!(index.status, 200, "SPA index");
    assert!(
        index.body_text().contains("uar-operator-spa"),
        "SPA content"
    );

    let status = standalone.stop_standalone().await;
    assert!(status.success(), "standalone exit: {status}");
}

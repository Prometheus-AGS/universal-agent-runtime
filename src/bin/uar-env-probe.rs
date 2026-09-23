//! Test-only child process: records what a process spawned by UAR can observe.
//!
//! Usage: `uar-env-probe <output-path> [--mcp] [--linger-ms <n>]`
//!
//! Appends one JSON line to `<output-path>` with the process id, its argv, its
//! complete environment, and the outcome of one bounded read from stdin
//! (`"eof"`, `"bytes:<n>"`, `"timeout"` or `"error"`). With `--mcp` the probe
//! skips the stdin read (stdin is the MCP protocol pipe) and then serves a
//! minimal MCP stdio server with one tool, `probe`, which exits the process
//! when called so a client observes transport loss. `--linger-ms` keeps the
//! process alive after recording so a process-table inspector can read it.
//!
//! Built only with the `test-probes` feature. Never ship it.

use std::io::{BufRead as _, Read as _, Write as _};
use std::sync::mpsc;
use std::time::Duration;

use serde_json::{Value, json};

const STDIN_READ_TIMEOUT: Duration = Duration::from_secs(1);

fn main() {
    let argv: Vec<String> = std::env::args_os()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect();
    let Some(output_path) = argv.get(1).cloned() else {
        eprintln!("usage: uar-env-probe <output-path> [--mcp]");
        std::process::exit(2);
    };
    let mcp_mode = argv.iter().skip(2).any(|argument| argument == "--mcp");

    let stdin = if mcp_mode {
        "protocol".to_owned()
    } else {
        probe_stdin()
    };
    let environment: serde_json::Map<String, Value> = std::env::vars_os()
        .map(|(key, value)| {
            (
                key.to_string_lossy().into_owned(),
                Value::String(value.to_string_lossy().into_owned()),
            )
        })
        .collect();
    let record = json!({
        "pid": std::process::id(),
        "argv": argv,
        "env": environment,
        "stdin": stdin,
    });
    if let Err(error) = append_record(&output_path, &record) {
        eprintln!("uar-env-probe: cannot write record: {error}");
        std::process::exit(3);
    }

    if mcp_mode {
        serve_mcp();
    }
    let linger = argv
        .iter()
        .position(|argument| argument == "--linger-ms")
        .and_then(|index| argv.get(index + 1))
        .and_then(|value| value.parse::<u64>().ok());
    if let Some(milliseconds) = linger {
        std::thread::sleep(Duration::from_millis(milliseconds));
    }
}

/// One bounded read. A blocked read on an open, empty pipe reports `timeout`.
fn probe_stdin() -> String {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buffer = [0_u8; 256];
        let outcome = match std::io::stdin().read(&mut buffer) {
            Ok(0) => "eof".to_owned(),
            Ok(count) => format!("bytes:{count}"),
            Err(_) => "error".to_owned(),
        };
        let _ = sender.send(outcome);
    });
    receiver
        .recv_timeout(STDIN_READ_TIMEOUT)
        .unwrap_or_else(|_| "timeout".to_owned())
}

fn append_record(path: &str, record: &Value) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let mut line = record.to_string();
    line.push('\n');
    file.write_all(line.as_bytes())?;
    file.flush()
}

fn serve_mcp() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };
        let Ok(message) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let Some(id) = message.get("id").cloned() else {
            continue;
        };
        let result = match message.get("method").and_then(Value::as_str) {
            Some("initialize") => json!({
                "protocolVersion": message
                    .pointer("/params/protocolVersion")
                    .cloned()
                    .unwrap_or_else(|| json!("2025-03-26")),
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "uar-env-probe", "version": "0.0.0" },
            }),
            Some("tools/list") => json!({
                "tools": [{
                    "name": "probe",
                    "description": "Records the child environment.",
                    "inputSchema": { "type": "object", "properties": {} },
                }],
            }),
            Some("tools/call") => std::process::exit(0),
            Some("ping") => json!({}),
            _ => {
                let error = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": "method not found" },
                });
                if writeln!(stdout, "{error}")
                    .and_then(|()| stdout.flush())
                    .is_err()
                {
                    break;
                }
                continue;
            }
        };
        let response = json!({ "jsonrpc": "2.0", "id": id, "result": result });
        if writeln!(stdout, "{response}")
            .and_then(|()| stdout.flush())
            .is_err()
        {
            break;
        }
    }
}

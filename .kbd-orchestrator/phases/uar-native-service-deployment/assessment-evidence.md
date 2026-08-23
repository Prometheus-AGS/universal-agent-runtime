# Assessment evidence

Date: 2026-08-23
Host: macOS arm64
Source: `1566df0404141aa64f095dcd3c45c36f059349cc`

| Claim | Command | Observed result | Limit |
|---|---|---|---|
| Port 1906 baseline | `lsof -nP -iTCP:1906 -sTCP:LISTEN` | No listener returned. | Point-in-time local host observation. |
| Existing install state | `find "$HOME/.uar" -maxdepth 3` | Found `config.yaml`, `data/`, `mcp.json`, `runtime/`, `static/`, and `uploads/`. | Names only; file contents and secrets excluded. |
| Local proxy models | `curl -fsS --max-time 5 http://127.0.0.1:8181/v1/models` | Returned `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, `gpt-5.4-nano`, `gpt-5.6-sol`, `gpt-5.6-terra`, and `gpt-5.6-luna`. | Provider availability may change; installer must rediscover instead of assuming this inventory forever. |
| Release binary baseline | `test -x target/release/universal-agent-runtime` | Release binary absent. | Does not assess buildability. |
| React bundle baseline | `test -f static/index.html && du -sh static` | Bundle present, approximately 39 MiB. | Existing bundle may not match the final source until rebuilt. |
| Current native configuration | Sanitized `~/.uar/config.yaml` inspection | HTTP host `127.0.0.1`, port `1906`, Surreal persistence, selected model `alibaba/qwen3.7-max`; no plaintext provider secrets were inspected or recorded. | Database-managed provider rows were not decoded during assessment. |

The repository `.prometheus/` directory is committed KBD history. Native service logs use machine-local paths that also contain a `.prometheus/logs` segment but are outside the repository checkout.
# Verified plan inputs — 2026-08-23

| Claim | Command | Observed output | Limit |
|---|---|---|---|
| Exact Windows SCM crate exists. | `cargo info windows-service@0.8.1` | crates.io reported version `0.8.1`, Rust version `1.71.0`, and `https://docs.rs/windows-service/0.8.1`. | Version/dependency metadata only; no runtime claim. |
| Release profile exists and carries the UI. | `sed -n '/^\\[features\\]/,/^\\[/p' Cargo.toml` | `server-full` includes `minimal`, A2A, telemetry, API docs, `admin-ui`, WASM, and the remaining server-full capabilities; `admin-ui` is documented as building/serving the React app. | Manifest evidence only. |
| Local platform tools. | `rustup target list --installed`; `command -v pwsh shellcheck plutil lsof` | Apple/iOS/Android/WASM targets installed; Windows target absent; PowerShell, ShellCheck, plutil, and lsof present; `systemd-analyze` absent. | Pre-execution inventory; the Windows Rust target must be added before compile verification. |

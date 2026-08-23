## 1. Inputs and telemetry

- [ ] 1.1 Add `--env-file` and `UAR_ENV_FILE`, load before telemetry/configuration, and fail closed on explicit file errors.
- [ ] 1.2 Add fail-closed `UAR_LOG_FILE` tracing output without changing stdout behavior when absent.

## 2. Lifecycle and listeners

- [ ] 2.1 Pin `windows-service = "=0.8.1"` for Windows targets and regenerate `Cargo.lock`.
- [ ] 2.2 Add the Windows-only `service` command, SCM status reporting, and Stop/Shutdown handler.
- [ ] 2.3 Add a crate-private process-scoped `start_server_with_shutdown` cancellation entrypoint, preserving the existing caller-owned HTTP-only token, and route Windows service cancellation through the new entrypoint into the existing graceful coordinator.
- [ ] 2.4 Make A2A gRPC resolve its socket from `server.host`; never fall back to wildcard.

## 3. Provider startup

- [ ] 3.1 Enrich YAML providers through the existing catalog path before initial persistence.

## 4. Cheap verification before the change-2 commit

- [ ] 4.1 Run `cargo check --locked -p universal-agent-runtime --no-default-features --features server-full` and package-scoped Clippy after source edits; record actual output.
- [ ] 4.2 Install the `x86_64-pc-windows-gnu` Rust target and run `cargo check --locked -p universal-agent-runtime --target x86_64-pc-windows-gnu --no-default-features --features server-full`; fix any Rust compile error before committing and record any linker-only limitation separately.
- [ ] 4.3 Confirm `Cargo.toml` reports package version `1.0.0`; if it does not, correct it within this change's owned manifest surface before committing.
- [ ] 4.4 Strict-validate the change before committing it independently.
- [ ] 4.5 Inspect the completed source for `Cli.env_file`, the `UAR_LOG_FILE` telemetry branch, Windows `Command::Service`, `start_server_with_shutdown`, gRPC resolution from `server.host`, and YAML `enrich_provider_config`; every named call site must exist before commit.

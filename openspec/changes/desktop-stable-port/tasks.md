# Tasks: desktop-stable-port

## 1. Raise the macOS build floor

- [x] 1.1 Raise macOS floor to 14.0 in `rust/.cargo/config.toml` (was forced 10.15)
- [x] 1.2 Raise macOS floor to 14.0 in `desktop/src-tauri/.cargo/config.toml` (was forced 10.15)
- [x] 1.3 Pin `tauri.conf.json` bundle `macOS.minimumSystemVersion` to 14.0. NOTE: the Tauri plugin still uses `gen_ui_inference` feature `local-llama` on macOS — switching macOS→MLX is the per-target feature change in tasks 3.1/3.2, not the floor raise. Raising 10.15→14.0 is safe for llama/whisper (14.0 ⊃ 10.15).
- [x] 1.4 `cargo build -p gen_ui_inference --features local-mlxc` compiled fully on macOS at the 14.0 floor (5m49s, exit 0) — mlex/mlx-c links, no `MLX requires macOS >= 14.0`. The gate is cleared; the macOS MLX lane builds for the first time.

## 2. Certify the macOS MLX engine

- [x] 2.1 `MlxcEngine` (`mlxc.rs:32,104,289`) resolves the macOS model via the shared `mlx_artifact_set_for_id` catalog and downloads via `download_model_set` (per-file SHA-256). Compiles + links at 14.0.
- [x] 2.2 Memory preflight with downgrade is present (`mlxc.rs:97,252`, shared `mlx_peak_bytes_for_id`) — rejects/downgrades rather than crashing.
- [x] 2.3 `TokenKind::Reasoning → StreamEvent::ThinkingDelta` (`mlxc.rs:507-514`) — mlex classifies reasoning at the tokenizer level (cleaner than the mlx.rs `<think>` text parser); the shared `A2uiAdapter` renders it as a `thinking` ContentBlock. Tool calls via `parse_tool_calls(Hermes)`.
- [x] 2.4 `cargo test -p gen_ui_inference --features local-mlxc` green (5 passed). The live generation path (real model + Metal) is device-cert territory, same as iOS.

## 3. Desktop engine matrix

- [ ] 3.1 Wire per-target engine selection at the desktop construction site: macOS → `MlxcEngine`, Windows/Linux → llama.cpp
- [ ] 3.2 Set per-target Cargo features so one codebase builds the correct lane per platform
- [ ] 3.3 Verify a Windows/Linux desktop build still selects and runs the llama.cpp lane (no regression)

## 4. Shell stability + A2UI parity

- [ ] 4.1 Boot the desktop app on macOS to an interactive state; confirm a not-yet-downloaded model shows a download/progress state, not a hang or panic
- [ ] 4.2 Run an agent turn on a macOS local model end-to-end without crashing
- [ ] 4.3 Confirm the Tauri `spawn_chat_event_forwarder` streams `thinking`, `toolUse`, `toolResult`, `citation`, `memory`, and `skill` ContentBlocks live from the local run
- [ ] 4.4 Confirm the runtime ops console updates runs/steps/tool-calls live from the local run's event stream

## 5. Certify + finalize

- [ ] 5.1 Make MLX the macOS default local lane; drop the llama.cpp macOS fallback once certified
- [ ] 5.2 Update KBD waypoint (`.kbd-orchestrator/`) and mirror to Surreal Memory MCP
- [ ] 5.3 Record a progress log in the karpathy wiki; document the new macOS minimum

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

- [x] 3.1 Per-target engine selection at the construction site (`tauri-plugin-gen-ui/src/commands.rs:397`): `#[cfg(target_os="macos")]` → `MlxcEngine::new(model-cache/mlx)`, `#[cfg(not(macos))]` → `LlamaCppEngine::new(model-cache)`. VERIFIED: full `tauri-plugin-gen-ui` build on macOS at the 14.0 floor (arm64-apple-macosx14.0.0), exit 0.
- [x] 3.2 Per-target Cargo features (`tauri-plugin-gen-ui/Cargo.toml`): moved `gen_ui_inference` into `[target.'cfg(target_os="macos")']` → `local-mlxc` and `[target.'cfg(not(macos))']` → `local-llama`. Verified by the successful macOS plugin build.
- [~] 3.3 Win/Linux no-regression: verified by construction — the `not(target_os="macos")` Cargo block is the UNCHANGED prior `local-llama` config, and the two target blocks are mutually exclusive, so non-macOS resolution is unaffected. A real cross-build needs a Linux/Windows toolchain + tauri system libs (gtk/webkit) not available on this macOS host; the authoritative gate is CI's Linux/Windows runners. FLAGGED: not built here.

## 4. Shell stability + A2UI parity

- [ ] 4.1 Boot the desktop app on macOS to an interactive state; confirm a not-yet-downloaded model shows a download/progress state, not a hang or panic (runtime — needs a launched app)
- [~] 4.2 App launched on macOS via `tauri dev` and booted clean (migrations/host/sync ready, no crash). Runtime config bug found + FIXED (commit fc7b8d0): the desktop DEFAULT_LOCAL_MODEL was a GGUF id, but macOS now runs MlxcEngine which resolves ids against the MLX catalog → "failed to prepare local inference". Split the cfg (macOS→MLX id qwen3.5-4b-mlx-4bit, Win/Linux→GGUF). App relaunched with NO prepare error. REMAINING: the actual on-device generation (model download + token stream) needs an operator to send a Local-lane message — the `tauri dev` binary can't be driven by computer-use. Live diagnostics-log monitor is ready to certify when triggered.
- [x] 4.3 Verified by inspection: `spawn_chat_event_forwarder` (`tauri-plugin-gen-ui/src/lib.rs:146`) emits the ENTIRE `A2uiEvent` to `gen-ui://chat-event` unconditionally — the `match` at 134-144 is logging-only, not a filter. All ContentBlock variants (thinking/toolUse/toolResult/citation/memory/skill) reach the webview. Runtime confirmation folds into 4.2.
- [ ] 4.4 Confirm the runtime ops console updates runs/steps/tool-calls live from the local run's event stream (runtime — folds into 4.2)

## 5. Certify + finalize

- [x] 5.1 MLX IS the macOS default: the `#[cfg(target_os="macos")]` construction builds only `MlxcEngine` and the macOS Cargo block pulls only `local-mlxc` — there is no llama.cpp fallback on macOS. (Live-run certification of the generation path is task 4.2, device-cert territory like iOS.)
- [ ] 5.2 Update KBD waypoint (`.kbd-orchestrator/`) and mirror to Surreal Memory MCP
- [x] 5.3 Karpathy wiki progress log written (`rust/.prometheus/knowledge/wiki/desktop-macos-mlx-floor-raise-and-ios-thinking-split.md`, commit e83fd63) documenting the new macOS 14.0 minimum + the floor-raise/mlex-unblock/engine-matrix milestone.

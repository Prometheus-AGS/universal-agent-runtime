## Why

The macOS/desktop (Tauri) port cannot yet run local inference: MLX requires a
macOS 14 build floor, but both `rust/.cargo/config.toml` and
`desktop/src-tauri/.cargo/config.toml` force `MACOSX_DEPLOYMENT_TARGET=10.15`
(inherited from whisper.cpp/llama.cpp), so the already-scaffolded `local-mlxc`
(`mlex`) engine fails to compile. iOS local inference is now certified on device
with the shared A2UI/thinking pipeline; desktop must reach the same stable,
runnable state so macOS behaves identically to iOS/Android/Web — the four
platforms that must match.

## What Changes

- **Raise the macOS build floor to 14.0** by removing the forced
  `MACOSX_DEPLOYMENT_TARGET=10.15` (rust + desktop `.cargo/config.toml`).
  **BREAKING** for any lingering whisper.cpp/llama.cpp-on-macOS build lane —
  those come off macOS (Apple-native STT/TTS replaces whisper; MLX replaces
  llama on Apple), matching the iOS switchover.
- **Certify the in-process `mlex` MLX engine (`MlxcEngine`, `local-mlxc`)** as
  the macOS default local lane behind the shared `InferenceProvider` seam:
  catalog-driven pinned/SHA-verified downloads, memory preflight, and
  `TokenKind::Reasoning → ThinkingDelta` (macOS already emits thinking; verify
  it renders as a collapsible block, matching mobile).
- **Establish the desktop engine matrix**: macOS → MLX (`mlex`); Windows/Linux →
  llama.cpp (unchanged, lower priority). Engine selection at the desktop
  construction site, per-target Cargo features.
- **A2UI/thinking parity on desktop**: the Tauri `spawn_chat_event_forwarder`
  already forwards `A2uiEvent`; confirm a local macOS run produces the same
  ContentBlocks (thinking, tool-use, citations, memory, skill) the mobile lane
  does — i.e. full agentic operation locally, not just chat.
- **Stable desktop shell run**: the app boots, selects the macOS local model,
  downloads/loads it, and runs an agent turn end-to-end without crashing.

## Capabilities

**New Capabilities**
- `desktop-local-inference` — the macOS in-process MLX (`mlex`) lane as the
  desktop local `InferenceProvider`: model catalog, download/verify, memory
  preflight, reasoning-split, and the per-target engine matrix (MLX macOS /
  llama.cpp Win-Linux). New `specs/desktop-local-inference/spec.md`.

**Modified Capabilities**
- `desktop-shell` — the shell must reach a stable runnable state with a working
  local-inference lane and A2UI/thinking parity; requirement-level change to how
  the desktop app boots and streams runtime events on macOS. Delta spec under
  `specs/desktop-shell/`.

## Impact

- **Build config** (BREAKING on macOS): `rust/.cargo/config.toml`,
  `desktop/src-tauri/.cargo/config.toml` — remove the forced 10.15 floor.
- **Engines**: `gen_ui_inference` (`mlxc.rs`, `catalog.rs`, feature wiring),
  desktop engine construction (tauri-plugin-gen-ui), per-target features.
- **Dependencies**: `mlex` (macOS-only, static mlx-c, no Python); whisper.cpp /
  llama.cpp come off the macOS graph.
- **Runtime UX**: macOS gains on-device inference with collapsible thinking and
  full agentic ContentBlocks, identical to iOS/Android; provider compatibility
  unchanged (same `InferenceProvider`/A2UI seams). Realtime state: local runs
  flow through the same `A2uiEvent` forwarder, so runs/steps/tool-calls/
  approvals/artifacts update live like the cloud path.
- **KBD workflow state**: yes — this change is tracked in `.kbd-orchestrator/`
  (the `desktop-stable-port` waypoint) and mirrored to Surreal Memory MCP; the
  waypoint advances as artifacts complete.

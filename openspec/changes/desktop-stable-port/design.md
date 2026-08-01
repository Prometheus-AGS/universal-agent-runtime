## Context

iOS local inference is certified on device: the in-process/bridged MLX lane runs
Qwen3-4B and the shared `A2uiAdapter` renders thinking, tool-use, citations,
memory, and skill ContentBlocks. macOS must reach the same stable state so the
four crucial platforms (macOS, iOS, Android, Web) behave identically.

Current desktop state:
- The `MlxcEngine` (`gen_ui_inference/src/mlxc.rs`, feature `local-mlxc`) is
  scaffolded and uses the `mlex` crate (Apple's mlx-c C API, statically
  vendored, no Python) and already classifies tokens into
  `TokenKind::Reasoning → ThinkingDelta`.
- It does NOT build because `MACOSX_DEPLOYMENT_TARGET=10.15` is force-set in both
  `rust/.cargo/config.toml` and `desktop/src-tauri/.cargo/config.toml` (inherited
  from whisper.cpp/llama.cpp), below MLX's macOS 14 CMake floor.
- The Tauri shell already forwards `A2uiEvent` to the webview
  (`tauri-plugin-gen-ui` `spawn_chat_event_forwarder`), so the rendering path is
  ready; the missing piece is a working local engine feeding it.

## Goals / Non-Goals

**Goals:**
- macOS desktop builds and runs local inference via the `mlex` `MlxcEngine`.
- Raise the macOS deployment floor to 14.0; remove whisper.cpp/llama.cpp from the
  macOS graph (Apple-native STT/TTS and MLX replace them, mirroring iOS).
- A per-target engine matrix: macOS → MLX, Windows/Linux → llama.cpp.
- Full-agentic A2UI parity on the desktop shell (thinking/tool/citation/memory/
  skill ContentBlocks stream live from a local run).

**Non-Goals:**
- Windows/Linux MLX (they stay on llama.cpp; lower priority).
- The Apple-native STT/TTS replacement implementation itself (separate change);
  here we only ensure whisper does not pin the macOS floor.
- The transport-free AG-UI adapter consolidation (separate change/ADR).
- Web/WebLLM (separate change).

## Decisions

- **Raise the macOS floor by removing the forced 10.15, not by per-crate
  override.** The `force = true` on `MACOSX_DEPLOYMENT_TARGET` exists only to
  satisfy whisper/llama on old macOS. With those off the macOS graph, the floor
  should follow the toolchain default (≥14 on current Xcode). Alternative
  considered: keep 10.15 and special-case `mlex` — rejected, because a mixed
  floor invites ABI/link surprises and the project policy explicitly allows
  raising minimums.
- **macOS local lane = in-process `mlex`, not MLX-Swift-over-FFI.** Desktop is
  Tauri/Rust with no Swift host, so the iOS C-vtable bridge doesn't apply; `mlex`
  gives in-process streaming with `TokenKind` classification. Alternative: run a
  Swift sidecar — rejected as heavier and off-pattern for a Rust desktop.
- **Engine selection via per-target Cargo features**, at the desktop
  construction site, so one codebase yields MLX on macOS and llama.cpp on
  Win/Linux. Mirrors the existing mobile feature gating (`local-mlx` iOS,
  `local-litert-lm` Android).
- **Reuse the shared seams unchanged**: `InferenceProvider`, the catalog,
  memory preflight, the reasoning-split (`ThinkingDelta`), and the
  `A2uiAdapter` → `A2uiEvent` → Tauri forwarder. No new rendering path.

## Risks / Trade-offs

- [Raising the macOS floor breaks an old-macOS build lane] → Those lanes are
  whisper/llama-only and being removed from macOS; document the new minimum and
  verify a clean desktop build on current macOS.
- [`mlex` is a young crate (0.1.x)] → It compiled+linked mlx-c cleanly on Apple
  Silicon in a prior spike; keep llama.cpp as the macOS fallback until the lane
  is certified by a real run, then flip the default.
- [Removing whisper from macOS regresses on-device transcription] → Out of scope
  here; the Apple-native STT/TTS replacement is a separate change. This change
  only stops whisper from pinning the floor.
- [GPU/thermal behavior on sustained desktop agent loops] → Accepted for v1;
  tune from measured evidence, same as iOS.

## Migration Plan

1. Remove `MACOSX_DEPLOYMENT_TARGET=10.15 force=true` from both cargo configs;
   ensure whisper/llama are off the macOS dependency graph.
2. Enable `local-mlxc` on the macOS desktop target; wire the engine matrix at the
   desktop construction site (per-target features).
3. Build + run the desktop app on macOS; download/load the model; run an agent
   turn; confirm thinking/tool/citation/memory ContentBlocks render.
4. Certify, then make MLX the macOS default (drop the llama.cpp macOS fallback).

Rollback: revert the cargo-config change and feature wiring; macOS falls back to
the llama.cpp lane (still present for Win/Linux).

## Open Questions

- Exact macOS minimum after the floor raise — follow the toolchain default vs.
  pin an explicit `14.0`? (Lean: pin `14.0` explicitly for reproducibility.)
- Which Qwen3 MLX model is the macOS default vs. the 12 GB-class tier — reuse the
  mobile catalog entries or add a desktop-sized entry?
- Does any remaining macOS dependency (cpal, etc.) still assume the old floor?

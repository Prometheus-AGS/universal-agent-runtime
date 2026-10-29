# Handoff: arm64 and iOS verification on the MacBook Pro (Apple Silicon)

Written 2026-10-29 on the Intel Mac that ran the macOS (x86_64) validation of
UAR 1.4.0-rc.1. This document is self-contained: a new session on the
MacBook Pro should read it, then the files it names, and nothing else from the
old machine is needed. No secret, token or key from the old machine is in this
repository or this document, and none should be copied across.

## 1. Where things stand

- **Release candidate:** tag `v1.4.0-rc.1` on `59abbfff` (GitHub release marked
  latest at the operator's request). GHCR tags `1.4.0-rc.1`, `v1.4.0-rc.1` and
  `latest` point at the multi-arch image built by main's run for that commit
  (index `sha256:7315267292ed6fc45bc9374252df5c61de996c0efea72703c7e21e3040a695a2`).
  The tag's own image run failed on arm64 (see §3.1); the image was retagged
  from main's identical build instead.
- **main** has moved past the tag: #453 (Dockerfile `cargo-wasi` fix), #454
  (console on `@a2ui` 0.14 and Vitest 5) and #455 (`UarSurface` optional
  catalog type fix). A later `1.4.0-rc.2` would carry them.
- **macOS x86_64:** 99 of 102 catalog scenarios pass
  (`.prometheus/verification/bdd-validation-catalog.md`). E04, E06 and E07 are
  blocked by missing product capabilities, being built in two other sessions
  (OpenSpec changes `route-requirements-dispatch` and `chat-image-input`, on
  their own branches, not merged).
- **Open PRs:** #440 (native tool-call identity, merged with main and
  mergeable; it will ship in 1.4.0) and #447 (`react-resizable-panels` 4.15.0,
  mergeable).
- **Operator to-do:** `versions.toml` still pins Vitest 4.1.10 while the
  frontend is on 5.0.5. Only the operator edits `versions.toml`.

## 2. What this machine is for

1. **arm64 Linux image proof** for #453 (§3.1). Fast here: no emulation.
2. **macOS arm64 platform cell** of the catalog (§3.2): a native Apple Silicon
   build of UAR, its own owned runtime, then the BDD scenarios.
3. **iOS** (§3.3): UAR as a library inside a Flutter test app on the iOS
   simulator, driven by buttons that call it over FFI, using models served
   locally on this machine (MLX/GPU).

Do them in that order unless the operator says otherwise. Each is a separate
claim: an x86_64 pass never transfers to arm64, and simulator and device are
distinct claims.

## 3. Tasks

### 3.1 arm64 image toolchain proof (#453)

The `v1.4.0-rc.1` image run failed on linux/arm64 because its layer cache
missed and the toolchain stage rebuilt `cargo-wasi` 0.1.28 from source. That
crate (archived, no prebuilt binaries) locks a `rustix` whose Linux aarch64
backend uses `rustc_layout_scalar_valid_range_*`, which the pinned nightly
`nightly-2026-07-18` rejects. #453 installs it unlocked. On macOS and on
linux/amd64 even the locked install succeeds, so only aarch64 Linux can prove
the fix (an emulated attempt on the Intel Mac was still compiling after two
hours).

Proof command, native here:

```bash
docker buildx build --no-cache --platform linux/arm64 --target toolchain --progress=plain -o type=cacheonly -f Dockerfile .
```

Pass: the log shows ``Installed package `cargo-wasi v0.1.28` `` and the stage
completes. Record the command, duration and result in
`.prometheus/session-log.md`. Optional next step: a full
`--platform linux/arm64` image build of `main`.

### 3.2 macOS arm64 catalog cell

1. **Build** (locked, same features as the x86_64 gate):
   `cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime`
   with a dedicated `CARGO_TARGET_DIR`, plus the identity proxy
   (`cargo build --locked -p uar-jwt-proxy`).
2. **Provision an owned runtime with fresh local secrets.** The old machine's
   runtime config lived in `target/` and is not in git; do not recreate it from
   memory. Write a new config under `target/` with: server host/port and
   `grpc_port`; SurrealKV persistence under `target/`; `llm.base_url` and a
   `liter` provider pointing at the local model gateway (§4); memory `db_path`
   under `target/`; `security.deployment_profile: trusted_local`,
   `jwt_required: false`, and a freshly generated JWT secret and settings admin
   key. Start UAR with `UAR_STATIC_DIR` set to a built console directory
   (`pnpm -C frontend build --outDir <dir> --emptyOutDir`).
3. **Identity outside the tests.** Start `uar-jwt-proxy` in front of UAR with
   `--config`, `--upstream`, `--listen` and `--grpc-listen` (see
   `docs/dev-tools.md`). Tests never send credentials; the proxy supplies the
   owner.
4. **Launch records the scenarios rely on.** `scripts/behaviors/support/restart-owned-uar.mjs`
   reads `target/prompt-macos-launch.json` (binary, config, pid, base_url,
   static_dir, binary/config SHA-256, log, status),
   `target/manual-identity-proxy/launch.json` (binary, config, upstream,
   base_url, grpc_base_url, pid) and `.prometheus/verification/manual-runtime.json`
   (base_url, runtime_url, runtime_pid, external_identity_pid, grpc_base_url,
   model, workspace_id). Read that script first and write records that satisfy
   its assertions for this machine. Its lifecycle adapter reads process
   environments with macOS `ctypes`; confirm it works on arm64.
5. **Run scenarios once each**, recording every attempt. Start with the ones
   most sensitive to architecture, then the rest of the catalog as time allows:
   - `knowledge`, `knowledge-document-delete` (xberg extraction, local
     embeddings through fastembed/ONNX Runtime);
   - `skill-classifier-backends` (all five backends, including `wasm` via
     wasmtime; build the guest per `scripts/acceptance/fixtures/intent-classifier-guest/README.md`);
   - `memory-retention` (E03; host fixture
     `scripts/acceptance/fixtures/provision-aged-memory.mjs`, which needs
     `cargo build --example surrealql_store_runner`);
   - `structured-output`, `owner-isolation`, `grpc-modes`, the SDK mode sets,
     and the browser scenarios (`browser-history-reload`,
     `browser-agent-selection`, `a2ui-browser`; Playwright needs Chrome and
     `npx playwright install ffmpeg`).
   Invocation pattern:
   `UAR_BASE_URL=… UAR_GRPC_TARGET=… UAR_WORKSPACE_ID=… UAR_MODEL=… node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/<name>.cucumber.mjs`.

### 3.3 iOS: UAR as a library in a Flutter test app

UAR on iOS always runs as a library inside a host app, never as a standalone
process. Build a small Flutter test app (Flutter is the most common host) whose
window has one button per scenario; each button calls the embedded runtime over
FFI and shows the result and its evidence.

- **Runtime side, already in the repo:** the Rust SDK's `ios_acceptance`
  staticlib example (`scripts/acceptance/ios/rust/lib.rs`, C ABI in
  `scripts/acceptance/ios/Host/Bridge.h`, built with
  `cargo build --locked --manifest-path sdks/rust/Cargo.toml --no-default-features --features ios-acceptance --example ios_acceptance --target aarch64-apple-ios-sim`).
  It owns the Tokio runtime, SDK runtime and a SurrealKV store, contains
  panics at the ABI and shuts down cleanly.
- **Scenarios:** the six in `scripts/acceptance/ios/scenarios.json` (arithmetic
  and streaming, a follow-up, protected-agent refusal, orchestrator selection,
  skill activation, reduction/cancellation/shutdown, plus the cold-restart
  readback), each a button.
- **Models:** serve a chat model and an embedding model on this machine
  (MLX), reachable from the simulator at `127.0.0.1` with no credentials, and
  point the embedded runtime at them. Whether UAR's embedded host can run
  generation on-device is unconfirmed; ask before building that.
- **Conflict to resolve first:** the existing harness's
  `scripts/acceptance/ios/private-config.schema.json` requires a `proxy_token`.
  That violates the standing rule that tests never carry credentials. Replace it
  with the no-credential local model endpoint, with the operator's agreement.
- **Decisions for the operator before code:** plain `dart:ffi` over the
  existing C ABI (no new dependency) versus `flutter_rust_bridge` (a new
  dependency, not pinned in `versions.toml`); where the app lives (suggested
  `scripts/acceptance/flutter-host/`).
- **Done when:** the app builds for the arm64 simulator, each button's
  scenario passes against the embedded runtime with the local models, and every
  attempt is recorded. A physical device is a separate, later claim.

## 4. Machine setup checklist

- Clone `Prometheus-AGS/universal-agent-runtime`, `git pull`, then
  `git submodule update --init` (PEM must be at `v4.3.0`, `ea38b265`).
- Rust: the repo's `rust-toolchain.toml` installs `nightly-2026-07-18` with
  `aarch64-apple-ios` and `aarch64-apple-ios-sim`. `cargo-component` 0.21.1 for
  the WASM classifier guest.
- Node 24 and pnpm 11 (`pnpm install --frozen-lockfile` at the root and in
  `frontend`). On the Intel Mac the default `node` was v10; check `node -v`.
- Docker with native linux/arm64 (§3.1).
- Xcode with an iOS simulator runtime; Flutter SDK. The Dart MCP server needs
  Dart 3.9 or later: register it with Flutter's bundled Dart
  (`claude mcp add --scope user dart -- <flutter>/bin/dart mcp-server`).
- Playwright Chrome channel and `ffmpeg`.
- A local model gateway compatible with the old `liter` provider shape
  (`http://127.0.0.1:4000/v1`, models such as `liter/MiniMax-M3`,
  `liter/glm-5.3`), or local MLX models; provider keys stay in the gateway's own
  environment, never in tests or the repo.

## 5. Rules that carry over

- Read `CLAUDE.md`, `versions.toml` and `.prometheus/gotchas.md` first.
  `versions.toml` is authoritative and only the operator edits it.
- Verification is BDD scenarios only, through public APIs against real models;
  the only other gate step is the build that produces the artifact. No unit
  tests or typecheck as gates (the commit hooks still run lint and typecheck).
- Tests never require, mint, copy or send JWTs, bearer tokens, API keys or
  login credentials. External infrastructure supplies identity.
- No secret values in tests, evidence, tracked files, terminal output or chat.
- Keep first failures; no automatic retries. Fix, then rerun only the failed
  step. Record every attempt in `.prometheus/session-log.md`.
- No commit, push or PR unless the operator asks. No attribution lines.
- GitHub Actions are for deployment only; run
  `pnpm github-actions-policy:validate` before and after touching a workflow or
  a script a workflow runs.
- Worktrees go under `~/.claude/worktrees/`, never inside the repo.

## 6. Read first

- `.prometheus/verification/bdd-validation-catalog.md` (scenario definitions,
  Mac results, platform plan in its platform table)
- `scripts/behaviors/README.md` and `scripts/behaviors/support/restart-owned-uar.mjs`
- `.prometheus/session-log.md` (entries dated 2026-10-28 and 2026-10-29)
- `.prometheus/gotchas.md` (latest entries: Node v10 PATH, protocol contracts
  before defects, zip 9.1.1, the host `surreal` CLI versus SurrealDB 3.3.0)
- `scripts/acceptance/ios/README.md`
- `docs/releases/v1.4.0-rc.1.md`

## 7. Prompt to start the session on the MacBook Pro

```text
I'm continuing UAR 1.4.0-rc.1 verification on this Apple Silicon MacBook Pro.
Read .prometheus/handoffs/macbook-pro-arm64-ios-2026-10-29.md in full, then the
files its "Read first" section lists, and follow its rules. Start with the
machine setup checklist: report what is missing on this machine and ask before
installing or downloading anything. Then propose the exact commands for task
3.1 (the arm64 toolchain proof for #453) and wait for my go-ahead before
running them. After that we'll do 3.2 (the macOS arm64 catalog cell) and 3.3
(the Flutter iOS host app), in that order; ask me the open decisions in 3.3
before writing any code. Tests never carry credentials, and no secret from any
machine goes into the repo, evidence or chat.
```

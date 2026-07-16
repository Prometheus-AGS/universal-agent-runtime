## 1. Port resolution logic

- [x] 1.1 In `src-tauri/src/lib.rs`, change `resolve_localhost_port` so the
      default path (no `TAURI_LOCALHOST_PORT`) tries `fallback`
      (`app_config.server.port`) first via a direct bind attempt, instead of
      immediately binding `127.0.0.1:0`.
- [x] 1.2 Keep the existing `TAURI_LOCALHOST_PORT` env var check as the
      top-priority path, unchanged (D3).
- [x] 1.3 On a successful bind of the configured port, return it directly —
      no persistence needed (it's already the stable, known-good default).

## 2. Conflict fallback and persistence

- [x] 2.1 When binding `app_config.server.port` fails (port in use), first
      check for a previously-persisted fallback port (see 2.3) and attempt
      that before falling back to a fresh ephemeral bind.
- [x] 2.2 If no persisted fallback exists or it's also unavailable, bind
      `127.0.0.1:0` to get an OS-assigned port (existing ephemeral-bind
      logic), then persist it per 2.3.
- [x] 2.3 Add a small persistence helper: write the resolved fallback port
      as a plain value (e.g. a one-line file or minimal JSON) to
      `app.path().app_config_dir()`, read it back on the next launch before
      attempting a fresh ephemeral bind (2.2).
- [x] 2.4 Ensure the persistence file path creation handles a missing
      app-config directory gracefully (create it if absent; log and fall
      through to ephemeral-without-persistence if the write fails — must
      never crash startup over a persistence failure).

## 3. Wiring and cleanup

- [x] 3.1 Update the call site in `run()` (`setup` closure) so
      `server_url`/`wait_for_ready` use the value returned by the updated
      `resolve_localhost_port` unchanged — no signature changes needed if
      the function keeps returning a single `u16`.
- [x] 3.2 Remove or update the function's doc comment to describe the new
      priority order: env var → configured port → persisted fallback →
      fresh ephemeral bind (persisted for next time).

## 4. Verification

- [x] 4.1 `cargo check --locked --no-default-features --features server-full`
      (per this project's build gate) passes with the updated `src-tauri`
      code.
      NOTE (2026-07-15): this command checks the root workspace lib, which
      does not compile src-tauri at all — task wording was imprecise. The
      actual src-tauri crate cannot currently be checked/built via `cargo
      check` in this environment: its `build.rs` fails on missing sidecar
      binaries (`src-tauri/binaries/` doesn't exist; tauri.conf.json's
      externalBin references mcp-server-filesystem/mcp-server-fetch, pre-
      existing gap, unrelated to this change — see spawned task
      "Provision missing Tauri sidecar binaries for src-tauri build").
      Confirmed instead: (a) this command passes clean against the root
      workspace after excluding src-tauri from it and giving src-tauri its
      own standalone `[workspace]` table (both required just to make the
      port-resolution code reachable by any checker at all — src-tauri was
      never wired into a workspace before this change); (b) the edited
      `resolve_localhost_port` logic verified correct via an isolated
      `rustc --edition 2024` compile + runtime assertions covering same-
      port reuse, conflict fallback, persisted-port reuse across calls, and
      env-var override priority — all passed.
      UPDATE (2026-07-16): the sidecar-binaries gap is now resolved
      (`scripts/build-tauri-sidecars.sh`, merged via PR #132/#133). A
      second, separate pre-existing blocker surfaced once src-tauri could
      actually compile: `src-tauri/src/lib.rs` called
      `config::load_llm_settings()` (doesn't exist) and
      `server::start_server()` with a stale 2-arg signature — fixed to use
      `Cli::parse_from` + `ConfigManager::load` + the current single-arg
      `start_server(Arc<ConfigManager>)`. That in turn surfaced a third,
      genuine blocker: `server::start_server`'s future is not `Send`
      (tracing/log values captured across `.await`), so it cannot run
      inside `tauri::async_runtime::spawn` (which requires `Send + 'static`
      — no other caller in this codebase had ever spawned it that way
      before). Fixed by running the embedded server on its own dedicated
      OS thread with its own `tokio::runtime::Runtime` via `block_on`
      instead, which has no `Send` bound on the future. `cargo check
      --manifest-path src-tauri/Cargo.toml` now passes clean (exit 0, zero
      warnings). This unblocks 4.2-4.4 below.
- [x] 4.2 Verified live (2026-07-16) by running `target/debug/app` twice in
      a row with no `TAURI_LOCALHOST_PORT` set: both launches logged
      `Starting embedded Axum server on http://127.0.0.1:54490` — the
      identical port both times. (In this dev environment port 1906, the
      configured default, is permanently held by the always-running
      uar-jwt-proxy-fronted UAR instance, so every launch here naturally
      exercises the conflict-fallback + persistence path rather than the
      configured-port-free path — see 4.3, which is the same evidence.)
      CAVEAT: the embedded Axum server logs its resolved URL and then
      fails during its own startup (`Failed to build embedding backend:
      ... unknown or disabled embedding backend: fastembed`) — a separate,
      pre-existing feature-gating bug (src-tauri only enables the `tauri`
      Cargo feature, not `local-models`/`desktop-full`, so fastembed isn't
      compiled in even though the default config selects it). This is
      unrelated to port resolution (which runs synchronously before the
      failing async server-init code and is unaffected by it) and out of
      scope for this change; flagged as a follow-on task
      (task_d05f930d) rather than fixed here. Full "webview reaches the
      origin" could not be observed end-to-end because of this separate
      bug — the port-resolution logic itself, which is what this change
      implements, was directly verified via live process runs + log +
      persisted-file inspection.
- [x] 4.3 Verified live: with port 1906 occupied (see 4.2), launch #1
      resolved and persisted fallback port 54490
      (`~/Library/Application Support/com.tauri.dev/desktop-port.txt`
      contained `54490`); launch #2 (fresh process, same occupied port)
      reused the identical persisted port 54490, not a new random one.
- [x] 4.4 Verified live: launching with `TAURI_LOCALHOST_PORT=48123` set
      logged `Starting embedded Axum server on http://127.0.0.1:48123`
      (override honored over both the configured port and the persisted
      fallback), and the persisted-port file remained unchanged at 54490
      afterward — confirming the env-var path does not corrupt persisted
      state.
- [x] 4.5 Ran `scripts/kbd-validate-progress.sh --mark-implementation-complete`
      for this change against
      `.kbd-orchestrator/phases/uar-hybrid-app-architecture/progress.json`.

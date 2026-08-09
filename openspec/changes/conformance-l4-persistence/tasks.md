> **Read `EXECUTION-CONTRACT.md` first.** It fixes the order (this change is
> third and last), the pinned command, and the verification format.
>
> **Local gate:** the cases added here are executed by the pinned local command
> from `conformance-baseline-gate` task 2.1. No GitHub Actions work belongs in
> this change.
>
> **This is the only change in the set that touches runtime source.** Task 1.1 is
> the entire permitted surface: one additive parameter. Anything beyond it is a
> stop condition.

## 1. Runtime seam — expose the shutdown that exists

- [ ] 1.1 Add an optional caller-supplied `CancellationToken` to
      `start_server_sidecar` (`src/server.rs:1357`). Thread it to
      `serve_on_listener` so it is used **instead of** the internally created
      token at `:1386` when supplied. When absent, behaviour is unchanged.
- [ ] 1.2 The existing signal handler (`:1388-1420`) must keep working: SIGINT
      and SIGTERM still drain the ingestion pool and still cancel. A test-owned
      token is an additional trigger, never a replacement.
- [ ] 1.3 Do not change `shutdown_future` (`:1425-1438`) or the
      `.with_graceful_shutdown` wiring (`:1441`, `:1453`). This task exposes a
      trigger; it does not redesign shutdown.
- [ ] 1.4 `cargo check --locked --no-default-features --features server-full
      --all-targets` exits 0.

## 2. Harness — allow a reusable DB path

- [ ] 2.1 `boot_test_server` currently derives its persistence path from
      `unique_temp_path` (`harness.rs:143-149`), which mints a fresh UUID per
      boot. Add a way for a caller to supply a fixed path so a second boot
      reopens the same store.
- [ ] 2.2 Default behaviour is unchanged: callers that do not supply a path
      still get a unique temp dir. Every existing case must keep passing
      untouched.
- [ ] 2.3 Return a handle allowing the caller to shut the server down and await
      its exit, so a reboot does not race the previous instance's file locks.
      SurrealKV holds its directory open — see `harness.rs:86`.

## 3. The round-trips

- [ ] 3.1 **C-12 persistence, target L4.** Boot on a fixed path, write a
      resource through the real API, shut down via the caller-owned token, await
      exit, boot again on the same path, read the resource back, assert it
      matches what was written. Rename to `l4_c12_persistence_round_trip`.
- [ ] 3.2 **C-13 sessions, target L4.** Same shape against the session surface,
      using the **current** contract — `X-UAR-Session-ID` with
      `POST /api/chat/completion`, not the retired `/api/sessions` route.
- [ ] 3.3 If either round-trip proves structurally impossible, publish it as
      `excluded_` with the blocking reason named. **"No shutdown hook exists" is
      not available as a reason** — that claim is refuted by `server.rs:1386-1453`.

## 4. Verification

- [ ] 4.1 Full matrix green, including the pre-existing 20 cases with no
      behaviour change.
- [ ] 4.2 Negative control: the round-trip must FAIL if persistence is broken.
      Temporarily point the second boot at a different path and confirm the
      assertion fails. A round-trip that passes against an empty store is not a
      round-trip.
- [ ] 4.3 `cargo check --features server-full --all-targets` exits 0; no new
      warnings.
- [ ] 4.4 Confirm the ordinary signal path still shuts the server down —
      this change must not make SIGTERM depend on a caller-supplied token.
- [ ] 4.5 Append one row per case to
      `.kbd-orchestrator/phases/uar-spec-conformance-2026-08/verification.md`
      in the format defined by `EXECUTION-CONTRACT.md`, including the negative
      control from 4.2 as its own row.

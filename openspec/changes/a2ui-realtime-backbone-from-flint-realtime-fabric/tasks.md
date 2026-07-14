## 1. Audit before building
- [x] 1.1 Confirmed `src/uar/a2ui/protocol.rs`'s wire-message DTOs are Deserialize-only, `pub(crate)`, and unused outside their own module — no existing A2UI surface-emission call site to hook.
- [x] 1.2 Confirmed `RunManager::subscribe` already gives every run a `tokio::sync::broadcast` channel — multi-client convergence for live subscribers already exists.
- [x] 1.3 Confirmed the real gap is durable replay for late-join clients (a `broadcast` channel has no replay-from-start), which is what `flint-realtime-fabric`/`frf-sdk-rust` is for.
- [x] 1.4 Confirmed `frf-sdk-rust` is a real, clean, importable crate with a genuine git remote (`git@github.com:Prometheus-AGS/flint-realtime-fabric.git`) — unlike Change 19's flint-forge situation, this one is practically wireable, but doing so means adding a new `vendor/git/` submodule per this repo's established convention (see `proposal.md`'s "Out of scope"), an operator decision.

## 2. StatePatch conversion + replay backbone
- [x] 2.1 `src/uar/a2ui/realtime.rs`: `A2uiWireKind` enum (CreateSurface/UpdateComponents/UpdateDataModel/DeleteSurface) + `surface_message_to_state_patch(surface_id, kind, payload) -> StatePatchOp`.
- [x] 2.2 `A2uiReplayBackbone` trait: `publish(run_id, op)`, `replay(run_id) -> Vec<StatePatchOp>`.
- [x] 2.3 `InMemoryReplayBackbone`: real, tested, in-process implementation. Explicitly documented as non-durable/single-process (the gap the deferred FRF-backed implementation would close).
- [x] 2.4 7 unit tests: each `A2uiWireKind` variant's op/path/value mapping (4 tests), replay ordering + multi-run isolation, replay-of-nonexistent-run returns empty, two independent readers converge on the same sequence.

## 3. HTTP endpoints
- [x] 3.1 `A2uiApiState` gets a new `realtime_backbone: Arc<InMemoryReplayBackbone>` field; both existing construction sites in `server.rs` updated to share one instance via a new `a2ui_realtime_backbone` binding.
- [x] 3.2 `POST /api/uar/runs/{run_id}/a2ui/surface-test-trigger` — converts the request body, publishes to the backbone, emits `NormalizedEvent::StatePatch` via `RunManager::emit_to_run` (mirrors the existing `.../a2ui/test-trigger` pattern).
- [x] 3.3 `GET /api/uar/runs/{run_id}/a2ui/surface-replay` — returns the backbone's replay for that run.
- [x] 3.4 Both routes registered in `build_response_router()`; module doc table at the top of `routes.rs` updated.

## 4. BDD feature file
- [x] 4.1 `tests/bdd/features/a2ui-live-update.feature` — the plan's 2 named scenarios (multi-client convergence, late-join reattach) as real Gherkin, following this repo's existing feature-file style.
- [ ] 4.2 **Deferred**: step definitions. Requires new test-harness infrastructure absent for every existing BDD scenario in this repo — a way to obtain a real `run_id` via direct API (not through the browser-driven chat UI) and raw SSE-stream consumption from test code. This is test-infrastructure work, not this change's product-code scope; disclosed in `proposal.md`.
- [x] 4.3 The late-join scenario is tagged `@pending-flint-realtime-fabric` in the feature file since it cannot be honestly satisfied by the in-memory backbone alone in a genuinely multi-process/durable sense (single-process replay works today; cross-process/durable replay needs the deferred FRF wiring).

## 5. Deferred (see proposal.md "Out of scope")
- [ ] 5.1 Wiring `frf-sdk-rust` as a live Cargo dependency (needs an operator decision on `vendor/git/` submodule vs. other approach).
- [ ] 5.2 An orchestrator call site that creates/updates real A2UI surfaces during a run — doesn't exist yet anywhere in the codebase; separate, larger product feature.
- [ ] 5.3 BDD step definitions (see 4.2).
- [ ] 5.4 Motion-based live transitions — Change 21's scope, not this change's.

## 6. Verification
- [x] 6.1 `cargo check --no-default-features --features server-full` — PASS.
- [x] 6.2 `cargo test --no-default-features --features server-full --lib a2ui::realtime` — 7/7 PASS.
- [x] 6.3 `cargo fmt --all -- --check` — 2 formatting nits found and fixed (line-wrapping only); re-verified clean.
- [x] 6.4 `openspec validate a2ui-realtime-backbone-from-flint-realtime-fabric --strict` — PASS.
- [ ] 6.5 **Deferred to the phase's consolidated validation pass**: full-workspace `cargo clippy` (scoped `cargo check` on the touched crate is clean; no clippy-specific pass run this turn).

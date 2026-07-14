## Why

The phase plan frames this change as "wire `flint-realtime-fabric` as the
SSE/fan-out backbone for A2UI live updates." Auditing the existing code
before writing anything found the premise only partially holds:

- **No A2UI emission call site exists yet.** `src/uar/a2ui/protocol.rs`'s
  wire-message DTOs (`CreateMessage`/`ComponentsMessage`/`DataMessage`/`DeleteMessage`)
  are `Deserialize`-only, `pub(crate)`, and referenced nowhere outside
  their own module (`#[expect(dead_code, ...)]` on the module confirms
  they're contract-validation scaffolding). The orchestrator does not
  currently create or update A2UI surfaces during a live run — there is no
  existing traffic to "become" StatePatch events yet.
- **Multi-client fan-out ("convergence") already exists**, for free.
  `RunManager` broadcasts every `NormalizedEvent` to all subscribers of a
  run via a `tokio::sync::broadcast` channel (`RunManager::subscribe`).
  Any number of SSE clients on the same `run_id` already converge on the
  same event stream without this change doing anything.
- **The real, genuinely-missing capability is durable replay for
  "late-join reattach."** A `broadcast` channel has a bounded buffer and
  no replay-from-start semantics: a client that connects after events
  already fired gets nothing before its subscribe point, and a slow
  client can miss events entirely. This is exactly what
  `flint-realtime-fabric` (a durable pub/sub broker;
  `frf-sdk-rust`'s `FrfClient::subscribe(..., Offset::BEGINNING)`) is
  built for — but adding it as a live Cargo dependency requires an
  operator decision on vendoring strategy (see "Out of scope" below).

## What Changes

- New `src/uar/a2ui/realtime.rs`: `surface_message_to_state_patch` (converts
  an A2UI wire message into a `StatePatchOp` rooted at
  `/a2ui/surfaces/{surface_id}`, so any future orchestrator call site that
  emits A2UI surface changes composes with the existing
  `NormalizedEvent::StatePatch`/`RunManager` broadcast pipeline for free)
  and an `A2uiReplayBackbone` trait with a real, tested
  `InMemoryReplayBackbone` implementation (in-process, per-run patch
  history, used for late-join replay).
- Two new HTTP endpoints on the existing A2UI runs router:
  - `POST /api/uar/runs/{run_id}/a2ui/surface-test-trigger` — converts a
    supplied A2UI surface message into a `StatePatchOp`, publishes it to
    the replay backbone, and emits it on the run's live SSE broadcast via
    the same `RunManager::emit_to_run` path every other run event uses.
    Mirrors the existing `.../a2ui/test-trigger` artifact-testing
    endpoint's shape.
  - `GET /api/uar/runs/{run_id}/a2ui/surface-replay` — returns every
    surface state-patch published for a run so far, in order: the
    late-join read path.
- New `tests/bdd/features/a2ui-live-update.feature`: the plan's 2 named
  scenarios (multi-client convergence, late-join reattach), written as
  real Gherkin.

## Capabilities

### New Capabilities

- `a2ui-live-update`: the `StatePatchOp` conversion, the replay-backbone
  abstraction, and the two new endpoints.

## Impact

- **No breaking changes.** Purely additive: a new module, a new field on
  `A2uiApiState` (`realtime_backbone`), two new routes.
- **`AppState`/`server.rs`:** one new shared `Arc<InMemoryReplayBackbone>`
  instance, cloned into both existing A2UI router-construction sites so
  replay is consistent regardless of which router a request hits.
- **No new runtime dependency** for this pass (the in-memory backbone
  uses only `std::sync`). `frf-sdk-rust` itself is not yet a Cargo
  dependency — see "Out of scope."

## Out of scope

- **Actually wiring `flint-realtime-fabric` as a live dependency.**
  `frf-sdk-rust` is a real, cleanly-separated, importable crate (unlike
  Change 19's flint-forge situation) with a genuine git remote
  (`git@github.com:Prometheus-AGS/flint-realtime-fabric.git`), so this is
  *practically* wireable — but this repo's established convention for
  consuming external git-sourced Rust crates is vendoring under
  `vendor/git/<repo>/` (see `sycophancy-core`, `surreal-memory-server`,
  `prometheus-parking-lot-rs` in the root `Cargo.toml`'s workspace
  `exclude` list), which means adding a new git submodule — a repo
  structure change with broader implications than this change's own
  scope, better made as an explicit operator decision (matching how
  Change 1's SDK relicensing and Change 6's `llm.api_key` secrecy
  scope were both left as explicit operator/follow-up decisions rather
  than forced through). The `A2uiReplayBackbone` trait is designed so an
  `FrfReplayBackbone` implementation can be dropped in later without
  changing any call site.
- **An orchestrator call site that actually creates/updates A2UI surfaces
  during a run.** As found in the audit above, this doesn't exist yet
  anywhere in the codebase — building it is a separate, larger product
  feature (likely spanning the orchestrator, the A2UI protocol module,
  and the frontend renderer), not implied by "wire the realtime backbone."
  The two new test-trigger/replay endpoints let this change be exercised
  and verified without that call site existing yet, the same way the
  pre-existing `.../a2ui/test-trigger` endpoint already does for
  artifact-input-request events.
- **BDD step definitions for `a2ui-live-update.feature`.** The feature
  file's 2 scenarios are real Gherkin, but implementing their step
  definitions needs new test-harness infrastructure this repo's BDD suite
  doesn't have for *any* existing scenario: a way to create a real,
  addressable `run_id` via a direct API call (every existing scenario
  gets its run indirectly through the browser-driven chat UI, with no
  `run_id` surfaced to test code) and raw SSE-stream consumption from
  Node/Playwright test code (no existing step definition does this
  either). Building that harness is test infrastructure work, not part
  of this change's product-code scope. Verified instead via 7 Rust unit
  tests in `realtime.rs` covering the conversion function and the replay
  backbone's guarantees directly.
- **Live transitions via Motion.** Named in the plan's done-condition
  ("§17") — that's frontend animation work belonging to Change 21
  (`a2ui-world-class-theming-a11y-i18n`), which already owns Motion
  integration per its own done-condition.

## Context

`proxy-integration-gate` shipped `tests/integration/live/{stub_llm.rs,
backend.rs}`: a resolvable `base_url` (real proxy or in-process stub) that
any HTTP client can point at. What's missing is the HTTP client's other
end — a real, running instance of the UAR server to actually exercise. No
existing test in this repo boots one: `AppState` (`src/lib.rs:67`) has ~20
fields; every current integration test either drives a narrow sub-router
via `axum_test::TestServer` (e.g. `tests/credentials_api_integration_test.rs`)
or a standalone mock server unrelated to `AppState`
(`tests/test_a2a_client.rs`). This change is the first to boot the real
server for a test.

## Goals / Non-Goals

**Goals:**
- Prove, via real HTTP requests against a real (if minimally-configured)
  server, that streaming (3 SSE modes), tool-loop, agent selection, memory,
  RAG, and credentials work end-to-end.
- Keep the harness's `AppState` construction honest: use `None` for optional
  services only when a case doesn't need them, not as a blanket shortcut that
  quietly stops testing anything real.
- Land the per-change feature-coverage contract (`MATRIX.md` + CI
  presence-check) so every later Round 1-4 change has an unambiguous
  completion criterion.

**Non-Goals:**
- A general-purpose, fully-configured `AppState` test harness for all future
  testing needs — scope this to exactly what the 8 baseline cases require;
  extend later if a specific future case needs more.
- Testing persistence/memory/RAG against real Postgres/SurrealDB in this
  tier — that's what `comprehensive-tests.yml`'s existing database
  integration tests (`tests/integration/database/`) are for. This tier proves
  the HTTP/orchestrator/streaming surface; in-memory doubles are acceptable
  here where a case doesn't specifically need real persistence semantics.
- Replacing the eval harness or the 80% line-coverage gate (unchanged
  non-goals carried from `proxy-integration-gate`).

## Decisions

**D1: Construct `AppState` field-by-field, confirming each `Option` before
defaulting to `None`.**
Rather than assume which fields can be `None`, go through the 8 cases and
determine per-field: streaming/tool-loop/agent-selection cases need real
`orchestrator`, `provider_registry`, `model_router`, `sessions`,
`run_manager` — these are non-optional and get real construction regardless.
Memory and RAG cases specifically need `memory_service: Some(...)`; if
`MemoryService::new` requires a real backend URL, use an embedded/in-memory
SurrealDB mode if the `surreal-memory` dependency supports one (confirm at
implementation time — do not assume), or fall back to a `#[ignore]`'d
case with a clear reason if no in-memory mode exists, rather than silently
skip coverage. Credential-chain resolution can likely use
`InMemoryCredentialStore` (already used by
`tests/credentials_api_integration_test.rs`) via `provider_service`.
Alternative considered: mock every service behind trait objects for maximum
isolation — rejected, over-engineering (Rule 2) for an 8-case suite; real
constructors are already cheap for most fields.

**D2: One harness function, not one per case.**
A single `fn boot_test_server(fixtures: FixtureSet, needs: ServiceNeeds) ->
TestServerHandle` (exact shape TBD at implementation time) takes a small
struct describing which optional services a case needs, builds `AppState`
accordingly, and returns a bound `axum_test::TestServer` or real
`TcpListener`-backed server (whichever `stream_mode: dual`'s SSE testing
requires — confirm which `axum_test` supports before choosing). Alternative
considered: one bespoke boot function per case — rejected, duplicates
`AppState` wiring 8 times.

**D3: Matrix + CI, carried from `proxy-integration-gate`'s D4 (see that
change's design.md).**
`tests/integration/live/MATRIX.md` maps `CH-## → case name(s)`; CI greps for
presence, advisory until CH-01..CH-04 land a case without drift.

## Risks / Trade-offs

- **[Risk]** Memory/RAG cases may have no viable in-memory/embedded backend,
  forcing a real DB dependency this tier was designed to avoid.
  **Mitigation:** confirm at implementation time (D1); if no embedded mode
  exists, mark those two cases `#[ignore = "needs a running SurrealDB/
  Postgres; see D1"]` rather than fabricate a fixture that doesn't prove
  anything, and note this explicitly in `MATRIX.md` and the phase plan.
- **[Risk]** `stream_mode: dual` (both OpenAI and AG-UI events on one
  connection) may need a raw `TcpListener` + streaming HTTP client rather
  than `axum_test::TestServer`, which could behave differently under test.
  **Mitigation:** verify `axum_test::TestServer`'s SSE support first; fall
  back to the raw-listener pattern from `tests/test_a2a_client.rs` if needed.
- **[Risk]** This change is itself an estimate that could reveal further
  sub-scope once started (mirroring `proxy-integration-gate`'s own history).
  **Mitigation:** if it does, apply the same discipline — pause, report,
  split further rather than silently descope requirements.

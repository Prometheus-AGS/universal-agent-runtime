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

- **[Risk — materialized] Memory/RAG cases have no hermetically-usable
  embedding provider.** The storage layer IS embedded (SurrealKV file, no
  network service — that part of the original mitigation was right). The
  gap is the embedding provider: `embedding_provider: "local"` requires
  `surreal-memory`'s `local-embeddings` Cargo feature, which this
  workspace's `Cargo.toml` does not enable anywhere (confirmed by grep, and
  by `memory_write_then_recall` actually hitting the resulting 503 at
  runtime — `start_server` swallows the construction error into
  `memory_service: None` with only a `tracing::error!`, no visible signal
  without going looking). `"openai"`/`"cohere"` need a real API key, which
  this hermetic tier can't depend on.
  **RAG (2.7) has a second, independent reason, found while diagnosing the
  first via a `tracing_subscriber` added to `harness.rs`:**
  `VectorMatcher::embed_batch`'s local-model path
  (`src/uar/runtime/matching/vector.rs:210-213`) has its real
  `model.forward(...)` call commented out and unconditionally returns
  all-zero placeholder vectors ("Burn inference running in generic
  placeholder mode") — not environment-specific, not a missing dependency,
  just incomplete code. Search after successful ingestion always returns
  `results: []` because query/document embeddings are always identical
  zeros. Also found along the way: `update_document_status`
  (`src/uar/persistence/providers/surreal.rs:524`) uses the SurrealQL
  function `type::thing(...)`, rejected by the pinned SurrealDB `=3.0.5`
  ("did you maybe mean type::record") — status writes silently fail, so a
  document that ingests successfully stays reported `"pending"` forever.
  Both are real, disclosable, pre-existing product gaps, not test-harness
  issues.
  **Resolution (per the original mitigation plan for this exact outcome):**
  `#[ignore]` both memory (2.6) and RAG (2.7) cases with reasons naming the
  actual root causes (missing `local-embeddings` feature; placeholder
  embeddings), not a vague "needs a DB" reason, and note the gap in
  `MATRIX.md`. Each root cause flagged separately via `spawn_task` for a
  dedicated follow-up session — enabling `local-embeddings` project-wide is
  a build-footprint/dependency decision (comparable to the `palace`
  feature's documented `rusqlite`-conflict trade-off next to it in
  `Cargo.toml`); wiring up real Burn inference is a real implementation
  task. Both are well out of scope
  for this test-infra change to decide unilaterally; flagged as a follow-up
  candidate, not resolved here.
- **[Risk]** `stream_mode: dual` (both OpenAI and AG-UI events on one
  connection) may need a raw `TcpListener` + streaming HTTP client rather
  than `axum_test::TestServer`, which could behave differently under test.
  **Mitigation:** verify `axum_test::TestServer`'s SSE support first; fall
  back to the raw-listener pattern from `tests/test_a2a_client.rs` if needed.
- **[Risk]** This change is itself an estimate that could reveal further
  sub-scope once started (mirroring `proxy-integration-gate`'s own history).
  **Mitigation:** if it does, apply the same discipline — pause, report,
  split further rather than silently descope requirements.
- **[Risk — materialized, fixed] Concurrent `boot_test_server` calls cause
  health-check timeouts.** Confirmed by running the full `live::` suite
  together (as opposed to per-module during development): 7/16 tests failed
  at the 10s health-check with `cargo test`'s default parallelism, while
  every module passed 100% in isolation. Booting a real server (embedded
  SurrealDB persistence, real orchestrator, real MCP subprocess spawns via
  the repo's `mcp.json`) several times concurrently is resource-heavy enough
  to matter.
  **Fix:** every test that calls `boot_test_server` (in `harness.rs` and
  `baseline_cases.rs`) is `#[serial]` (`serial_test`, matching the pattern
  already used in `backend.rs` for a different reason). This forces
  server-boot tests to run one at a time within the binary; cheap
  non-booting tests (`stub_llm.rs`'s direct fixture tests) remain
  unaffected. Relevant for task 3.0's CI-timing concern too: a CI runner is
  likely more resource-constrained than this dev machine, so serialization
  isn't just a local nicety — it's load-bearing for CI reliability.

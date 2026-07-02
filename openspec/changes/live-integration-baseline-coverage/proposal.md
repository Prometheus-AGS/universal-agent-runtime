## Why

`proxy-integration-gate` shipped an interchangeable live/recorded backend
mechanism (stub LLM server + `UAR_LIVE_INTEGRATION_BACKEND` selection +
health-check script) but deliberately stopped short of proving any actual
feature works — that required a minimal real-server test harness, which
turned out to be its own separately-sized problem (no existing test in this
repo boots a full `AppState`; see `proxy-integration-gate`'s design.md
Risks). This change closes that gap: build the minimal harness, prove the
phase's baseline features work end-to-end against it, and wire the result
into CI as the per-change feature-coverage contract every later Round 1-4
change depends on.

## What Changes

- **Minimal server-boot harness**: construct enough of a real `AppState` to
  serve chat-completion HTTP/SSE traffic through the actual orchestrator —
  real `provider_registry` + `model_router` (the actual point of this tier),
  pointed at the backend resolved by `proxy-integration-gate`'s
  `tests/integration/live/backend.rs`. Every other `AppState` field starts
  from the lightest construction that lets the route under test actually
  run: `Option` fields (`persistence`, `memory_service`, `live_bus`,
  `ingest_service`, `api_key_service`, `provider_service`,
  `compiler_service`, `settings_manager`) stay `None` unless a specific
  baseline case needs one; non-optional fields
  (`sessions`, `run_manager`, `vector_matcher`, `skill_service`,
  `native_skill_registry`, `federated_agent_registry`, `actor_system`,
  `governance_engine`, `prompt_cache_provider`, `user_settings_store`,
  `a2ui_registry`, `agent_sessions`) get their existing cheap/in-memory
  constructors. **Confirmed per case, not assumed up front** — see design.md.
- 8 baseline feature cases against the real HTTP server: streaming chat under
  `stream_mode: openai`, `agui`, and `dual`; an MCP tool-loop round-trip;
  agent selection via the `agent_id` request field; a memory write followed
  by a recall; a RAG document ingest followed by a retrieval; and
  credential-chain resolution. Each case runs against both the `recorded` and
  (locally) the `live` backend.
- `tests/integration/live/MATRIX.md`: a living `CH-## → live case(s)` table.
  Every later Round 1-4 change is required to append its row in the same PR
  that lands its feature — the phase's "100% feature coverage" contract,
  distinct from and additive to the existing 80% line-coverage gate in
  `comprehensive-tests.yml`.
- CI wiring: an additive job running the recorded-backend baseline cases plus
  a matrix-presence check, advisory (non-blocking) until CH-01/02/03/04 have
  each added a case without matrix drift.
- Docs: `tests/integration/live/README.md` (or an `evals/README.md`
  extension) distinguishing this gate from the eval harness's model-quality
  gate, and confirming the tooling runs identically from Codex, Claude Code,
  Cursor, and OpenCode.

## Capabilities

- **Modified Capabilities:**
  - `live-integration-testing` — adds baseline feature-case coverage, the
    per-change matrix contract, and CI wiring on top of the backend-selection
    mechanism `proxy-integration-gate` added. (Delta spec written against
    `proxy-integration-gate`'s in-flight spec since the capability has not
    yet archived to `openspec/specs/`; both changes target the same
    capability name by design.)

## Impact

- **Affected code:** new `tests/integration/live/harness.rs` (or similar) for
  the minimal `AppState` construction, 8 new test cases in
  `tests/integration/live/`, `tests/integration/live/MATRIX.md`, a small
  addition to `.github/workflows/` (or an extension of
  `comprehensive-tests.yml`).
- **Affected config:** none new.
- **Dependencies:** none new — reuses `proxy-integration-gate`'s
  `backend::resolve()` and `stub_llm` fixtures, and existing `AppState`
  constructors already used by `src/server.rs`.
- **KBD workflow state:** yes — `.kbd-orchestrator/phases/uar-next-harness/`
  plan Amendment A3 inserts this change immediately after
  `proxy-integration-gate` in Round 1; CH-01..CH-04's "add a matrix row"
  completion criterion depends on `MATRIX.md` existing, i.e. on this change.

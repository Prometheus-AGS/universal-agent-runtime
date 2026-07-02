# Live Integration Feature Coverage Matrix

This is the per-change **feature-coverage contract** for the `uar-next-harness`
phase (plan Amendment A2). Every change that lands a user-facing runtime
feature MUST add a row here mapping its `CH-##` identifier to at least one
live integration test case (`tests/integration/live/`). CI greps this file for
the change's `CH-##` token and fails when a plan-referenced change lands
without a matching row (see `.github/workflows/live-integration.yml`).

This gate is **distinct from**:
- the 80% line-coverage gate (`comprehensive-tests.yml`) — this is *feature*
  coverage, not line coverage; 100% line coverage is explicitly rejected (A2.1).
- the eval harness (`evals/`) — that gates *model-output quality*; this gates
  *feature correctness*.

## Backend

Each case runs against one of two interchangeable backends selected by
`UAR_LIVE_INTEGRATION_BACKEND` (`tests/integration/live/backend.rs`):
- **recorded** (default; CI) — in-process stub LLM (`stub_llm.rs`); deterministic.
- **live** (operator; local) — the real proxy at `127.0.0.1:8181` via
  `scripts/live-integration.sh`; non-deterministic.

Cases that assert exact model content are **recorded-only** by nature (a live
model won't reproduce canned text). Only `backend_parametric_chat_smoke` is
wired through the switch with content-tolerant assertions, so it runs on both.

## Baseline cases (pre-existing; no `CH-##` — established by CH-22b)

| Feature | Test case | Backend | Status |
|---|---|---|---|
| Streaming chat, `stream_mode: openai` | `streaming_chat_openai_mode` | recorded | ✅ |
| Streaming chat, `stream_mode: agui` | `streaming_chat_agui_mode` | recorded | ✅ |
| Streaming chat, `stream_mode: dual` | `streaming_chat_dual_mode` | recorded | ✅ |
| MCP/native tool-loop round-trip | `tool_loop_round_trip` | recorded | ✅ |
| Agent selection via `agent_id` | `agent_selection_resolves_both_builtin_agents` | recorded | ✅ |
| Memory write → recall | `memory_write_then_recall` | recorded | ⏸️ `#[ignore]` — needs `local-embeddings` Cargo feature (not enabled); see design.md Risk 1 / `task_188b4179` |
| RAG ingest → retrieve | `rag_ingest_then_retrieve` | recorded | ⏸️ `#[ignore]` — zero-vector placeholder embeddings + SurrealQL `type::thing` bug; see `task_188b4179`, `task_7c2fd7ee` |
| Credential-chain CRUD (encrypted, authed) | `credential_chain_put_then_list` | recorded | ✅ |
| Dual-backend parity smoke | `backend_parametric_chat_smoke` | **both** | ✅ |

## Per-change rows (append one per landing feature change)

| CH-## | Feature | Test case(s) | Notes |
|---|---|---|---|
| _(none yet — CH-01..CH-04 add theirs on landing)_ | | | |

<!--
When CH-01 (a2a-grpc-enable) lands, add e.g.:
| CH-01 | A2A gRPC task round-trip | `a2a_grpc_task_roundtrip` | |
CH-03 (provider-health-failover): a case inducing a 429 and asserting failover.
CH-04 (prompt-dialect-engine): a case asserting dialect params in the captured request.
CH-21 (agui-spec-alignment): AG-UI vocabulary conformance.
-->

## CI enforcement status

**Advisory (non-blocking)** for this change's own landing (per design.md open
question). Promotion criteria to **blocking**: once CH-01/02/03/04 have each
added a case here without matrix drift, flip the `continue-on-error` flag in
`.github/workflows/live-integration.yml` to make the matrix-presence check
gating.

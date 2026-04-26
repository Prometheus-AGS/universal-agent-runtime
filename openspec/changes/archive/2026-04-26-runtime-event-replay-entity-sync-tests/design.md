## Context

The runtime console already renders runtime entities from the Prometheus entity graph. Runtime event ingestion is centralized in `frontend/src/entities/runtime-ingest.ts`, entity schemas and types are registered in `frontend/src/entities/schemas.ts` and `frontend/src/entities/types.ts`, and console pages read graph state through `useEntities(...)` in `frontend/src/admin/pages/runtime-console-page.tsx`.

The current validation gap is not layout coverage; `runtime-console-live-visual-tests` already covers shell navigation and stable console rendering. The missing proof is that replayed runtime events drive the same graph state and visible UI updates that live SSE, AG-UI, A2UI, memory, approval, provider, and routing updates are expected to drive in production.

The implementation must preserve frontend layering:

- Components render graph-backed state and call hooks only.
- Hooks expose store state/actions only.
- Stores own state and call services.
- Services own HTTP/SSE/stream I/O.

## Goals / Non-Goals

**Goals:**

- Provide deterministic replay fixtures for representative runtime event envelopes.
- Verify `ingestRuntimeEvent(...)` and `ingestAgUiEvent(...)` normalize events into the expected entity types and stable entity ids.
- Verify replayed updates merge into existing graph entities rather than producing duplicate or stale state.
- Verify runtime console pages display replayed run, step, tool, approval, artifact, memory, provider health, route decision, AG-UI, and A2UI state without a manual refresh.
- Keep replay tests independent from real provider keys, network calls, and live backend processes where possible.
- Record OpenSpec/KBD progress and verification evidence as part of the validation hardening workflow.

**Non-Goals:**

- Do not redesign the runtime console UI in this change.
- Do not change backend protocol semantics for OpenAI, Anthropic, AG-UI, A2UI, MCP, or liter-llm routing.
- Do not add new provider integrations or real provider API calls.
- Do not replace the Prometheus entity graph library or introduce a parallel frontend state model.
- Do not make Surreal Memory the source of truth for this test workflow; `.kbd-orchestrator/` remains authoritative.

## Decisions

### 1. Fixture-first replay coverage

Use static TypeScript fixtures that model the runtime event envelope variants already accepted by `ingestRuntimeEvent(...)` and `ingestAgUiEvent(...)`.

The fixture set should cover:

- `run_started`, `run_updated`, `run_finished`, and `run_failed`
- `step_started`, `step_updated`, and `step_finished`
- `tool_call_started`, `tool_call_delta`, `tool_call_finished`, and `tool_call_failed`
- `approval_requested` and `approval_updated`
- `artifact_created` and `artifact_updated`
- `memory_recalled` and `memory_updated`
- `provider_health`
- `model_route_decision`
- `a2ui_surface`
- AG-UI event envelopes passed through `ingestAgUiEvent(...)`

Rationale: fixtures keep the test deterministic and avoid real network dependencies. The alternative was to build a replay HTTP endpoint first, but that would test backend plumbing before proving the frontend normalization contract.

### 2. Test the existing ingest boundary directly

Add unit-style frontend tests around `frontend/src/entities/runtime-ingest.ts` that reset the graph store, replay fixture events, and assert graph entities by type and id.

The tests should verify:

- Event type to entity type mapping.
- Id fallback behavior using explicit ids, payload/data ids, run ids, and sequence values.
- Payload/data merge behavior.
- Update behavior when later events target an existing id.
- AG-UI event normalization into `RuntimeAgUiEvent`.

Rationale: the ingest boundary is the lowest-cost place to catch regressions. Testing only through Playwright would make failures harder to diagnose and slower to run.

### 3. Add a browser-visible replay path only for tests

Expose replay fixtures to Playwright through test-only browser evaluation or test fixtures, not through a production navigation affordance. The E2E test can navigate to runtime console pages, call the imported ingest functions or a guarded test helper from the browser context, then assert visible state updates.

The visible assertions should cover:

- Cockpit stat tiles for active runs, tool calls, approvals, and memory events.
- Live Runs and Execution Timeline rows.
- Provider Health rows.
- Runs detail artifacts and tool calls.
- Tool Approvals pending/updated state.
- Protocol console AG-UI, A2UI, and liter-llm route decision counts.

Rationale: the user-facing requirement is realtime UI refresh, but production should not gain a synthetic replay endpoint solely for tests. If Vite import boundaries make direct browser evaluation impractical, a small test-only helper can be guarded by `import.meta.env.MODE === "test"` or an equivalent Playwright-only initialization path.

### 4. Keep realtime stream work scoped to verification

This change should validate that the graph update path works for replayed events. It should not rewrite `frontend/src/entities/sync.ts` or the live SSE/Electric adapters unless tests expose a real defect in the existing integration.

Rationale: replay tests are a hardening step, not a transport refactor. Keeping transport changes out reduces blast radius and keeps the change focused.

### 5. Treat provider compatibility as observable state

Provider health and model route decision fixtures should use realistic liter-llm-style fields such as `provider_id`, selected provider/model, capability requirements, latency, and routing reason. Tests should assert that these become visible in the runtime/provider/protocol surfaces.

Rationale: provider compatibility work needs a stable UI observability contract. The tests should prove that route decisions and health changes can be inspected without requiring real OpenAI, Anthropic, OpenRouter, Moonshot, Qwen, Fireworks, DeepSeek, or Minimax credentials.

### 6. Update workflow state after each artifact and verification step

Each `/opsx:continue`, `/opsx:apply`, `/opsx:verify`, and archive step should update `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json` and `.kbd-orchestrator/current-waypoint.*`.

Rationale: this keeps Codex, Claude Code, Cursor, and OpenCode aligned on the same source-of-truth workflow state. Surreal Memory may mirror the state when available, but file state remains authoritative.

## Risks / Trade-offs

- [Risk] Browser replay tests may require test-only access to frontend internals. -> Mitigation: prefer a narrow helper that only imports existing ingest functions and fixture data, and keep it gated to test mode.
- [Risk] Store reset APIs in `@prometheus-ags/prometheus-entity-management` may not expose a clean public reset path. -> Mitigation: inspect the store API during implementation and isolate reset logic in the test harness; avoid mutating production code if the existing API is sufficient.
- [Risk] Replay fixtures can drift from backend event shapes. -> Mitigation: document the fixture envelope fields and align names with `EVENT_TYPE_MAP`; add backend fixture parity later only if protocol tests reveal mismatches.
- [Risk] Playwright UI assertions can become brittle if copy changes. -> Mitigation: use stable test ids for structural assertions and visible text only where it represents user-facing runtime state.
- [Risk] The existing global OpenSpec validation is already blocked by unrelated `implement-opencode-suggestions` issues. -> Mitigation: validate this change directly and record the unrelated blocker separately in KBD progress.


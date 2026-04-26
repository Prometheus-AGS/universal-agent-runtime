## 1. Workflow Bootstrap

- [x] 1.1 Read `.kbd-orchestrator/current-waypoint.*`, this change proposal, design, specs, and the frontend layering rules before implementation.
- [x] 1.2 Confirm Codex, Claude Code, Cursor, and OpenCode workflow state still points to OpenSpec plus KBD as the shared source of truth for this phase.
- [x] 1.3 Record implementation start for `runtime_event_replay_tests` in `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json`.

## 2. Replay Fixtures

- [x] 2.1 Add deterministic frontend runtime replay fixtures for run lifecycle, run step, tool call, approval, artifact, memory, provider health, model route decision, and A2UI surface events.
- [x] 2.2 Add deterministic AG-UI replay fixtures for event envelopes passed through `ingestAgUiEvent(...)`.
- [x] 2.3 Ensure fixtures use synthetic ids, provider/model names, payloads, and timestamps and do not require live provider keys, live model calls, or external network access.

## 3. Entity Graph Ingest Tests

- [x] 3.1 Add frontend tests that reset or isolate the Prometheus entity graph store before each replay test.
- [x] 3.2 Test `ingestRuntimeEvent(...)` maps replayed event types to `RuntimeRun`, `RuntimeRunStep`, `RuntimeToolCall`, `RuntimeApproval`, `RuntimeArtifact`, `RuntimeMemoryEvent`, `RuntimeA2uiSurface`, `RuntimeModelRouteDecision`, and `RuntimeProviderHealth`.
- [x] 3.3 Test replayed events with explicit ids, payload/data ids, run ids, and sequence values produce stable entity ids.
- [x] 3.4 Test later replayed events targeting an existing id update the existing graph entity instead of producing duplicate logical state.
- [x] 3.5 Test `ingestAgUiEvent(...)` creates `RuntimeAgUiEvent` entities with run id, event type, sequence, payload, and updated timestamp.

## 4. Runtime Console Replay Visibility

- [x] 4.1 Add a test-only replay helper or Playwright fixture that can ingest runtime replay fixtures without exposing a production replay UI.
- [x] 4.2 Add Playwright coverage showing `/admin/runtime` updates active runs, execution timeline, provider health, approval count, tool call count, and memory event count after replay without refresh.
- [x] 4.3 Add Playwright coverage showing `/admin/runs` updates run detail, artifacts, and tool calls after replay without refresh.
- [x] 4.4 Add Playwright coverage showing `/admin/approvals` displays replayed pending approvals and updated approval statuses without refresh.
- [x] 4.5 Add Playwright coverage showing `/admin/protocols` displays replayed AG-UI event counts, A2UI surface counts, and liter-llm route decision counts without refresh.
- [x] 4.6 Preserve the existing unseeded runtime console empty-state visual tests.

## 5. Validation

- [x] 5.1 Run the targeted runtime event replay/entity-sync frontend test command and confirm it exits with status code 0.
- [x] 5.2 Run `bun run lint` from `frontend/` and confirm it exits with status code 0.
- [x] 5.3 Run `bun run typecheck` from `frontend/` and confirm it exits with status code 0.
- [x] 5.4 Run the targeted runtime console Playwright suite and confirm it exits with status code 0.
- [x] 5.5 Run `openspec validate runtime-event-replay-entity-sync-tests` and confirm it exits with status code 0.
- [x] 5.6 Run `git diff --check` from the repository root and confirm it exits with status code 0.

## 6. Workflow Evidence

- [x] 6.1 Write a KBD verification report summarizing replay fixtures, entity graph tests, Playwright replay coverage, validation commands, and any blockers.
- [x] 6.2 Update `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json` so `runtime_event_replay_tests` records verified or complete status when validation passes.
- [x] 6.3 Update `.kbd-orchestrator/current-waypoint.*` with the next recommended runtime-console-validation-hardening change after this one completes.

ASSESSMENT: runtime-console-validation-hardening
Project: universal-agent-runtime
Date: 2026-04-26
Codebase baseline: The runtime-console foundation is committed, current-change OpenSpec validation and TypeScript typecheck pass, but the phase cannot close until lint, visual/runtime verification, workflow memory mirroring, global OpenSpec hygiene, and Moonshot provider status are resolved.
Cross-tool progress: 1 prior phase reflected by Codex; no other tool progress recorded for this phase yet.

IMPLEMENTATION STATUS
- Frontend lint closure: PARTIAL - `bun run lint` fails with 8 errors and 9 warnings. Blocking errors are unused variables in `frontend/e2e/chat-agent-selection.spec.ts:38`, `frontend/src/admin/pages/providers-page.tsx:354`, `frontend/src/features/chat/capability-toggles.tsx:164`, plus React hook `set-state-in-effect` violations in `frontend/src/components/model-selector.tsx:64`, `frontend/src/features/chat/agent-selector.tsx:62`, `frontend/src/features/chat/capability-toggles.tsx:178`, and `frontend/src/pages/chat-page.tsx:85`.
- Frontend type safety: DONE - `bun run typecheck` passes with `tsc -b`.
- Runtime console entity model: DONE - runtime entity tables are registered in `frontend/src/entities/sync.ts`, and runtime ingestion helpers exist in `frontend/src/entities/runtime-ingest.ts`.
- Runtime console live UI verification: PARTIAL - `frontend/src/admin/pages/runtime-console-page.tsx` renders runtime graph surfaces, but this assessment found no committed Playwright visual/mobile checks proving the shell, command palette, protocol views, and live graph updates work end to end.
- AG-UI/A2UI/runtime stream integration: PARTIAL - `frontend/src/stores/chat-stream-store.ts` handles AG-UI and A2UI stream payloads, and `frontend/src/entities/runtime-ingest.ts` can normalize runtime events, but the hardening phase still needs tests proving streamed events flow into visible runtime entity views without manual refresh.
- Surreal Memory workflow mirror: PARTIAL - `mcp.json` registers `surreal_memory`, and `/mcp/memory` exists in backend config and memory modules, but there is no automated KBD workflow mirror round-trip for project, phase, task, assessment, plan, blocker, and verification entities.
- OpenSpec current-change validation: DONE - `openspec validate runtime-console-entity-workflow` passes.
- OpenSpec global validation: PARTIAL - `openspec validate --changes` fails because older `openspec/changes/implement-opencode-suggestions/` requirement deltas lack SHALL/MUST wording in seven spec files.
- Provider/protocol compatibility closure: PARTIAL - focused backend tests for OpenAI/Anthropic normalization, provider fallback URLs, and knowledge search persistence pass. Moonshot Kimi k2.6 remains unresolved because the supplied key was rejected by the provider during live testing.
- Runtime-console change archive readiness: PARTIAL - `runtime-console-entity-workflow` should not be archived until lint is clean, runtime UI visual checks pass, Surreal Memory mirror tests pass or are explicitly deferred, and global OpenSpec validation is repaired or the unrelated invalid change is isolated.

CROSS-TOOL PROGRESS
- `runtime-console-ux`: reflected (by codex) - prior reflection set this phase as the next waypoint and identified the hardening gaps.
- `runtime-console-validation-hardening`: assessed (by codex) - this file records the gap baseline for closing all prior reflection items.

SPEC GAP SUMMARY
- `runtime-console-entity-workflow` requires workflow state changes to be mirrored to `surreal_memory`; implementation has MCP registration but lacks a proven mirror writer/round-trip test.
- `runtime-console-entity-workflow` requires visible console surfaces to update when runtime entities update; entity types exist, but end-to-end live update tests are missing.
- `runtime-console-entity-workflow` requires operator protocol/provider inspection; UI surfaces exist, but automated desktop/mobile acceptance and stream replay checks are missing.
- Global OpenSpec validation is not clean because `implement-opencode-suggestions` has malformed requirements. This is outside the runtime-console change but blocks repository-wide validation confidence.
- Provider matrix status has one unresolved vendor path: Moonshot Kimi k2.6 authentication failure. The hardening phase must either validate with a working key or record an explicit unsupported/credential-blocked status.

BUILD HEALTH
- build check: PASS - `bun run typecheck`.
- current-change spec check: PASS - `openspec validate runtime-console-entity-workflow`.
- global spec check: FAIL - `openspec validate --changes` fails on `implement-opencode-suggestions`.
- frontend lint: FAIL - `bun run lint` reports 8 errors and 9 warnings.
- backend focused tests: PASS - `cargo test --quiet normalize_legacy_openai_base_url`, `cargo test --quiet unwrap_record_id_supports_table_key_shape`, and `cargo test --quiet llm::registry::tests::test_seed_from_llm_config_enriches_provider_base_url`.
- test coverage: PARTIAL - backend provider/knowledge persistence tests exist; frontend runtime-console live update, visual, and memory mirror tests are missing.

CONSTRAINT CHECK
- AGENTS.md violations: FRONTEND LAYERING RISK - `frontend/src/entities/sync.ts` owns `fetch`/`EventSource` directly. If entity sync is treated as a service layer this may be acceptable, but the hardening phase should either move it under `frontend/src/services/`/stores or document it as entity infrastructure outside component/hook/store layering. Components still appear to render via hooks and graph state.
- constraints.md violations: N/A - no `.kbd-orchestrator/constraints.md` file exists.
- zero-warning policy: VIOLATED - frontend lint warnings remain, and the hardening phase must close or explicitly justify them.

GOAL PROGRESS
- Close all frontend lint errors and warnings: NOT MET - current lint output still blocks the goal.
- Prove runtime console desktop/mobile UX and live updating: NOT MET - no committed visual/live update test evidence exists.
- Prove Surreal Memory workflow mirror round-trip: NOT MET - MCP endpoint registration exists, but mirror persistence/recovery tests are missing.
- Restore meaningful global OpenSpec validation: NOT MET - unrelated active change still fails validation.
- Resolve Moonshot provider status: NOT MET - current status is provider authentication failure.
- Archive or mark `runtime-console-entity-workflow` ready for archive: NOT MET - dependent hardening gaps remain.

CLOSURE CRITERIA FOR THIS PHASE
- `bun run lint` exits 0 with no errors and no warnings, or warnings are moved behind a documented accepted rule exception.
- `bun run typecheck` remains green after lint fixes.
- Playwright or equivalent browser checks cover desktop and mobile runtime console navigation, command palette, protocol/provider surfaces, and non-overlapping responsive layout.
- A runtime event replay test proves run/tool/approval/artifact/memory/provider/AG-UI/A2UI updates enter the entity graph and update visible UI without manual refresh.
- A Surreal Memory workflow mirror test proves create, retrieve, update, and conflict-resolution behavior for KBD project/phase/task/assessment/plan/blocker/verification entities.
- `openspec validate --changes` passes, either by repairing `implement-opencode-suggestions` or archiving/removing that invalid active change through the OpenSpec workflow.
- Moonshot provider is either validated with a working key or documented as blocked by credentials with UI/provider status that makes the failure explicit.
- `runtime-console-entity-workflow` is verified and archived, or left active with a narrow documented reason.

ASSESSMENT COMPLETE

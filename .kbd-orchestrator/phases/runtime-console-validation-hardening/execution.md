EXECUTION: runtime-console-validation-hardening
Project: universal-agent-runtime
Date: 2026-04-26T04:22:54-05:00
Selected backend: hybrid
Dispatched to: OpenSpec canonical workflow; Codex for current orchestration
Backend rationale: This phase needs traceable acceptance criteria across frontend UI, realtime state, provider diagnostics, MCP memory, and OpenSpec workflow cleanup. OpenSpec remains the canonical execution surface, while bounded implementation changes can be handed to the tool named in the phase plan when that tool has a better fit.
Backend entrypoint: /opsx:continue openspec-global-validation-cleanup
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/runtime-console-validation-hardening/plan.md

EXECUTION SCOPE

- frontend-lint-zero-warning: make frontend lint and typecheck clean with zero frontend lint warnings.
- runtime-console-live-visual-tests: prove runtime console layout, navigation, command palette, and responsive rendering across desktop/mobile.
- runtime-event-replay-entity-sync-tests: prove runtime events normalize into the entity graph and update visible UI without manual refresh.
- surreal-memory-workflow-mirror-tests: prove KBD workflow state can round-trip through the UAR Surreal Memory MCP mirror.
- openspec-global-validation-cleanup: repair or retire invalid active OpenSpec deltas so repository-wide change validation passes.
- moonshot-provider-status: resolve or explicitly classify Moonshot Kimi k2.6 compatibility without committing credentials.
- runtime-console-archive-readiness: run final verification and archive the runtime console workflow when closure gates pass.

DISPATCH CONTRACTS

- frontend-lint-zero-warning -> OpenSpec + Codex
  Entry: archived at openspec/changes/archive/2026-04-26-frontend-lint-zero-warning/
  Progress file: .kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json
  Handoff: Completed and archived. Refiner QA passed.

- runtime-console-live-visual-tests -> OpenSpec + Codex
  Entry: archived at openspec/changes/archive/2026-04-26-runtime-console-live-visual-tests/
  Progress file: .kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json
  Handoff: Completed and archived. Refiner QA passed.

- runtime-event-replay-entity-sync-tests -> OpenSpec + Codex
  Entry: archived at openspec/changes/archive/2026-04-26-runtime-event-replay-entity-sync-tests/
  Progress file: .kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json
  Handoff: Completed and archived. Refiner QA passed.

- surreal-memory-workflow-mirror-tests -> OpenSpec + Codex
  Entry: archived at openspec/changes/archive/2026-04-26-surreal-memory-workflow-mirror-tests/
  Progress file: .kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json
  Handoff: Completed, verified, spec-synced, refiner QA passed, and archived.

- openspec-global-validation-cleanup -> OpenSpec + Codex
  Entry: /opsx:continue openspec-global-validation-cleanup
  Progress file: .kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json
  Handoff: Scaffold exists at openspec/changes/openspec-global-validation-cleanup/. Create proposal next, then repair invalid active requirement deltas or archive/remove stale invalid change through OpenSpec workflow until `openspec validate --changes` passes.
  HANDOFF NOTE for Codex:
  1. Read .kbd-orchestrator/current-waypoint.json.
  2. Read openspec/changes/openspec-global-validation-cleanup/ and inspect openspec/changes/implement-opencode-suggestions/.
  3. On start: update progress.json status -> IN_PROGRESS, started_by -> codex.
  4. On each artifact/task done: update artifacts_done/tasks_done in progress.json.
  5. Acceptance is `openspec validate --changes` passing or a documented narrow exception.

- moonshot-provider-status -> OpenSpec + Codex
  Entry: /opsx:new moonshot-provider-status
  Progress file: .kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json
  Handoff: Re-test Moonshot only through configured environment secrets, then document credential-blocked or compatible status in provider diagnostics without storing secrets.

- runtime-console-archive-readiness -> OpenSpec + Codex
  Entry: /opsx:new runtime-console-archive-readiness
  Progress file: .kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json
  Handoff: Run final KBD/OpenSpec/frontend/backend verification and archive runtime-console-entity-workflow if all gates pass.

APPROVAL GATES

- Do not commit or persist provider API keys or user secrets.
- Run artifact-refiner QA before archiving changes with three or more modified files unless the change is documentation-only.
- Do not archive runtime-console-entity-workflow until frontend lint/typecheck/e2e, OpenSpec validation, and focused backend protocol/provider gates pass or a narrow external-credential exception is documented.

FALLBACK CONDITIONS

- If a tool backend cannot maintain inspectable progress in progress.json, fall back to OpenSpec-only execution.
- If a change expands beyond its scoped acceptance criteria, split it into a follow-up OpenSpec change before implementation.
- If provider validation is blocked by invalid credentials, classify the provider as credential-blocked and proceed only with non-secret diagnostics.

VERIFICATION REQUIREMENTS

- openspec validate <change-id>
- openspec validate --changes
- bun run lint from frontend/
- bun run typecheck from frontend/
- targeted Playwright suites for runtime console desktop/mobile and live update behavior
- focused backend/provider/memory tests for affected runtime protocols

PROGRESS LEDGER

- [DONE] frontend-lint-zero-warning - OpenSpec + Codex
- [DONE] runtime-console-live-visual-tests - OpenSpec + Codex
- [DONE] runtime-event-replay-entity-sync-tests - OpenSpec + Codex
- [DONE] surreal-memory-workflow-mirror-tests - OpenSpec + Codex
- [IN_PROGRESS] openspec-global-validation-cleanup - OpenSpec + Codex
- [PENDING] moonshot-provider-status - OpenSpec + Codex
- [PENDING] runtime-console-archive-readiness - OpenSpec + Codex

OUTPUTS

- .kbd-orchestrator/phases/runtime-console-validation-hardening/execution.md
- .kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json
- .kbd-orchestrator/current-waypoint.json
- .kbd-orchestrator/current-waypoint.md
- .refiner/artifacts/runtime-event-replay-entity-sync-tests/refinement_log.md
- openspec/changes/archive/2026-04-26-surreal-memory-workflow-mirror-tests/
- openspec/changes/openspec-global-validation-cleanup/

BLOCKERS

- openspec validate --changes is currently blocked by unrelated invalid requirement wording in openspec/changes/implement-opencode-suggestions/.

REFLECTION HANDOFF

- kbd-reflect should compare the completed changes against the original runtime-console-validation-hardening assessment gaps, with special attention to live UI state, replayable protocol evidence, provider diagnostics, memory mirror recoverability, global OpenSpec validation, and final OpenSpec archive readiness.

EXECUTION READY

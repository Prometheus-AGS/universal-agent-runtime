# Handoff in — perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion > fix-embedded-sse-offline-reconnect

**Spawned by:** perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion

## Why this child was spawned

The parent strengthened its embedded SSE scenario by removing page reload and
manual runtime replay. The replacement scenario then timed out twice without a
second `/api/uar/sync/stream` request. Assessment also found a separate static
contract defect: the server emits named `entity.change` events with
`{table, action, id, record}`, while the embedded client listens only to
`onmessage` and expects `{entity_type, action, id, data}`. This is the fourth
supported-product defect encountered by the parent, so its execution contract
requires a narrowly scoped child rather than another inline repair.

## Inputs (paths from the parent node)

- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/assessment.md
- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/plan.md
- openspec/changes/screen-by-screen-validation/specs/product-validation-evidence/spec.md
- openspec/changes/screen-by-screen-validation/tasks.md
- tests/bdd/test-results/features-local-first-resil-d7113-t-duplicating-runtime-state/error-context.md

## Success criteria

- The embedded client and server share the named-event and payload contract.
- A focused unit test observes initial delivery, a forced error, one replacement
  connection, post-reconnect delivery, no duplicate delivery, and retry
  cancellation on unsubscribe.
- The live browser scenario instruments the EventSource used by the registered
  adapter, observes a second real request after a forced error, and shows one
  post-reconnect entity update without page reload or manual runtime replay.
- The prior timeout remains recorded as the negative observation; the corrected
  commands and actual outputs are recorded in the child `verification.md`.
- TypeScript Tier 0, focused Tier 1, child-completion Tier 2, the single required
  browser acceptance scenario, strict OpenSpec validation, and independent
  artifact review pass.

## Expected deliverables

- One completed OpenSpec change: `fix-embedded-sse-offline-reconnect`.
- Minimal embedded adapter implementation and focused unit/browser regression
  tests within the child scope.
- `verification.md`, artifact-refiner receipts, append-only learning, and a
  child `handoff-out.md` naming the exact parent tasks that may resume.
- A child commit that excludes `.claude/settings.local.json`, parent
  certification artifacts, generated browser output, and unrelated user work.

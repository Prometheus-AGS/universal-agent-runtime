# Handoff in — uar-post-lifecycle-verification-readiness › orchestrator-routing-reasoning-repair

**Spawned by:** uar-post-lifecycle-verification-readiness (operator decision, 2026-10-27), as a sibling of `uar-verification-simplification`.

## Why this child was spawned

The manual BDD track found two suspected product defects while running C34 (intent routing follows the live eligible roster). They need source changes, a build and an owned restart, which the manual one-scenario loop must not do inline.

## Inputs

- `.prometheus/verification/c34-live-roster-routing-attempt-00{1,2,3}-fail.json` and their evidence under `.prometheus/verification/live-roster-routing/`
- `scripts/behaviors/{features/live-roster-routing.feature,live-roster-routing.cucumber.mjs,steps/live-roster-routing.steps.mjs}`
- `.prometheus/verification/bdd-validation-catalog.md` (C34 contract, ceilings)
- `src/uar/runtime/graph/nodes/router_node.rs`, `src/uar/runtime/graph/delegation.rs`

## Success criteria

See goals.md G1–G3. C34 passes on Mac with the answer oracle that excludes reasoning, and its evidence shows genuine final answers.

## Expected deliverables

assessment.md, plan.md, an OpenSpec change with a spec delta; after approval, the implementation, the C34 receipt and handoff-out.md back to the BDD track.

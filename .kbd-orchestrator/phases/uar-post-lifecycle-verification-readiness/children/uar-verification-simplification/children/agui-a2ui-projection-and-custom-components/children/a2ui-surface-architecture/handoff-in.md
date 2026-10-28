# Handoff in — uar-post-lifecycle-verification-readiness› uar-verification-simplification› agui-a2ui-projection-and-custom-components› a2ui-surface-architecture

**Spawned by:** uar-post-lifecycle-verification-readiness› uar-verification-simplification› agui-a2ui-projection-and-custom-components

## Why this child was spawned

Change 1 of the parent (`openspec/changes/a2ui-surface-state-patches`) went
through four planning revisions. Each fresh-critic review found at least one
new critical defect (1, 1, 1, 2). Every revision had the server compute
materialized surface state and diff patches, which needs a Rust copy of
`@a2ui/web_core`'s reducer that must match the browser exactly on untrusted
input. Those two goals conflict. Surface state also lives in the run's
in-memory event stream (512-event ring, run-scoped), while surfaces outlive
runs. The operator opened this child to settle the architecture before any
further planning.

Note: `kbd-new-child.sh` wrote this file, `goals.md` and `scope.json` one
level up (`uar-verification-simplification/children/a2ui-surface-architecture`)
while the runtime nested the phase here. They were moved here on 2026-10-28.

## Objectives (operator-accepted framing, 2026-10-28)

1. Live: users see surfaces appear and change as the agent works.
2. Recoverable: reload, reconnect, a second device or the console shows the current surface.
3. Interactive across turns: an action's continuation run can update the same surface.
4. Extensible: registered custom components are validated by the server and rendered by every client from one catalog definition.
5. Safe: untrusted agent output cannot crash the server, inflate storage, or inject code.

## Options under comparison

- A. Server reducer with diff patches (the change 1 path).
- B. Server-authoritative reducer that sends whole surfaces.
- C. Message log as source of truth; clients reduce.
- D. One shared reducer implementation (WASM or embedded JS).
- E. A narrowed UAR A2UI profile (combines with B or C).

## Inputs (paths from the parent node)

- `../../../agui-a2ui-projection-and-custom-components/assessment.md` (top-level child directory, where the parent's authored artifacts live)
- `../../../agui-a2ui-projection-and-custom-components/plan.md`
- `openspec/changes/a2ui-surface-state-patches/design.md` (revisions 1-4 and their reviews)

## Success criteria

- An assessment with cited upstream evidence and a complete UAR consumer inventory.
- An analysis scoring A-E against the five objectives, with a recommendation and its costs.
- The loop stops before Plan for the operator's decision.

## Expected deliverables

- `assessment.md` and `analysis.md` in this directory.
- A decision request for the operator; the decision goes in `handoff-out.md` once made.

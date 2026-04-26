## Why

The runtime console validation-hardening phase has completed its supporting hardening changes. The remaining work is to prove that the phase is archive-ready, close the final unchecked validation task in `runtime-console-entity-workflow`, and record a single KBD handoff for reflection.

## What Changes

- Run and record the final phase gate: active OpenSpec validation, frontend lint/typecheck, targeted frontend tests, focused backend tests, and static asset cleanliness.
- Mark the runtime console entity workflow lint/test validation task complete when the gate passes.
- Archive `runtime-console-entity-workflow` after verification.
- Record refiner/KBD evidence for final phase closure.

## Capabilities

### New Capabilities

- `runtime-console-phase-archive-readiness`: Runtime console refactor phases can be closed only after final validation evidence is recorded, dependent changes are archived, and the canonical runtime-console OpenSpec change is either archived or documented with a narrow blocker.

### Modified Capabilities

- None.

## Impact

- OpenSpec/KBD workflow state only.
- No new runtime features are intended.
- The readiness change may update task checkboxes and archive completed OpenSpec changes after validation passes.

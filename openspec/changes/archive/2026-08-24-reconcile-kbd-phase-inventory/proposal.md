## Why

The canonical KBD run still reports legacy phases as pending even when their own reflection artifacts record completion or later phases supersede their remaining work. This makes status output misleading and risks deleting worktrees or branches without first checking whether they contain unique work.

## What Changes

- Inventory every registered top-level phase and record an evidence-backed disposition: complete, cancelled as superseded or no longer relevant, or retained as active.
- Apply legal KBD state transitions so canonical status matches the recorded disposition without rewriting historical phase artifacts by hand.
- Preserve the evidence for partial phases whose remaining outcomes were completed by named successor phases.
- Audit phase-associated Git worktrees and branches for dirty, ignored, unique, or unmerged content before removing only those proven obsolete.
- Verify that no stale active phase cursor or phase-associated worktree remains after reconciliation.

## Capabilities

### New Capabilities

- `kbd-phase-inventory-governance`: Defines evidence, transition, and Git-safety requirements for reconciling a KBD phase estate.

### Modified Capabilities

None.

## Impact

This change updates KBD orchestration metadata, records the reconciliation evidence, and may remove obsolete phase-associated local Git worktrees and branches. It does not change product runtime behavior, public APIs, application types, or GitHub Actions.

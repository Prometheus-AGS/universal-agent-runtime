## Why

Catalog matching and tool attribution alone cannot show that the right governed skill was activated or that it helped complete the task.

## What Changes

- Preserve explicit selection and conversation > agent > global precedence.
- Restore immutable active skill bodies before final budget validation.
- Require a frozen labeled evaluation dataset, baseline and numerical acceptance criteria before implementation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `skill-activation-runtime`: Extend the existing behavior with this change's testable contracts.

## Impact

scope: Existing skill catalog, ranking, activation, immutable body reattachment, attribution and local behavior fixtures.

Dependencies: harness-task-graph-lifecycle The phase index defines the shared-file serialization order. All mutation authority remains in trusted hosts; dependency pins remain unchanged.

Runtime UX: explicit typed failures and additive redacted events; preserve existing wire contracts and frontend/entity ownership. Provider compatibility: only documented and tested destinations qualify; unsupported paths remain explicit. Realtime state derives from persisted host state, never agent-only memory.

KBD impact: this Spec stage supplies proposals for Plan; implementation tasks remain unchecked. Five F4–F8 external-candidate research prerequisites and the exact provider-profile evidence matrix must be resolved before affected implementation commitments. No deployment, release, runtime test result or completed runtime fix is claimed.


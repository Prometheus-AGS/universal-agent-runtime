## Why

Completion-only evaluations and historical coverage reports cannot certify tool-inclusive production behavior or supported runtime profiles.

## What Changes

- Extend the existing runner with governed production-path evaluations and seeded regression gates.
- Replace obsolete product-test CI requirements with local deterministic and separately reported live evidence.
- Reproduce reported failures and coverage debt; require supported-profile evidence and a separately scoped local certification milestone.

## Capabilities

### New Capabilities

- `runtime-profile-certification`: Reproducible local certification across declared runtime profiles.

### Modified Capabilities

- `eval-harness`: Extend the existing behavior with this change's testable contracts.

## Impact

scope: Existing eval runner, deterministic tools/provider fixtures, baseline artifacts, coverage/profile evidence and local verification plans. Workflow edits only if Plan inventories an actual policy violation.

Dependencies: harness-knowledge-evidence The phase index defines the shared-file serialization order. All mutation authority remains in trusted hosts; dependency pins remain unchanged.

Runtime UX: explicit typed failures and additive redacted events; preserve existing wire contracts and frontend/entity ownership. Provider compatibility: only documented and tested destinations qualify; unsupported paths remain explicit. Realtime state derives from persisted host state, never agent-only memory.

KBD impact: this Spec stage supplies proposals for Plan; implementation tasks remain unchecked. Five F4–F8 external-candidate research prerequisites and the exact provider-profile evidence matrix must be resolved before affected implementation commitments. No deployment, release, runtime test result or completed runtime fix is claimed.


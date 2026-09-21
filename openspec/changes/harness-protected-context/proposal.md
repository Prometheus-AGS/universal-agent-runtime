## Why

Context reducers and tool-result ingestion can discard payloads before dispatch. A valid call/result pair is insufficient when its original data has already been truncated.

## What Changes

- Preserve canonical tool/data records and meaningful empty-text assistant messages across every reduction and resume path.
- Add a pure destination-budget contract that compresses only host-marked prose and rejects protected overflow.
- **BREAKING**: replace silent orphan removal, pair dropping, and canonical tool-result truncation with explicit invalid-history, incomplete-acquisition, or overflow outcomes.

## Capabilities

### New Capabilities

- `protected-context-budget`: Lossless protected data with explicit destination input budgets.

### Modified Capabilities

- `conversation-history-integrity`: Extend the existing behavior with this change's testable contracts.

## Impact

scope: Host context reduction, message persistence, native/MCP/graph tool receipt ingestion, request counting, and regression fixtures.

Dependencies: None; first implementation change. The phase index defines the shared-file serialization order. All mutation authority remains in trusted hosts; dependency pins remain unchanged.

Runtime UX: explicit typed failures and additive redacted events; preserve existing wire contracts and frontend/entity ownership. Provider compatibility: only documented and tested destinations qualify; unsupported paths remain explicit. Realtime state derives from persisted host state, never agent-only memory.

KBD impact: this Spec stage supplies proposals for Plan; implementation tasks remain unchecked. Five F4–F8 external-candidate research prerequisites and the exact provider-profile evidence matrix must be resolved before affected implementation commitments. No deployment, release, runtime test result or completed runtime fix is claimed.

## Why

Family-level dialect detection and separately prepared retries do not establish that the final provider request uses a compatible model template or fits its destination.

## What Changes

- Resolve versioned exact provider/endpoint/model templates, serialization contracts, supported settings and counting bounds.
- Prepare each attempt from canonical content; apply constrained routing and rebuild after failover.
- Reject unsupported compatibility explicitly and report redacted per-attempt budget/profile evidence.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `prompt-assembly`: Extend the existing behavior with this change's testable contracts.
- `model-path-resiliency`: Extend the existing behavior with this change's testable contracts.

## Impact

scope: Existing model routing, dialect resolution, compiler override propagation, prompt assembly, provider drivers and request-level fixtures.

Dependencies: harness-protected-context The phase index defines the shared-file serialization order. All mutation authority remains in trusted hosts; dependency pins remain unchanged.

Runtime UX: explicit typed failures and additive redacted events; preserve existing wire contracts and frontend/entity ownership. Provider compatibility: only documented and tested destinations qualify; unsupported paths remain explicit. Realtime state derives from persisted host state, never agent-only memory.

KBD impact: this Spec stage supplies proposals for Plan; implementation tasks remain unchecked. Five F4–F8 external-candidate research prerequisites and the exact provider-profile evidence matrix must be resolved before affected implementation commitments. No deployment, release, runtime test result or completed runtime fix is claimed.

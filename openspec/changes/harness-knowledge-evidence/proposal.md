## Why

Retrieval provenance and lexical answer matching cannot establish that the model used authorized evidence or represented stale and conflicting sources truthfully.

## What Changes

- Recheck evidence authorization at use, retry and resume.
- Tie answer claims to source evidence with supported, insufficient, conflicting and stale outcomes.
- Certify tenant isolation, revocation and retrieved-instruction containment through the existing runtime.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rag-provenance`: Extend the existing behavior with this change's testable contracts.
- `knowledge-rag-product-certification`: Extend the existing behavior with this change's testable contracts.

## Impact

scope: Existing retrieval, evidence verification, prompt assembly and redacted audit paths plus local certification fixtures; existing ingestion and React journey retained.

Dependencies: harness-skill-activation-quality The phase index defines the shared-file serialization order. All mutation authority remains in trusted hosts; dependency pins remain unchanged.

Runtime UX: explicit typed failures and additive redacted events; preserve existing wire contracts and frontend/entity ownership. Provider compatibility: only documented and tested destinations qualify; unsupported paths remain explicit. Realtime state derives from persisted host state, never agent-only memory.

KBD impact: this Spec stage supplies proposals for Plan; implementation tasks remain unchecked. Five F4–F8 external-candidate research prerequisites and the exact provider-profile evidence matrix must be resolved before affected implementation commitments. No deployment, release, runtime test result or completed runtime fix is claimed.


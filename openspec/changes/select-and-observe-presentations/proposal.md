## Why

Existing A2UI publication does not describe requested mode, client negotiation, eligible template selection or fallback provenance.

## What Changes

Add optional negotiation and deterministic fallback, frozen run template selection, governed template rendering and truthful AG-UI/run observability while preserving legacy behavior.

## Capabilities

### New Capabilities
- `presentation-selection`: Add optional negotiation and deterministic fallback, frozen run template selection, governed template rendering and truthful AG-UI/run observability while preserving legacy behavior.

### Modified Capabilities

None. Preserve existing AG-UI/A2UI conformance behavior; new opt-in contracts extend it.

## Impact

Host domain, persistence/policy/runtime and frontend typed entities/UI as applicable. No new dependencies, deployment workflows or release gates. Tests at the end of the complete Presentation phase.

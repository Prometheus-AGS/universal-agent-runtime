## Why

The inspected RunPolicy has no Presentation resource field; recorded prior completion cannot supply eligibility for the approved template domain.

## What Changes

Extend scoped policy and immutable run policy with Presentation eligibility; add graph-backed assignment controls and carry ceilings across delegated runs.

## Capabilities

### New Capabilities
- `presentation-policy`: Extend scoped policy and immutable run policy with Presentation eligibility; add graph-backed assignment controls and carry ceilings across delegated runs.

### Modified Capabilities

None. Preserve existing AG-UI/A2UI conformance behavior; new opt-in contracts extend it.

## Impact

Host domain, persistence/policy/runtime and frontend typed entities/UI as applicable. No new dependencies, deployment workflows or release gates. Tests at the end of the complete Presentation phase.

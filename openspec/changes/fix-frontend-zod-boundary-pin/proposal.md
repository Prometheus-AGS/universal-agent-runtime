# Restore the frontend schema boundary

## Why
The consolidated main frontend fails TypeScript compilation because first-party consumers select Zod 4 while the authoritative pin and A2UI core exchange Zod 3 schema objects. The already-selected A2UI core 0.12 constructor also requires a protocol version and no longer infers processor component implementation types from its constructor.

## What Changes
- Restore Zod 3.25.76 in the frontend and UAR renderer manifests and both active lockfiles.
- Adapt the two UAR catalogs and typed display processor to the selected A2UI API without changing wire version, components or authority.
- Build the completed frontend and record exact evidence.

## Capabilities
### New Capabilities
None.
### Modified Capabilities
- frontend-build-tooling: require compatible schema objects and versioned A2UI catalog construction across the first-party frontend.

## Impact
Only frontend dependency manifests/lockfiles and the existing A2UI catalog/processor construction sites. No provider/runtime API, realtime state, schema migration, UI redesign or governance change. The user explicitly requested this repair on main after consolidation; existing KBD phase history is unchanged. No general harness/tool workflow changes.

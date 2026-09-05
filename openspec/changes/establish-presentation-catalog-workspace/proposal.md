## Why

The approved Presentation domain is a reusable UI template. Existing artifact schemas and development-only A2UI testing do not provide owner-scoped production template management.

## What Changes

Introduce owner-scoped revisioned Presentation records, safe template validation, persistence and authenticated CRUD, plus a production graph-backed registry/editor/preview. Preserve the development-only tester.

## Capabilities

### New Capabilities
- `presentation-catalog`: Introduce owner-scoped revisioned Presentation records, safe template validation, persistence and authenticated CRUD, plus a production graph-backed registry/editor/preview. Preserve the development-only tester.

### Modified Capabilities

None. Preserve existing AG-UI/A2UI conformance behavior; new opt-in contracts extend it.

## Impact

Host domain, persistence/policy/runtime and frontend typed entities/UI as applicable. No new dependencies, deployment workflows or release gates. Tests at the end of the complete Presentation phase.

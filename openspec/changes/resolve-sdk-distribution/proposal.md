## Why

Assessment C4: the rust/typescript/python SDKs carry placeholder metadata,
undeclared licensing posture vs the AGPL-3.0-only runtime, zero tests, no
CI builds, ~15%% API coverage and no streaming in TS/Python - unshippable
as 1.0 companions.

## What Changes

- Make a deliberate SDK licensing decision (permissive client SDKs vs AGPL)
  and record it; fix authorship/versions/publishability.
- Either bring SDKs to a minimal supported surface (chat/runs streaming,
  knowledge, agents) with tests and CI builds, or withdraw them from 1.0
  and remove customer-facing references.

## Capabilities
### New Capabilities
- `sdk-distribution`

## Impact
sdks/*, CI, docs site, README.

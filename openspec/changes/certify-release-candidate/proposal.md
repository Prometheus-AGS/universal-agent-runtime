## Why

No immutable release candidate has exercised the complete production matrix or external installation path.

## What Changes

- Set all product surfaces to `1.0.0`, then cut and certify candidate tag
  `v1.0.0-rc.3` from that immutable commit (the next unused RC ordinal).
- Install on clean supported platforms and execute the stable matrix.
- Require external adopter validation and preserve evidence.

## Capabilities
### New Capabilities
- `release-candidate-certification`

## Impact
Release operations and evidence; failures create focused follow-up changes rather than being waived.

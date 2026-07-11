## Why

The migration is incomplete until the temporary legacy allowlist reaches zero and CI prevents regression.

## What Changes

- Remove all production layering allowlist entries.
- Make import/direct-fetch checks blocking.
- Document narrow infrastructure exceptions.

## Capabilities
### New Capabilities
- `react-boundary-release-gate`

## Impact
Frontend code and CI policy; no intended product behavior change.

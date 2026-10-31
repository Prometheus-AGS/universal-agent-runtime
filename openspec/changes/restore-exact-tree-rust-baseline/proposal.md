## Why
Issue #462 observes a duplicate fixture field hidden behind cfg(test). #460 reported test results obtained after a temporary patch; release-binary builds do not establish committed library-test health.

## What Changes
- Remove the duplicate initializer while retaining replay/provenance assertions.
- Add a serial local library-test compilation safeguard for the three named profiles, integrated with the existing pre-commit workflow.
- Require a clean, committed source identity for acceptance mode; distinguish compilation from runtime qualification.
- Record KBD phase outcome, commands and honest evidence status.

## Capabilities
### New Capabilities
- `rust-test-baseline`: exact-source compilation and regression evidence for supported library-test profiles.

### Modified Capabilities
None.

## Impact
Runtime UX/provider compatibility/realtime state are unchanged. Changes affect one Rust test fixture, local hook/script and documentation. KBD waypoint and phase artifacts identify the bounded operator-directed work; previous program tasks remain historical/pending. No deployment workflow or global runner is introduced.

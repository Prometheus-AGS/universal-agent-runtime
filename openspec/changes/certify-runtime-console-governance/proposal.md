## Why

Runtime Console panels are now wired, but the pages still bypass architecture boundaries and Cedar denial is incorrectly convertible into human approval.

## What Changes

- Move Console/approval I/O and graph mutation into store/service ownership.
- Certify Cockpit, Protocols, Runs, Approvals and live feeds.
- Implement non-overridable `Deny` distinct from `RequireApproval`.

## Capabilities
### New Capabilities
- `runtime-console-governance-certification`

## Impact
Governance/runtime Rust code, approval API, Console React feature, E2E tests.

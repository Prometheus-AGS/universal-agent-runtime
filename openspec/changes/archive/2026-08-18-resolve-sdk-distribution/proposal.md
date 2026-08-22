## Why

Assessment C4 originally found the Rust, TypeScript, and Python SDKs
unshippable. Subsequent Grade-A work implemented the 1.0 surface, tests,
examples, docs, MIT licensing, and version parity, but this change was never
reconciled. Residual placeholder authorship, an unresolved Rust publication
prerequisite chain, stale install commands, and a routine-test GitHub Actions
workflow still conflict with the accepted SDK and repository policies.

## What Changes

- Reaffirm and canonically record the accepted ADR-0007 decision: ship all
  three SDKs at 1.0.0 under MIT with the existing parity contract.
- Fix the remaining authorship, Rust dependency, and install-command metadata.
- Record the complete Rust publication prerequisite chain without claiming the
  unpublished runtime or SDK can already be uploaded.
- Verify each SDK's tests, package contents, examples, and generated
  documentation locally; retire the legacy routine CI workflow as required by
  repository policy.

## Capabilities
### New Capabilities
- `sdk-distribution`

## Impact
sdks/*, the retired routine CI workflow, docs site, README.

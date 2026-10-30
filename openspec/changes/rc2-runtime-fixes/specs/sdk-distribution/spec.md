## ADDED Requirements

### Requirement: The embedded SDK pins the runtime version being released
The release gate SHALL fail when the Rust SDK's exact `universal-agent-runtime` path dependency pin differs from the root package version.

#### Scenario: Version bump without the pin
- **WHEN** `Cargo.toml` declares a version that `sdks/rust/Cargo.toml` does not pin exactly
- **THEN** `pnpm sdk-runtime-pin:validate` exits non-zero and names both versions.

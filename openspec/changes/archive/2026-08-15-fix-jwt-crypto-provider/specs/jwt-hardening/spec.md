## ADDED Requirements

### Requirement: The runtime can execute the JWT algorithms it is configured with
The runtime SHALL have a cryptographic provider available to `jsonwebtoken` at
build time and SHALL install the selected provider before each runtime JWT
operation, so signing and verifying execute rather than panic. All UAR-owned
workspace packages SHALL select the same pinned provider. Dependency resolution
alone SHALL NOT be treated as evidence: provider features are additive, and
`jsonwebtoken` selects a panic provider when both or neither built-in is active.

#### Scenario: A token round-trips through the real code path
- **WHEN** a token is signed with the configured secret and then verified through the runtime's own verification path
- **THEN** the verification succeeds and the decoded subject matches the signed subject, with no panic

#### Scenario: An invalid signature is rejected rather than panicking
- **WHEN** a token signed with a different secret is presented
- **THEN** verification returns an error and the process does not panic

#### Scenario: Negative control for the round-trip
- **WHEN** the round-trip test is run against a build with no crypto feature enabled
- **THEN** the test fails, demonstrating it detects the missing provider rather than passing vacuously

#### Scenario: UAR-owned initialization is idempotent
- **WHEN** UAR acquires the process provider slot with RustCrypto and invokes its initializer again
- **THEN** the cached UAR-owned initialization succeeds and JWT operations remain available

#### Scenario: An earlier process owner fails closed
- **WHEN** any `jsonwebtoken` provider was installed before UAR invokes its provider guard
- **THEN** UAR returns a structured provider-conflict error because version 11.0.0 cannot publicly identify the installed provider

#### Scenario: A different provider initialized before UAR fails closed
- **WHEN** a non-RustCrypto `jsonwebtoken` provider is installed before UAR invokes its provider guard
- **THEN** UAR returns a structured provider-conflict error and does not perform a JWT operation

#### Scenario: Server startup acquires provider ownership
- **WHEN** any UAR server entrypoint reaches the shared startup funnel
- **THEN** RustCrypto is installed before routes are initialized or readiness is reported

#### Scenario: Workspace provider selection is singular
- **WHEN** the complete Cargo workspace feature graph is inspected
- **THEN** `jsonwebtoken` 11.0.0 has `rust_crypto` active and `aws_lc_rs` inactive

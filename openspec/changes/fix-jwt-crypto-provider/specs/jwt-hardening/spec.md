## ADDED Requirements

### Requirement: The runtime can execute the JWT algorithms it is configured with
The runtime SHALL have a cryptographic provider available to `jsonwebtoken` at
build time, so that signing and verifying a token executes rather than panics.
Dependency resolution alone SHALL NOT be treated as evidence of this: the
provider is selected by a Cargo feature, and the crate's default features select
none.

#### Scenario: A token round-trips through the real code path
- **WHEN** a token is signed with the configured secret and then verified through the runtime's own verification path
- **THEN** the verification succeeds and the decoded subject matches the signed subject, with no panic

#### Scenario: An invalid signature is rejected rather than panicking
- **WHEN** a token signed with a different secret is presented
- **THEN** verification returns an error and the process does not panic

#### Scenario: Negative control for the round-trip
- **WHEN** the round-trip test is run against a build with no crypto feature enabled
- **THEN** the test fails, demonstrating it detects the missing provider rather than passing vacuously

## ADDED Requirements

### Requirement: Shipped SDKs are licensed, tested and built in CI
Any SDK distributed with the 1.0 release SHALL have a deliberate license,
real authorship metadata, version parity with the product, CI-built tests,
and documented coverage including streaming consumption.

#### Scenario: SDK ships
- **WHEN** an SDK is listed in customer docs for 1.0
- **THEN** its package metadata, license file, tests and CI job exist and pass

#### Scenario: SDK withdrawn
- **WHEN** an SDK is not brought to standard for 1.0
- **THEN** it is removed from customer-facing docs and marked experimental in-repo

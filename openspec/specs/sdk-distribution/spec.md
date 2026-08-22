# sdk-distribution Specification

## Purpose

Define the licensing, local verification, customer installation metadata, and
honest release-order contract for the Rust, Python, and TypeScript SDKs.

## Requirements

### Requirement: Selected SDKs are licensed, locally verified, and release-ordered
Any SDK selected for the 1.0 release SHALL have a deliberate license,
real authorship metadata, version parity with the product, locally observed
tests and package metadata/contents, and documented coverage including
streaming consumption. A package whose manifest names an unpublished sibling
SHALL retain that dependency and record the complete prerequisite/remediation
chain. It SHALL NOT be reported as registry-publishable until that chain passes.
Routine SDK tests SHALL NOT run in GitHub Actions.

#### Scenario: SDK ships
- **WHEN** an SDK is listed in customer docs for 1.0
- **THEN** its package metadata, license file, tests, package contents, examples,
  and generated documentation exist and pass their local verification commands

#### Scenario: Routine verification stays local
- **WHEN** SDK tests and build checks are defined
- **THEN** they are recorded as local commands and no routine CI workflow runs them

#### Scenario: Rust publication is prerequisite-blocked
- **WHEN** the Rust SDK's embedded feature depends on the unpublished runtime
- **THEN** the release record names every path-only runtime dependency and the
  remediation order before runtime and SDK publication

#### Scenario: SDK withdrawn
- **WHEN** an SDK is not brought to standard for 1.0
- **THEN** it is removed from customer-facing docs and marked experimental in-repo

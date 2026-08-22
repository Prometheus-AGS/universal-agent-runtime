# Customer Documentation Specification

## Purpose

Define the customer-facing documentation and published API metadata required
for the supported UAR product surface.

## Requirements

### Requirement: Customer documentation describes the whole product accurately
The README and docs site SHALL describe runtime, frontend, SDKs, skills and
deployment with mermaid architecture/flow/scenario diagrams including fabric
context, and the published OpenAPI version SHALL match the product version.

#### Scenario: New customer orientation
- **WHEN** a customer reads the README
- **THEN** they see mermaid architecture/flow/scenario diagrams, the fabric relationship, SDK and skills sections, deployment quickstart, and a working docs-site link

#### Scenario: Customer guide coverage
- **WHEN** a customer opens the documentation site
- **THEN** architecture, SDK, skills, deployment, and security guides describe the supported server-full product and link to the relevant operational contracts

#### Scenario: Docs build breaks on broken links
- **WHEN** a docs page links to a missing target
- **THEN** the site build fails rather than shipping the broken link

#### Scenario: Published API metadata matches the runtime
- **WHEN** a client reads the generated OpenAPI document
- **THEN** its version matches `CARGO_PKG_VERSION` and it documents the actual chat, runs, providers, skills, knowledge, authentication, and realtime route groups

#### Scenario: Repository root contains no captured test output
- **WHEN** a customer inspects a release source tree
- **THEN** the reviewed `TEST_EXECUTION_REPORT.md`, `output*.txt`, and `u00261` scratch artifacts are absent

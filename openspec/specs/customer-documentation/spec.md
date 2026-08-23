# Customer Documentation Specification

## Purpose

Define the customer-facing documentation and published API metadata required
for the supported UAR product surface.

## Requirements

### Requirement: Customer documentation describes the whole product accurately
The README and docs site SHALL orient readers to runtime, frontend, SDKs,
skills, knowledge, security, operations, and deployment through current portal
authorities and profile-bounded diagrams. The public route inventory SHALL
resolve every required product surface to a document. Package metadata, a Git
tag, generated reference source, or a successful build MUST NOT be represented
as registry publication, deployed health, certification, or cross-profile
support. The published OpenAPI version SHALL match the product version.

#### Scenario: New customer orientation
- **WHEN** a customer reads the README
- **THEN** they see the branded hero, current architecture and execution diagrams, profile boundaries, source quickstart, SDK/skills guidance, deployment boundary, and canonical portal link

#### Scenario: Customer guide coverage
- **WHEN** a customer opens the documentation site
- **THEN** current architecture, API/protocol, SDK, skills, knowledge, security, operations, configuration, installation, deployment, and upgrade guides link to their relevant contracts

#### Scenario: Required product route
- **WHEN** a route is marked required in the frozen product route inventory
- **THEN** its declared Docusaurus document ID MUST exist and state its profile and evidence limits

#### Scenario: Docs build breaks on broken links
- **WHEN** a docs page links to a missing target
- **THEN** the site build MUST fail rather than shipping the broken link

#### Scenario: Published API metadata matches the runtime
- **WHEN** a client reads the generated OpenAPI document
- **THEN** its version MUST match `CARGO_PKG_VERSION` and it MUST describe its implemented route summary without claiming exhaustive router coverage

#### Scenario: Repository root contains no captured test output
- **WHEN** a customer inspects a release source tree
- **THEN** reviewed scratch test-output artifacts MUST be absent

#### Scenario: Evidence is insufficient for an availability claim
- **WHEN** a README has only package metadata, a tag, source, generated documentation, or build output
- **THEN** it MUST describe only that observed fact and MUST NOT infer registry publication, deployment health, certification, or support for another profile

#### Scenario: Profile-specific behavior is documented
- **WHEN** behavior or evidence differs between `server-full`, `minimal`, and `embedded-mobile`
- **THEN** the public guide reports those results separately and makes no aggregate or cross-profile claim

### Requirement: Native deployment documentation is operationally complete
The README and product documentation SHALL cover native installation, upgrade, uninstall, credential refresh, configuration precedence, program/config/state/log locations, service control, listener exposure, troubleshooting, and platform-specific verification for macOS, Linux, and Windows.

#### Scenario: Operator follows platform instructions
- **WHEN** an operator selects a supported native platform
- **THEN** the documentation identifies prerequisites, exact lifecycle commands, default paths/ports, preservation behavior, log inspection, and the limits of locally observed evidence

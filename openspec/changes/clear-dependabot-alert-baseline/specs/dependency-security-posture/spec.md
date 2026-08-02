## ADDED Requirements

### Requirement: Authoritative Dependency Graph and Zero-Alert Baseline

Each first-party project surface SHALL retain only dependency lockfiles that can be reproduced by its declared and CI-used package manager, and a security-remediation change MUST verify the live GitHub Dependabot feed reports zero open actionable alerts after the remediated commit is pushed.

#### Scenario: Duplicate lockfile cannot reproduce the current manifest

- **WHEN** a secondary package-manager lockfile fails fresh resolution against the current manifest while the declared package manager resolves successfully
- **THEN** the secondary lockfile MUST be removed and all CI audit/install jobs for that surface MUST use the authoritative dependency graph

#### Scenario: Retained lockfiles are remediated

- **WHEN** an open Dependabot alert names a retained lockfile and a patched dependency line exists
- **THEN** the manifest or lockfile MUST resolve the affected package outside the vulnerable range without dismissing the alert

#### Scenario: Remediation completion is verified remotely

- **WHEN** the remediation commit has been pushed and GitHub has processed its dependency graphs
- **THEN** the Dependabot alerts API MUST return zero open alerts before the remediation is declared complete

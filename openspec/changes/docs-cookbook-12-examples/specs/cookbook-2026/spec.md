# cookbook-2026 Specification

## ADDED Requirements

### Requirement: Cookbook examples are runnable and validated in CI

The repository SHALL maintain a `docs/cookbook/` directory with at least 12
examples: 4 runtime, 4 SDK (across Rust, Python, and TypeScript), and 4 A2UI.
A validation script SHALL compile or typecheck every example and execute the
ones that do not require a live UAR server or external LLM backend.

#### Scenario: Runtime example compiles

- **WHEN** `tools/validate-cookbook.sh` runs
- **THEN** every Rust runtime example builds successfully

#### Scenario: Runtime example runs without external services

- **WHEN** a runtime example does not depend on a live LLM or database
- **THEN** `tools/validate-cookbook.sh` executes it and it exits 0

#### Scenario: SDK example compiles or typechecks

- **WHEN** `tools/validate-cookbook.sh` runs
- **THEN** every Rust and Python SDK example compiles, and every TypeScript SDK example typechecks

#### Scenario: A2UI examples are deferred

- **WHEN** A2UI examples are blocked on upstream `@a2ui` integration work
- **THEN** they are represented as placeholders and skipped with a clear message

## ADDED Requirements

### Requirement: Cookbook validation is part of CI

A GitHub Actions workflow SHALL run `tools/validate-cookbook.sh` on every pull
request and push to `main`.

#### Scenario: Pull request validation

- **WHEN** a pull request changes the cookbook, SDK examples, or validation script
- **THEN** the cookbook workflow runs and fails on regressions

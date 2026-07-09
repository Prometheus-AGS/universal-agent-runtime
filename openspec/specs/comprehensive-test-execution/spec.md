# comprehensive-test-execution Specification

## Purpose
TBD - created by archiving change fix-comprehensive-tests-ci-gate. Update Purpose after archive.
## Requirements
### Requirement: Comprehensive Test Workflows Progress Past Pre-flight

`comprehensive-tests.yml` and `tests-full.yml`'s pre-flight/prerequisite configuration-validation steps SHALL pass so that every downstream job (Code Quality, Security Audit, Build Verification, Docker Integration Tests, Comprehensive Tests, Performance Benchmarks) is genuinely dispatched rather than unconditionally skipped.

#### Scenario: Required config files all exist

- **Given** `test-config.yaml`, `docker-compose.test.yaml`, and `Dockerfile.test` all exist at the repo root
- **When** `comprehensive-tests.yml`'s Pre-flight job runs
- **Then** it MUST pass, and every downstream job MUST be dispatched (not skipped)

#### Scenario: A required config file goes missing again

- **Given** one of the three required config files is deleted or renamed in a future change
- **When** the Pre-flight job runs
- **Then** it MUST fail with a specific message naming the missing file, and this MUST be treated as a P0 regression, not a low-priority backlog item — per this project's history of the same gap silently persisting for the project's entire lifetime

### Requirement: Coverage Thresholds Reflect Measured Reality, Not Aspiration

`test-config.yaml`'s coverage thresholds SHALL be documented as either a real measured baseline or an explicitly-labeled interim placeholder — never presented as an achieved target without evidence.

#### Scenario: Thresholds are interim placeholders

- **Given** exact current coverage percentages have not been measured via the real coverage toolchain (`grcov`/`cargo-llvm-cov`, Playwright V8 coverage)
- **When** `test-config.yaml` sets threshold values
- **Then** those values MUST be low enough not to immediately fail typical current coverage, and MUST be labeled in an inline comment as unmeasured interim placeholders pending real coverage measurement — not copied verbatim from an aspirational, never-achieved spec document


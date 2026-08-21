## ADDED Requirements

### Requirement: Runtime recovers from supported failures
UAR SHALL terminate, retry, resume or surface external failures according to documented bounded policies without orphaning work or duplicating side effects.

#### Scenario: MCP server restart
- **WHEN** an MCP server crashes during a tool call and restarts
- **THEN** the run reaches a documented terminal/retry state and later calls can reconnect without restarting UAR

#### Scenario: Streaming soak
- **WHEN** the release candidate runs the defined multi-hour streaming workload
- **THEN** error, memory, latency and duplicate-event thresholds remain within the published limits

### Requirement: Product certification runs locally
UAR SHALL run product, installed-artifact, supply-chain, security, load, stress,
soak, and release-certification checks locally rather than in GitHub Actions.
GitHub Actions SHALL be limited to deployment execution and
deployment-specific validation.

#### Scenario: Immutable local resilience candidate
- **WHEN** operational resilience certification is run
- **THEN** a clean local checkout builds and certifies one exact source commit for at least 10,800 seconds and retains machine-readable evidence locally

#### Scenario: Non-deployment workflow is introduced
- **WHEN** a GitHub Actions workflow invokes a non-deployment test, lint, typecheck, audit, benchmark, certification, or implicit build-time test
- **THEN** the local workflow-policy validator fails before the workflow is committed

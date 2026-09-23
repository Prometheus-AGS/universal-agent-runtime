## ADDED Requirements

### Requirement: Starter suite and local evaluation gates
The repository SHALL retain evals/starter.yaml with declared scorers and a deterministic local run requiring no model or API key. Real-model runs SHALL be separate local evidence with destination and environment recorded. All product testing, linting, coverage, typechecking and release certification SHALL run locally; GitHub Actions SHALL be limited to deployment execution and deployment-specific validation, including no product tests hidden inside build/package scripts.

#### Scenario: Starter suite ships and is valid
- **WHEN** the repository is checked out
- **THEN** evals/starter.yaml exists, declares scorers and loads/scores through the harness

#### Scenario: Local deterministic gate
- **WHEN** the local deterministic suite runs
- **THEN** it needs no API key or model access and a behavioral regression causes nonzero exit

#### Scenario: Live environment missing
- **WHEN** a required local live-provider run lacks credentials or a working endpoint
- **THEN** the evidence is explicitly environment-blocked, never a successful certification; deterministic results remain separately reported

### Requirement: Governed agent behavior is evaluated through production paths
The existing eval runner SHALL support tool-inclusive cases through the production trusted orchestration boundary with deterministic provider fixtures and local governed tools. Completion-only suites SHALL remain supported but SHALL NOT certify agentic behavior. Required cases SHALL score tool selection, argument validity, results, multi-step completion, approval/deny, graph/subagent correlation, context integrity, cancellation and shared budgets. Fixtures SHALL be established with each affected change, before final certification.

#### Scenario: Full successful task
- **WHEN** a recorded provider requests multiple governed tool steps before answering
- **THEN** the runner executes the actual orchestration path, preserves receipts and emits correlated scored traces including the final task outcome

#### Scenario: Seeded regression detection
- **WHEN** a fixture deliberately drops a tool result, bypasses a deny, duplicates an effect, misattributes child usage or hides cancellation
- **THEN** the corresponding deterministic assertion fails and the local strict-baseline gate exits nonzero without accepting a baseline update as certification

#### Scenario: Reproducible artifacts
- **WHEN** evaluation finishes or fails
- **THEN** retained authorized artifacts record source revision, suite/dataset/profile versions, environment, commands, scorer thresholds, trace identities, baseline/delta and actual outcome, with content redacted according to policy

## REMOVED Requirements

### Requirement: Starter suite and two-tier CI gate
**Reason**: Product tests in pull-request or scheduled GitHub Actions violate the repository deployment-only policy.
**Migration**: Use “Starter suite and local evaluation gates”; inventory and remove any remaining non-deployment workflow testing locally under the mandatory policy validator before and after such edits.

## MODIFIED Requirements

### Requirement: Eval harness is runnable via a CLI subcommand

The binary SHALL expose an `eval` subcommand to run and inspect suites, while
preserving the default (no-subcommand) behavior of starting the server.

#### Scenario: Default invocation runs the server
- **WHEN** the binary is run with no subcommand
- **THEN** it starts the server exactly as before this change

#### Scenario: Run a suite
- **WHEN** `eval run <suite>` is invoked
- **THEN** the suite is loaded, each case is completed via the orchestrator and scored, results are persisted, the run is compared to the baseline, and a per-scorer report is printed

#### Scenario: Regression sets a non-zero exit code
- **WHEN** `eval run <suite>` detects a regression against the baseline (without `--update-baseline`)
- **THEN** the process exits with a non-zero status (a local regression gate), otherwise it exits zero

#### Scenario: Update baseline
- **WHEN** `eval run <suite> --update-baseline` is invoked
- **THEN** the run's summary is saved as the suite's baseline and the process exits zero

#### Scenario: Inspect baseline / list results
- **WHEN** `eval baseline <suite>` or `eval list` is invoked
- **THEN** the stored baseline summary, or the result files, are printed

### Requirement: Strict baseline gating

The eval runner SHALL support a strict mode in which a missing baseline is a
failure rather than a pass, so a regression gate cannot silently pass before a
baseline has been established. Local certification SHALL run in this strict mode. Strict mode SHALL be opt-in; without it, a run with no baseline remains
clean (a run may still establish expectations).

#### Scenario: Strict run with no baseline fails
- **WHEN** `eval run <suite>` is invoked with the strict-baseline option and no baseline exists for the suite
- **THEN** the process prints a clear message and exits non-zero, without reporting a (vacuous) clean comparison

#### Scenario: Non-strict run with no baseline is clean
- **WHEN** `eval run <suite>` is invoked without the strict-baseline option and no baseline exists
- **THEN** the run completes and exits zero (unchanged prior behavior)

#### Scenario: Scheduled tier is strict
- **WHEN** the previously scheduled certification gate is migrated to a local run
- **THEN** it uses strict-baseline mode, so a missing baseline fails certification until an explicitly reviewed baseline is seeded; no GitHub Actions test job runs

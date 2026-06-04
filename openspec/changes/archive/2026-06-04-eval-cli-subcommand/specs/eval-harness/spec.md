## ADDED Requirements

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
- **THEN** the process exits with a non-zero status (a CI gate), otherwise it exits zero

#### Scenario: Update baseline
- **WHEN** `eval run <suite> --update-baseline` is invoked
- **THEN** the run's summary is saved as the suite's baseline and the process exits zero

#### Scenario: Inspect baseline / list results
- **WHEN** `eval baseline <suite>` or `eval list` is invoked
- **THEN** the stored baseline summary, or the result files, are printed

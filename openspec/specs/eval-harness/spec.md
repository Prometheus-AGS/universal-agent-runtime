# eval-harness Specification

## Purpose
TBD - created by archiving change eval-domain-and-rule-scorers. Update Purpose after archive.
## Requirements
### Requirement: Eval domain model

The system SHALL provide a typed, serializable eval domain: an eval case (id +
input + optional expected output + metadata), an eval suite (name + cases), a
score (scorer name + value in 0.0–1.0 + optional detail), and an eval result
(suite + case id + model + scores + timestamp).

#### Scenario: Domain round-trips
- **WHEN** an eval result is serialized and deserialized
- **THEN** its case id, scores, and metadata are preserved unchanged

### Requirement: Scorer contract

The system SHALL define a `Scorer` contract that maps a `(case, output)` pair to
a normalized `Score` whose value is always within 0.0–1.0. Scoring SHALL be
deterministic for rule-based scorers and SHALL NOT perform IO.

#### Scenario: Score is normalized
- **WHEN** any built-in scorer scores an output
- **THEN** the returned value is between 0.0 and 1.0 inclusive

### Requirement: Built-in rule-based scorers

The system SHALL provide rule-based scorers: exact-match and contains (against
the case's expected output), JSON-validity, non-empty, a regex/pattern match,
and a sycophancy scorer (higher value = less sycophantic) derived from the
existing sycophancy detector.

#### Scenario: Exact match
- **WHEN** the output equals the case's expected output
- **THEN** the exact-match scorer returns 1.0, otherwise 0.0

#### Scenario: Contains
- **WHEN** the output contains the expected substring
- **THEN** the contains scorer returns 1.0, otherwise 0.0

#### Scenario: JSON validity
- **WHEN** the output parses as valid JSON
- **THEN** the json-valid scorer returns 1.0, otherwise 0.0

#### Scenario: Sycophancy scorer
- **WHEN** a clean (non-sycophantic) output is scored
- **THEN** the sycophancy scorer returns a high value (≈1.0), and a flagged output returns a lower value

### Requirement: Eval suites load from golden files

The system SHALL load an `EvalSuite` from a file path, supporting JSON and YAML
by extension. A malformed or missing file SHALL produce an error (not a panic).

#### Scenario: Load a JSON suite
- **WHEN** a `.json` suite file with cases is loaded
- **THEN** an `EvalSuite` with those cases is returned

#### Scenario: Load a YAML suite
- **WHEN** a `.yaml`/`.yml` suite file with cases is loaded
- **THEN** an `EvalSuite` with those cases is returned

#### Scenario: Missing/malformed file errors
- **WHEN** the path does not exist or the content does not parse
- **THEN** the loader returns an error and does not panic

### Requirement: Runner executes cases through a completion provider and scores them

The system SHALL run a suite by, for each case, obtaining an output from a
pluggable completion provider and applying the configured scorers, producing one
`EvalResult` per case (with a timestamp). The runner SHALL NOT depend on a
specific LLM client (it depends on the provider abstraction), so it is testable
without a live model.

#### Scenario: Cases scored into results
- **WHEN** a suite of N cases is run with a provider and a set of scorers
- **THEN** N `EvalResult`s are produced, each containing one score per scorer

#### Scenario: Per-case completion error is contained
- **WHEN** the provider returns an error for a case
- **THEN** that case yields a failed result (a completion score of 0.0 with the error detail) and the remaining cases still run

### Requirement: Eval results are persisted to files

The system SHALL persist a run's `EvalResult`s to a JSON file under a results
directory, named by suite and timestamp, so runs are retained for comparison.

#### Scenario: Results written
- **WHEN** a run's results are saved for suite `s`
- **THEN** a JSON file `<dir>/s-<timestamp>.json` is written containing those results, re-loadable into `EvalResult`s

### Requirement: Per-scorer summary and baseline

The system SHALL compute a per-scorer mean summary over a run, and SHALL store
and load a named baseline summary per suite.

#### Scenario: Summary is per-scorer mean
- **WHEN** a run has multiple cases each scored by scorer `X`
- **THEN** the summary's value for `X` is the mean of those scores

#### Scenario: Baseline round-trips
- **WHEN** a baseline summary is saved and loaded for a suite
- **THEN** the loaded summary equals the saved one; loading a missing baseline yields none

### Requirement: Delta-vs-baseline regression detection

The system SHALL flag a scorer as regressed when its current mean drops below the
baseline mean by more than a configured threshold, and SHALL roll up to an
overall regressed flag. With no baseline, there SHALL be no regression.

#### Scenario: Regression detected
- **WHEN** a scorer's current mean is below its baseline mean by more than the threshold
- **THEN** that scorer is marked regressed and the report's overall flag is true

#### Scenario: Within threshold is not a regression
- **WHEN** a scorer's current mean is equal to or above the baseline (or below by ≤ threshold)
- **THEN** it is not marked regressed

#### Scenario: No baseline
- **WHEN** there is no baseline for the suite
- **THEN** the report shows no regressions (the run can establish a baseline)

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

### Requirement: Suites declare their scorers

An eval suite SHALL be able to declare the scorers applied to its cases, and the
runner SHALL build scorers from that declaration. When a suite declares no
scorers, the system SHALL fall back to a default scorer set so existing suites
behave unchanged.

#### Scenario: Declared scorers are used
- **WHEN** a suite declares a list of scorers (e.g. `json_valid`, `pattern_match`)
- **THEN** the runner scores each case with exactly those scorers

#### Scenario: No declaration falls back to the default set
- **WHEN** a suite declares no scorers
- **THEN** the runner applies the default set (non-empty + sycophancy, plus exact-match + contains when every case has an expected output)

#### Scenario: Existing suites deserialize unchanged
- **WHEN** a suite file without a `scorers` field is loaded
- **THEN** it loads successfully and uses the default scorer set

### Requirement: LLM-as-judge scorer

The system SHALL provide an LLM-as-judge scorer that grades a candidate output
against a rubric via a completion provider and returns a normalized score. The
scorer SHALL parse the model's verdict deterministically and SHALL NOT panic on
malformed output. Judge scores SHALL be advisory — reported and persisted, but
not part of the hard regression gate.

#### Scenario: Rubric grading
- **WHEN** a suite declares an `llm_judge` scorer with a rubric
- **THEN** each case's input + candidate output are sent to the provider with the rubric, and the parsed score is recorded

#### Scenario: Deterministic verdict parse
- **WHEN** the judge response contains a JSON object `{ "score": <0.0–1.0>, "reason": <text> }`
- **THEN** the score is extracted and clamped to 0.0–1.0

#### Scenario: Malformed verdict is contained
- **WHEN** the judge response cannot be parsed into a verdict
- **THEN** the scorer returns 0.0 with a detail explaining the failure, and does not panic

### Requirement: End-to-end run pipeline is covered by an automated test

The system SHALL cover the eval run pipeline (load a suite, run each case
through a completion provider, score, summarize, persist, and compare to a
baseline) with an automated test that uses a deterministic recorded provider, so
the pipeline is verifiable without a live model.

#### Scenario: Deterministic pipeline run
- **WHEN** a suite is run through the pipeline with a recorded provider
- **THEN** each case produces scored results, the summary reflects the recorded outputs, and results persist and reload unchanged

#### Scenario: Regression verdicts are exercised
- **WHEN** the summary is compared to baselines representing no-baseline, equal, and a drop beyond the threshold
- **THEN** the comparison reports clean, clean, and regressed respectively

#### Scenario: Provider failure is contained
- **WHEN** the provider has no recorded output for a case
- **THEN** that case yields a contained failure result and the run still completes

### Requirement: Starter suite and two-tier CI gate

The repository SHALL ship a starter eval suite and SHALL run it in CI as a
regression gate across two tiers: a deterministic structural check on every pull
request that requires no model or API key, and a scheduled real-model run that
gates on regression against a baseline and degrades gracefully when no API key
is configured.

#### Scenario: Starter suite ships and is valid
- **WHEN** the repository is checked out
- **THEN** `evals/starter.yaml` exists, declares scorers, and loads + scores through the harness

#### Scenario: PR tier requires no key
- **WHEN** the pull-request CI runs
- **THEN** the starter suite is loaded and scored with a deterministic provider, with no API key and no model call

#### Scenario: Scheduled tier gates on regression
- **WHEN** the scheduled job runs and an API key secret is present
- **THEN** the suite runs against the real model and the job exits non-zero on regression against the baseline

#### Scenario: Scheduled tier without a key is skipped
- **WHEN** the scheduled job runs and no API key secret is present
- **THEN** the job skips the real-model run without failing


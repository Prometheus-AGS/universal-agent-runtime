## ADDED Requirements

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

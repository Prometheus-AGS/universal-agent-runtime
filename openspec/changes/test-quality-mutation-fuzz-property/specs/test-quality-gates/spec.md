# Test quality gates

## Purpose

Define the test-quality gates for the UAR runtime: mutation testing,
fuzzing, property-based tests, and conventional-commit release checks.
These gates complement the line-coverage gate defined in the
`coverage-baseline` capability.

## ADDED Requirements

### Requirement: Mutation testing with cargo-mutants

The repository MUST run mutation testing on a nightly schedule. The
workflow MUST use `cargo-mutants` with `--no-shuffle` and publish the
results to `docs/mutation-history/`.

#### Scenario: Mutation testing runs on schedule
- **WHEN** the nightly `mutation.yml` workflow triggers
- **THEN** `cargo mutants --no-shuffle` runs against the Rust workspace
- **AND** the mutation report is committed or published to
  `docs/mutation-history/`

#### Scenario: A mutation survives
- **WHEN** `cargo-mutants` finds a surviving mutant
- **THEN** the report records the source file, line, and mutant
- **AND** the result is tracked historically in `docs/mutation-history/`

### Requirement: Fuzz targets with cargo-fuzz

A `fuzz/` directory MUST exist with at least four initial targets:
`chunker`, `rag_verification`, `mcp_message_parser`, and
`json_schema_validator`. Each target MUST be buildable with
`cargo fuzz run <target>`.

#### Scenario: A fuzz target runs locally
- **WHEN** a developer runs `cargo fuzz run chunker`
- **THEN** the fuzzer builds and begins exercising the chunker input
  surface
- **AND** crashes are reproducible from the `fuzz/corpus/` artifacts

#### Scenario: A new parser is added
- **WHEN** a new message parser or validator is introduced
- **THEN** a corresponding fuzz target is added or the existing
  `mcp_message_parser`/`json_schema_validator` target is extended

### Requirement: Property-based tests with proptest

The Rust test suite MUST include property-based tests using
`proptest`. The minimum required properties are:

1. Settings store serde roundtrip: any valid settings value serializes
   and deserializes to an equivalent value.
2. Retrieval RRF invariants: reciprocal rank fusion scores are
   monotonic and bounded for arbitrary input lists.
3. Governance policy hot-reload semantics: policy files parse and
   produce deterministic effective policies after reload cycles.

#### Scenario: Property test catches a regression
- **WHEN** a code change breaks a previously held invariant
- **THEN** the corresponding `proptest` case fails in CI
- **AND** the failure includes a minimal counterexample

### Requirement: Conventional commits and release automation

The JS workspace MUST enforce conventional commit messages via
`commitlint` and `lefthook`. `release-plz` MUST be configured to check
conventional commits before producing a release PR.

#### Scenario: A non-conventional commit is attempted
- **WHEN** a commit message in the JS workspace does not match the
  conventional commit format
- **THEN** `lefthook` blocks the commit locally
- **AND** CI does not produce a release PR for it

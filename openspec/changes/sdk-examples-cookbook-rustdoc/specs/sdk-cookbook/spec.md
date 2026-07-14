# SDK cookbook

## Purpose

Give developers a runnable, CI-verified set of examples across all three
first-party UAR SDKs (Rust, Python, TypeScript), backed by hosted API
reference documentation, so the SDK surface stays demonstrably correct
as the SDKs evolve.

## ADDED Requirements

### Requirement: The Rust SDK cookbook covers every public client API group
`sdks/rust/examples/` SHALL contain at least 12 runnable
`cargo run --example <name>` programs, collectively exercising every
public method group on `Client` (`chat`, `runs`, `knowledge`, `ingest`,
`tools`, `embeddings`).

#### Scenario: A developer builds the full example set
- **WHEN** a developer runs `cargo build --examples --locked` inside
  `sdks/rust/`
- **THEN** every example under `sdks/rust/examples/` compiles with no
  errors

#### Scenario: A developer runs the self-contained error-handling example
- **WHEN** a developer runs `cargo run --example error_handling` inside
  `sdks/rust/` with no UAR server running
- **THEN** the example exits `0` and prints the `miette::Diagnostic`
  `code` and `help` text for the resulting transport error, without
  requiring network access to a live server

### Requirement: Every SDK example is validated by an automated smoke test
`tools/validate-examples.sh` SHALL exist at the repository root and, when
run with no arguments, SHALL compile or typecheck every example under
`sdks/rust/examples/`, `sdks/python/examples/`, and
`sdks/typescript/examples/`, exiting non-zero if any example fails to
compile/typecheck.

#### Scenario: CI runs the smoke test on every push and pull request
- **WHEN** a commit is pushed to `main` or a pull request targeting
  `main` is opened or updated
- **THEN** the `sdk-examples` job in `.github/workflows/ci.yml` runs
  `bash tools/validate-examples.sh`
- **AND** the job fails if any example in any of the three SDKs fails to
  compile or typecheck

#### Scenario: An operator runs the smoke test against a live server
- **WHEN** an operator sets `VALIDATE_EXAMPLES_LIVE=1` and `UAR_BASE_URL`
  to a reachable UAR server before running `tools/validate-examples.sh`
- **THEN** the script additionally executes every example end-to-end
  against that server and reports pass/fail per example, instead of
  reporting compile-only checks as skipped

### Requirement: Each first-party SDK produces hosted API reference documentation
Each of `sdks/rust/`, `sdks/python/`, and `sdks/typescript/` SHALL have a
locally verifiable documentation build command that produces zero-error
API reference output, suitable for hosting (docs.rs for Rust on
crates.io publish, Sphinx/ReadTheDocs or GitHub Pages for Python, GitHub
Pages via typedoc for TypeScript).

#### Scenario: A maintainer verifies Rust API docs before a release
- **WHEN** a maintainer runs `cargo doc --no-deps -p
  universal-agent-runtime-sdk` inside `sdks/rust/`
- **THEN** the command completes with zero warnings and generates
  `target/doc/universal_agent_runtime_sdk/index.html`

#### Scenario: A maintainer verifies Python API docs before a release
- **WHEN** a maintainer runs `python -m sphinx -b html sdks/python/docs
  <output-dir>` after installing `sdks/python[dev]`
- **THEN** the build succeeds and produces HTML output with no build
  errors

#### Scenario: A maintainer verifies TypeScript API docs before a release
- **WHEN** a maintainer runs `npm run docs` inside `sdks/typescript/`
- **THEN** typedoc completes with zero errors and generates
  `sdks/typescript/docs/api/`

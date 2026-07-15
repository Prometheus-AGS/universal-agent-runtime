# dev-portal-2026 Specification

## Purpose

Provide a single, hosted developer portal that combines narrative documentation, API references generated from source, and architecture decision records for the Universal Agent Runtime project.

## Requirements

### Requirement: Hosted developer portal

The project SHALL publish a static documentation site to GitHub Pages that includes Docusaurus-based narrative docs, rustdoc for the Rust runtime and SDK, and TypeDoc for the TypeScript SDK.

#### Scenario: Docusaurus IA

- **WHEN** a visitor navigates to the docs site
- **THEN** the information architecture includes sections for architecture, configuration, SDKs (Rust, Python, TypeScript), RAG, A2UI, governance, supply chain, and contributing

#### Scenario: API reference hosting

- **WHEN** the docs site is built
- **THEN** it links to generated rustdoc under `/docs/api/rust` and to generated TypeDoc under `/docs/api/typescript`

### Requirement: Prose quality

The project SHALL lint documentation prose with a UAR-specific Vale configuration.

#### Scenario: Style violation

- **WHEN** a contributor runs `docs:lint`
- **THEN** Vale reports violations against the UAR style rules

### Requirement: Architecture decisions are documented

The project SHALL maintain architecture decision records (ADRs) that document the major grade-A decisions.

#### Scenario: ADR lookup

- **WHEN** a developer opens `docs/adr/`
- **THEN** they find a template, an index, and at least 10 ADRs covering the grade-A decisions

## ADDED Requirements

### Requirement: Docusaurus information architecture

The project SHALL organize the developer portal into the sections required for the grade-A release.

#### Scenario: Architecture section

- **WHEN** a visitor opens `/docs/category/architecture`
- **THEN** they see an introduction to the UAR architecture

#### Scenario: SDK sections

- **WHEN** a visitor opens `/docs/category/rust-sdk`, `/docs/category/python-sdk`, or `/docs/category/typescript-sdk`
- **THEN** they see SDK-specific documentation and links to generated API references

#### Scenario: Contributing section

- **WHEN** a visitor opens `/docs/category/contributing`
- **THEN** they see the contribution guidelines, license split, and commit conventions

### Requirement: Vale prose linting

The project SHALL run a UAR-specific prose linter against the documentation.

#### Scenario: Lint command

- **WHEN** a contributor runs `pnpm docs:lint`
- **THEN** Vale executes using `.vale.ini` and the UAR style rules

### Requirement: Architecture decision records

The project SHALL publish ADRs that capture the grade-A decisions.

#### Scenario: ADR template

- **WHEN** a contributor proposes a new architectural decision
- **THEN** they use the template in `docs/adr/0001-record-architecture-decisions.md`

#### Scenario: Grade-A decisions documented

- **WHEN** a reviewer inspects `docs/adr/`
- **THEN** they find at least 10 ADRs covering license, coverage, error handling, configuration, supply chain, SDKs, RAG, A2UI vendoring, A2UI renderer, and docs/visual regression

### Requirement: GitHub Pages deployment workflow

The project SHALL automatically build and deploy the docs site on pushes to `main`.

#### Scenario: Docs deployment

- **WHEN** a change is pushed to `main`
- **THEN** `.github/workflows/docs.yml` builds the Docusaurus site, runs Vale, and deploys to GitHub Pages

#### Scenario: API reference wiring

- **WHEN** the docs workflow runs
- **THEN** it includes placeholder steps for rustdoc and TypeDoc generation and stages them under `/docs/api/rust` and `/docs/api/typescript`

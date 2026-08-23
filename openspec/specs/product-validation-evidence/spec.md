# Product Validation Evidence Specification

## Purpose

Define the evidence required to certify that shipped product screens perform
their primary functions in a live browser.

## Requirements

### Requirement: Every product screen is functionally validated with recorded evidence
Each shipped screen SHALL have a validated purpose/function record backed by a
live browser test run with video evidence, covering the operator's functional
matrix (agents, skills, RAG, memory, auth, isolation, local-first).

#### Scenario: Orchestrator answers live with proof
- **WHEN** the validation suite asks the orchestrator and default agents defined questions in the browser
- **THEN** expected answers stream back and the run is captured on video in the certification bundle

#### Scenario: Admin screen validation
- **WHEN** the suite exercises an admin screen's primary function (e.g. create KB, disable skill, mint API key)
- **THEN** the function succeeds end-to-end and the screen's record links its video segment

#### Scenario: An observed screen defect blocks validation
- **WHEN** a live primary-function scenario exposes a supported product defect
- **THEN** the defect is recorded and repaired at the smallest bounded product
  surface before that scenario may be reported as passing

### Requirement: Native inference evidence is genuine and bounded
An inference claim for a native service SHALL identify the actual provider and model, traverse the installed UAR boundary, and retain an observed model-produced response. Mocked, recorded, replayed, stubbed, or hard-coded responses SHALL NOT satisfy the claim. Verification SHALL use short requests rather than a soak.

#### Scenario: Native inference is certified
- **WHEN** an installed-service inference requirement is evaluated
- **THEN** evidence identifies the provider/model, source SHA, profile/platform, command or UI action, observed response, timeout, and request limit without retaining credentials

### Requirement: Native release evidence separates API, UI, provider, and platform claims
Evidence SHALL record API and shipped-UI observations for local proxy, Kimi K3, and MiniMax M3 through installed UAR, along with source SHA, server-full profile, platform, timeout/output limits, and redaction. macOS runtime, Linux-template, and Windows compile/template results SHALL be separate.

#### Scenario: Required provider is unavailable
- **WHEN** a credential, endpoint, capacity, or model prevents a required genuine response
- **THEN** the phase stops before reflection and reports that exact inference requirement as unmet without substituting synthetic evidence

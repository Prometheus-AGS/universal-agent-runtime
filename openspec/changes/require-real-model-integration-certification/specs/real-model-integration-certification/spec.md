## Purpose

Ensure every inference-readiness claim is backed by a full runtime request that
reaches a real loaded model and observes genuine model inference output.

## ADDED Requirements

### Requirement: Inference certification uses real model inference
UAR SHALL count an inference test toward integration, soak, resilience, release,
or production-readiness certification only when the request traverses the
supported packaged runtime boundary, reaches a real loaded model through the
configured provider path, performs model inference, and returns the resulting
output through UAR.

#### Scenario: Certifying inference request
- **WHEN** an executor runs an inference integration or certification test
- **THEN** the request reaches a real model and performs actual inference
- **AND** the retained evidence identifies the provider and model and proves a
  genuine model response was observed through UAR

#### Scenario: Real locally hosted model
- **WHEN** a locally hosted model is used instead of an external provider
- **THEN** the test qualifies only if the model weights are loaded and the model
  performs actual inference for each certifying request

### Requirement: Synthetic model responses never substitute for integration
UAR SHALL NOT count mocked, stubbed, recorded, replayed, hard-coded, or synthetic
model responses as inference integration, soak, resilience, release, or
production-readiness evidence.

#### Scenario: Real inference is unavailable
- **WHEN** credentials, model capacity, model weights, network access, budget, or
  another prerequisite for real inference is unavailable
- **THEN** the executor stops and reports the inference claim as unverified
- **AND** the executor does not substitute a synthetic provider or synthetic
  response

#### Scenario: Fast isolated test uses a model double
- **WHEN** a unit or component test uses a model double to exercise isolated code
- **THEN** it is labeled non-certifying and is not described or counted as an
  inference integration, soak, resilience, release, or production-readiness test

### Requirement: Long-running inference tests justify and exercise their duration
UAR SHALL run a multi-hour inference test only when real inference occurs
throughout the test and the duration is tied to a named failure-detection or
production-load objective.

#### Scenario: Proposed multi-hour inference test uses synthetic responses
- **WHEN** a proposed multi-hour test would send mocked, stubbed, recorded,
  replayed, hard-coded, or synthetic model responses
- **THEN** the executor does not run the test

#### Scenario: Proposed duration lacks an objective
- **WHEN** a proposed multi-hour duration has no documented failure model,
  traffic-volume target, operating-period target, or statistical detection goal
- **THEN** the executor stops and obtains an explicit objective before consuming
  the test time

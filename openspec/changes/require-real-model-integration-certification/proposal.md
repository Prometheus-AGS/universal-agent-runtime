## Why

Mocked inference tests consumed multi-hour release time while leaving the actual
real-model request path unverified. Certification evidence must exercise real
inference against a real model so it answers the production question it claims
to test.

## What Changes

- Prohibit mocked, stubbed, recorded, or synthetic model providers from
  satisfying integration, soak, resilience, release, or production-readiness
  certification.
- Require every inference certification request to reach a real model and
  observe a real inference response through the supported runtime boundary.
- Permit mocked-provider tests only as non-certifying unit or component checks;
  they cannot replace or count toward required integration evidence.
- Record the rule in both repository agent-policy files so future executors stop
  rather than schedule a noncompliant certification.

## Capabilities

### New Capabilities

- `real-model-integration-certification`: Defines the mandatory real-provider,
  real-model inference boundary for integration and release evidence.

### Modified Capabilities

None.

## Impact

- Affects `CLAUDE.md`, `AGENTS.md`, and future integration, soak, resilience,
  release, and production-readiness test plans.
- Existing mocked certifications cannot support inference-readiness claims and
  must not be repeated as substitutes for real-model integration.
- No runtime API, provider compatibility, dependency, UI, realtime state, or KBD
  workflow-state implementation changes are made by this policy-only change.

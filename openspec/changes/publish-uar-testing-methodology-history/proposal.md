## Why

UAR's retained test documentation spans coverage floors, unit and component
checks, recorded providers, conformance matrices, browser journeys, negative
controls, long-running soak plans, and genuine-model functional acceptance. The
methods changed after expensive runs produced evidence for a different boundary
than the one being claimed. Readers need one current explanation of what each
test class proves, what it cannot prove, and when it belongs in the delivery loop.

## What Changes

- Publish a dated testing-methodology history, evidence taxonomy, negative-control
  guide, and local verification guide.
- Distinguish diagnostic model doubles from certifying genuine-model inference.
- Explain why duration alone does not turn synthetic traffic into operational or
  inference evidence.
- Preserve the tier ladder, delivery-first cadence, per-profile/source-SHA limits,
  and deployment-only GitHub Actions policy.
- Add local controls that reject evidence classes without limits, synthetic
  inference certification, unpaired fail-closed claims, profile transfer,
  duration-only soak claims, and routine Actions testing.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `documentation-publication-contract`: Require a source-traceable evidence
  taxonomy with explicit negative controls, profile limits, and private-history
  synthesis boundaries.
- `dev-portal-2026`: Add stable public testing-history and local-verification
  guides without using GitHub Actions for routine documentation checks.

## Impact

- Documentation pages, testing-history manifest, local validators, and the
  coverage ADR's supersession note.
- No runtime, test implementation, React application, dependency, package-lock,
  workflow, or deployment behavior changes.
- No product test, real-model request, soak, production build, browser run, or
  GitHub Actions test is executed by this content change.

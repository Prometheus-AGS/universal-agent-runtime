## Context

The strongest retained lesson is not that one test type replaced every other
type. It is that evidence must match the claimed boundary. Unit and recorded
provider checks remain useful diagnostics. They do not certify model inference.
A long synthetic run exercises plumbing for a long time; it still does not cross
the model boundary. UAR's 1.0 closeout therefore replaced an incomplete
release-tail plan with five bounded real-model functions through the packaged API
and shipped UI, while explicitly cancelling the uncompleted soak, supply-chain,
release-candidate, and publication claims.

## Goals / Non-Goals

**Goals:** publish the evidence ladder, negative-control discipline, timing rules,
profile/source-SHA limits, and historical corrections with explicit non-claims.

**Non-Goals:** change tests, thresholds, tier hooks, workflows, runtime code, or
release status; rerun historical evidence; claim that coverage equals correctness;
or require expensive testing while content is still being written.

## Decisions

### 1. Organize by the question evidence answers

The taxonomy distinguishes static/type evidence, focused unit/component evidence,
synthetic/recorded boundary diagnostics, packaged functional integration,
genuine-model inference, load/soak/resilience, and deployment validation. Every
class carries both `proves` and `doesNotProve` fields.

### 2. Keep real-model inference a narrow certifying boundary

Inference evidence must traverse the supported packaged UAR boundary, reach a
real loaded model through the configured provider, and return its output through
UAR. A response string alone does not prove provider/model routing; retained
evidence also identifies the provider/model and effective policy where relevant.

### 3. Require observed negative controls for fail-closed claims

A guard or test that is only observed passing can be vacuous. The guide requires
the same assertion to be observed failing under a deliberate, bounded inversion,
then restored exactly and observed passing. Controls are required for fail-closed
claims, not indiscriminately for every test.

### 4. Keep broad checks behind delivery boundaries

The standing tier ladder remains: Tier 0 at its edit boundary, Tier 1 when an
implementation unit is complete, Tier 2 at phase completion, and Tier 3 at a
milestone/release. Stack-specific rules may consolidate related edits into one
cohesive slice. Unchanged expensive commands are not repeated without a source
change or contract requirement.

### 5. Keep Actions deployment-only

All routine unit, integration, conformance, lint, formatting, type, and docs
checks run locally. GitHub Actions may build/deploy an artifact and validate the
resulting deployment. This explicitly supersedes ADR-0003's old CI mechanism,
not its still-documented local coverage target.

## Risks / Trade-offs

- Real-model evidence is nondeterministic and can cost money; unavailable
  prerequisites correctly leave the claim unverified.
- Negative controls add effort and can damage source if restoration is casual;
  the guide requires bounded mutation and exact restoration.
- Local verification depends on retained commands/output rather than hosted
  checkmarks; incomplete evidence records are easier to lose.
- A compact taxonomy can be misread as a universal command list; each profile and
  change contract remains authoritative for its exact commands.

## Migration Plan

1. Add the testing-history manifest and four public guides.
2. Add bounded validators and negative controls, composed into publication.
3. Mark the coverage ADR's hosted-CI mechanism superseded.
4. Run local source checks after content completion, then strict OpenSpec and
   artifact-refiner validation.
5. Record evidence, transition KBD, and commit independently.

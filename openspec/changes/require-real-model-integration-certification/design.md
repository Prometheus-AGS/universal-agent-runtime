## Context

See `proposal.md` for motivation. `CLAUDE.md` and `AGENTS.md` are parallel
repository policy surfaces consumed by different agent harnesses. Their
project-specific sections are outside auto-managed regions and survive context
bootstrap regeneration.

The current operational certification routes inference requests to a local
deterministic provider double. Running that workload for multiple hours does not
verify the real provider/model boundary and cannot remain acceptable release
evidence.

## Goals / Non-Goals

**Goals:**

- Give every agent harness the same fail-closed real-inference rule.
- Define full integration as the packaged UAR request path reaching a real loaded
  model and returning genuine inference output.
- Prevent long-running synthetic inference workloads from consuming release time.
- Preserve fast isolated tests only as explicitly non-certifying diagnostics.

**Non-Goals:**

- Redesign or run the replacement real-model integration suite in this change.
- Select a provider, model, credential source, spending budget, or duration.
- Change runtime APIs, provider routing, or model behavior.

## Decisions

### Put the same rule in both repository policy files

Add one identical `Real-model integration testing` section to the
project-specific region of `CLAUDE.md` and `AGENTS.md`. This is preferred to a
rule in only one file because the repository is operated by multiple harnesses.
The managed base regions are not edited because regeneration would overwrite
them.

### Fail closed when real inference is unavailable

An executor stops and reports the integration claim as unverified when a real
model cannot be reached. It may not silently fall back to a provider double.
This is preferred to a best-effort fallback because a passing fallback recreates
the exact false-confidence failure this change addresses.

### Separate isolated diagnostics from inference certification

Fast unit or component tests may still use model doubles when they test isolated
logic, but they must be labeled non-certifying. They cannot satisfy or be
reported as integration, soak, resilience, release, or production-readiness
evidence. Multi-hour synthetic inference workloads are prohibited entirely.

### Require a reason for elapsed time

Real inference alone does not justify a multi-hour run. A long duration must map
to a named failure model, production traffic target, operating-period target, or
statistical detection objective. This prevents elapsed time from being treated
as evidence by itself.

## Risks / Trade-offs

- **[Risk] Real-provider tests cost money and can fail because of external
  capacity or networking.** → Report that boundary explicitly; never replace it
  with synthetic success evidence.
- **[Risk] Real model output is nondeterministic.** → Assert protocol, routing,
  streaming, tool, usage, and completion invariants rather than exact prose.
- **[Risk] Existing mock-based evidence is no longer sufficient.** → Mark the
  affected inference claims unverified and replace them with real-model evidence
  before release certification.

## Migration Plan

1. Add the policy section to both agent instruction files.
2. Validate the new OpenSpec contract and the two-file policy diff.
3. Treat existing synthetic inference runs as non-certifying diagnostics.
4. Plan the replacement real-model integration suite as a separate change with
   explicit provider/model, budget, workload, and duration objectives.

Rollback would remove the policy and re-allow synthetic certification; that
would restore the false-confidence failure and is not an acceptable operational
fallback.

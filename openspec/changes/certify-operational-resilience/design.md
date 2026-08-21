## Context

See `proposal.md` for motivation and
`specs/operational-resilience-certification/spec.md` for the required failure
and soak behavior. The installed-artifact harness already exercises lifecycle,
provider failures, MCP process loss, parallel load, streaming reconnects, cold
backup/restore, and a non-root container. The remaining work is to bind that
harness to one committed source revision and retain certifying evidence.

The uncomfortable current fact is that `.github/workflows/operational-resilience.yml`
sets the soak to 60 seconds and the job timeout to 120 minutes. That lane can
validate workflow wiring, but it cannot produce the required multi-hour result.

## Goals / Non-Goals

**Goals:**

- Produce replayable machine-readable evidence from an installed `server-full`
  archive and image built from one exact Git SHA.
- Keep pull-request feedback short while making manual and scheduled executions
  certifying only after at least 10,800 seconds of streaming/reconnect soak.
- Fail closed on lifecycle, failure recovery, MCP restart, load, duplicate-event,
  latency, memory-growth, backup/restore, non-root, health, or signal failures.
- Retain the complete result directory as one GitHub Actions artifact.

**Non-Goals:**

- This change does not create a release-candidate or GA tag, publish an image or
  GitHub release, or claim external-install/operating-period evidence.
- The 60-second pull-request lane is a deployment-validation preflight and is
  never reported as the multi-hour certification.
- No product runtime behavior is changed unless the installed-artifact run
  exposes a supported-product defect, which remains a stop condition under the
  outer execution contract.

## Decisions

### Use two explicit duration lanes

Pull requests run a 60-second preflight. `workflow_dispatch` accepts a numeric
`soak_duration_seconds` input defaulting to 10,800, and scheduled runs use
10,800. Manual and scheduled executions reject any duration below 10,800. The
deterministic job timeout is 300 minutes so a three-hour soak plus build and
packaging can complete.

This is preferred to silently changing every event to three hours, which would
make pull-request feedback unusable, and to accepting an arbitrary short manual
duration, which could be mislabeled as certification.

### Certify the installed process boundary

`scripts/certify-operational-resilience.sh` remains the entry point. Its
deterministic Rust test is supplemental; the authoritative product evidence is
the nested `scripts/certify-release-candidate.sh` result from the installed
archive and image. That path boots the packaged binary, exercises HTTP and MCP
process boundaries, restarts the runtime, restores a cold backup, and runs the
container as UID 65532.

This is preferred to treating the synthetic Rust checks alone as release proof.

The embedded entity-management repository is a nested pnpm workspace with its
own authenticated pnpm 10.33.0 pin. Its developer-engine contract admits pnpm
versions from 10.33.0 through 11.x so an outer pnpm 11 task can invoke the
nested dependency closure without replacing the repository's pinned default.
The workflow installs and builds that workspace from its own boundary before
the outer build. UAR advances the gitlink to upstream commit `959839a`, where a
clean Corepack cache accepts the corrected integrity digest and a detached
clean-worktree proof builds core then React under pnpm 11.15.0. This preserves
both repositories' authenticated toolchain contracts instead of disabling
Corepack verification.

### Pin fail-closed thresholds in the retained result

The certifying lane requires:

- parallel installed-runtime requests: 20, failures: 0;
- streaming errors: 0 and duplicate events: 0;
- streaming p95 latency: at most 2,000 ms;
- peak RSS growth: at most 262,144 KiB;
- provider outage, rate limit, malformed stream, MCP crash, transport loss, and
  tool timeout surface explicitly before a later recovery succeeds;
- cold-backup and restored tree digests match;
- health/readiness succeed, the container UID is non-root, persistence is
  writable, and SIGTERM exits with status 0.

The harness writes the configured duration, observed duration, thresholds,
source SHA, and candidate label into JSON. A missing or malformed result is a
failure, not an omitted measurement.

### Bind and retain one evidence directory

The workflow passes `github.sha` as `UAR_CANDIDATE_SOURCE_SHA`; the harness
refuses a checkout/source mismatch. It uploads the complete
`target/resilience-certification/` directory with `if-no-files-found: error`,
including deterministic output, installed-runtime JSON, and logs. The evidence
receipt records the workflow run ID, source SHA, artifact name, duration, and
result hashes.

This is preferred to copying selected console excerpts into prose, which would
lose machine-verifiable provenance and negative-control output.

## Risks / Trade-offs

- **[Risk] A three-hour run consumes substantial hosted-runner time.** → Keep the
  pull-request lane at 60 seconds and reserve certification for manual/scheduled
  runs.
- **[Risk] A source edit after the run makes the evidence stale.** → Bind every
  result to the exact SHA and rerun before downstream candidate certification.
- **[Risk] The workflow definition must be present on the default branch for a
  manual dispatch.** → Validate and merge the workflow correction before using
  its new input; do not reinterpret an older 60-second run as certifying.
- **[Risk] Runner variance can move latency and RSS.** → Keep the published
  absolute limits, retain raw measurements, and fail rather than weakening a
  threshold after observing a miss.

## Migration Plan

1. Add the typed duration input, event-specific duration gate, and 300-minute
   timeout to the deployment-validation workflow.
2. Run local syntax, deterministic harness, result-schema, OpenSpec, and diff
   checks without claiming the time-bound soak.
3. Commit and push the exact candidate source.
4. After the workflow correction exists on the default branch, dispatch the
   certifying lane for at least 10,800 seconds and retain its artifact.
5. Record actual commands, outputs, run identity, hashes, and limits in
   `verification.md`; only then complete the evidence tasks and archive.

Rollback is deletion of the workflow input/duration-selection step. Existing
60-second preflight behavior remains available on pull requests throughout.

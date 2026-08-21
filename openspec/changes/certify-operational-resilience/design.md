## Context

See `proposal.md` for motivation and
`specs/operational-resilience-certification/spec.md` for the required failure
and soak behavior. The installed-artifact harness already exercises lifecycle,
provider failures, MCP process loss, parallel load, streaming reconnects, cold
backup/restore, and a non-root container. The remaining work is to bind that
harness to one committed source revision and retain certifying evidence.

The uncomfortable current fact is that the first correction tried to turn
`.github/workflows/operational-resilience.yml` into a three-hour product-test
runner. That violated the repository's older deployment-only GitHub Actions
decision. Run `32458212074` was canceled and is not certification evidence.

## Goals / Non-Goals

**Goals:**

- Produce replayable machine-readable local evidence from an installed
  `server-full` archive and image built from one exact Git SHA.
- Run the certifying streaming/reconnect workload locally for at least 10,800
  seconds from a clean detached checkout.
- Fail closed on lifecycle, failure recovery, MCP restart, load, duplicate-event,
  latency, memory-growth, backup/restore, non-root, health, or signal failures.
- Retain the complete result directory with the OpenSpec evidence and bind it to
  the source commit with cryptographic hashes.
- Keep GitHub Actions limited to actual deployment execution and
  deployment-specific validation.

**Non-Goals:**

- This change does not create a release-candidate or GA tag, publish an image or
  GitHub release, or claim external-install/operating-period evidence.
- This change does not move local product checks into any hosted CI service.
- No product runtime behavior is changed unless the installed-artifact run
  exposes a supported-product defect, which remains a stop condition under the
  outer execution contract.

## Decisions

### Certify only from a local immutable checkout

The local entrypoint requires an explicit duration of at least 10,800 seconds
for a certifying run, an exact candidate source SHA, and a clean detached
checkout. A short local preflight may prove wiring but cannot be recorded as
multi-hour certification. Product-test, installed-artifact, security, load,
stress, soak, and release-certification workflows are removed from GitHub
Actions; a local policy validator rejects their return.

This is preferred to relabeling a hosted product-test workflow as deployment
validation. The deployment-only boundary predates this change and takes
precedence over a task plan.

### Certify the installed process boundary

The local candidate builder and `scripts/certify-operational-resilience.sh`
remain the entry points. The latter's
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
The local builder installs and builds that workspace from its own boundary before
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

### Bind and retain one local evidence directory

The local launcher passes the detached checkout's exact SHA as
`UAR_CANDIDATE_SOURCE_SHA`; the harness refuses a checkout/source mismatch. It
retains the complete result directory, including deterministic output,
installed-runtime JSON, logs, the candidate archive digest, duration, and
result hashes. The final evidence is copied into the change only after the
immutable run finishes and its manifest validates.

The tested source commit is frozen before the run. A direct child evidence
commit may add only verification artifacts, hashes, and task/progress updates
that name that tested parent. Any implementation, script, dependency, workflow,
or product-documentation change after the run creates a new candidate and
requires a complete rerun.

This is preferred to copying selected console excerpts into prose or relying on
an external workflow URL, either of which would weaken replayable provenance.

## Risks / Trade-offs

- **[Risk] A three-hour local run occupies one machine and target directory.** →
  Use a dedicated detached worktree and target directory, retain progress logs,
  and preserve the single-writer build discipline.
- **[Risk] A source edit after the run makes the evidence stale.** → Bind every
  result to the exact SHA and rerun before downstream candidate certification.
- **[Risk] An old hosted run could be mistaken for current evidence.** → Record
  the canceled run as superseded and accept only the local manifest whose source
  SHA and hashes match the frozen candidate.
- **[Risk] Runner variance can move latency and RSS.** → Keep the published
  absolute limits, retain raw measurements, and fail rather than weakening a
  threshold after observing a miss.

## Migration Plan

1. Remove non-deployment GitHub Actions workflows and add a local policy
   validator that fails if they return.
2. Add a local immutable-candidate builder/launcher and run short syntax,
   deterministic harness, result-schema, OpenSpec, and diff checks without
   claiming the time-bound soak.
3. Commit the exact candidate source and create a clean detached worktree.
4. Run the local certifying lane for at least 10,800 seconds and retain its
   complete evidence directory.
5. Record actual commands, outputs, source identity, hashes, and limits in
   `verification.md`; only then complete the evidence tasks and archive.

Rollback is restoration of the local-only scripts and contracts from the prior
commit, never restoration of non-deployment testing in GitHub Actions.

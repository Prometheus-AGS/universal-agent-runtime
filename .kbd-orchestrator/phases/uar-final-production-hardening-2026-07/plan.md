# Plan — uar-final-production-hardening-2026-07

Project: Universal Agent Runtime
Date: 2026-07-13
Mode: operator-locked production completion
Progress: 19/24 formally complete

> EXECUTION LOCK: Complete changes 20–24. CI and tests are evidence, not the work queue. Do not poll workflows while an actionable release step remains. Batch fixes, keep Windows nonblocking, and obey operator priority over historical context.

## Current product truth

- The `server-full` BossFang sidecar implementation is complete.
- Consolidated local Rust, frontend, distribution, and contract validation passed.
- Release/platform/resilience/supply-chain/candidate/GA integrations are implemented.
- Remaining work is immutable evidence, external/time-bound validation, and operator-authorized release effects.

## Completion map

| # | Change | State | Completion requirement |
|---:|---|---|---|
| 20 | `align-release-workflow-platforms` | IMPLEMENTED / EVIDENCE_PENDING | Supported Linux/macOS candidate jobs and retained artifacts; Windows remains Experimental |
| 21 | `certify-operational-resilience` | IMPLEMENTED / EVIDENCE_PENDING / TIME_BOUND | Machine-readable lifecycle, recovery, non-root, and soak evidence |
| 22 | `produce-supply-chain-artifacts` | IMPLEMENTED / EVIDENCE_PENDING | Signed checksums, SBOMs, provenance, images, manifest, independent verification |
| 23 | `certify-release-candidate` | IMPLEMENTED / EVIDENCE_PENDING / TIME_BOUND / AUTHORIZATION | Immutable RC, clean installs, three external installs, operating period, approval |
| 24 | `release-1-0-0` | IMPLEMENTED / EVIDENCE_PENDING / AUTHORIZATION | No-rebuild GA promotion and public endpoint verification |

## Execution sequence

### 1. Stabilize the source

- Keep PR #98 reviewable and free of unrelated edits.
- Treat only demonstrated Stable Linux/macOS defects as source blockers.
- Do not chase Experimental Windows failures.
- Obtain operator authorization before merging to main.

### 2. Freeze one immutable candidate

- Record source, lockfile, catalog, and model digests.
- Obtain operator authorization before creating `v1.0.0-rc.1`.
- Make no source change after certification begins; if source changes, restart the candidate sequence once.

### 3. Certify concurrently

- Platform lane: Linux/macOS archives, install, startup, health.
- Resilience lane: lifecycle, failure recovery, non-root, backup/restore, soak.
- Supply-chain lane: checksums, SBOMs, provenance, signatures, digest-addressed image and manifest.
- Candidate lane: clean artifact journeys and documentation procedures.

These lanes run asynchronously. Inspect completed results; do not sit polling them. A failure becomes active work only when it exposes a supported product defect.

### 4. Complete external/time-bound evidence

- Record three external installs without checkout-specific knowledge.
- Complete the specified operating period.
- Bind all evidence to the immutable candidate SHA.

### 5. Promote unchanged source to GA

- Confirm candidate/source identity.
- Obtain operator authorization for tag and publication.
- Promote without rebuilding.
- Verify every public artifact and endpoint.
- Archive OpenSpec changes and run KBD reflection.

## Verification policy

- Implementation stage: static inspection and cohesive `cargo check` only.
- Certification stage: one consolidated supported-product sequence.
- Experimental Windows failures are retained as evidence but are nonblocking.
- No additional tests are written or run merely to create activity.

## Approval gates

- Merge to main.
- Candidate and GA tags.
- GitHub/GHCR publication and signing identity.
- Acceptance of external-install and operating-period evidence.

## Stop condition

Stop only at 24/24 or at a genuine missing authorization, external/time-bound condition, or supported-product defect. Report the precise condition; do not substitute workflow monitoring.

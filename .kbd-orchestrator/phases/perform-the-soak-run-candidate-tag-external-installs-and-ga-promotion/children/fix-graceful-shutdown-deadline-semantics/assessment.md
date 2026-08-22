# ASSESSMENT: fix-graceful-shutdown-deadline-semantics

Project: universal-agent-runtime
Date: 2026-08-22

## Codebase baseline

- Candidate source SHA: `32afa53d510c8b840b3e98b2be9d9f5dee149531`.
- Active parent phase: `perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion`.
- The full local operational-resilience command ran from a clean detached checkout.
- Deterministic operational-resilience tests passed 5/5.
- The 10,800-second soak passed with 10,196 requests, 0 errors, 0 duplicate events, p95 13 ms against a 2,000 ms limit, and peak RSS growth 5,376 KiB against a 262,144 KiB limit.
- Native archive install, readiness, SIGTERM exit 0, backup/restore, and restart passed.
- Certification failed at the final non-root container lifecycle. Docker sent SIGTERM, then SIGKILL at its 30-second deadline; the container exited 137. The certification script correctly rejected the non-zero process exit and did not write `non-root-container.json` or `results.json`.

## IMPLEMENTATION STATUS

- Existing server code intercepts SIGINT/SIGTERM, cancels in-flight runtime work, shuts down the ingestion pool, and broadcasts an HTTP shutdown token to primary and companion Axum listeners.
- The Axum shutdown future then sleeps for the complete `shutdown_timeout_secs` before returning. Because Axum begins graceful drain only when that future resolves, the implementation delays drain initiation by 30 seconds.
- The candidate log proves the sequence: SIGTERM at `21:36:37.181843Z`; pool drained and HTTP shutdown token cancelled by `21:36:37.183811Z`; server reported graceful stop at `21:37:07.189286Z`. Docker's outer 30-second stop deadline expired at the same boundary and produced exit 137.
- No product fix exists in the active child.
- The deterministic lifecycle test in `tests/operational_resilience.rs` is synthetic channel/task coverage and does not exercise the UAR process signal boundary.
- The integration child-process harness separates caller-owned HTTP-token cancellation from actual SIGTERM and checks successful process exit, but it does not assert the canonical no-active-connection one-second limit.
- The release-candidate script's `docker stop --time 30` boundary is a valid external orchestrator test, but using the same 30-second value as UAR's internal maximum leaves no scheduling margin. This harness detail must not substitute for correcting the product semantics.

## CROSS-TOOL PROGRESS

- The KBD child phase exists and is active through the canonical runtime at revision 256.
- The child creation wrapper failed after canonical creation because its own `child_label` shell variable was unset. Canonical activation repaired the projection without manually editing the waypoint.
- No OpenSpec change has been created for this child.
- No child implementation or verification task has been completed.

## SPEC GAP SUMMARY

- The canonical graceful-shutdown specification requires immediate shutdown initiation, process exit within one second when no connections are active, up to the configured deadline for active work, and forced termination when the deadline expires.
- Current behavior violates the no-active-connection scenario by always waiting the full timeout.
- Current behavior does not enforce the timeout as a maximum drain deadline; it uses the timeout as a pre-drain delay.
- Existing tests do not directly prove immediate no-active process exit or forced completion at the configured deadline.
- The minimum change needs an OpenSpec delta that makes deadline semantics and a real process-boundary negative control explicit.

## BUILD HEALTH

- At source SHA `32afa53d510c8b840b3e98b2be9d9f5dee149531`, native release/test construction completed and deterministic operational-resilience tests passed 5/5.
- Full candidate certification outcome: failed.
- Failed requirement: non-root container SIGTERM must exit 0 before the orchestrator kill deadline.
- Observed negative result: exit 137 after Docker SIGKILL.
- The soak, backup/restore, native restart, and deterministic results remain valid evidence for this SHA, but no evidence transfers to a corrected SHA without a complete fresh certification run.

## CONSTRAINT CHECK

- The defect is observed at a real process/container boundary; a shutdown deadline is therefore justified defensive behavior.
- Work must remain local. GitHub Actions are not an allowed test runner.
- Tier 0 applies after each edit, Tier 1 after the focused unit is complete, and the full certification remains the parent release gate.
- Existing unrelated working-tree changes must be preserved.
- The child write scope is currently documentation-only and must be explicitly widened during Plan before implementation.
- Candidate source immutability means any product or certification-script change creates a new candidate SHA and restarts the full 10,800-second certification from zero.

## GOAL PROGRESS

- Goal 1, correct `shutdown_timeout_secs` semantics: missing.
- Goal 2, prove non-root SIGTERM exits 0 before the outer deadline: missing; current observed result is exit 137.
- Goal 3, freeze a corrected candidate and rerun complete local certification: missing.
- Assessment conclusion: the test is valid and the failure is a major release-blocking operational defect. It is not evidence of an authentication, tenancy, or data-corruption defect. It is material because common container orchestrators can kill UAR during rolling updates before orderly connection and resource closure completes.

## ASSESSMENT COMPLETE

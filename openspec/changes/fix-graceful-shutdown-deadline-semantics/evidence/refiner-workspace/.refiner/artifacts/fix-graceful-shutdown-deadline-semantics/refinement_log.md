# Refinement log — `fix-graceful-shutdown-deadline-semantics`

## Iteration 1 — 2026-08-22T10:50:01Z

- Specify: derived five blocking constraints from the child OpenSpec, KBD
  surface, outer immutable-candidate rules, and observed exit-137 failure:
  absolute deadline, owned cleanup, container margin, fail-closed evidence,
  and bounded scope.
- Plan: preserve behavioral failures first, implement the minimum private
  coordinator and resource ownership, prove focused units locally, exercise a
  real non-root held request, then commit before the parent soak restarts.
- Execute: removed the mandatory pre-drain delay; armed an executor-independent
  watchdog at signal observation; made forced and graceful outcomes exclusive;
  explicitly terminated MCP and live-query owners; retained ingestion/A2A
  joins; observed SurrealKV release before exit; and added the held-work
  container journey.
- Verification: process controls passed 9/0; MCP 4/0; live-query 1/0;
  same-path C-12 1/0; SIGINT caller control 1/0; check, scoped Clippy, strict
  OpenSpec, shell, dependency, visibility, and scoped diff gates exited 0.
- Negative evidence: the baseline process control failed 6 intended
  assertions; the different-path C-12 control failed at the intended 404
  assertion; the prior immutable container exited 137 after Docker SIGKILL.
- Container evidence: after correcting the manual control to the image's real
  port, Docker health was `healthy`, UID was 65532, stop elapsed was 30,489
  ms, held curl exited 18, UAR exited 0 with only `deadline_enforced`, and no
  SIGKILL event was present.
- Uncomfortable result: the same-path C-12 runs still emit SurrealKV's teardown
  warning that no runtime is available during nested helper destruction. It is
  not hidden or called clean. The stronger external same-path acquisition
  passes while the original helper remains alive, so the ownership assertion
  is based on acquisition rather than absence of a warning.
- Scope: the root `.refiner` tree, local settings, unrelated KBD projection
  churn, static output, dependencies, public APIs, protocols, providers,
  GitHub Actions, push, PR, release, and GA remain outside this artifact.
- Reflect: all five blocking constraint IDs match state, constraints, and
  verification evidence. State, constraints, manifest, three progressive
  checkpoints, manifest references, strict OpenSpec, and structured container
  receipts passed deterministic validation. The prior history-free artifact
  critic and judge passed the corrected plan, and the later history-free
  lifecycle critic and judge passed the only implementation-discovered scope
  expansion.
- Decision: converge after the Persist checkpoint and active/history replay.
  The parent soak is a handoff condition, not evidence that may be fabricated
  inside this child refinement.
- Persist: write the five-phase progressive state, constraints, manifest,
  verification summary, decisions, and exact validation receipt into the
  contained OpenSpec evidence tree. Finalize only after the Persist snapshot;
  then require active/history byte identity before accepting convergence.

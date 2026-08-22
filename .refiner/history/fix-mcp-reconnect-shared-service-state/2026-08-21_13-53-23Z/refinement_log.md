# Refinement log — `fix-mcp-reconnect-shared-service-state`

## Iteration 1 — 2026-08-21T12:30:48Z

- Specify: derived five blocking constraints from the child OpenSpec, KBD scope,
  and installed failure: shared replacement state, non-replay, authorization,
  immutable evidence, and truthful bounded handoff.
- Plan: bind the direct-content artifact to source `ae77d570`, replay raw SSE
  and process evidence, validate schemas and requirements, then require
  history-free critic and judge review before persistence.
- Execute: replayed the raw parser successfully; persisted crash and timeout SSE,
  the exact five-row process trace, candidate result, process summary, validation
  output, source/tool hashes, exact Tier 0/Tier 1 receipts, and scope limits.
- Uncomfortable result: the prior candidate failed after `echo, crash` and never
  reached a replacement process. The child exists because prose recovery claims
  were false at the installed process boundary.
- Reflect: pending history-free critic and independent judge.

## Iteration 2 — 2026-08-21T13:44:08Z

- Reflect delta: the history-free critic found that an old filtered view could
  reconnect with configuration A after an A→B upsert, all state/checkpoints
  stored constraint IDs instead of full objects, and the installed result named
  five raw files that were not retained. The independent judge missed these
  blockers, so the critic finding controlled the correction.
- Execute: paired every shared service with its authoritative reconnect entry
  and generation; a reconnect now swaps only when its generation remains
  current. Added the A→B upsert, crash, reconnect regression, which passed 1/0.
- Execute evidence: retained all result-referenced files byte-for-byte, restored
  full constraint-object parity, expanded the manifest references, and rebuilt
  the immutable candidate from source `f0298d76ea3c39853020c8a33e13f136c07a1806`.
- Installed result: local macOS arm64 release, local Linux arm64 image, focused
  operational suite 5/0, and the 60-second installed preflight all passed. The
  raw crash and timeout each failed once without replay and crossed to new PIDs.
- Reflect: a fresh history-free critic and independent judge both returned PASS.
  They independently replayed source ancestry, hashes, retained references,
  full constraint-object parity, raw process evidence, strict OpenSpec, scope,
  and the authorization/non-replay semantics.
- Constraint status: 5/5 blocking constraints satisfied. No regression was
  found from the corrected Execute state; the manifest gained the durable
  artifact-refiner validation receipt and lost no prior reference.
- Convergence: terminate. The parent three-hour soak, release-candidate tag,
  external installs, deployment validation, and GA remain deliberately outside
  this child and must resume from source `f0298d76`.

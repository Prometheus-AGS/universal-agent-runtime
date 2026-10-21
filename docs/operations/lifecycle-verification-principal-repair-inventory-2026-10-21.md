# Embedding principal repair source boundary

Execute phase: `uar-outstanding-issues-v1-1-1::function-first-release-certification`.
Canonical waypoint3189, plan82. Task9.7 is source-complete and the state writer
has released its files and handoff. This is implementation evidence, not a pass.

The previous approved embedding command compiled in14m32s and failed its one
test in3.67s at knowledge-base creation because its raw fixture owner did not
satisfy the existing verified-principal boundary. The earlier digest errors
did not recur. The full captured terminal result is in the attempt2 log.

Changed source: `tests/embedding_lifecycle_failover.rs`. The fixture now signs
and verifies its fixture-only JWT through `authenticate_bearer_context`, derives
the canonical owner through `session_owner_key`, and carries the verified key
and context across all storage, ingestion and router search operations. Its
existing local model, continuity, incompatible-space and cleanup assertions
remain. No new test, production authorization change, anonymous fallback,
dependency change, warning cleanup or shutdown-budget API was added.

Handoff: `lifecycle-approved-gate-principal-repair-2026-10-21.md`. No other writer
owns an unfinished source repair in this cohort. The companion JSON records
exact saved bytes, base Git objects, canonical ledgers and collected handoffs;
it excludes itself from the digest. It is authored directly from saved source,
without running an acceptance/source-identity checker.

Next command: only the identical operator-approved embedding command in the
proposal, with the previously recorded Cargo PATH environment repair. No flags,
feature profile or test selection change. If the repaired authorization failure
recurs, the command hangs or broader scope is required, stop affected work and
report its exact evidence. No repeated unchanged attempt is allowed.

All other gates remain held. Native candidate/payload source binding remains
blocked by uncommitted repairs and the old32cb3bcf pin. Full release preparation,
independent review, frozen embedding evidence and reflection remain unfinished.
The finite shutdown observation budget remains a proposed decision, not an
implemented contract. The checklist distinguishes these limits explicitly.

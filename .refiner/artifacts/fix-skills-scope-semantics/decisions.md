# Decisions — `fix-skills-scope-semantics`

## Iteration 1 — 2026-08-18T17:20:08Z

- **Decision:** continue to independent review before convergence.
- **Iteration:** 1 of 5.
- **Blocking violations remaining locally:** 0.
- **Rationale:** the earlier scoped-governance work already delivered durable
  state, precedence, cold restart, origin serialization, and delete protection.
  This reconciliation adds only the missing session-policy proof and built-in
  edit boundary.
- **Uncomfortable result:** the plan grouped M4 with H2/H3/O1, but M4 is a
  separate matching-quality enhancement rather than part of either observed
  release-blocking defect. Expanding into it would delay working software and is
  explicitly deferred.
- **Termination decision:** converge after the history-free critic and judge
  independently returned PASS. The critic's non-blocking schema-receipt warning
  was corrected before finalization.

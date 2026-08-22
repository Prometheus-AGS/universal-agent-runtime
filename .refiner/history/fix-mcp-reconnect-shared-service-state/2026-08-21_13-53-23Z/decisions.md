# Decisions — `fix-mcp-reconnect-shared-service-state`

## Iteration 1 — 2026-08-21T12:33:05Z

- **Decision:** continue to independent review before convergence.
- **Iteration:** 1 of 5.
- **Blocking violations remaining locally:** 0.
- **Rationale:** implementation, raw process evidence, negative controls, Tier 0,
  focused Tier 1, installed preflight, strict OpenSpec, and local workflow-policy
  checks pass; independent artifact review remains mandatory.
- **Commit exclusions:** operator settings, generated `static/`, prior screen
  evidence/refiner histories, unrelated KBD projections, parent certification
  evidence, and every GitHub Actions workflow.

## Iteration 2 — 2026-08-21T13:44:08Z

- **Decision:** continue to corrected independent review before convergence.
- **Iteration:** 2 of 5.
- **Blocking violations corrected:** stale reconnect configuration after upsert,
  incomplete constraint snapshots, and dangling installed-evidence references.
- **Rationale:** the generation-guarded shared state and exact A→B regression
  close the implementation defect; a fresh immutable candidate now binds all
  retained local evidence to `f0298d76`.
- **Uncomfortable thing:** the first judge issued PASS despite the reachable
  stale-config rollback. The critic's reproducible code path overruled it.

### Iteration 2 Decision

- **Decision:** terminate refinement and persist the corrected artifact.
- **Iteration:** 2 of 5.
- **Blocking violations remaining:** 0.
- **Rationale:** both fresh history-free reviews passed all five blocking
  constraints after the stale-configuration rollback, incomplete checkpoint,
  and dangling-reference defects were corrected and independently replayed.
- **Next focus:** resume the parent local three-hour certification from immutable
  source `f0298d76`; do not reuse the invalidated earlier candidate.

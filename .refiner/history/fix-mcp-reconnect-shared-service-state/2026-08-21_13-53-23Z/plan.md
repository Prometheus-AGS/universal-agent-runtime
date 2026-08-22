# Plan — `fix-mcp-reconnect-shared-service-state`

## Stages

1. Bind the verification artifact to source
   `f0298d76ea3c39853020c8a33e13f136c07a1806` and the retained installed
   evidence hashes.
2. Replay the raw crash and timeout SSE against the exact process trace and
   verify the expected five-mode sequence and two PID transitions.
3. Reconcile every OpenSpec requirement with focused tests, installed evidence,
   fail-closed negative controls, and an explicit scope limit.
4. Validate child and parent OpenSpec strictly, validate the GitHub Actions
   deployment-only policy, run scoped diff checks, and validate artifact JSON
   schemas and references.
5. Submit only the frozen artifact, governing contracts, source diff, and
   evidence to a history-free artifact critic and independent judge.
6. Correct any blocker in a new iteration; otherwise persist and finalize.

## Deterministic validation plan

| Constraint | Validation |
|---|---|
| `shared-replacement-state` | Exact focused registry tests, including A→B upsert then crash/reconnect, and installed PID sequence |
| `fail-closed-no-replay` | Raw SSE/process parser replay plus six synthetic rejected controls |
| `authorization-preserved` | Focused excluded-server/tool assertions and scope audit |
| `immutable-evidence-integrity` | Source identity, byte comparisons, SHA-256, file existence, JSON parse, and manifest references |
| `truthful-bounded-handoff` | Verification-row audit, strict OpenSpec, local Actions-policy validation, diff check, and independent review |

## State updates

- Create `artifact_manifest.json`, `constraints.json`, `specification.md`,
  `plan.md`, `refinement_log.md`, `decisions.md`, and
  `dist/verification-summary.md`.
- Maintain progressive phase checkpoints in `state.json`.
- Finalize only after both independent reviews pass.

## Commit exclusions

- `.claude/settings.local.json`
- generated `static/` output
- prior screen-validation evidence and refiner histories
- unrelated KBD projections and hook logs
- parent certification evidence not produced by this child
- any GitHub Actions workflow change

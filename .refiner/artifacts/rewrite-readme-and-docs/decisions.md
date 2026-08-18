# Decisions — `rewrite-readme-and-docs`

### Iteration 1

- **Decision**: continue
- **Iteration**: 1 of 5
- **Blocking violations remaining**: 3 of 4 constraints
- **Rationale**: review found a false router claim, unrendered Mermaid fences,
  and stale pnpm/SDK publication wording.
- **Next focus**: correct those exact findings and refresh focused evidence.

### Iteration 2 pre-review

- **Decision**: continue
- **Iteration**: 2 of 5
- **Blocking violations remaining**: pending independent re-review
- **Rationale**: the identified defects are corrected and deterministic checks
  pass, but the corrected candidate has not yet passed anti-sycophancy review.
- **Next focus**: independent artifact critic and judge re-review.

### Iteration 2 checkpoint correction

- **Decision**: continue
- **Iteration**: 2 of 5
- **Blocking violations remaining**: 1 provenance defect
- **Rationale**: the judge passed the implementation and evidence but found
  three phase checkpoints absent from state.
- **Next focus**: backfill and validate the missing checkpoint references, then
  request bounded re-review without rerunning product tests.

### Iteration 2 final

- **Decision**: terminate
- **Iteration**: 2 of 5
- **Blocking violations remaining**: 0
- **Rationale**: the independent critic and judge passed the corrected product,
  evidence, scope, and complete checkpoint chain; all four constraints pass.
- **Next focus**: finalize the persisted artifact and archive the completed
  OpenSpec change.

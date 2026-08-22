# Decisions — `harden-jwt-defaults`

## Iteration 1 — 2026-08-18T16:30:39Z

- **Decision:** continue to independent review before convergence.
- **Iteration:** 1 of 5.
- **Blocking violations remaining locally:** 0.
- **Rationale:** required-auth fallback rejection, shared registered-claim
  validation, and UAR-issued-token continuity have focused passing evidence.
- **Uncomfortable result:** the repository's scoped Clippy command still emits
  572 pre-existing warnings. It exits 0, but no warning-free result is claimed.

## Iteration 2 — 2026-08-18T16:49:59Z

- **Decision:** continue to final independent re-review.
- **Iteration:** 2 of 5.
- **Blocking violations remaining locally:** 0.
- **Rationale:** effective-secret validation now runs after Vault resolution,
  configuration tests are hermetic, and the public docs and `nbf` contract match
  the implementation's 60-second clock-skew allowance.
- **Uncomfortable result:** the first review correctly rejected receipts that
  still named the pre-fix source hash and 19-test configuration result. Those
  receipts are superseded by iteration 2 rather than silently rewritten.
- **Termination decision:** converge after both independent reviewers returned
  PASS on the corrected source, receipts, OpenSpec delta, and commit exclusions.
- **Last correction before convergence:** the judge caught stale Cargo filtered
  counts; exact replays replaced them with observed current values.

# Decisions — `fix-pnpm-lock-submodule-consistency`

## Iteration 1 — 2026-08-20T16:59:49Z

- **Decision:** continue to independent review before convergence.
- **Iteration:** 1 of 5.
- **Blocking violations remaining locally:** 0.
- **Rationale:** the retained lock fixes the observed importer mismatch without
  accepting unrelated range movement; independent review remains mandatory.
- **Independent review:** pending artifact critic and judge.
- **Commit exclusions:** operator settings, `frontend/pnpm-lock.yaml`, static
  output, parent screen-validation artifacts, and old certification bundles.

## Iteration 2 — 2026-08-20T17:10:00Z

- **Decision:** continue after correcting the first independent-review blocks.
- **Iteration:** 2 of 5.
- **Blocking violations found:** false minimum-delta claim, no clean full
  install, no positive supply-chain receipt, stale scope receipt, and absent
  refiner validation receipt.
- **Correction:** restore both noncausal edges, retain both causally required ws
  versions, run the clean full install, and refresh all hashes and receipts.
- **Independent review:** first critic and judge both BLOCK; corrected-candidate
  critic PASS and independent judge PASS.
- **Final decision:** terminate iteration 2 with 5/5 constraints satisfied.

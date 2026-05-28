See plan.md. Verification:
- bash scripts/ci-grep-gates.sh exits 0 locally
- pnpm --filter ./frontend test ≥ 40/40
- pnpm --filter ./frontend build clean

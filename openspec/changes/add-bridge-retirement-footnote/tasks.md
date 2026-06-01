See plan.md. Verification:
- pnpm run ci-gates exits 0
- pnpm --filter ./frontend test ≥ 40/40
- pnpm --filter ./frontend build clean
- Mermaid block renders cleanly on GitHub preview

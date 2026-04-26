# Verification: runtime-console-archive-readiness

Verified at: 2026-04-26T04:44:28-05:00
Verifier: codex

## Final Gate

- PASS: `openspec validate --changes`
- PASS: `bun run typecheck` from `frontend/`
- PASS: `bun run lint`
- PASS: `UAR_FRONTEND_E2E_PORT=18080 bun run test:e2e -- runtime-console-visual.spec.ts runtime-event-replay.spec.ts` from `frontend/` (9 passed)
- PASS: `cargo test workflow_mirror --lib`
- PASS: `cargo test provider_catalog_status --lib`
- PASS: `openspec validate runtime-console-entity-workflow --strict`
- PASS: `openspec archive runtime-console-entity-workflow -y`
- PASS: `git diff --check`
- PASS: generated `static/index.html` asset churn is clean

## Notes

- Playwright emitted expected Vite proxy ECONNREFUSED messages for backend API calls during UI-only e2e runs; tests use deterministic fixtures and all assertions passed.
- Moonshot live credential verification remains classified as credential-blocked until a safe runtime credential is configured outside the repository.

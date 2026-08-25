PLAN: fix-runtime-settings-namespace-routes
Project: universal-agent-runtime
Date: 2026-08-25
OpenSpec available: YES
Changes to implement: 1

CHANGE LIST (ordered)
1. fix-settings-namespace-read-routes: Canonicalize settings namespace GET routes and prove the installed LaunchAgent bundle
   - Scope: KBD gitlink | frontend API | focused Vitest | installed-service Playwright | native install evidence
   - Depends on: upstream `codex/kbd-run-rollover` commit `f1e58b25b0a9926c24d1bb0ddb6c0678d16c6f49`
   - Recommended agent: Codex
   - Est. complexity: M
   - Customer value: HIGH
   - Details: At Execute start, merge `origin/main` and pin the pushed KBD commit. Apply the one-line read-boundary conversion, add transport and installed-service regression coverage, then run the complete local gate/install/live-proof sequence while preserving provider IDs and operator-owned files. Backend aliases, persistence, payloads, save behavior, and adjacent refactors are explicit scope cuts.

EXECUTION ROUND ORDER
Round 1: merge origin/main and pin the exact upstream KBD commit
Round 2: implement the GET conversion and focused API test
Round 3: add the local installed-service Playwright proof
Round 4: run consolidated local certification, install, compare live state, and close out evidence

FILES INTENDED FOR EXECUTE
- `crates/prometheus-skill-system` (gitlink only)
- `frontend/src/features/settings/api/settings-api.ts`
- `frontend/src/features/settings/api/settings-api.test.ts`
- `frontend/playwright.installed-settings-routes.config.ts`
- `frontend/e2e/settings-routes-installed.spec.ts`
- `openspec/changes/fix-settings-namespace-read-routes/**`
- `.prometheus/session-log.md`, `.prometheus/decisions.md`, and `.prometheus/gotchas.md` append-only evidence as applicable
- KBD phase artifacts/projections through canonical commands; no hand edits to generated waypoint/progress JSON

VERIFICATION
- `pnpm typecheck`
- `pnpm lint`
- focused settings API Vitest command
- `pnpm frontend:boundaries`
- `pnpm test`
- `pnpm build`
- `node scripts/validate-static-bundle.mjs static`
- `openspec validate fix-settings-namespace-read-routes --strict`
- `cargo build --locked --release --no-default-features --features server-full`
- native installer plus health/readiness/provider identity comparison and installed Playwright proof on port 1906

COMMANDS TO RUN
/opsx:apply fix-settings-namespace-read-routes

PLAN COMPLETE

# C-14c verification

Run date: 2026-08-08

## Verdict

Implementation verification passes. The legacy admin shell and theme contract are absent, the two retained utilities have feature ownership, direct legacy dependencies are retired, the extended boundary gate is effective, and the production initial graph remains under budget.

## Cheap and focused gates

| Gate | Result |
|---|---|
| `pnpm -C frontend typecheck` | Pass |
| `pnpm -C frontend lint` | Pass |
| `bash scripts/ci-grep-gates.sh` | Pass: boundaries 0, platform adapter clean, Flat 2.0 baseline 376 with 0 new violations, HSL debt 0 |
| `pnpm -C frontend settings:structure` | Pass: 11 modules, page 549/600 lines, 29 settings tabs and 29 accessibility controls |
| Focused Vitest seam | Pass: 6 files, 26 tests covering AG-UI projection, PGlite legacy content, chunk rendering, direct admin routing, A2UI ownership, and MCP ownership |
| `git diff --check` | Pass |

The full frontend suite is intentionally deferred to C-14d, the final C-14 wave verification change, per the phase tier contract. C-14c used the affected cross-seam suite plus production build evidence.

## Browser and production evidence

| Gate | Result |
|---|---|
| `playwright test e2e/runtime-console-visual.spec.ts e2e/settings-decomposition.spec.ts --workers=1` | Pass: 6/6 across desktop and mobile |
| `pnpm -C frontend build:manifest` | Pass: 8,053 modules transformed |
| Production A2UI tester search | Pass: no `a2ui-testing-page` or `A2uiTestingPage` entry among 524 manifest entries; retained receipt records the manifest hash and query |
| `pnpm -C frontend budget:bundle --output .../bundle-budget.json` | Pass: 231,433/250,000 gzip bytes across 10 counted files plus one excluded PGlite file; 18,567 bytes headroom |

The browser harness ran without the backend, so Vite logged expected proxy `ECONNREFUSED` messages. All route, shared-shell, mobile-overlay, command-palette, and decomposed-settings assertions passed.

The production build retains existing PGlite direct-`eval` warnings from `@electric-sql/pglite@0.5.4`; C-14c did not introduce or modify that dependency.

## Retirement and dependency evidence

- `frontend/src/admin/` contains no files.
- Product source has no terminal-theme mutation and no direct import of `@radix-ui/*`, `@tanstack/react-query`, or `highlight.js`. Two route tests mention `data-admin-theme` only to assert its absence.
- `frontend/package.json` has zero direct declarations from the three retired dependency groups.
- `pnpm -C frontend install --frozen-lockfile` passes.
- `pnpm why` attributes retained Radix packages to `cmdk`, `vaul`, `radix-ui`, and `@assistant-ui/react`; the full ownership record is in `dependency-receipt.md`.
- The dependency-manager operation reported removing 41 resolved packages and no manifest dependency was added; the exact historical delta is receipt evidence, while current manifest/lockfile/`pnpm why` checks independently prove the resulting ownership state.

## Architecture and specification evidence

- The production boundary scan reports zero violations.
- Negative fixtures are rejected with ten deterministic component/hook layer rules, the exact §6.3 app/feature/shared/platform matrix, and cross-feature implementation-path/public-entry rules. The cross-feature fixtures include both a direct `ui` implementation and a `ui/index.ts` barrel to prevent unauthorized subdirectory barrels from being treated as public APIs.
- `openspec validate retire-admin-and-legacy-deps --strict` passes.
- `openspec validate frontend-architecture-boundaries --type spec --strict` passes.

## Protected-path evidence

The closeout digest over all eight protected paths matches the entry baseline exactly:

`07e74ad94dc137e9574e411bc99d6f0fcd631879c5a0e52a1b87ca999cf43dc4`

Therefore C-14c did not alter the pre-existing submodule pointer, Rust API changes, or staged license deletions.

## Security boundary

No new trust boundary or security hardening was added. Existing markdown sanitization, backend routes, provider/model contracts, and realtime wire behavior remain outside this structural-retirement change.

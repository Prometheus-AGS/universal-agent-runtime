# Verification: settings-page-decomposition

## Outcome

C-14b replaces the 3,336-line settings UI monolith with 11 production TSX modules organized by navigation, shared primitives, schema rendering, registry composition, and cohesive domain panels. The route-level `settings-page.tsx` is 104 lines; the largest production module is the governance/agents group at 549/600 lines. The feature root still exports only `SettingsPage`.

The deterministic structure gate preserves all 29 ordered navigation keys, all 29 ordered panel-registry keys, the default `provider` selection, the generic-schema registry entries, and the narrow feature-root export. The Flat 2.0 baseline was mechanically remapped from the old monolith to the extracted paths while remaining exactly 384 findings with zero new debt.

## Observed runtime defect and correction

The first real-browser settings smoke exposed an existing React 19 external-store failure: `getDirty(namespace)` created a new empty snapshot on every read when no draft existed, producing “The result of getSnapshot should be cached” followed by a maximum-update-depth crash in `ProviderPanel`. The settings form cache now returns one stable empty snapshot, and its unit test locks identity stability. Initial namespace load rejection is consumed by the hook because the store already retains the actionable panel error state, avoiding the observed unhandled rejection. The corrected focused and browser suites pass.

## Automated evidence

- `pnpm -C frontend settings:structure`: pass; 11 production TSX modules, largest 549/600 lines, 29 navigation and registry keys preserved.
- `pnpm -C frontend typecheck`: pass.
- `pnpm -C frontend lint`: pass with zero warnings/errors.
- `node scripts/check-frontend-boundaries.mjs`: pass with zero production violations.
- `node scripts/check-flat2-style.mjs`: pass at 384 tracked legacy findings and zero new findings.
- `node scripts/check-hsl-token-codemod.mjs`: pass with zero migrated or deferred admin call sites.
- Focused settings verification: 2 files and 8 tests passed after the snapshot correction.
- Full frontend suite: 67 files and 322 tests passed after the correction.
- `pnpm -C frontend build:manifest`: pass; only the existing PGlite direct-eval dependency warnings remain.
- `pnpm -C frontend budget:bundle --output ../openspec/changes/settings-page-decomposition/bundle-budget.json`: pass at 242,520/250,000 decimal gzip bytes across 12 manifest files, leaving 7,480 bytes of margin.
- `openspec validate settings-page-decomposition --strict`: pass.
- `openspec validate frontend-configuration-surfaces --type spec --strict`: pass.
- `git diff --check`: pass.

## UI audit, critique, polish, and responsive evidence

The extraction mechanically moved the component bodies and retained the inspected JSX, classes, visible copy, navigation order, controls, validation, loading/error/saved states, JWT gating, and responsive flex behavior. The retained automated evidence covers the route composition and selected representative panels rather than exhaustively proving every moved panel. Manual audit/critique/polish found no intentional aesthetic or interaction change; the frontend-design consultation correctly constrained this task to the established compact utilitarian settings treatment. UI Pro Max, Impeccable, ux-designer, and named Vercel skills were absent, as recorded in `ui-routing-summary.md`.

The targeted Playwright smoke passes 2/2: desktop 1440×900 preserves provider default and prompt-caching navigation; mobile 390×844 preserves the stacked settings surface and visible navigation. Expected proxy `ECONNREFUSED` diagnostics remain because the frontend-only harness has no backend at 127.0.0.1:1906; after the stable-snapshot fix, those failures render through the settings error state without crashing the page.

## Scope and security receipt

C-14b did not introduce a dependency or change an authentication, untrusted-content, protocol, persistence, provider/model, REST, entity, or backend trust boundary. The JWT gate and error redaction behavior are unchanged. `protected-path-receipt.txt` records the same protected-path status observed at closeout as at entry, but the inherited entry evidence did not retain hashes or a standalone receipt and therefore is not independently reproducible. No staging or commit occurred during C-14b.

## Independent review

The fresh artifact-only review returned **PASS** with no critical findings. Its warnings led to an exact feature-root export assertion, correction of a misleading settings-store retirement comment, and narrower claims here about exhaustive behavior and protected-path proof. It also recorded preserved layering debt: JWT availability is still derived in the user-settings panel, without a direct store/service import or a new C-14b regression.

## C-14c handoff

- Retire the remaining admin shell, `A2uiTestingPage`, `McpHealthPage`, terminal-theme wrapper, eligible dependencies, and truly retired stores without deleting the live `features/settings/model/settings-store.ts` used by `useSettings`.
- Preserve the internal settings modules and `settings:structure` gate; do not widen them through the feature root.
- Before relocating the admin-shell feed subscription, expose `useRuntimeConsoleFeeds` through a narrow runtime model entry instead of the mixed runtime root.
- Rerun the manifest bundle gate after every public-entry contraction because the measured initial margin remains narrow.

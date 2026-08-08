# Isolated Adversarial Review Packet — C-13

Review only this packet and the source paths listed below. Do not use conversation history, prior reviews, or unrelated repository files as evidence. Do not modify files.

## Verdict contract

Return JSON with:

- `verdict`: `PASS` or `FAIL`
- `critical`: defects that invalidate a binding acceptance criterion
- `warnings`: concrete nonblocking defects or material risks
- `suggestions`: optional improvements

Every finding must name a file, make one falsifiable claim, cite direct code/evidence, and propose the smallest correction. Separate observed defects from hypothetical concerns.

## Binding acceptance criteria

1. The production manifest's complete static `/threads` JavaScript closure is at most 250,000 decimal gzip bytes.
2. Exactly one named statically reachable `vendor-pglite` JavaScript chunk is reported/excluded. PGlite WASM/data and the schema seed are reported. Mermaid and Shiki remain dynamic and fail closed if eager or unverifiable.
3. A fresh real Chromium context opens `idb://uar-threads` and the hydrated thread-list commit is marked within 1,000ms of navigation start. The fixture performs no test-only prewarm and cannot pass an unmounted/empty application accidentally.
4. The 500-event trace lane commits within 100ms with fewer than 40 mounted rows, and representative 2,000-line finalized Markdown commits within 250ms.
5. A versioned schema-only seed is semantically reproduced from current migrations, contains exact migration versions, the ordered-definition digest, and zero rows in every product table, is used only when `/pglite/uar-threads` does not exist, and does not alter existing-database resume/migration behavior.
6. Ordinary pull-request CI runs deterministic negative proofs, one production-manifest build, the bundle gate, and all three Chromium fixtures serially. Any regression exits non-zero and evidence is retained.
7. The change does not alter provider/model/backend/protocol contracts, entity-graph synchronization order, C-14-owned removals, `.gitmodules`, `crates/prometheus-skill-system`, `src/uar`, or user-staged license deletions.

## Source packet

- `.github/workflows/ci.yml`
- `frontend/package.json`
- `frontend/performance-budgets.json`
- `frontend/playwright.performance.config.ts`
- `frontend/vite.config.ts`
- `frontend/vite.config.js`
- `frontend/tsconfig.json`
- `frontend/src/App.tsx`
- `frontend/src/app.test.tsx`
- `frontend/src/pages/chat-page.tsx`
- `frontend/src/features/chat/chat-thread-view.tsx`
- `frontend/src/components/layout/left-sidebar.tsx`
- `frontend/src/hooks/use-thread-ui.ts`
- `frontend/src/stores/thread-registry-store.ts`
- `frontend/src/lib/db-context.tsx`
- `frontend/src/platform/pglite/assets.ts`
- `frontend/src/platform/pglite/assets.test.ts`
- `frontend/src/platform/pglite/client.ts`
- `frontend/src/platform/pglite/migrations.ts`
- `frontend/src/platform/pglite/pglite-seed-v3.tar.gz`
- `frontend/e2e/performance-budget.spec.ts`
- `frontend/src/test/performance-budget.ts`
- `frontend/src/test/performance-budget.test.ts`
- `frontend/src/features/chat/ui/run-trace-timeline.stories.tsx`
- `frontend/src/shared/markdown/markdown-bubble.stories.tsx`
- `frontend/src/shared/markdown/markdown-build-contract.test.ts`
- `scripts/build-pglite-seed.mjs`
- `scripts/check-frontend-budgets.mjs`
- `scripts/test-frontend-budget-gate.mjs`
- `scripts/check-markdown-lazy-chunks.mjs`
- `openspec/changes/ci-bundle-and-perf-budget/proposal.md`
- `openspec/changes/ci-bundle-and-perf-budget/design.md`
- `openspec/changes/ci-bundle-and-perf-budget/tasks.md`
- `openspec/changes/ci-bundle-and-perf-budget/.openspec.yaml`
- `openspec/changes/ci-bundle-and-perf-budget/files.txt`
- `openspec/changes/ci-bundle-and-perf-budget/verification-output.txt`
- `openspec/changes/ci-bundle-and-perf-budget/specs/frontend-build-tooling/spec.md`
- `.refiner/artifacts/ci-bundle-and-perf-budget/dist/verification-summary.md`

## Deterministic receipts

- Initial JS: 242,082 / 250,000 gzip bytes across the counted closure.
- PGlite seed: 4,605,535 bytes; migrations 1,2,3; ordered-definition SHA-256 `a4cf692ceb10f55dae41490a46353edb64e98283d3311873d0077e65db24aab7`; actual public-schema catalog exactly matches a fresh migration replay at SHA-256 `1d1e4bd08d2b14a3308bf1028ce01113cff5b9f30b8b31f3d59a6eff568452ac`; zero rows in every product table.
- Cold thread list: consolidated 973.3ms; repeated proof 943.7ms, 921.6ms, 925.6ms.
- Trace lane: 13.3ms / 100ms, timestamped only when the selected virtual row and 500-row structure predicate holds.
- Markdown: 130.2ms / 250ms, timestamped only when the sentinel, heading, link, and list predicates hold.
- Full frontend: 63 files, 317 tests passed.
- Typecheck, lint, architecture boundary, Flat 2.0, negative proofs, production build, bundle gate, strict OpenSpec: passed.

Pay special attention to whether the first hydrated browser-frame mark is described honestly as a frame-boundary proxy for first paint, whether IndexedDB existence detection can seed only a genuinely new database, whether migration/seed drift fails closed, whether the 4.6MB seed is correctly owned in bundle evidence, whether the engine graph must exactly cover the manifest-static JavaScript closure, and whether CI executes the same verified path.

## Raw scope receipts

`openspec/changes/ci-bundle-and-perf-budget/files.txt` is the complete declared C-13 changed-file inventory. It contains no protected path.

`git diff --cached --name-status` after verification:

```text
D  LICENSE-COMMERCIAL.md
D  sdks/rust/LICENSE-AGPL
```

These are the two operator-owned staged deletions received before C-13 and remain staged exactly as received.

Protected-path status after verification:

```text
 M .gitmodules
 M crates/prometheus-skill-system
 M src/uar/api/adapters.rs
 M src/uar/api/openapi.rs
 M src/uar/api/routes.rs
 M src/uar/api/sse.rs
```

The execution handoff identified every entry above as pre-existing/operator-owned and instructed C-13 not to touch them. They do not appear in `files.txt`, and no C-13 operation targeted those paths. Provider, backend, protocol, realtime, and C-14 removal paths are likewise absent from the declared inventory. The raw command/evidence receipt is preserved in `verification-output.txt`.

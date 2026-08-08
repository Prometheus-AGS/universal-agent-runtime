## Context

The production Vite manifest currently reports a 977,298-byte gzip static JavaScript closure from the application entry. The binding migration target is at most 250,000 gzip bytes after reporting and excluding the named PGlite chunk family plus the lazy Mermaid and Shiki engines. The current build only emits a 1,100 kB warning; it does not turn the product budget into a merge-blocking result.

The three latency paths are at different layers. The thread list becomes available only after the real IndexedDB-backed PGlite instance opens and hydrates the thread registry. The run trace already has a deterministic 500-event projection test and a Chromium Storybook interaction that measures the virtualized lane. Finalized Markdown uses the shared `MarkdownBubble` pipeline and currently has no 2,000-line browser measurement. CI already installs the frontend workspace, runs Vitest, and builds the frontend on every pull request, so C-13 extends that job instead of creating a second product build or reopening the existing Chromatic workflow.

This change crosses Vite output, application loading boundaries, browser fixtures, repository scripts, and CI. It must preserve the UI → hooks → stores → services/platform layering and must not alter provider, AG-UI, A2UI, entity-graph, or persistence semantics.

## Goals / Non-Goals

**Goals:**

- Make the 250,000-byte gzip initial-JavaScript limit a deterministic, fail-closed manifest gate.
- Reduce the measured initial closure with intentional loading boundaries while keeping `/threads` and its required runtime code in the counted startup path.
- Prove Mermaid and Shiki remain lazy, and prove the one named, statically reachable PGlite JavaScript chunk plus its WASM/data assets are reported outside the charged application total.
- Enforce the cold-PGlite thread-list, 500-event trace-lane, and 2,000-line finalized-Markdown limits in Chromium.
- Run the bundle and latency gates in the existing ordinary pull-request frontend job and supply negative fixtures that prove threshold violations fail.
- Produce concise machine-readable measurements that can be retained as verification evidence.

**Non-Goals:**

- Reopen or replace Storybook/Chromatic visual regression or reprovision `CHROMATIC_PROJECT_TOKEN`.
- Complete the four explicitly deferred tasks in `docs-storybook-visual-regression-perf-budget`.
- Remove TanStack Query, `src/admin/`, legacy stores, Highlight.js, or Radix declarations; those remain C-14 work.
- Change database schemas, synchronization order, backend routes, provider routing, message/event protocols, or rendered feature behavior. The generated seed contains the existing migrations only.
- Hide arbitrary required startup JavaScript behind an exclusion list; the binding plan's sole PGlite family exception remains named, statically reachable, and reported.
- Add a bundle-analysis or performance-testing dependency when Node, Vite, Vitest, Storybook, and Playwright already provide the required primitives.

## Decisions

### 1. Measure the emitted initial static JavaScript closure

The bundle gate will read Vite's production manifest, require exactly one application entry, and recursively traverse that entry's static `imports`. It will resolve every referenced emitted JavaScript asset beneath the configured build root, gzip the actual bytes with a fixed compression configuration, deduplicate shared files, and compare the summed byte count with the decimal limit of 250,000 bytes. It will emit the entry, visited files, raw bytes, gzip bytes, exclusions, and verdict as stable JSON plus a short human-readable summary.

The gate fails closed for a missing or malformed manifest, an ambiguous entry, an unresolved import, a referenced missing file, a non-JavaScript member in the JavaScript closure, a path escaping the build root, or an over-budget total. Cycles are legal but traversed once. Exactly one statically reachable chunk named `vendor-pglite` is reported and excluded as the binding plan's PGlite-family exception; missing, duplicate, or dynamically hidden PGlite ownership fails. Exactly one PGlite data asset, main WASM, and versioned schema seed are required and reported with raw byte sizes; optional runtime-support assets such as `initdb.wasm` are reported when emitted. Mermaid and Shiki must remain dynamic and absent from the static closure, which is cross-checked against the existing markdown-engine production graph contract. When `--output` is requested, both passing and failing results are retained as machine-readable JSON.

Alternatives considered:

- Vite's chunk warning was rejected because it measures individual uncompressed chunks, not the transitive initial gzip payload.
- A hard-coded filename allowlist was rejected because content hashes and code-splitting output change.
- Browser transfer-size measurements were rejected as the primary bundle gate because cache state, compression negotiation, and server headers make them less reproducible than emitted-byte gzip measurement.
- Subtracting arbitrary JavaScript or an unnamed PGlite match was rejected. The sole PGlite JavaScript exception is tied to the stable Vite group name, must remain statically reachable, and is included in evidence; Mermaid and Shiki still fail if eager.

### 2. Keep the default route honest and split non-startup surfaces

`/threads` is the default destination, so its Chat page, shell, thread hydration, and required static imports remain part of the counted entry closure. The Admin and About route surfaces will move behind named `React.lazy` boundaries with a shared, accessible Suspense fallback. Further feature boundaries may be introduced only when the manifest proves they are not required for first interactive thread rendering and their loading fallback preserves current behavior.

The implementation will iterate from manifest evidence: apply one cohesive loading boundary, rebuild once for cheap feedback, and retain only boundaries that materially reduce the counted closure without shifting code that the default route immediately requires. Vite's existing named vendor grouping remains auditable; the budget is enforced on the closure total rather than on arbitrary per-package chunk sizes.

Alternatives considered:

- Lazily importing the default Chat route was rejected because the browser requests it immediately and excluding it would understate startup cost.
- Broad manual chunk splitting was rejected because moving bytes between static chunks does not reduce the transitive closure.
- Removing dependencies owned by C-14 was rejected because it would violate the ordered migration scope.

### 3. Measure product latency at the real rendering boundary

One shared budget assertion utility will accept a named measurement, observed milliseconds, and limit, then produce a stable result and throw on a non-finite, negative, or over-limit observation. Fast unit fixtures will prove each configured limit and invalid measurement fails without sleeping.

The product measurements are:

- **Thread list, 1,000ms:** a dedicated Playwright performance spec uses a fresh Chromium browser context with empty browser storage, navigates to `/threads`, and measures from navigation start to a browser performance mark emitted after the hydrated thread-list commit. It exercises the production `DbProvider`, the versioned schema-only PGlite seed, `idb://uar-threads`, hydration hook, store, and shell; the fixture does not mock PGlite or perform a test-only prewarm.
- **Run trace, 100ms:** the existing 500-event Chromium Storybook fixture remains the canonical UI measurement. Its click-to-layout-effect timing, virtualization assertions, selected distant row, and mounted-row bound are retained, and the shared assertion records the result. The separate 20ms projection unit test remains diagnostic rather than substituting for the UI limit.
- **Finalized Markdown, 250ms:** a new Chromium Storybook fixture renders one deterministic 2,000-line Markdown document through `MarkdownBubble`, starts timing immediately before the finalized state is committed, and stops in a layout effect after the rendered sentinel is present. The fixture uses representative headings, paragraphs, emphasis, links, and lists but no Mermaid or Shiki block, so it measures the required core finalization path rather than lazy engine startup.

Browser measurements run serially in the supported Chromium environment. Each fixture performs one measured product action and reports the raw observed duration; there is no relaxed CI multiplier. Structural assertions accompany the clock assertion so a fast empty or unmounted surface cannot pass.

Alternatives considered:

- Happy DOM timing was rejected for the three acceptance budgets because it does not execute IndexedDB/PGlite or browser layout faithfully.
- A mocked database was rejected because the requirement explicitly names cold IndexedDB.
- Repeated runs with an averaged threshold were rejected because they would warm the database and could conceal a single over-budget cold path.
- Mermaid or syntax-highlighting blocks were excluded from the Markdown fixture because their separately lazy engine startup is expressly outside the initial budget and would measure a different contract.

### 4. Extend the existing frontend CI job

The existing `frontend` job in `.github/workflows/ci.yml` remains the single ordinary pull-request gate. Its production build will emit a manifest, then run the bundle checker. A targeted performance command will execute the three Chromium fixtures serially. Package scripts expose the same commands locally, and the CI step uploads or prints their JSON result when the runner supports artifact retention.

The bundle check consumes the build that CI already produces; it does not perform a hidden second build. The performance command owns its local preview/dev-server lifecycle through the existing Playwright configuration. Relevant script, fixture, Vite, source, and workflow edits are therefore covered by the workflow's existing all-PR trigger.

Alternatives considered:

- A separate workflow with path filters was rejected because it could silently miss changes outside the filter that affect the entry graph or latency.
- Folding performance checks into the unrestricted full test command was rejected because a named command and result artifact make the acceptance gate independently reproducible and diagnosable.

### 5. Keep negative proofs deterministic and repository-owned

Bundle fixtures will contain small synthetic manifests and emitted assets for an under-budget closure, an over-budget closure, a missing import, a malformed manifest, and an unexpectedly eager lazy engine. They execute the same production checker with temporary roots and explicit limits. Performance assertion tests supply deterministic below, exact-limit, above-limit, non-finite, and negative observations for all three named budgets. CI runs these proof tests before trusting the positive product measurements.

Alternatives considered:

- Deliberately slowing browser tests was rejected as nondeterministic and wasteful.
- Testing a second simplified parser was rejected because fixtures must exercise the production gate implementation.

## Risks / Trade-offs

- [Shared CI runner timing noise causes false failures] → Keep browser fixtures isolated and serial, measure one explicit action, use the binding thresholds without a multiplier, and retain raw results for diagnosis.
- [Code splitting makes the metric look smaller without improving `/threads`] → Keep the default route and its required imports static and counted; allow lazy boundaries only for surfaces not needed on initial thread rendering.
- [Manifest schema or output paths change during a Vite upgrade] → Validate every consumed field and fail with an actionable schema/path error instead of silently producing zero bytes.
- [A forbidden engine becomes eager under a renamed hashed chunk] → Determine ownership from the manifest and existing module-graph evidence, never filename hashes alone.
- [Cold IndexedDB is accidentally warmed by test setup] → Use a fresh browser context and one navigation measurement. The only bootstrap input is the checked production schema seed; the fixture performs no test-only prewarm or storage setup.
- [A fast but incomplete render passes] → Pair every timing assertion with DOM, virtualization, row-count, or sentinel assertions proving the expected content committed.
- [Admin/About lazy loading changes focus or loading semantics] → Use one shared accessible fallback and retain focused route tests for navigation and resolved content.
- [Generated `static/` output obscures review] → Treat it as build evidence; source scripts, config, fixtures, and checked-in production assets remain the reviewable contract according to the repository's existing static-bundle policy.

## Migration Plan

1. Add the reusable budget definitions, assertion utility, manifest checker, and negative fixtures.
2. Emit a production manifest and record the current closure through the new checker.
3. Introduce measured non-default route/feature loading boundaries until the honest `/threads` entry closure is at or below 250,000 gzip bytes.
4. Add the cold-PGlite and finalized-Markdown Chromium fixtures, and connect the existing 500-event trace fixture to the shared result contract.
5. Add local package commands and extend the existing frontend pull-request job to run negative proofs, the manifest gate, and serial Chromium budgets.
6. Run C-13 verification, retain measurement evidence, and transition/archive only after strict OpenSpec and all scoped gates pass.

The production database bootstrap is semantically reproducible: `pglite:seed` compiles the current migration source into an empty versioned data-directory archive, while `pglite:seed:check` requires the exact migration-version set, a SHA-256 digest of the ordered migration definitions, an exact public-schema catalog match against a fresh replay of those migrations (tables, columns, constraints, and indexes), and zero rows in every product table. This is a semantic schema/data guarantee, not a claim that Postgres archive bytes are identical across independent builds. The application uses that archive only when `/pglite/uar-threads` does not yet exist; established databases never receive `loadDataDir` and continue through ordinary migrations. The main PGlite WASM compilation begins concurrently with data/seed loading. The cold-start fixture records the first browser-frame callback after the hydrated commit; this is a frame-boundary proxy for first paint, not a claim that pixels have already been presented, and it avoids charging Playwright polling delay as product rendering time.

Rollback is source-level and reversible: remove the CI steps and budget scripts, restore eager route imports, and remove the added fixtures. No persisted data or backend migration is involved. A rollback reopens the known 977,298-byte and unguarded-latency condition, so it must not be represented as satisfying Goal 12.

## Open Questions

None. The target thresholds, excluded engine classes, default route, supported Chromium environment, CI workflow, and operator credential state are all resolved by the binding plan and current repository state.

# frontend-build-tooling Specification

## Purpose
TBD - created by archiving change migrate-vite-rolldown-codesplitting. Update Purpose after archive.
## Requirements
### Requirement: Vendor Chunk Splitting Uses a Non-Deprecated Rolldown API

The frontend production build SHALL configure vendor chunk splitting via
Rolldown's `build.rolldownOptions.output.codeSplitting.groups` API rather
than the deprecated `manualChunks` function form, and MUST preserve the
same vendor chunk groupings (`vendor-react`, `vendor-assistant`,
`vendor-query`, `vendor-hljs`) that existed before the migration.

#### Scenario: Production build emits the expected vendor chunks

- **Given** `frontend/vite.config.ts` configures chunk splitting via
  `codeSplitting.groups`
- **When** `pnpm run build` runs
- **Then** the output MUST include separate chunks matching
  `vendor-react`, `vendor-assistant`, `vendor-query`, and `vendor-hljs`,
  with the same package-matching logic as the prior `manualChunks`
  function

#### Scenario: Build config uses no deprecated chunk-splitting API

- **Given** `frontend/vite.config.ts` is inspected for chunk-splitting
  configuration
- **When** checking for the deprecated `manualChunks` function form or
  the removed object form
- **Then** neither MUST be present — only `codeSplitting.groups` MUST
  configure vendor chunk grouping

### Requirement: Manifest-Derived Initial JavaScript Budget

The frontend build tooling SHALL derive the initial JavaScript closure from the production Vite manifest by starting at the single application entry and recursively following every static import. It MUST gzip the actual emitted JavaScript files with deterministic settings, count every file once, and fail when the total exceeds 250,000 decimal bytes. JavaScript required to render the default `/threads` route MUST remain in the counted startup closure rather than being excluded through an immediately requested route boundary.

#### Scenario: Initial closure satisfies the budget

- **GIVEN** a production manifest with one application entry and a complete static import graph
- **WHEN** the emitted JavaScript files gzip to 250,000 bytes or fewer in total
- **THEN** the budget gate MUST pass and report the entry, visited files, raw bytes, gzip bytes, limit, and passing verdict

#### Scenario: Initial closure exceeds the budget

- **GIVEN** a valid production manifest and emitted static import graph
- **WHEN** the emitted JavaScript files gzip to more than 250,000 bytes in total
- **THEN** the budget gate MUST fail with a non-zero result and report the observed total and limit

#### Scenario: Required default-route code is accounted for

- **GIVEN** `/threads` is the application's default destination
- **WHEN** the production entry graph is inspected for budget accounting
- **THEN** the shell, thread-list hydration path, and JavaScript required for the first `/threads` render MUST be statically reachable and included in the measured closure

#### Scenario: Manifest evidence is incomplete or unsafe

- **GIVEN** the manifest is missing, malformed, has zero or multiple application entries, references an unknown import or missing file, or resolves a file outside the build root
- **WHEN** the budget gate runs
- **THEN** the gate MUST fail closed with an actionable diagnostic and MUST NOT report a passing zero-byte result

#### Scenario: Shared and cyclic imports are deterministic

- **GIVEN** multiple manifest records reach the same emitted JavaScript file or the graph contains a cycle
- **WHEN** the budget gate traverses the initial closure
- **THEN** each emitted file MUST be counted exactly once and traversal MUST terminate deterministically

### Requirement: Lazy Engine Exclusion Integrity

The initial-JavaScript budget SHALL report and exclude exactly one named, statically reachable `vendor-pglite` JavaScript chunk plus exactly one required PGlite data asset, main WASM, and versioned schema seed with their raw byte sizes, matching the binding plan's PGlite-family exception. Mermaid and Shiki MUST remain named dynamic engine entries outside the initial static closure. The production gate MUST fail when required PGlite ownership is missing, duplicated, type-confused, dynamically hidden, or when engine evidence is missing, malformed, or shows either engine becoming eager. A requested machine-readable output MUST be written for passing and failing evaluations.

#### Scenario: Named PGlite family remains reported outside the charged total

- **GIVEN** the production build emits one statically reachable chunk named `vendor-pglite` plus PGlite WASM or data assets
- **WHEN** the initial JavaScript budget is calculated
- **THEN** the named PGlite JavaScript and WASM/data bytes MUST be reported as exclusions and MUST NOT be added to the charged application total

#### Scenario: Required PGlite asset ownership is incomplete

- **GIVEN** the production manifest omits or duplicates the PGlite data asset, main WASM, or versioned schema seed, or maps one to an unexpected emitted type
- **WHEN** the initial JavaScript budget is calculated
- **THEN** the gate MUST fail and retain a machine-readable failure result when an output path was requested

#### Scenario: PGlite ownership cannot be hidden

- **GIVEN** the named PGlite JavaScript chunk is absent from the static closure, duplicated, or available only through a dynamic boundary
- **WHEN** the initial JavaScript budget is calculated
- **THEN** the gate MUST fail instead of passing an application total that silently omits database ownership

#### Scenario: Mermaid and Shiki remain lazy

- **GIVEN** the production build contains the named Mermaid and Shiki dynamic entries
- **WHEN** the manifest closure and production module graph are checked
- **THEN** neither engine nor its owned modules MUST appear in the initial static closure

#### Scenario: A lazy engine becomes eager or unverifiable

- **GIVEN** Mermaid or Shiki appears in the initial static closure, a named engine entry is absent, or its graph evidence is malformed
- **WHEN** the production bundle gate runs
- **THEN** the gate MUST fail even if the measured gzip total is at or below 250,000 bytes

### Requirement: Cold IndexedDB Thread-List First-Paint Budget

The frontend performance gate SHALL measure the production thread-list path in a fresh Chromium browser context with empty browser storage. The measurement MUST exercise the real `idb://uar-threads` PGlite open from the checked versioned schema seed, hydration store path, and user-visible thread-list region without a mocked database or test-only prewarm step. The browser MUST emit the measured mark at the first browser-frame callback after the hydrated thread-list commit, as a frame-boundary proxy for first paint, and that mark MUST occur within 1,000ms of navigation start.

#### Scenario: Cold thread list paints within budget

- **GIVEN** a fresh Chromium context with no existing UAR IndexedDB or local-storage state
- **WHEN** the browser navigates once to `/threads`
- **THEN** the real hydrated thread-list region or its empty state MUST be visibly committed within 1,000ms of navigation start
- **AND** the fixture MUST assert the expected user-visible region so an empty or unmounted application cannot pass on timing alone

#### Scenario: Cold thread list exceeds its threshold

- **GIVEN** a recorded cold thread-list measurement greater than 1,000ms
- **WHEN** the shared performance assertion evaluates the result
- **THEN** the performance gate MUST fail and identify the observed duration and 1,000ms limit

### Requirement: Five-Hundred-Event Trace-Lane Budget

The frontend performance gate SHALL render a deterministic 500-event run through the production trace-lane component in Chromium. The measured action MUST commit the virtualized lane within 100ms and MUST prove that the expected 500-event tree is represented while fewer than 40 rows are mounted.

#### Scenario: Virtualized trace lane renders within budget

- **GIVEN** a deterministic run projection containing 500 persisted events and a distant selected event
- **WHEN** the Chromium fixture mounts the production trace lane
- **THEN** the selected lane and virtualized tree MUST commit within 100ms
- **AND** the tree MUST represent all 500 events while mounting fewer than 40 tree rows

#### Scenario: Trace-lane rendering exceeds its threshold

- **GIVEN** a recorded trace-lane measurement greater than 100ms
- **WHEN** the shared performance assertion evaluates the result
- **THEN** the performance gate MUST fail and identify the observed duration and 100ms limit

### Requirement: Two-Thousand-Line Markdown Finalization Budget

The frontend performance gate SHALL render a deterministic 2,000-line finalized Markdown document through the production `MarkdownBubble` pipeline in Chromium. The document MUST include representative core Markdown structures without Mermaid or Shiki blocks, and the finalized content MUST commit within 250ms.

#### Scenario: Finalized Markdown commits within budget

- **GIVEN** a deterministic 2,000-line document containing representative headings, paragraphs, emphasis, links, and lists
- **WHEN** the Chromium fixture commits the document through `MarkdownBubble` as finalized content
- **THEN** the rendered sentinel and expected content structure MUST be present within 250ms
- **AND** the fixture MUST prove the complete document path committed rather than timing an empty fallback

#### Scenario: Markdown finalization exceeds its threshold

- **GIVEN** a recorded finalized-Markdown measurement greater than 250ms
- **WHEN** the shared performance assertion evaluates the result
- **THEN** the performance gate MUST fail and identify the observed duration and 250ms limit

### Requirement: Deterministic Negative Budget Proofs

The repository SHALL include deterministic negative fixtures that exercise the same production bundle checker and performance assertion used by CI. The proofs MUST cover under-limit, exact-limit, over-limit, invalid-measurement, malformed-manifest, missing-import, and unexpectedly eager-engine outcomes without intentionally sleeping or slowing a browser.

#### Scenario: Bundle checker negative fixtures block invalid output

- **GIVEN** synthetic emitted assets and manifests representing over-budget, malformed, missing-import, and eager-engine cases
- **WHEN** the production bundle checker evaluates each fixture with an explicit build root and limit
- **THEN** every invalid fixture MUST fail for its expected reason and a valid under-budget control MUST pass

#### Scenario: Performance assertion proves every threshold

- **GIVEN** deterministic observations below, exactly at, and above each configured latency limit
- **WHEN** the shared performance assertion evaluates the observations
- **THEN** below-limit and exact-limit observations MUST pass while above-limit, negative, and non-finite observations MUST fail

#### Scenario: Seed metadata and actual schema diverge

- **GIVEN** a versioned seed whose migration versions and stored migration digest are current but whose actual public tables, columns, constraints, or indexes differ from a fresh migration replay
- **WHEN** the deterministic seed proof runs
- **THEN** the proof MUST fail on the schema-catalog mismatch before the seed can be accepted

### Requirement: Ordinary Pull-Request CI Enforcement

The existing frontend job in the primary CI workflow SHALL enforce the bundle and latency budgets on every pull request targeting `main`. It MUST build the frontend once with a production manifest, run negative proofs, evaluate that same build with the manifest gate, and run all three supported Chromium performance fixtures serially. Any failed budget or proof MUST fail the job.

#### Scenario: Pull-request frontend job runs all budget gates

- **GIVEN** a pull request targets `main`
- **WHEN** the primary frontend CI job reaches its build and performance steps
- **THEN** it MUST emit a production manifest, run the bundle negative proofs, evaluate the built closure, and execute the thread-list, trace-lane, and Markdown Chromium measurements serially

#### Scenario: Budget regression blocks the pull request

- **GIVEN** the manifest closure exceeds 250,000 gzip bytes, lazy-engine integrity fails, a latency result exceeds its limit, or a negative proof does not behave as expected
- **WHEN** the primary frontend CI job runs
- **THEN** the job MUST exit non-zero and expose the named measurement or fixture that failed

#### Scenario: Existing visual-regression workflow remains independent

- **GIVEN** Storybook/Chromatic visual regression and `CHROMATIC_PROJECT_TOKEN` are already configured
- **WHEN** the C-13 budget gates are added to primary CI
- **THEN** the visual-regression workflow and its deferred tasks MUST remain unchanged and MUST NOT be treated as substitutes for the new bundle or latency gates

### Requirement: Root Workspace Lock Matches Every Committed Workspace Manifest

The repository SHALL commit a root dependency lock that is accepted by frozen
installation for every committed workspace manifest, including manifests
reached through workspace-owned submodules. A workspace manifest or submodule
advance MUST NOT be accepted as a certifiable source candidate while the root
lock is stale.

#### Scenario: Frozen installation accepts the committed workspace

- **WHEN** dependency installation runs in a clean checkout with frozen lock enforcement
- **THEN** installation SHALL succeed using the committed root lock
- **AND** the root lock content and digest SHALL remain unchanged

#### Scenario: Workspace submodule manifest advances without lock reconciliation

- **WHEN** a committed workspace submodule manifest no longer matches the committed root lock importer
- **THEN** frozen installation MUST exit non-zero and identify the stale lock
- **AND** source-bound build or certification evidence MUST NOT be minted from a non-frozen resolution

#### Scenario: Lock reconciliation preserves unrelated resolved versions

- **WHEN** the root lock is reconciled only to describe a committed workspace manifest advance
- **THEN** resolved versions unaffected by that manifest change SHALL remain unchanged
- **AND** any dependency upgrade MUST be authorized and verified as a separate dependency change

### Requirement: Every Active pnpm Workspace Root Has a Frozen-Compatible Lock

The repository SHALL commit a lock for every independently active pnpm
workspace root, including a nested workspace used by product build, test, or
certification commands. Each lock MUST match all committed manifests reachable
from its workspace declaration, including manifests supplied by pinned
submodules, and reconciliation MUST preserve resolved versions unrelated to the
manifest changes that made the lock stale.

#### Scenario: Nested workspace accepts frozen installation

- **WHEN** dependency installation runs from an independently active nested pnpm
  workspace in a clean checkout with frozen lock enforcement
- **THEN** installation SHALL succeed using that workspace's committed lock
- **AND** both the nested lock and repository-root lock content and digest SHALL
  remain unchanged

#### Scenario: Nested workspace submodule advances without lock reconciliation

- **WHEN** a manifest reached through a nested workspace's pinned submodule no
  longer matches the nested workspace's committed lock importer
- **THEN** frozen installation MUST exit non-zero and identify the stale lock
- **AND** source-bound build, test, or certification evidence MUST NOT be minted
  from a non-frozen resolution

#### Scenario: Nested lock reconciliation preserves unrelated resolutions

- **WHEN** a nested lock is reconciled only to describe committed manifest or
  submodule-manifest changes
- **THEN** pre-existing package and snapshot resolutions unaffected by those
  changes SHALL remain unchanged
- **AND** any independent dependency upgrade MUST be authorized and verified as
  a separate dependency change

#### Scenario: Every command uses the lock for its actual execution root

- **WHEN** a build, test, or certification command selects a nested pnpm
  workspace as its execution root
- **THEN** lock validation evidence SHALL name and hash that nested lock
- **AND** a successful repository-root frozen install MUST NOT substitute for
  validation of the nested workspace lock

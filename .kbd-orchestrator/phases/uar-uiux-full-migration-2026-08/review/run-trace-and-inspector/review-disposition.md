# C-11 round-one finding disposition

## Corrected

- Roving tree focus: keyboard selection now carries an explicit pending focus target across the controlled projection rerender, maps both virtual and non-virtual rows, and focuses the committed row. A consecutive ArrowDown regression proves focus advances twice.
- Checkpoint inspection: the selected checkpoint now exposes node, iteration, timestamp, state, and messages as deterministic escaped text; resume remains explicitly latest-checkpoint behavior.
- Collapsed-root phase selection: `selectPhase` reopens both the run root and selected phase before reprojecting.
- Returned-run query behavior: row selection and resume handoff share `selectRun`, which writes the existing `?run=` route parameter. A MemoryRouter regression proves `run-2` persists.
- Repeated copy announcement: the live-region text now includes a monotonically increasing sequence so consecutive successful or failed copies cause a DOM text change.

## Disproved by current source

- `useMessage` is imported from `@assistant-ui/react` at `enhanced-thread.tsx:12` and was already used by several components before the two anchor additions.
- `RuntimeRunStepEntity`, `steps`, and `TimelineRow` remain live in the separate cockpit composition at `runtime-console-page.tsx:171,213,248`; C-11 replaced only the runs detail composition.
- `src/uar/a2ui/realtime.rs:52-94` documents and implements exactly four coarse A2UI patch forms rooted at whole surface, components, or data model. Nested JSON-patch paths are deliberately not part of the backend contract.

## Retained contracts

- The tree scroll owner keeps a 256px intrinsic height plus `flex-1`; the flex basis still grows with available space, while the intrinsic baseline prevents the already observed virtual spacer from expanding the viewport to roughly 22,000px.
- Strict response schemas are the C-11 fail-closed trust-boundary behavior and are covered by malformed-response tests. No additive-field backend failure is observed.
- `RunTracePanel` is mounted once for the selected run. The singleton hook has one current consumer; speculative multi-consumer reference counting is outside the observed C-11 problem.

## Corrected evidence

- Six focused files: 35 tests pass.
- Supported Chromium 500-event story: one test passes.
- Typecheck, lint, frontend boundaries, Flat 2.0, strict OpenSpec, scoped diff checks, and artifact-refiner validation pass.
- Full frontend tests and production build remain deferred to the C-12 Wave 4 boundary.

## Round-two disposition

### Corrected

- Local snapshot lifecycle is now an independent `snapshot` action state. Subscription setup reports loading/success/error, catches PGlite initialization failure, renders an alert in the panel, and still starts checkpoint/replay/agent requests after local failure. A focused regression proves the error and continued remote calls.
- A stable selection no longer re-scrolls on each live snapshot append. The effect tracks the last scrolled node and only scrolls for an actual selection/focus transition; a rerender regression proves a same-selection append leaves scroll untouched.
- Checkpoint refresh preserves the operator's selected checkpoint while that id remains present; focused coverage proves it.

### Packet-scope correction

- The repository is an intentionally cumulative dirty worktree containing already accepted C-02 through C-10 changes. The round-two packet showed the cumulative `HEAD` hunk for `frontend/package.json`, so the judge attributed Tailwind, markdown, Chromatic, and earlier dependency work to C-11. C-11 itself added only `"@tanstack/react-virtual": "3.14.9"` at `frontend/package.json:63` plus the matching importer and exact 3.14.9 package resolution at `frontend/pnpm-lock.yaml:121-123,3581-3585,10213-10217`.
- `pnpm -C frontend install --frozen-lockfile` passes with the lockfile up to date. The corrected packet excludes cumulative dependency history and carries this exact manifest/lock evidence instead.
- The same cumulative-history issue caused the MarkdownBubble warning: that renderer migration was completed and accepted in C-08. C-11's `enhanced-thread.tsx` delta is limited to importing `chatMessageAnchorId`, reading the already-imported `useMessage`, and adding stable id/data/tab-index attributes.
- `runtime-console-page.tsx` currently imports both `useCallback` and `useNavigate` at lines 1-2; typecheck passes. The round-two normal `HEAD` hunk omitted unchanged import context.

### Retained

- Conversation focus uses a bounded 60-frame post-navigation search and has a passing persisted-message integration test. No cold-load timeout failure was observed in C-11; a new observer/timeout channel would be speculative outside the accepted contract.

### Corrected evidence

- Six focused files: 38 tests pass.
- Supported Chromium 500-event story: one test passes.
- Frozen frontend install, typecheck, lint, frontend boundaries, Flat 2.0, strict OpenSpec, scoped diff checks, and artifact-refiner validation pass.

## Final review disposition

- Round three passes at 0 critical / 3 warnings / 1 suggestion with a verified-distinct `k3` judge and anti-sycophancy score 0.0.
- Adopted: pending keyboard focus is cleared only after the destination element exists, and virtual-range changes retry the focus. A 500-event End-key regression proves the distant selected row receives focus.
- Adopted: an unknown or newly resumed `?run=` id now renders “Waiting for the selected run to appear” instead of silently falling back to another run; the route-handoff regression covers the state.
- Retained nonblocking: phase-bar segments remain exact run timing even when filters remove their tree node; changing visibility/filters would alter the accepted filter contract. Async unsubscribe remains idempotent and awaited; no teardown rejection is observed.
- Final deterministic evidence: six focused files pass with 39 tests; supported Chromium passes one 500-event mount test; frozen install, typecheck, lint, boundaries, Flat 2.0, strict OpenSpec, and scoped diff checks pass.

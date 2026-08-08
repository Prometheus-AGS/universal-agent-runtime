# C-11 Artifact Refinement Verification

Change: `run-trace-and-inspector`

## Passing evidence

- `pnpm -C frontend typecheck`
- `pnpm -C frontend lint`
- `node scripts/check-frontend-boundaries.mjs` — zero production violations
- `node scripts/check-flat2-style.mjs` — 385 tracked legacy findings, zero new findings
- Six focused C-11 files — 39 tests passed
- Supported Chromium 500-event mount story — one test passed; mounted rows remain viewport-bounded and the story enforces less than 100 milliseconds
- `openspec validate run-trace-and-inspector --strict`
- Scoped `git diff --check`

## Constraint evaluation

- `c11-reactive-trace-contract`: satisfied by live PGlite repository, typed feature API, projection, store/hook, and focused tests.
- `c11-accessible-responsive-surface`: satisfied by the one-tree responsive composition, listbox/tree/tab semantics, keyboard interaction coverage, 44px targets, ember focus, semantic theme tokens, bounded virtualization, and the supported Chromium story.
- `c11-inert-inspector-boundary`: satisfied by deterministic escaped JSON, replay-patch validation through the existing A2UI validator/reducer, executable-content rejection tests, and truthful scoped status feedback.
- `c11-change-gates`: satisfied at the per-change implementation tier. The C-11-owned file inventory excludes backend/provider/protocol/submodule contracts; the two pre-existing staged license deletions remain staged and untouched.

## Isolated review remediation

- Round 1 blocked at 1 critical / 8 warnings / 1 suggestion with a verified-distinct `k3` judge and anti-sycophancy score 0.01785714365541935.
- The critical roving-focus finding is corrected and covered by a controlled consecutive-key regression across committed projection rerenders.
- Actionable warnings are corrected: selected checkpoint inspection is now visible/inert, collapsed-root phase selection reopens its ancestor, and returned-run handoff persists to the route query. Repeat copy announcements now change on every operation.
- Disproved findings: `useMessage` is already imported and used elsewhere; `RuntimeRunStepEntity` and `TimelineRow` remain live in the cockpit page; the Rust A2UI replay bridge explicitly emits only the three supported coarse path shapes.
- Retained intentional contracts: the scroll owner uses a 256px intrinsic baseline plus `flex-1` after a rendered 500-event spacer defect; strict wire schemas fail closed at the existing response trust boundary; the single mounted `RunTracePanel` is the only current hook consumer.
- Round 2 blocked at 2 critical / 4 warnings / 1 suggestion with the same verified-distinct judge and anti-sycophancy score 0.01785714365541935. The real local-subscription blocker and live-scroll/checkpoint findings are corrected. The dependency blocker came from a cumulative `HEAD` hunk spanning prior accepted changes; the exact C-11 manifest/lock pair passes `pnpm -C frontend install --frozen-lockfile`.
- Round 3 passes at 0 critical / 3 warnings / 1 suggestion with verified-distinct `k3` and anti-sycophancy score 0.0. Virtual focus now retries after a distant row mounts, and an unknown/resumed `?run=` id renders an explicit pending state rather than another run. The remaining phase/filter and unsubscribe observations are recorded nonblocking tradeoffs without an observed failure.

## Manual audit, critique, and polish

- Wide/narrow: `RunTracePanel` reflows the same timeline and inspector instances at the `xl` breakpoint; it does not mount a second mobile tree.
- Keyboard/accessibility: phases, filters, tree rows, tabs, checkpoint controls, replay, resume, copy, and conversation handoff retain keyboard paths, accessible names, selected/expanded state, and 44px targets.
- Themes/Flat 2.0: the C-11 surfaces use semantic background, foreground, phase, destructive, and ember tokens shared by light/dark themes. They add no new visible line, shadow, blur, gradient, or outline-variant finding.
- States: empty filters/checkpoints, checkpoint and agent loading/error, replay idle/loading/success/error, resume disabled/loading/error, and live PGlite append behavior have explicit code paths and focused evidence.
- Trust boundary: raw payloads render only as escaped React text; replay patches are validated and reduced to inert A2UI metadata before projection. The polish pass removed the false successful-validation label from idle/loading replay states.

## Tier note

Full frontend Vitest, production build, and Wave 4 aggregate evidence remain intentionally deferred until C-12 completes. They were not run as C-11 implementation feedback.

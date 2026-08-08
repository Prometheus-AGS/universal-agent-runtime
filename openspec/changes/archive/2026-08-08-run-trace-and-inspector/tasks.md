## 1. Reconcile Authority and Resolve the Library Candidate

- [x] 1.1 Re-read the updated KBD execution authority, C-11 plan row, target trace contract, existing `runtime-console` capability, and C-07 persistence implementation before editing application code.
- [x] 1.2 Resolve `cand-010` in favor of exact `@tanstack/react-virtual` 3.14.9 using current package metadata and official headless, measured-row, stable-key, and indexed-scroll documentation.
- [x] 1.3 Complete the available frontend-design and Vercel React performance/composition reviews, the primary-source runtime DevTools research, and the binding UI/UX distillation; record manual fallbacks for unavailable UI/UX Pro Max, Impeccable, and `ux-designer` tooling.
- [x] 1.4 Create and strictly validate the C-11 proposal, design, and `runtime-console` delta specification.

## 2. Establish the Reactive Persistence Boundary

- [x] 2.1 Add exact `@tanstack/react-virtual` 3.14.9 to the frontend dependency and lock manifests without changing unrelated packages.
- [x] 2.2 Enable the installed PGlite `live` extension in `UarDb` and add a typed selected-run snapshot subscription that returns the selected `run` plus ordered `run_event` rows and an async-safe unsubscribe.
- [x] 2.3 Preserve existing one-shot `getRuns`/`getRunEvents` behavior and add focused platform tests proving live initial delivery, append/update delivery, run isolation, and unsubscribe behavior.

## 3. Add Typed Run Trace Contracts

- [x] 3.1 Add kebab-case run trace types for selected-run context, checkpoints, resume responses, replay patch operations, tree nodes, visible rows, event timing, filters, and action/error state.
- [x] 3.2 Add the feature service for PGlite snapshot subscription, `GET /api/uar/runs/{run_id}/checkpoints`, `POST /api/uar/runs/{run_id}/resume`, and `GET /api/uar/runs/{run_id}/a2ui/surface-replay` with encoded path parameters and typed response validation.
- [x] 3.3 Reuse the existing agent catalog service to resolve a complete runtime agent artifact for resume rather than constructing one from partial run metadata.
- [x] 3.4 Add focused service tests for exact URLs, methods, headers, resume body/session context, successful responses, malformed responses, HTTP errors, and independent checkpoint/replay failure.

## 4. Build the Pure Trace and Replay Projection

- [x] 4.1 Implement one-pass `run → phase → event` projection using canonical phase attribution, a `lifecycle` fallback, first-event phase order, stable IDs, and exactly-once event leaves.
- [x] 4.2 Implement event-kind counts/filtering, ancestor retention, expansion flattening, stable selection fallback, sibling position metadata, and event/message lookup maps.
- [x] 4.3 Implement trace segment percentages with a 3 percent visual floor while retaining exact duration/percentage labels.
- [x] 4.4 Implement factual timing derivation for start, preceding gap, explicit duration, and correlated start/end spans; label unmatched instantaneous events without inventing next-event duration.
- [x] 4.5 Implement replay-patch validation and A2UI v0.9.1 envelope reconstruction through the existing `validateA2uiMessage` and `reduceA2uiMessage` trust-boundary path.
- [x] 4.6 Add projection tests for ordering, every-event-once invariants, phase attribution, filters, expansion, selection preservation/fallback, timing pairs, raw payload preservation, replay order, invalid replay rejection, and executable-content rejection.
- [x] 4.7 Add a deterministic 500-event fixture and prove the pure projection completes within its 20 millisecond allocation.

## 5. Own Run Trace State Behind a Narrow Hook

- [x] 5.1 Add the run trace Zustand store with selected context, live snapshot lifecycle, filters, expanded nodes, selected event, checkpoints, replayed surfaces, and independent local/network loading and error states.
- [x] 5.2 Make run switching unsubscribe before resubscribe, preserve stable selection during live appends, and keep the local trace usable when checkpoint or replay requests fail.
- [x] 5.3 Add store actions for phase/event selection, filter toggles, expansion, replay refresh, inert checkpoint inspection, agent-artifact resolution, and latest-checkpoint resume with returned-run handoff.
- [x] 5.4 Add `use-run-trace.ts` selector/action façades so components neither import the store directly nor call services/PGlite.
- [x] 5.5 Add focused store/hook tests for subscription cleanup, stale-run isolation, concurrent endpoint loading, scoped errors, disabled resume prerequisites, successful returned-run handoff, failed resume preservation, and stable render selectors.

## 6. Build and Integrate the Runtime Trace Surface

- [x] 6.1 Add `run-trace-bar.tsx` as a labelled horizontal listbox with canonical phase colors, exact text labels, 3 percent minimum segments, Left/Right/Home/End handling, and phase-to-timeline scrolling.
- [x] 6.2 Add `run-trace-timeline.tsx` with labelled/counting filter toggles, an ARIA tree and roving tab stop, full tree keyboard handling, stable selected-state communication, and non-virtual rendering through 200 visible rows.
- [x] 6.3 Integrate `@tanstack/react-virtual` above 200 visible rows with stable keys, dynamic measurement, bounded overscan, active-row retention, indexed scrolling, and complete `aria-level`/`aria-posinset`/`aria-setsize` metadata.
- [x] 6.4 Add `run-inspector.tsx` with Base UI-backed Payload, Timing, and Raw AG-UI tabs; escaped deterministic JSON; inert checkpoint/replay metadata; explicit copy action; and polite success/failure announcements.
- [x] 6.5 Add `run-trace-panel.tsx` as one responsive semantic composition that reflows registry, trace/timeline, and inspector without duplicating a mobile tree and keeps compact targets at least 44 CSS pixels.
- [x] 6.6 Use Flat 2.0 surface fills, spacing, typography, state icons/text, and the 3 pixel ember focus treatment without adding visible borders, outline variants, shadows, blur, or gradients.
- [x] 6.7 Replace only the existing `RuntimeRunsPage` detail composition with the C-11 feature, preserving graph run selection, route/query behavior, artifacts/tool context, and the later C-14 relocation boundary.
- [x] 6.8 Add stable chat message anchors and an integration callback that selects the persisted thread, navigates to `/threads`, and focuses the matching message for `Open in conversation` without coupling the timeline to router or UI-store implementation.
- [x] 6.9 Add focused interaction/accessibility tests for phase navigation, filters, tree keys, expansion, selection, virtualized ARIA metadata, inspector tabs, escaped payloads, copy announcements, replay status, resume states, responsive semantics, and conversation jump.
- [x] 6.10 Add the supported-browser 500-event mount check proving interactivity within 100 milliseconds and a mounted row count bounded by viewport plus overscan.

## 7. Verify and Close C-11

- [x] 7.1 Pass `pnpm -C frontend typecheck`, `pnpm -C frontend lint`, `node scripts/check-frontend-boundaries.mjs`, and the Flat 2.0 style gate from their required working directories.
- [x] 7.2 Pass only the focused C-11 platform, service, projection, store/hook, component, interaction, accessibility, and performance tests during this change.
- [x] 7.3 Pass strict OpenSpec validation and diff-integrity checks proving C-11 did not alter provider/protocol contracts, backend routes, unrelated dependencies, `.gitmodules`, the Prometheus skill-system submodule, or user-staged license deletions.
- [x] 7.4 Complete the manual audit, critique, and polish fallback across wide/narrow layouts, keyboard-only operation, both themes, loading/empty/error/live states, Flat 2.0 hierarchy, and raw-data trust boundaries; remediate actionable findings.
- [x] 7.5 Run artifact-refiner validation for `run-trace-and-inspector` and remediate every failure before archival.
- [x] 7.6 Run isolated adversarial review against the final C-11 artifact/diff, apply actionable findings, and record anti-sycophancy evidence.
- [x] 7.7 Run OpenSpec verification, transition canonical C-11 to `complete` with `committedLocally: true`, sync the `runtime-console` capability, append the phase waypoint, and archive `run-trace-and-inspector` before starting C-12.
- [x] 7.8 Record that full frontend Vitest, production build, and Wave 4 aggregate evidence remain intentionally deferred until C-12 completes; do not run them as C-11 implementation feedback.

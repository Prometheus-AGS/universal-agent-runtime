## Context

C-07 persists one `run` row plus ordered, normalized `run_event` rows in browser PGlite. Terminal runs include seven phase timings (`context`, `skill`, `memory`, `retrieval`, `reasoning`, `tool`, and `generate`), while each event retains its stable event ID, wire sequence, AG-UI type, normalized kind, timestamp, and raw payload. The current `/admin/runs` surface reads only normalized runtime graph entities, renders an unbounded flat step list, and does not consume the local event record.

The backend already exposes three read/action contracts needed by this change:

- `GET /api/uar/runs/{run_id}/checkpoints` returns ordered persisted checkpoint metadata and state.
- `POST /api/uar/runs/{run_id}/resume` accepts a complete agent artifact plus optional input/session and returns a new run ID and stream URL.
- `GET /api/uar/runs/{run_id}/a2ui/surface-replay` returns ordered state-patch operations for late-join A2UI reconstruction.

The target is a dense runtime instrument rather than a dashboard of cards: a proportional phase lane provides the run silhouette, a navigable event tree provides sequence and causality, and a stable inspector provides detail without losing position. Surface fills, spacing, typography, and the existing ember focus token establish hierarchy; visible borders, gradients, blur, and shadows remain prohibited by the Flat 2.0 gate.

UI/UX routing distillation: the available `frontend-design` guidance supports an intentional industrial/utilitarian direction with one memorable phase lane rather than generic panel chrome. The available Vercel React performance and composition guidance favors stable keyed projections, one-pass map/set derivation, deferred filtering, memoized expensive rows, and a state/actions/meta boundary that keeps UI independent of Zustand and transports. Chrome DevTools' official accessibility guidance reinforces complete keyboard and screen-reader navigation, while its performance guidance favors reducing render work and measuring the actual trace interaction. TanStack Virtual's official API supports headless markup, dynamic measurement, stable keys, overscan, and indexed scrolling. UI/UX Pro Max, Impeccable, and `ux-designer` are not exposed in this session, so their audit/critique/polish roles become explicit manual checks before C-11 verification.

## Goals / Non-Goals

**Goals:**

- Project the C-07 local record into a proportional trace bar, a hierarchical and filterable event timeline, and an offline event inspector.
- Keep a 500-event trace within the plan's 100 ms render budget and virtualize visible tree rows once the projection exceeds 200 rows.
- Preserve live local-first updates without making components poll, fetch, or import PGlite directly.
- Make trace, tree, filters, tabs, replay status, and resume actions fully keyboard-operable and screen-reader legible.
- Consume checkpoint listing, latest-run resume, and A2UI surface replay through typed feature contracts.
- Integrate with the current `RuntimeRunsPage` while leaving its later C-14 relocation behavior-preserving.

**Non-Goals:**

- Changing AG-UI, A2UI, provider, model-routing, or MCP wire contracts.
- Adding or changing backend routes, checkpoint persistence, or the server's checkpoint-restoration semantics.
- Replacing the runtime entity graph or moving the full admin page tree before C-14.
- Rendering untrusted raw payloads as HTML, markdown, scripts, or executable A2UI content.
- Building the C-12 content-block catalog or the C-13 global bundle/performance gate.

## Decisions

### 1. Add a self-contained chat feature slice and preserve strict layering

The implementation will use kebab-case files under the target feature boundary:

```text
frontend/src/features/chat/
  api/run-trace-api.ts
  model/run-trace-projection.ts
  model/run-trace-store.ts
  model/run-trace-types.ts
  model/use-run-trace.ts
  ui/run-inspector.tsx
  ui/run-trace-bar.tsx
  ui/run-trace-panel.tsx
  ui/run-trace-timeline.tsx
```

`run-trace-api.ts` is the feature service and owns PGlite access plus the three HTTP calls. `run-trace-store.ts` owns selected-run context, durable snapshots, filters, expanded nodes, selected event, replay state, checkpoints, loading/error state, and resume actions. `use-run-trace.ts` exposes narrow selector/action façades. UI files render those façades only. The existing runs page passes the selected graph run's ID, agent ID, and thread/session context into `RunTracePanel`; it does not gain transport or persistence logic.

Alternative considered: extend the global runtime-console store. Rejected because its 15-second provider/schema polling lifecycle is unrelated to per-run local persistence, and coupling the trace to that timer would obscure ownership and miss active-stream updates.

### 2. Subscribe to the durable PGlite record instead of maintaining a parallel event cache

The PGlite client will initialize the already-installed `live` extension and expose a platform-level run snapshot subscription. The subscription watches the selected `run` row and its ordered `run_event` rows, emits typed `PersistedRun`/`PersistedRunEvent` data, and returns an async-safe unsubscribe. The feature service adapts that platform contract; the Zustand store holds only the latest projection and ephemeral interaction state.

This preserves PGlite as source of truth and gives the timeline updates immediately after each persisted event without a component timer or a second event ledger. Switching runs unsubscribes the prior live query before attaching the next. Terminal rows remain inspectable entirely offline; checkpoint/resume/replay actions report network unavailability independently.

Alternative considered: poll `getRunEvents()` while a run is active. Rejected because it introduces latency, redundant full-table reads, timer ownership, and manual-refresh behavior despite PGlite's existing reactive query capability.

### 3. Model hierarchy in UAR and virtualize only the visible flattened projection

The event model is deterministic:

1. One synthetic run root owns the selected run.
2. Phase nodes are created in first-event order for the seven known phases plus `lifecycle` for unattributed run events.
3. Every persisted `run_event` appears exactly once beneath its phase. Phase attribution reuses `phaseOfAguiEvent` against the persisted payload; unattributed events use `lifecycle`.
4. Filters are applied to event leaves. A phase remains visible when it has a matching descendant. Expansion then produces one ordered `VisibleTraceRow[]` carrying stable key, depth, parent, position, sibling count, and expansion metadata.

The projection performs one scan over events and uses maps/sets for grouping, filtering, lookup, and selected-row preservation. Tree semantics remain UAR-owned; `@tanstack/react-virtual` 3.14.9 receives only the visible row array. At 200 rows or fewer all rows render normally. Above 200 rows, `useVirtualizer` uses stable event/node keys, a conservative row estimate, dynamic `measureElement`, small overscan, and `scrollToIndex` for keyboard selection.

Alternative considered: a tree-grid or table library. Rejected because no installed table abstraction owns this causal tree, adopting another component system would conflict with D1, and TanStack Virtual's headless adapter solves the measured-list problem without taking markup or state ownership.

### 4. Derive timing once, with explicit semantics

The trace bar reads persisted `phaseTimings`, omits zero-duration phases from the visual lane, and gives every present phase `max(3%, duration / total)` flex weight. If minimums exceed 100%, the browser flex calculation normalizes them while the accessible label continues to report exact milliseconds and percentage. Each segment uses `--color-phase-*`, visible phase text at sufficient width, a title, and an offscreen exact label.

Inspector timing is derived during the same projection pass:

- `start`: the event's persisted `at` timestamp.
- `gap`: milliseconds since the preceding persisted event, or zero for the first event.
- `duration`: an explicit numeric `durationMs`/`duration_ms` when present; otherwise a matched start/end span for message, reasoning, tool-call, step, or run event families using their stable correlation ID; otherwise `null` and displayed as `instant` rather than an invented interval.

Alternative considered: use time until the next event as every event's duration. Rejected because that conflates idle gap with work duration and would present fabricated timing as fact.

### 5. Use one responsive trace composition with complete keyboard semantics

On wide layouts the runs registry, trace/timeline, and inspector occupy three grid areas. On narrower layouts the same nodes reflow into a run selector, trace/timeline, and inspector below; no duplicate mobile tree is created. The event timeline is the primary scroll owner so virtualizer measurements remain stable.

The phase bar is a labelled horizontal `listbox`; Left/Right, Home/End, and activation move between phases and scroll the first matching timeline row into view. Filter chips are toggle buttons with `aria-pressed`, text labels, counts, and 44 px compact targets. The timeline is an ARIA tree with a roving tab stop; Up/Down, Left/Right, Home/End, Enter, and Space navigate, expand/collapse, and inspect visible rows. Virtualized tree items publish `aria-level`, `aria-posinset`, and `aria-setsize` so absent DOM siblings do not erase structural context. Selection uses surface fill, an icon/text state, and the 3 px ember focus treatment rather than color alone.

Activating an event always selects it in the inspector. Message/reasoning events with persisted thread/message identity also expose an explicit `Open in conversation` action. The integration callback selects the thread through the existing UI hook, navigates to `/threads`, and focuses a stable message anchor; it does not place router or store knowledge inside the timeline component.

### 6. Keep inspector data inert and selection stable

`run-inspector.tsx` uses the existing Base UI-backed Tabs wrapper with exactly three panels:

- **Payload**: deterministic pretty JSON plus event summary and, when applicable, validated replayed A2UI surface metadata.
- **Timing**: start, duration, preceding gap, sequence, and wire sequence.
- **Raw AG-UI**: the verbatim persisted `{ type, ...payload }` representation in a copyable `<pre>`.

All payload output is text content produced by `JSON.stringify`; it never enters `innerHTML`, the markdown pipeline, a URL, or dynamic code execution. Copy uses the Clipboard API only on explicit activation and announces success/failure through a polite live region. When live rows update, selection remains keyed by `eventId`; if the selected event disappears, the store selects the nearest surviving visible row and announces the change.

Alternative considered: render payloads through the shared markdown renderer for readability. Rejected because this is a raw protocol inspector and markdown interpretation would change the bytes/operators are trying to diagnose while widening the trust boundary.

### 7. Treat checkpoint resume as a typed server dispatch

Loading a run requests checkpoint metadata in parallel with A2UI replay; a failure in either network read does not hide the offline trace. Resume is enabled only when the selected runtime graph run identifies a runtime agent and the agent catalog provides its complete artifact. The store submits that artifact and the selected run's thread/session ID to `POST /api/uar/runs/{run_id}/resume`, then hands the returned run ID/stream URL to the page integration for selection and navigation.

The UI describes this action as starting a resumed run from the server's latest available checkpoint. It does not deserialize checkpoint `state` or `messages` into client state and does not claim stronger restoration guarantees than the server response provides. Checkpoint state and messages are inspectable as inert JSON.

Alternative considered: reconstruct and post checkpoint state from the browser. Rejected because the route does not accept that contract, it would duplicate runtime policy/state ownership, and it would create a new untrusted execution boundary outside this change.

### 8. Reconstruct A2UI replay through the existing validator and reducer

The replay service validates each response item as `{ op, path, value }`. The model adapter maps the known `/a2ui/surfaces/{id}` path forms back to an A2UI v0.9.1 message envelope, passes it through `validateA2uiMessage`, and reduces valid messages in publish order with `reduceA2uiMessage`. Unknown operations, invalid paths, executable content, and invalid component graphs become an inspector error entry and are never rendered.

The resulting surfaces are stored alongside the trace snapshot and summarized in the inspector. This shares the same validator/reducer as live A2UI rather than creating a second renderer or trusting replay because it originated from persistence.

Alternative considered: display replay operations only as JSON. Rejected because it would technically call the endpoint without delivering late-join surface reconstruction, which is the endpoint's product purpose.

### 9. Verify behavior and budget at the layer that owns each claim

Focused tests will cover:

- service URL/method/body/response contracts for checkpoints, resume, and replay;
- phase attribution, one-event-once tree construction, filtering, expansion, timing pairing, and stable selection;
- replay patch reconstruction through the existing A2UI trust-boundary validator;
- phase-listbox and tree keyboard behavior, virtualized ARIA metadata, tabs, copy announcements, resume states, and conversation jump;
- live PGlite subscription update/unsubscribe behavior;
- a deterministic 500-event fixture whose projection stays below 20 ms and whose mounted virtualized timeline stays below the 100 ms product budget while keeping rendered row count bounded.

Typecheck, lint, frontend boundaries, Flat 2.0, focused tests, strict OpenSpec validation, and diff-integrity are C-11 gates. Full frontend tests and production build remain deferred to the Wave 4 boundary after C-12, per the execution plan.

## Risks / Trade-offs

- **[Risk] Dynamic row measurement can shift scroll position when wrapped summaries expand.** → Estimate the largest normal row, use stable keys and measured elements, keep overscan small, and verify selection after expansion.
- **[Risk] ARIA tree virtualization removes siblings from the DOM.** → Publish explicit level/position/set-size metadata, keep the active row mounted through the virtualizer range, and test the roving-tab-stop contract with keyboard-only navigation.
- **[Risk] Live queries rerun when every event is appended.** → Subscribe only to the selected run, preserve C-07 delta coalescing, perform one-pass projection, and virtualize above 200 rows.
- **[Risk] Checkpoint or replay routes may be unavailable while local history remains valid.** → Separate offline trace state from network-action errors and keep inspection fully usable.
- **[Risk] A runtime run may not identify a loadable agent artifact.** → Keep Resume visibly disabled with a text explanation; never synthesize an artifact from partial run metadata.
- **[Risk] The backend's latest-resume implementation owns restoration semantics not visible to the client.** → Assert only request/response integration in C-11, surface checkpoint metadata, and avoid client claims about internal state fidelity.
- **[Risk] Phase minimum widths can visually overstate many tiny phases.** → Keep exact duration/percentage in text and accessibility labels and document the 3% width as a discoverability floor, not a quantitative scale.
- **[Trade-off] Synthetic phase nodes add rows beyond the persisted event count.** → They provide the required hierarchy and collapse behavior; the budget fixture includes these nodes.

## Migration Plan

1. Add `@tanstack/react-virtual` 3.14.9 and enable the installed PGlite live extension.
2. Add typed run snapshot subscription and feature API contracts without changing the database schema or backend.
3. Add pure projection/timing/replay adapters and their focused tests.
4. Add the feature store and narrow hook façades.
5. Add the trace bar, virtualized timeline, inspector, and responsive composition.
6. Replace only `RuntimeRunsPage`'s current step-detail body with the feature composition, preserving routes and graph-run selection.
7. Run C-11 cheap/focused gates, manual audit/critique/polish, artifact refinement, isolated review, OpenSpec verification, canonical KBD completion, and archive.

Rollback removes the feature composition and dependency and restores the prior `RuntimeRunsPage` body. The C-07 database rows, migrations, backend endpoints, and provider/protocol behavior remain untouched, so no data rollback is required.

## Open Questions

None. Backend checkpoint-restoration fidelity is explicitly outside the C-11 client contract; any change to that semantic requires its own observed backend change and tests rather than an implicit UI workaround.

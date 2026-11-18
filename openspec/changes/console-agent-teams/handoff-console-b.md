# Handoff: console writer B (routing test and run view)

Writer: console B (uar-console). Nothing below was executed: no typecheck, lint, test, build, pnpm, vite or git state change (project verification boundary). Every claim is from reading source and the recorded payloads in `observations/`.

## (a) Rules applied

Vercel React Best Practices, Composition Patterns, Prometheus Entity Management (entity-graph-optimize) and Impeccable Operate plus craft-floor shaped this work. Components render and call hooks only; every business record (members, tasks, attempts, waits, artifacts, peer messages, run summary, routing progress, watch state) comes from `@/platform/entities` hooks, and all I/O is a domain action (`watchTeamRun`, `refreshTeamRunNow`, `startRoutingTest`, `resumeRoutingTest`, `cancelAttempt`, `loadPackageExport`, `loadTeamDefinitions`) called from an effect or an event handler. Subscriptions sit at the smallest rendered boundary (`rerender-derived-state`, `rerender-split-combined-hooks`, `rerender-no-inline-components`): each lane is a `memo` component receiving only primitives; each attempt part (model, usage, time, diagnostic, output, wait, task title) reads its own fields; the attempt card reads only `status` and `runId`. Cross-record views the domain has no hook for (lane order, role holders, per-member messages) are folded by `Collect`, a recursion over components that calls one narrow hook per id, so no hook runs in a loop and no graph data is copied into state. `useState` holds only UI-local mechanics (open disclosures, form entries, clamp and copy state, cancel form); the routing-test progress stays in the graph. No setter runs in a render body; the default role is derived in render, not mirrored by an effect; command ids are created in initializers and handlers. Polling is the domain's `watchTeamRun`, started in one effect and stopped by its cleanup; ingest never moves focus or scroll, and expansion state survives refreshes because every list is keyed by stable ids. Operate mode: familiar stacked rows, labelled `min-h-11` controls, unavailable actions are `aria-disabled` with adjacent visible reason (`GatedAction`), status is icon plus text, one polite `role="status"` per attempt for transitions, motion only for state (spinner, `motion-safe`), and Flat 2.0 (fills and spacing, no borders, shadows, outline variants).

## (b) Files created

All under `/Volumes/T7/Projects/prometheus/universal-agent-runtime/frontend/src/features/teams/ui/run/`:

- `team-run-view.tsx` (entry: `TeamRunView`)
- `refresh-status.tsx`, `routing-test.tsx`, `routing-reservation.tsx`, `routing-progress.tsx`
- `member-lanes.tsx`, `member-lane.tsx`, `attempt-card.tsx`, `attempt-parts.tsx`, `wait-detail.tsx`, `cancel-attempt.tsx`, `task-parts.tsx`, `member-records.tsx`, `output-view.tsx`, `status-label.tsx`, `disclosure.tsx`, `collect.tsx`
- `run-hooks.ts`, `run-model.ts`, `run-model.test.ts` (authored, not run)

Largest file is `run-model.ts` at about 290 lines. No file outside `ui/run/**` was touched.

Imports from writer A's files (they must keep these names): `../../model/use-readiness` `useActionGate`; `../../model/format` `formatCount`, `formatDateTime`, `formatMicrocost`, `formatSeconds`; `../../model/errors` `errorInfoOf`; `../gated-action` `GatedAction`; `../runtime-notice` `RuntimeNotice`.

## (c) `TeamRunView` contract

```tsx
import { TeamRunView } from "./run/team-run-view";
<TeamRunView workspace={workspace} teamId={teamId} />
```

Props are exactly `{ workspace: string; teamId: string }`. It must be rendered inside a React Router context (`Link` to `/admin/runs?run=<runId>` and `/admin/approvals`). It starts `watchTeamRun` on mount and stops it on unmount; writer A must not start a second watch (a second one would only share the ref-counted loop). It needs the team record already in the graph (A mounts it after that) and loads team definitions and the package export itself when their load state is idle. It does not add a page-level scroller. Headings: `h3` Run, `h4` sections, `h5` lane role.

## (d) Domain gaps and mismatches

1. **`TeamsErrorInfo` drops the error body's `diagnostic`.** The observed 409 carries `diagnostic {code, retryable, action}` (for example `TEAM_CAPABILITY_UNSUPPORTED` with `action: "rebind"`). The routing panel can only show `code`, `detail` (the same code string) and message through `RuntimeNotice`; the suggested action is lost. Needs `diagnostic` on `TeamsErrorInfo`. Attempt diagnostics (`TeamAttemptEntity.diagnostic`) are complete and shown.
2. **No derived cross-record hooks.** Lane ordering, role holders (for "no member holds the role") and per-member peer messages are built with the `Collect` fold. Suggested domain hooks: `useMemberIdsByAttention(ws, teamId)`, `useRoleHolders(ws, teamId)`, `useMemberPeerMessageIds(ws, teamId, memberId)`, `useAttemptWaitId(ws, teamId, attemptId)` (I match a yielded attempt's wait by scanning the member's waits).
3. **The watcher does not restart itself when work is admitted after it stopped.** `watchTeamRun` stops when the run is not live; `startRoutingTest` and `resumeRoutingTest` refresh once but do not reschedule. The console calls `refreshTeamRunNow` after each routing submit and each owed-step retry, which kicks the loop. Another admission path (outside this view) would not.
4. **Owed-step actions exist only for tests started from this console.** The progress record is the only place the owed step is stored. A task created elsewhere that is `ready`, unclaimed or claimed but not admitted shows in the lane or the unassigned list as a plain task with no action.
5. **`useTeamField(..., "definition" | "package")` returns an object**, so `RoutingSetup` re-renders when the team record is replaced by a refresh. It is isolated in one small component and passes primitives down; effects depend on id, version and digest strings, not on the object.
6. **Member and team `status` are not used as a signal** (always `inactive`). A lane's state is derived from its attempts. The record status appears only when it is `revoked` or `stopped`.
7. **Lane attention ignores owed routing steps** (the rank comes from the member's open or latest attempt). An unadmitted test is shown inside the lane but does not move it up the order.
8. **Duplicate first request.** Writer A's header calls `refreshTeamRun` once on mount and the watcher's first tick calls it again; both share an in-flight request only if they overlap.
9. The `humanizeToken` display of `diagnostic.action` and `category` only capitalises the token (`change-settings` becomes "Change settings"); I did not invent meaning for tokens. The raw code and state reason remain visible.
10. The cost field is entered in currency units (default `1` for 1,000,000 microunits) and labelled with the team budget currency; the wire value is `round(cost * 1e6)`.

## (e) Implemented without an observed payload

Observed and built against real shapes: failed attempts with and without `diagnostic`, the 409 admit refusal, running attempts, succeeded attempt with string `output`, usage and `pricingIdentity` ("Priced as ..." line shown when it differs from the route), an artifact (`content` string, `id`, `memberId`, `taskId`), empty peer messages and artifacts, empty `waits`.

Built from source reading only (no recorded payload):

- `yielded` with waits: label "Waiting on tasks" while the wait is `yield_requested`, `waiting` or `blocked` (decided by the runtime's open-attempt rule), "Handed off" once the wait is settled; wait state, target tasks with their status, reason code, "Resumed as attempt <id>", Cancel only while `waiting` or `blocked`.
- `cancellation_requested` ("Cancelling", no action) and the cancel flow (reason form, command id reused on retry, runtime detail on refusal).
- `uncertain` (note, effect disposition, no cancel).
- Continuations (a new attempt in the same lane; no special marker).
- Delegated tasks ("Delegated by <role>" from `team_delegate` receipts), "Depends on" and "Blocked by <task>".
- Peer messages (sender or recipient role, mode, text, delivery status).
- Artifacts beyond one string (non-string content renders as formatted JSON).
- `<think>` folding (including an unterminated block), clamp, expand and copy.
- Unadmitted routing tests in a lane, or under a role while no member is assigned; the claim and admit retry; 401, 429 and network error banners in the refresh status.
- Routing blocks: definition budget currency not `USD`, no available member for the role, no task slots, reservation over remaining budget, invalid inputs.

## (f) Unverified

Everything. Specifically:

- Type correctness (no `tsc`): the generic `Collect` props inference when `useValue` takes a context type wider than `RunContext`, the `GatedAction` `type="submit"` and `aria-busy` pass-through, `memo` on named components, `NativeSelect` `id` forwarding, `Link` from `react-router` v8, and the lucide icon names (`Hourglass`, `ArrowRightLeft`, `Ban`, `Clock`, `Circle`, `ShieldAlert`, `Send`, `Copy`, `Check`; `Loader2`, `HelpCircle`, `AlertTriangle` are the aliases already used in the repo).
- Lint: Flat 2.0 `no-restricted-syntax` (grep of the folder shows no `border`, `divide-`, `ring-1`, `shadow-`, blur, gradient, or `variant="outline"` literal), `react-hooks` rules (refs, purity, hooks in `Collect`'s prop-passed `useValue`), `unicorn/filename-case`, `react-refresh/only-export-components` (hooks live in `run-hooks.ts`).
- `run-model.test.ts` (authored, not run).
- Behaviour in a browser: reordering of lanes during a poll (DOM nodes move; focus inside a lane that changes rank could be dropped by the browser), the per-attempt `role="status"` announcements, layout at 320 to 1920 px, contrast of `text-success` and `text-warning`, clamp and expand, copy permission, the cancel form focus.
- Against a running service: the whole routing path needs an instance id and execution ownership (per the design the service cannot bind or admit today). The wait, cancel, uncertain, peer-message and delegation paths have never run against a payload.
- That `refreshTeamRunNow` restarts a stopped watcher after an admit (read from `watch.ts`: `kick` is a no-op only when the watcher has been stopped by its holders).
- A run view for a team with many members (the `Collect` chain nests one component per member; fine at the design's member counts, unmeasured).

# Handoff: console writer A (page, readiness, definition detail, flows)

Writer: console A (uar-console). Nothing below was executed: no typecheck, lint, test, build, pnpm or git state change (project verification boundary). All claims are from reading source.

## (a) Rules applied

Vercel React Best Practices, Composition Patterns, Prometheus Entity Management (entity-graph-optimize) and the Impeccable Operate and craft-floor references shaped this work as follows. Every business record (definitions, packages, exports, bindings, teams, members, run summary) is read only through hooks exported by `@/platform/entities`; no component imports a store, the graph, a transport or the entity-management package, and all I/O is a domain action (`useTeamActions`) called from an effect or an event handler. Subscriptions sit at the smallest rendered boundary: each list row, definition section, budget line, roster item and readiness consumer is its own component reading one field or one derived primitive (`rerender-derived-state`, `rerender-split-combined-hooks`); the page shell reads only the URL. `useState` is used only for UI-local mechanics (open sheet, form entries, preflight result held between Check and Install, file analysis); unsaved binding entries are not a graph entity. No setter is called in a render body; the one reactive write (workspace into the URL) is an effect, and the workspace control resets by remounting on a `key`, not by mirroring props. Refs are touched only in handlers and effects (`rerender-use-ref-transient-values`); command ids are created in handlers, never in render. Lists are keyed by stable ids, the live-first team order renders each id into one of two groups so DOM order equals visual order, and a layout effect hands focus back if a row moves group (no focus or scroll movement on refresh). Composition: small explicit components (`GatedAction`, `RuntimeNotice`, `DiagnosticList`, `ReferenceField`) instead of boolean-mode props. Operate mode: familiar shell (list + detail like Agents), one `Sheet` per task with focus return, labelled `min-h-11` controls, unavailable actions use `aria-disabled` plus visible adjacent reason, state is icon plus text, Flat 2.0 lint contract respected (no borders, no `variant="outline"`, surfaces by fill and spacing), no decorative motion.

## (b) Files

Created under `/Volumes/T7/Projects/prometheus/universal-agent-runtime/frontend/src/features/teams/`:

- `index.ts` (exports `TeamsPage`)
- `model/`: `format.ts`, `errors.ts`, `readiness.ts` (+ `readiness.test.ts`), `use-readiness.ts`, `use-teams-bootstrap.ts`, `teams-location.ts`, `use-definition.ts`, `use-sheet-helpers.ts`, `package-analysis.ts` (+ `package-analysis.test.ts`), `binding-diagnostics.ts`, `binding-draft.ts` (+ `binding-draft.test.ts`), `launch-input.ts` (+ `launch-input.test.ts`)
- `ui/`: `teams-page.tsx`, `workspace-control.tsx`, `readiness-strip.tsx`, `teams-list.tsx`, `teams-welcome.tsx`, `definition-detail.tsx`, `team-detail.tsx`, `install-package-sheet.tsx`, `binding-sheet.tsx`, `binding-model-step.tsx`, `binding-reference-fields.tsx`, `launch-sheet.tsx`, `gated-action.tsx`, `runtime-notice.tsx`

All four test files are authored, not run. `ui/run/**` was not touched.

Changed (shell wiring):

- `frontend/src/app/shell/nav-destinations.ts`: `"teams"` id, `Users` icon, Teams destination in the Work group after Agents (not a compact tab).
- `frontend/src/pages/admin-page.tsx`: import and `teams: () => <TeamsPage />` in `PAGE_MAP`.
- `frontend/src/app/shell/nav-destinations.test.ts`: expectations updated for the new destination (work list, `/admin/teams` match, not a Configure path). Not run.

Interface effects: new route `/admin/teams` (query `workspace`, `team`, `definition`); localStorage key `uar.teams.workspace` (prefill only); no change to `platform/**`.

## (c) `TeamRunView` contract assumed

`team-detail.tsx` renders, after the header, roster and budget sections:

```tsx
import { TeamRunView } from "./run/team-run-view";
<TeamRunView workspace={workspace} teamId={teamId} />
```

Named export `TeamRunView`, props exactly `{ workspace: string; teamId: string }`, mounted only after the team record is in the graph, keyed by `workspace:teamId` (remounts when either changes), inside a padded `space-y-6` column that already scrolls (do not add another page-level scroller). The header calls `refreshTeamRun(workspace, teamId)` once on mount (shared with the watcher's own call if concurrent). The run view must start and stop `watchTeamRun` itself; the header does not.

Shared helpers writer B may reuse (all under `features/teams/model`): `use-readiness.ts` `useActionGate(workspace, "routing")` returns the reason string or null (routing needs identity, workspace, instance id and execution ownership); `ui/gated-action.tsx` (`GatedAction`, aria-disabled button with adjacent reason), `ui/runtime-notice.tsx` (`RuntimeNotice` shows the runtime's code and detail verbatim), `model/errors.ts` (`errorInfoOf`), `model/format.ts` (`formatMicrocost`, `formatSeconds`, `formatDateTime`). Heading ids used for focus return: `teams-detail-heading` (focus fallback after a sheet closes).

## (d) Domain gaps and mismatches

1. No hook returns teams ordered by activity. I render each team id into a "working" group and a "rest" group (each row renders only if `useTeamIsLive` matches), which gives live-first with DOM order equal to visual order, at the cost of 2N cheap subscriptions and a focus-handoff layout effect. A domain hook such as `useTeamIdsByActivity(workspace)` would replace this.
2. `TeamMemberSpec` and the export model carry no `responsibility`, so members show role, kind, min to max and agent definition id only.
3. Preflight diagnostics have no file, so install diagnostics are grouped by disposition with blocking first, not by file.
4. Binding diagnostics: `/models/N` goes to a group above step one; `/modelBindings/N/...` is shown beside requirement N (the console builds `modelBindings` in requirement order, so N is that row); `/modelBindings/N/credentialRef`, `/storage/connectionRef`, `/policyRevision`, `/effectiveLimits`, `/effectiveBudget` go beside their fields; everything else is listed first as general.
5. `credentialRef` exists per `modelBindings` entry, but design Decision 8 lists one credential reference field. I apply the single value to every model binding.
6. `teamExecutionAvailable === false` (from capabilities) is treated as a blocked execution-ownership fact, worded "The runtime reports team execution as unavailable." This is my reading of a documented field, not something the design states.
7. `resetTeamsAccess()` is not used: the spec says no further team requests until reload, so the 401 state offers "Reload after signing in" (`window.location.reload()`), and refresh is disabled while identity is not ok.
8. Binding creation always makes a new binding (new id, revision 1). Editing or re-versioning an existing binding is not offered; `buildDeploymentBinding` supports it if wanted.
9. Workspace values are validated as visible ASCII without spaces because the value is sent as an HTTP header (a non-Latin-1 value would make `fetch` throw). Not a runtime rule; a local boundary check.
10. The domain's load key `executions:<workspace>` (used by `loadTeamExecutions`) is not in `teamLoadKeys`; I do not read it.

## (e) Unresolved decisions

- Activity prefetch: the list calls `loadTeamExecutions` for the newest 20 teams on list load, on id change and on Refresh (one request per team, concurrency 2, shared 10 req/s bucket). `LIST_ACTIVITY_PREFETCH_LIMIT` in `model/use-teams-bootstrap.ts` is the knob. A workspace with more than 20 teams shows older rows as "Activity not loaded".
- Launch gate follows the design table (identity, workspace, instance id; not ownership) plus "at least one binding". If launch should also require ownership, change `REQUIRED.launch` in `model/readiness.ts`.
- Mobile reachability: Teams is a Work destination but not a compact tab; I did not check how non-compact destinations are reached on narrow screens.
- Install matches manifest paths that include folders to chosen files by unambiguous file name (a browser multi-select has no folder structure). Several files with one name are reported missing.
- Install selects the first manifest file of kind `TeamDefinition` afterwards (falls back to the first preflight definition). A package with several team definitions selects the first.
- Quality gate (two isolated Impeccable critiques, adversarial review) is not run; it is the lead's step 5.1.

## (f) Unverified

Everything. Specifically:

- Type correctness of every file (no `tsc`): in particular `GatedAction`'s props derived from the base-ui `Button`, the base-ui `Input type="file"` change event typing, `RefObject<string | null>` mutation in `teams-list.tsx`, spread-with-computed-key casts in `binding-sheet.tsx`, and the platform hook return types I relied on (`useConfiguredModels().items`, `useTeamDefinitionField(..., "members")`).
- Lint: Flat 2.0 `no-restricted-syntax` (I avoided the word "border", `ring-1`, `shadow-*`, gradients and `variant="outline"` in literals), `react-hooks` rules (refs, purity, static components), `unicorn/filename-case`, `react-refresh/only-export-components`.
- Behaviour in a browser: focus return after each sheet (two animation frames), the focus handoff when a team row changes group, base-ui `Sheet` and `AlertDialog` stacking (install confirm opens over the sheet), `<details>` toggling in the binding sheet, layout at 320 to 1920 px, contrast of `text-success` and `text-warning`, reduced motion.
- The four authored tests, `digest` availability under the test environment (`byteDigest` needs Web Crypto), and `nav-destinations.test.ts` plus any other test that counts Work destinations.
- Binding preflight and install, team launch and every ownership-dependent path against a running service (per the design, the service today cannot bind or launch). Install, readiness and empty states are the only paths that can run today.
- That `refreshTeamRun` on mount and writer B's watcher coexist without a duplicate-request or ordering problem.
- Wording of the explanations for the three references is mine, not taken from the runtime.

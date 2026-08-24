# ANALYSIS: fix-broken-session-configuration-ui

Project: Universal Agent Runtime, with a controlled upstream Prometheus Entity Management change
Date: 2026-08-23
Mode: Stack specified (React 19, Zustand 5, Prometheus Entity Management, Playwright)
Research package: `.prometheus/research/fix-broken-session-configuration-ui-entity-state-architecture.research/`

## Outcome

The installed Session Configuration freeze is caused by an unbounded external-store publication path, not by React reconciliation in the abstract. Opening the sheet mounts `ModelSelector`; its store downloads the 2,611,291-byte `/api/models` catalog (316 providers, 7,248 models) and calls `graph.upsertEntity` once per model. Chrome becomes unresponsive while the installed UAR service remains healthy.

UAR is bypassing Prometheus Entity Management's intended architecture:

- it registers no entity transports;
- feature hooks use raw graph selectors and REST calls;
- `useModelsStore` and `useChatSessionConfigStore` duplicate graph-owned state in separate Zustand stores;
- the existing `AgentSessionEntity` is not the authority for the panel;
- feature code mutates the graph row by row.

Prometheus Entity Management 3.0.2 is not itself a zero-`setState`, zero-rerender system. Its exact source uses React state for CRUD edit buffers and mutation lifecycle, and its React bindings subscribe to Zustand. React and Zustand rerender a subscriber when its selected snapshot changes. The durable objective is one explicit state authority, narrow stable selectors, and a constant number of store publications—not an impossible ban on every rerender.

The research found a confirmed upstream 3.0.2 defect. `useEntities` performs one `upsertEntities`, then one `setEntityFetched` write per row, then one `setListResult`: N+2 success publications. The same pattern exists in the core list engine, entity query, legacy view, and Electric adapter. `createGraphTransaction` does not batch publications. The defect remains in both signed tag `v3.0.2` (`f29a7016`) and latest `origin/main` (`e2521001`). The upstream repair is required work, but it is not a blocker for the panel fix: the configured-provider boundary reduces this panel to twelve rows. UAR can move immediately to published 3.0.2 and the bounded path while the upstream repair proceeds in parallel.

The panel is also a known dead facade. It sends `model_override` and `context_strategy`; backend `AgentSessionConfig` accepts `model` and no session `context_strategy`. The frontend never loads persisted session configuration, and the turn builder uses the selected agent's model. A visual-only freeze fix would preserve non-functional controls.

## Corrections to Assess

- The current installed `/api/uar/providers` response contains five configured providers and twelve configured models, not the earlier static estimate of four providers and 79 models.
- Server work is required to the extent necessary to establish one typed session contract and ensure saved session policy affects effective inference. Unsupported context controls must either receive a real backend contract or be removed; ignored JSON is not acceptable.
- A keyed React form with component-local business state is no longer the recommended architecture. The project requires business state to remain explicit and inspectable.
- The current UAR product resolves the checked-out `3.0.0-rc.1` workspace package. It must move to exact published 3.0.2 immediately; the upstream patch is a parallel correction, not a reason to delay that goal.

## Gap-to-decision table

| Gap | Decision | Limit |
|---|---|---|
| Published package baseline | Move UAR from the `3.0.0-rc.1` workspace link to exact published 3.0.2 and reconcile both lockfiles | This satisfies the original dependency goal; the configured-data bound, not 3.0.2 alone, fixes the panel |
| Upstream list ingestion | Add one atomic fetched-list graph action and migrate every list-fetch path that currently loops `setEntityFetched` | Preserve merge, lifecycle, sync, list, replace, append, stale, error, and pagination semantics |
| UAR entity architecture | Register transports at application boot and expose domain hooks through `frontend/src/platform/entities` | Feature components do not call services or raw graph mutation APIs |
| Model selector | Normalize only `/api/uar/providers` configured models | Opening the sheet must not request `/api/models` |
| Committed session state | Use canonical `AgentSession` keyed by thread/session ID and load it through GET | One typed frontend/backend field contract |
| Unsaved form state | Use a separate `AgentSessionDraft` graph entity keyed by session plus editor identity | Do not expose drafts through canonical shared patches; cancel cannot mutate committed state |
| Functional session behavior | Persist `model`, reload it, and prove it controls effective inference routing | Unsupported context controls are implemented fully or removed, never silently ignored |
| Sheet spacing | Resolve the established sheet/container spacing token during the mandatory design-system audit, then apply it to body/control margins and padding at compact and desktop sizes | Do not invent a pixel threshold during Analyze |
| React recurrence prevention | Require Vercel React and Entity Management skills/instructions; add scoped architecture/static gates for render-body setters, raw feature-store mutation, duplicate entity caches, and mutation loops | Do not prohibit UI-only local state or necessary field rerenders |
| Functional regression | Add one short local installed-browser scenario with browser console/network, responsiveness, interaction, effective-inference, computed-style, and server-log evidence | No GitHub Actions product test and no soak |

## Required execution order

Two tracks begin after Plan. They share no files and may run in parallel. The UAR
track is the user-visible critical path; the upstream track must not delay it.

### Track A1 — UAR 3.0.2 adoption and architecture migration

1. Inventory package declarations. The only UAR product declaration is `frontend/package.json`; `frontend/packages/prometheus-entity-management/**` belongs to the submodule's own workspace and is not a second product declaration. Reconcile both root `pnpm-lock.yaml` and `frontend/pnpm-lock.yaml`.
2. Replace `workspace:*`/`3.0.0-rc.1` product resolution with exact published Entity Management/Core 3.0.2. Record the lockfile transition explicitly.
3. Export transport and domain-hook APIs only through `frontend/src/platform/entities`.
4. Register configured Provider, Model, AgentSession, and AgentSessionDraft contracts at the application boundary.
5. Retire the panel's dependence on REST-wrapper Zustand stores when those records are graph-owned.

### Track A2 — functional Session Configuration repair

1. Load canonical `AgentSession` state when the sheet opens.
2. Initialize a distinct `AgentSessionDraft` graph entity. Each field subscribes only to its selected draft value; the panel shell does not subscribe to the entire draft.
3. Derive model options from atomically ingested configured providers/models.
4. Save the backend's typed field names, replace canonical state once, and remove the draft. Cancel removes only the draft.
5. Ensure policy resolution and the inference request use the saved session model.
6. Implement context-strategy persistence end to end or remove the unsupported controls from this change. Do not preserve a decorative control.
7. Apply the design-system-resolved body/control spacing token across supported viewports.

### Track A3 — prevention and bounded functional proof

1. Add project instructions requiring Vercel React Best Practices, Composition Patterns, and the relevant Prometheus Entity Management skill before React/entity work.
2. Add scoped enforcement that detects synchronous render-body state setters, per-row graph mutation loops in feature code, and direct feature imports that bypass the platform/domain-hook boundary.
3. After code completion, run one short local installed-service scenario. It must open and remain interactive within two seconds, issue no `/api/models` request, show configured models, save/reopen the session, prove the saved model routes a real inference turn, prove cancel isolation, verify resolved spacing at compact and desktop widths, and record browser console/network plus UAR service logs.

### Track B — upstream Entity Management repair

Work in an isolated worktree from updated `origin/main`; the existing upstream
checkout is dirty and midway through another KBD phase.

1. Create an upstream OpenSpec change for atomic fetched-list ingestion.
2. Add a core action that merges fetched rows, marks all entity/sync lifecycle records fetched with one timestamp, and applies replace/append list metadata in one Zustand `set`.
3. Migrate `useEntities`, core `fetchList`, `useEntityQuery`, legacy `useEntityView`, and list-like adapters that currently perform per-row lifecycle writes.
4. Add a notification-count regression using 7,248 rows. After the existing fetch-start notification, successful ingestion must publish once regardless of row count. The negative control against 3.0.2 must observe N+2 success publications.
5. Verify data, lifecycle, list metadata, merge strategy, replace, append, and error semantics.
6. Version the affected core and React packages for the next patch release, commit, push, and open the upstream PR. Do not edit or reset the dirty checkout. If repository state prevents a safe code change, create the GitHub issue with the exact reproduction instead of patching UAR around it.

Track convergence: if the corrected package is published before UAR closes, update
UAR from exact 3.0.2 to that release and rerun the same bounded functional proof. If
publication is not available, UAR may close on exact 3.0.2 because the panel ingests
only twelve configured models; the upstream PR/issue remains an explicit deliverable,
not a hidden blocker.

## Candidate conclusions

### Adopt

- Exact published Prometheus Entity Management/Core 3.0.2 as the immediate replacement for the current workspace `3.0.0-rc.1` product resolution.
- The corrected patch pair when it becomes published; adoption may converge before phase close but does not block writing the UAR fix.
- Existing Playwright 1.62.1 for the bounded installed-browser scenario.
- Existing GET/POST session policy endpoints after their JSON contract is made explicit and matched by the frontend.

### Adapt

- Entity Management registered transports and domain hooks behind UAR's platform boundary.
- A distinct graph-owned `AgentSessionDraft` entity for inspectable unsaved business state.
- Zustand stable narrow selectors so only a changed field subscriber rerenders.
- Vercel React Best Practices and Composition Patterns as mandatory pre-code references, backed by executable boundaries.

### Reference only

- Entity Management 3.0.2 as the reproduced baseline and negative control.
- React's local-state guidance: it explains why local drafts can be valid generally but does not override this project's explicit-business-state rule.
- Official React lint. It permits convergent guarded render updates and cannot enforce UAR's stricter boundary alone.

### Reject

- Using `useEntities` 3.0.2 unchanged.
- Treating `createGraphTransaction` as a batching primitive.
- Shared canonical patches for unsaved form values.
- A blanket ban on all React state or rerenders.
- Direct DOM mutation to avoid React reconciliation.
- A new cache/query library.
- A UAR-private fork or copied implementation of the upstream graph action.
- A spacing-only or freeze-only correction that leaves the dead facade.

## Evidence

- Exact NPM tarballs: Entity Management 3.0.2 SHA-256 `3c3160c1...c51ec202`; Core 3.0.2 SHA-256 `1ea1988f...c69d444d`.
- Signed upstream source: `v3.0.2` at `f29a7016`; updated `origin/main` at `e2521001`; both contain the per-row `setEntityFetched` loop.
- Independent direct verification commands were run after the research daemon failed: `npm view` supplied the exact tarball URLs; `shasum -a 256` produced the recorded hashes; `git fetch origin --prune`, `git show v3.0.2:.../use-entities.ts`, and `git show origin/main:.../use-entities.ts` confirmed the same loop. These claims do not depend on daemon output.
- UAR source: `b8c4fde214e250dc39080330ee4c130d102c78f7`.
- Installed service: `/healthz` HTTP 200; `/api/models` 2,611,291 bytes/7,248 models; `/api/uar/providers` 3,949 bytes/five providers/twelve models.
- Server logs: the installed service continued normal operational polling during the browser freeze and showed no matching request failure. Post-fix evidence must capture the relevant log interval rather than infer server health from process existence.
- Source contract mismatch: `session-config-panel.tsx` sends `model_override` and `context_strategy`; `AgentSessionConfig` accepts `model` and no context strategy; `use-chat-runtime.ts` sends `agentConfig.model`.
- Research daemon: two stage-0 failures with zero sources/tokens, including after its documented restart; disk fallback used and recorded in the research manifest.

## Uncomfortable finding

The dead session override was already documented in an archived BDD design, and the test phase deliberately proved agent switching instead of the panel's claimed session override. The UI was allowed to continue presenting controls whose save path did not affect inference. The correction must prove task completion, not merely that the sheet renders and closes.

## Unresolved review findings after the bounded two-round vet

The second isolated review found two critical planning defects. Both are corrected
in this revision, but the review skill permits only two artifact-review rounds, so
the next stage must retain the findings explicitly:

1. **Original Goal 1 had been silently superseded.** Corrected by preserving an
   immediate, explicit `3.0.0-rc.1` workspace to exact published 3.0.2 migration
   and recording the additional upstream repair as an Analyze-stage goal amendment.
2. **The upstream release had been made an unjustified hard blocker.** Corrected by
   running the UAR and upstream tracks in parallel. Twelve configured models cannot
   reproduce the 7,248-row freeze path, so UAR implementation proceeds immediately.

The review also warned that the backend dead-facade work expanded the original UI
scope and that daemon failure weakened provenance. The goal amendment now names the
functional session contract explicitly, and the tarball hashes/tag/source loop were
independently reverified with direct registry and Git commands.

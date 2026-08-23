# Entity-state architecture for Session Configuration

## Executive finding

UAR is not using Prometheus Entity Management as designed. Feature code bypasses registered transports and domain hooks, duplicates graph-owned data in ad-hoc Zustand stores, and hydrates a 7,248-model catalog through 7,248 imperative graph writes. That is the direct cause of the observed browser freeze.

The stronger claim that any `setState` or rerender is inherently wrong is contradicted by the exact 3.0.2 package source. Entity Management uses React state for isolated CRUD edit buffers and Zustand subscriptions for reactive views. React and Zustand rerender a subscriber when its selected snapshot changes. The correct target is not zero rerenders; it is one state authority, narrow stable selectors, bounded publications, and rerenders only in components whose selected value changed.

The investigation also found two defects that make a narrow visual fix unacceptable:

1. Entity Management 3.0.2 has an upstream list-ingestion defect. `useEntities` calls `upsertEntities` once, then calls `setEntityFetched` once per row, then writes the list result. A 7,248-row response produces 7,250 publications. The same pattern exists in the core list engine, entity-query path, legacy view path, and Electric query adapter. The defect is present in the signed `v3.0.2` tag and latest `origin/main`.
2. Session Configuration is a dead facade. The panel sends `model_override` and `context_strategy`; the backend accepts `model` and has no session `context_strategy` field. The frontend does not load the saved session config, and the turn builder uses the selected agent's model. A user can click Save without changing effective inference behavior.

## Evidence

The installed service returned:

- `/healthz`: HTTP 200;
- `/api/models`: 2,611,291 bytes, 316 providers, 7,248 models;
- `/api/uar/providers`: 3,949 bytes, five providers, twelve configured models.

The browser became unresponsive when the sheet mounted. The service remained healthy and its operational log showed its normal background activity rather than a matching server request failure. Repository tracing connects the sheet to `ModelSelector`, `useModelsStore.load`, the full catalog request, and one `upsertEntity` call for every catalog model.

The exact published package evidence is stronger than README intent:

- `useEntityCRUD` uses React state for edit/create buffers and mutation status.
- `useEntityAugment` documents graph patches as shared overlays, not isolated form drafts.
- `useEntities` has one batch data write followed by N lifecycle writes.
- `createGraphTransaction` snapshots for rollback but invokes store actions immediately; it is not a publication batch.

## Architecture decision

### 1. Fix Entity Management upstream first

Create an isolated upstream change from current `origin/main`, not from the dirty local checkout. Add a core graph action that ingests a fetched list in one Zustand `set` operation:

- merge all canonical entity rows using the registered merge strategy;
- mark all row lifecycle and sync metadata fetched using one timestamp;
- replace or append list IDs and pagination metadata in the same mutation;
- preserve existing merge, deduplication, stale, error, and pagination semantics.

Migrate every list-fetch path that currently performs `upsertEntities` plus a `setEntityFetched` loop. Add an observed notification-count regression: ingesting 7,248 rows must produce one success publication, independent of row count, after the existing fetching-start publication. Verify entity data, entity lifecycle metadata, list metadata, replace mode, append mode, and merge-strategy behavior. Version and publish the corrected core and React packages before UAR consumes them. If publishing authority is unavailable at Execute, file a GitHub issue with the failing notification-count reproduction and stop UAR adoption before pretending 3.0.2 is sufficient.

### 2. Make UAR a real Entity Management consumer

Register application-owned transports at boot and expose domain hooks through `frontend/src/platform/entities`. Feature components must not call services or raw graph mutation APIs. Retire the Session Configuration dependencies on `useModelsStore` and `useChatSessionConfigStore` where the graph owns the same records.

The model selector uses the configured-provider inventory, not the complete catalog. The configured inventory must be normalized into canonical Provider and Model entities through the corrected atomic ingestion path. This bounds both transfer size and graph work.

### 3. Keep committed and unsaved session state distinct

Use the existing `AgentSession` entity as committed server-confirmed state, keyed by thread/session ID. Load it through the GET session-config transport. Create a separate `AgentSessionDraft` entity keyed by session ID plus editor identity when the sheet opens. Controls select only their own draft field. Save reads the draft imperatively inside the domain mutation, sends the typed backend contract, replaces the canonical `AgentSession` once, and removes the draft. Cancel removes the draft without touching committed state.

Do not use the canonical entity's shared `patches` entry for unsaved form input: package documentation says those patches are visible to every subscriber. Do not keep session business configuration in component-local React state: the project requires business state to remain explicit and inspectable. UI-only state such as focus or popover openness may remain local.

### 4. Repair the functional contract

Choose one typed session configuration contract and make both sides match. For the existing backend, the model field is `model`, not `model_override`. Either add a real session-level context-strategy contract and route it through policy resolution, or remove the unsupported controls from this change; silently sending ignored fields is prohibited. The frontend must load persisted session state, and effective inference must consume the saved conversation policy.

## Alternatives rejected

- **Keep the full catalog but memoize options.** Rejected because memoization happens after transfer and graph mutation; it does not remove the publication storm.
- **Use `useEntities` 3.0.2 unchanged.** Rejected because its lifecycle loop remains O(N) publications.
- **Use `createGraphTransaction` as a batch.** Rejected because it supplies rollback, not deferred publication.
- **Store unsaved values in canonical patches.** Rejected because other subscribers would observe uncommitted values.
- **Ban all rerenders or mutate DOM directly.** Rejected because controlled UI must reconcile changed values; direct mutation would hide state from the graph and break React's ownership model.
- **Apply only padding and a keyed remount.** Rejected because the panel would remain functionally dishonest and the upstream list defect would remain available to recur elsewhere.

## Risks and uncomfortable findings

- The upstream repository's working checkout is heavily dirty and on a different KBD phase. Editing it in place risks mixing unrelated work. The fix must use an isolated worktree from the updated remote.
- A generic atomic ingestion API can become too broad. Its first contract should cover the existing fetched-list semantics only; do not turn it into an arbitrary graph transaction system.
- Moving every control to one draft selector can still rerender the whole panel. Split field subscriptions or field components so a changed field does not invalidate unrelated controls.
- The current archived BDD design knowingly documented the session override as a dead facade. Prior testing proved agent switching instead of the panel's claimed behavior. This was not an unknown regression; it was carried forward as accepted debt and then presented in the UI as functional.
- The current browser scenario can pass while Chrome is frozen because it checks only that the body remains visible. The prevention test must observe responsiveness and effective behavior, not mere rendering.

## Verification contract for the eventual changes

After code completion, run short local functional checks only:

1. Upstream notification-count test proves one success publication for 7,248 fetched rows and fails against 3.0.2.
2. UAR opens the sheet within a bounded two seconds without requesting `/api/models`.
3. Only configured provider/model entities appear; the local proxy remains present.
4. Change a session model, save, close, reopen, and observe the persisted value.
5. Send a real inference turn and prove the saved session model determines the effective route.
6. Cancel an edited draft and prove committed session data is unchanged.
7. Inspect browser console/network and UAR server logs for attributable failures.
8. Verify sheet spacing at compact and desktop widths using the design-system token selected during Spec/Plan, not an invented pixel value.

All product verification runs locally. No GitHub Actions product tests and no soak run.

## Research limitations

The prometheus-research daemon failed twice at initialization with zero sources and zero tokens, including after its documented LaunchAgent restart. The job was cancelled and the skill's disk-backed fallback was used. Context7 did not index the private package. Exact NPM tarballs, embedded source maps, signed Git source, current UAR source, and official React/Zustand documentation form the evidence base. The Feynman grading tool was unavailable, so no learning-grade score is claimed.

# Handoff: teams domain (`frontend/src/platform/entities/teams`)

Writer: domain (uar-console). Nothing below was executed: no typecheck, lint, test or build was run (project verification boundary).

## (a) Rules applied

Vercel React Best Practices, Composition Patterns and the Prometheus Entity Management skill (entity-graph-optimize, `rerender-*`, `js-index-maps`, `js-early-exit`, `rendering-conditional-render`) shaped this domain as follows. Every business record (definitions, packages, bindings, teams, members, tasks, attempts, waits, artifacts, peer messages, run summary, routing-test progress) lives in the normalized graph and reaches components only through hooks exported by `@/platform/entities`; components never import the graph, a store, or the transport. Each load fetches first and then publishes its whole response, including its lists and its load state, in one `batchGraphWrites` call (one store publication), never per row. Hooks subscribe to one field or one derived primitive (a boolean, a number, a string or a stable graph reference), and list hooks return `useMemo`-mapped id arrays that stay stable while the list is. Derived values (`useTeamIsLive`, `useTeamBudgetRemaining`, `useTaskBlockedBy`) are computed in the selector and return primitives, so a poll that changes nothing re-renders nothing. Workspace is a parameter of every hook and action, never hidden state. Ids are owner- and workspace-qualified and length-prefixed (precedent: `presentations/domain.ts:15-18`), hooks resolve the owner from a per-session runtime record (fresh `TEAMS_SESSION_ID`), and load states, run watches and in-flight routing steps are stamped with that session, so a persisted graph never shows one principal's records to another and never shows a stale "loading" or "running". UI-local mechanics (open sheets, form entries, "Use example value") stay in `useState`; the only transient workflow state in the graph is the routing-test progress, which must survive a reload to be retried safely.

## (b) Public surface (import from `@/platform/entities`)

Workspace-scoped calls take `workspace: string` first. "Domain id" means the server's id (`teamId`, `memberId`, ...), not the graph id.

### Actions: `useTeamActions()` returns `teamActions` (or import `teamActions`)

Loads publish with one atomic write and reject with `TeamsApiError` on failure (the error is also in the load state). Concurrent calls for one key share a request. After a 401 every load except `loadCapabilities` fails locally until `resetTeamsAccess()`.

| Action | Signature | Use |
|---|---|---|
| `loadCapabilities` | `(options?) => Promise<CollaborationCapabilities>` | First call on page open; admits the owner (`bindingOwnerId`) and sets instance id, backend, `teamExecutionAvailable`. Also the 401 probe/retry. |
| `loadExecutionOwner` | `(options?) => Promise<ExecutionOwnerView>` | Fills the ownership fact (read-only; no reclaim exists). |
| `ensureOwner` | `() => Promise<string>` | Rarely needed; every load calls it. |
| `resetTeamsAccess` | `() => void` | Clear the remembered 401 (a "Try again" button), then call `loadCapabilities`. |
| `loadTeamDefinitions` / `loadPackages` | `(options?) => Promise<void>` | Catalog lists. |
| `loadPackageExport` | `(pkg: ImmutableDefinitionRef, options?) => Promise<PackageExportModel>` | Fetches `packages:export`, parses it, stores it. A refused export resolves with `status: "refused"` and `refusalMessages`; role and aliases are then manual. |
| `preflightTeamPackage` | `(input: PackageSourceInput) => Promise<PackagePreflightResponse>` | Install sheet. `input = {manifest: string, files: Record<string,string>, expectedCatalogRevision?, commandId?}`. |
| `installTeamPackage` | `(input: PackageSourceInput) => Promise<PackageInstallResponse>` | Permanent. Reloads definitions and packages afterwards (best effort). |
| `buildDeploymentBinding` | `(input: BuildBindingInput) => Promise<{document, expectedRevision}>` | Binding sheet; see below. |
| `loadBindings` | `(workspace, options?) => Promise<void>` | |
| `preflightTeamBinding` / `installTeamBinding` | `(workspace, {binding, expectedRevision, commandId?}) => Promise<BindingPreflightResponse / BindingInstallResponse>` | Pass the `{document, expectedRevision}` from the builder as `{binding: document, expectedRevision}`. Install reloads bindings. |
| `loadTeams` | `(workspace, options?) => Promise<void>` | List, newest first. |
| `loadTeam` | `(workspace, teamId, options?) => Promise<void>` | Team with members and tasks. |
| `createTeam` | `(workspace, {deploymentBindingId, teamDefinition, input, memberSlots?, commandId?}) => Promise<TeamInstance>` | Publishes the new team to the teams list. |
| `createTask` | `(workspace, teamId, {title, role, input, outputContract?, dependsOn?, taskId?, commandId?}) => Promise<TeamInstance>` | `outputContract` must be a JSON Schema object (default `{}`). |
| `claimTask` | `(workspace, teamId, taskId, {memberId?, commandId?}) => Promise<TeamInstance>` | Requires `ready`; defaults to the first available member holding the role. |
| `admitTask` | `(workspace, teamId, taskId, {reservation, contextArtifactIds?, commandId?}) => Promise<TeamExecutionAttempt>` | Low level. Prefer the routing-test actions for anything retryable (see (d)). |
| `cancelAttempt` | `(workspace, teamId, attemptId, {reason, commandId?}) => Promise<TeamExecutionAttempt>` | Body sent: `{commandId, expectedTeamRevision, reason}`. |
| `refreshTeamRun` | `(workspace, teamId, {sideRecords?}, options?) => Promise<{live, signature}>` | One atomic refresh: team + execution (+ artifacts + peer messages when `sideRecords`). |
| `loadTeamArtifacts` / `loadTeamPeerMessages` | `(workspace, teamId, options?) => Promise<void>` | |
| `loadTeamExecutions` | `(workspace, teamIds, concurrency = 2) => Promise<void>` | For list ordering: loads execution for many teams in one publish. Costs one request per team. |
| `watchTeamRun` | `(workspace, teamId) => () => void` | Start in an effect, return the stop function as cleanup. 3 s while live, 15 s after an error or 429, artifacts and messages only when the attempt/wait signature changes, stops when not live or on 401. Shared across callers of the same team (ref-counted). |
| `refreshTeamRunNow` | `(workspace, teamId) => Promise<void>` | Manual refresh; wakes a stopped watcher. |
| `startRoutingTest` | `(input: RoutingTestInput) => Promise<TeamRoutingProgress>` | `{workspace, teamId, role, request, title?, reservation?, contextArtifactIds?, memberId?, taskId?}`. Creates, claims, admits. **Resolves (does not throw) on a command failure**; the failure is in the progress record. |
| `resumeRoutingTest` | `(workspace, teamId, taskId) => Promise<TeamRoutingProgress \| null>` | Performs only the step still owed. |
| `dismissRoutingTest` | `(workspace, teamId, taskId) => void` | Stops tracking (the task stays). |

`BuildBindingInput`: `{package, ownerId, workspaceId, runtimeInstanceId, runtimeCapabilities, modelBindings: {requestedAlias, providerId, modelId, credentialRef}[], effectiveLimits, effectiveBudget (without currency), storage: {backend, connectionRef}, policyRevision, bindingId?, existingRevision?, version?, requiredCapabilities?, provenance?}`. Emits every required field: profile, kind, id (`urn:uar:console:binding:<uuid>` by default), version, provenance (non-empty), requiredCapabilities (default `[BINDING_CAPABILITY]`, each must be in `runtimeCapabilities`), extensions, exportClass, package, ownerId, workspaceId, runtimeInstanceId, revision (`existingRevision + 1`; `expectedRevision` returned for the request), modelBindings, skillBindings `[]`, storage with `durableTransactions: true`, policyRevision, effectiveLimits, effectiveBudget with **currency forced to "USD"**, contextGrants `[]`, representationGrantRefs `[]`, status `ready`, effectiveBindingReceiptRef `null`, contentDigest. It throws a `TeamsApiError` of kind `precondition` (nothing sent) for an empty instance id, empty model bindings, a missing reference, or an unreported capability. It never invents `credentialRef`, `connectionRef` or `policyRevision`.

Pure helpers, also exported: `canonicalJson(value)`, `sha256Digest(textOrBytes)`, `byteDigest(textOrBytes)` (`"sha256:<hex>"`), `contentDigestOf(doc)`, `withContentDigest(doc)`, `documentReference(doc)`, `parsePackageExport(outcome)`, `owedRoutingStep(progress)`, `definitionKey({id, version})` (`id@version`), `isTeamRunLive(attempts, waits)`, `isAttemptInFlight(status)`, `isWaitLive(state)`, constants `TEAM_ATTEMPT_STATUSES`, `TEAM_ATTEMPT_IN_FLIGHT_STATUSES`, `TEAM_ATTEMPT_FINAL_STATUSES`, `TEAM_WAIT_LIVE_STATES`, `DEFAULT_ROUTING_RESERVATION` (128000 tokens, 1000000 microunits, 120 s), `BINDING_CAPABILITY`, `COLLABORATION_PROFILE`, `WATCH_LIVE_INTERVAL_MS`, `WATCH_BACKOFF_INTERVAL_MS`, `runSignature`, and `teamLoadKeys`.

### Errors

`TeamsApiError extends Error` with `.info: TeamsErrorInfo`, `.kind`, `.status`, `.code`, `.detail`, `.uncertain`. `TeamsErrorInfo = {kind, status, code, detail, message, capability, uncertain}` is plain data and is what hooks return. Kinds: `unauthorized` (401), `workspace_required` (400, or an empty workspace locally), `conflict` (409, `message` is the runtime's `detail` verbatim), `rate_limited` (429, no body), `not_found`, `invalid` (422), `forbidden` (403), `unavailable` (503; `capability` is set for `capability_unsupported`), `runtime`, `network` (`uncertain` for writes), `malformed`, `precondition` (local; nothing was sent). A 409 `code` may be a `TEAM_*` code instead of `collaboration_revision_conflict`; never read "revision changed" from it.

### Hooks

Readiness and loading

- `useTeamsOwnerId(): string | null`: owner admitted this session. `useTeamsOwnerId()`.
- `useTeamsRuntimeField(field): T | null` with field `instanceId | catalogBackend | teamExecutionAvailable | ownsExecution | capabilitiesLoadedAt | executionOwnerLoadedAt`. `null` = not loaded this session; `instanceId === ""` = host has not configured one. `useTeamsRuntimeField("instanceId")`.
- `useTeamsHasCapability(name: string): boolean`.
- `useTeamExecutionClaimField(field): ExecutionClaim[K] | null`, fields `epoch | acquiredAt | state | serviceInstanceId | incarnationId | catalogId | auditReceiptId`. `useTeamExecutionClaimField("epoch")`.
- `useTeamLoadStatus(key): "idle" | "loading" | "ready" | "error"`, `useTeamLoadError(key): TeamsErrorInfo | null`, `useTeamLoadCompletedAt(key): number | null`. Keys from `teamLoadKeys`: `capabilities`, `executionOwner`, `definitions`, `packages`, `packageExport(packageKey)`, `bindings(ws)`, `teams(ws)`, `team(ws, id)`, `run(ws, id)`, `artifacts(ws, id)`, `peerMessages(ws, id)`. `useTeamLoadError(teamLoadKeys.capabilities)` is how the strip sees a 401.
- `useTeamLoadKeys()` returns `teamLoadKeys` for components that prefer a hook.

Catalog

- `useTeamDefinitionKeys(): string[]` (`id@version`). `useTeamDefinitionField(key, field)` fields: `title purpose version definitionId digest package members budget limits instructions`.
- `usePackageKeys(): string[]`. `usePackageField(key, field)` fields: `manifest definitionKeys installedAt digest version packageId`.
- `usePackageExportStatus(packageKey): "exported" | "refused" | undefined`, `usePackageExportRefusal(packageKey): string[] | undefined`.
- `useExportedCoordinatorRole(packageKey, {id, version}): string | null` (the team definition's `coordinatorRole`).
- `useExportedModelRequirements(packageKey, {id, version}): ModelRequirement[]` where `ModelRequirement = {requirementKey, agentDefinitionId, role, capabilities, preferredAliases, alsoUsedBy}`, de-duplicated by alias set.
- `useBindingIds(workspace)`, `useBindingIdsForPackage(workspace, packageKey)`: binding ids. `useBindingField(workspace, bindingId, field)` fields: `revision activationSupported packageKey package document preflightDiagnostics effectiveBindingReceipt updatedAt`.

Teams, members, tasks

- `useTeamIds(ws)` newest first. `useTeamField(ws, teamId, field)`: `revision status definition package binding input createdAt updatedAt`. `status` is "inactive" or "revoked" and means nothing for activity.
- `useMemberIds(ws, teamId)` (by ordinal). `useMemberField(ws, teamId, memberId, field)`: `role kind ordinal status definition revision`.
- `useTaskIds(ws, teamId)`, `useUnassignedTaskIds(ws, teamId)`, `useMemberTaskIds(ws, teamId, memberId)`. `useTaskField(ws, teamId, taskId, field)`: `title role input outputContract output dependsOn status revision assigneeMemberId reviewerMemberId stateReason createdAt updatedAt`.
- `useTaskBlockedBy(ws, teamId, taskId): string | null`: first dependency that failed or was cancelled.
- `useTaskDelegatorMemberId(ws, teamId, taskId): string | null`: from `team_delegate` receipts.

Attempts, waits, derived activity

- `useAttemptIds(ws, teamId)`, `useMemberAttemptIds(ws, teamId, memberId)` (oldest first), `useTaskAttemptIds(ws, teamId, taskId)`. `useAttemptField(ws, teamId, attemptId, field)`: `runId taskId memberId status executionOutcome effectDisposition accountingState reservation usage output stateReason diagnostic effectiveModels continuationOfWaitId contextArtifactIds queueSequence createdAt updatedAt`.
- `useWaitIds(ws, teamId)`, `useMemberWaitIds(ws, teamId, memberId)`. `useWaitField(ws, teamId, waitId, field)`: `attemptId taskId memberId targetTaskIds predicate state continuationAttemptId reasonCode wakeOutcomes continuationInput`.
- `useTeamIsLive(ws, teamId): boolean` is the only activity signal. `useTeamAttemptCount(ws, teamId): number`. `useTeamLastActivityAt(ws, teamId): string | null`. `useMemberOpenAttemptId(ws, teamId, memberId): string | null`: newest in-flight attempt, or the yielded attempt of a live wait.
- `useTeamBudgetLimit(ws, teamId, "tokens" | "costMicrounits" | "elapsedSeconds"): number | null` and `useTeamBudgetRemaining(...)`: limit minus the sum of every attempt's `usage ?? reservation`, which is how admission counts. `useTeamTaskSlotsLeft(ws, teamId): number | null`: definition `maxPendingTasks` minus all tasks ever created. `useTeamRunSummaryField(ws, teamId, field)`: `committed reserved uncertainAttemptIds budget limits delegations continuations loadedAt`.
- `useArtifactIds(ws, teamId)`, `useMemberArtifactIds(ws, teamId, memberId)`, `useArtifactField(ws, teamId, artifactId, field)`: `taskId memberId attemptId content createdAt`.
- `usePeerMessageIds(ws, teamId)` (oldest first), `usePeerMessageField(ws, teamId, messageId, field)`: `senderMemberId senderAttemptId recipientMemberId recipientTaskId mode payload acceptedAt delivery`.

Run watch and routing test

- `useTeamRunLastUpdatedAt(ws, teamId): number | null` (ms), `useTeamRunWatchError(ws, teamId): TeamsErrorInfo | null`, `useTeamRunWatchStatus(ws, teamId): "watching" | "stopped" | "idle"`, `useTeamRunWatchInterval(ws, teamId): number | null`.
- `useRoutingTestTaskIds(ws, teamId)`. `useRoutingProgressField(ws, teamId, taskId, field)`: `step failedStep lastError memberId attemptId role title request reservation updatedAt createCommandId claimCommandId admitCommandId`. `useRoutingOwedStep(...): "create" | "claim" | "admit" | null`. `useRoutingRunningStep(...): "create" | "claim" | "admit" | null`.

Also exported: types `TeamDefinitionEntity`, `TeamPackageEntity`, `TeamBindingEntity`, `TeamInstanceEntity`, `TeamMemberEntity`, `TeamTaskEntity`, `TeamAttemptEntity`, `TeamWaitEntity`, `TeamRunSummaryEntity`, `TeamRoutingProgress`, `ModelRequirement`, `PackageExportModel`, `ExportedTeam`, the wire types in `contracts.ts` (`TeamReservation`, `TeamBudget`, `ConversionDiagnostic`, `PackageSourceRequest`, ...), and the input types of the actions.

## (c) Files

Created under `/Volumes/T7/Projects/prometheus/universal-agent-runtime/frontend/src/platform/entities/teams/`:
`contracts.ts`, `digest.ts`, `digest.test.ts` (authored, not run), `entities.ts`, `keys.ts`, `export-model.ts`, `binding-builder.ts`, `ingest.ts`, `store.ts`, `loads.ts`, `commands.ts`, `watch.ts`, `domain.ts`, `selectors.ts`, `use-teams.ts`, `use-team-run.ts`, `registration.ts`, `index.ts`, `api/teams-errors.ts`, `api/teams-transport.ts`.

Changed: `frontend/src/platform/entities/index.ts` (one line: `export * from "./teams";`) and `frontend/src/entities/bootstrap.ts` (import and call `registerTeamEntities()`).

No dependency, pin or submodule was touched. `@prometheus-ags/entity-graph-core` (already a declared dependency, 5.0.0) is imported for `batchGraphWrites` and `GraphBatch`/`GraphState` types, because `@prometheus-ags/prometheus-entity-management` does not re-export `batchGraphWrites`.

`contracts.ts` is about 600 lines because it is types only; every other file is under 400.

## (d) Decisions, corrections applied, and what I found wrong or missing

Coordinator corrections (each checked against source):

1. Applied. Attempt statuses are the eight listed (`settlement.rs:298`, `scope.rs:202`, `admission.rs:148`, `waits.rs:298-305`); `yielded` is permanent and resumption creates a new `queued` attempt (`continuations.rs:169-176`). Wait states: `yield_requested`, `waiting`, `blocked` are live; `resumed`, `invalidated` are settled (`waits.rs:235,262-265,320`). No status-only terminal predicate is exported; `isTeamRunLive(attempts, waits)` is the polling rule and drives `watchTeamRun`. `TEAM_ATTEMPT_FINAL_STATUSES` (succeeded, failed, cancelled) is for display only. `TeamWait` is modeled in full (`attemptId` is `authority.attemptId`); continuations and `team_delegate` receipts are kept in the run summary. Extra finding for lane actions: cancel of a `yielded` attempt is allowed only while its wait is `waiting` or `blocked`, and cancel of an `uncertain` attempt is refused with a conflict (`settlement.rs:298-300`), so offer Cancel accordingly.
2. Applied. No approval field is modeled.
3. Verified. Cancel body is `{commandId, expectedTeamRevision, reason}`.
4. Verified and applied. `parsePackageExport` classifies files by `kind`, and exposes `coordinatorRole` and per-agent `models[]`.
5. Verified (`bindings.rs:761-785`: the first `modelBindings` entry whose `requestedAlias` is in the entry's `preferredAliases` wins) and applied: one requirement per agent `models[]` entry, de-duplicated by alias set (`requirementKey`), with `alsoUsedBy`. The binding sheet should offer each requirement's `preferredAliases` as the choices for `requestedAlias`.
6. Applied (see `buildDeploymentBinding`). The binding's `effectiveBudget.currency` is forced to `"USD"`.
7. Applied, with one addition the correction missed. A same-`commandId` retry is not enough: admission digests the whole request including `expectedTeamRevision` and `expectedTaskRevision` (`admission.rs:31-35`), and so do create and claim (`claims.rs` request digest, `check_receipt`). After a commit the team revision has moved, so a retry that re-reads revisions is refused as "commandId used for another request". The progress record therefore stores the exact admit request first sent (`admitRequest`). Retry logic for admit: read `/execution` first; if an attempt for the task already exists and is not `queued`, mark admitted; if it is `queued` (committed, never dispatched) resend the stored request byte-identically with the same `commandId`; if none exists, rebuild with fresh revisions and the same `commandId` (no receipt exists yet). Create and claim retries first look at the fresh team (task exists, task assigned) and skip if already done. Progress record: `{taskId, createCommandId, claimCommandId, admitCommandId, step, failedStep, lastError, running, runningSession, admitRequest, ...}`. I added `step: "pending"` (nothing sent yet) to the four steps you listed. `createTask` also rejects a non-JSON-Schema `outputContract` server-side; `{}` is sent by default.
8. Applied. `TeamInstanceEntity.status` and `TeamMemberEntity.status` are carried but documented as not an activity signal. Activity is `useTeamIsLive`, `useTeamAttemptCount`, `useTeamLastActivityAt`. The team list cannot know which teams are live until their execution is loaded; call `loadTeamExecutions(workspace, visibleTeamIds)` (one request per team) before sorting active-first. This is a cost the UI writer must choose.
9. Applied. Graph ids are `len:owner|len:workspace|len:id...`; hooks gate on the per-session admitted owner (capabilities `bindingOwnerId`). Caveat: after a reload, hooks show persisted records of the same owner before the first refresh completes (stale-while-revalidate); use `useTeamLoadStatus`/`...CompletedAt` if a view must not show them. Catalog records (definitions, packages, exports) are owner-qualified too.
10. Noted. Admission also needs the team definition's budget currency to be `"USD"`, and the console cannot change a definition. `useTeamDefinitionField(key, "budget")` exposes it; the UI should block the routing test with a stated reason when it is not `"USD"`.

Other decisions and deviations from the brief:

- No entity transports are registered (schemas only). The generic transport path identifies records by the server's id and takes no workspace, so it would write unqualified, cross-principal ids into the persisted graph. All I/O is in `api/teams-transport.ts`, called only by domain actions.
- `isTerminalAttemptStatus` is not exported (see correction 1).
- `TeamPackage` omits `sourceJson` from the stored record (large, duplicates `manifest`).
- Preflight diagnostics are `{field, disposition, message}`: `field` is the RFC 6901 pointer (`bindings.rs:363-366,406-410`), and `reasonCode` is dropped by the runtime. For model-resolution diagnostics the pointer is `/models/N` into the **agent definition**, not the binding, so it cannot be matched to a binding field by pointer; map `/models/N` to the Nth entry of the agent's `models[]`, i.e. to a `ModelRequirement` (indices are per agent, so a team with several agents is ambiguous: show these diagnostics as a group above the model step, not beside a field). `/runtimeInstanceId` and `/requiredCapabilities` do map to binding fields.
- Top-level keys of `GET /capabilities` other than `executionProfile` are snake_case (`uar_version`, `assistant_operations`); only the camelCase parts the console reads are typed.
- A 401 is remembered per session: every load except `loadCapabilities` then fails locally without a request, and the watcher stops. `resetTeamsAccess()` clears it.
- `watchTeamRun` does not pause on a hidden tab; add that in the page if the shared 10 req/s bucket matters.
- Team `output` and artifact `content` are stored as received (they can be large and are persisted in the local graph). I did not truncate.
- Peer-message and artifact lists exist per team and per member; per-task artifact lists do not.

## (e) Unverified

Nothing was executed. Specifically unverified:

- Type correctness of every file (no `tsc`), lint cleanliness (for example unused imports or rules I could not see), and the `digest.test.ts` assertions, including that `crypto.subtle` exists under the `happy-dom` test environment and that the `../../../../../tests/fixtures/collaboration/` URL resolves from the test file.
- That `GraphState` is the exact state type `useGraphStore` passes to selectors, and that `batch.setListIds` creates a list that `state.lists[key].ids` exposes the way the presentations hooks read it (read from the `.d.ts` only).
- That `@prometheus-ags/entity-graph-core` resolves from the app bundle as a direct import (it is a declared dependency and is symlinked in `frontend/node_modules`) and that its `graphStore` is the same instance as the one the entity-management package exports (seen in `chunk-JKXSJWRS.mjs`, a re-export).
- Wire shapes beyond the Rust structs I read: `TeamInstance` JSON (members' `definition`, tasks' optional fields), `effectiveBindingReceipt` content, and the `GET /capabilities` field names in a running service. The digest implementation matches `canonical.rs` for strings, booleans and integers; a source float such as `1.0` would hash differently, which cannot occur in documents the console builds.
- That `useTeamBudgetRemaining` equals the runtime's own remaining budget; it uses the same arithmetic as `admission.rs:138-196` over the attempts the graph holds, so it is only as current as the last refresh.
- Behavior against the running service (which, per the design, cannot bind, launch or admit today).
- Persisted-graph hydration with the new entity types, and the local-first replay path (no transports are registered, so none is expected).

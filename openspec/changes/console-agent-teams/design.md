## Context

Revision 3 (2026-11-18). Revisions 1 and 2 were each reviewed and each found not ready; the review record is at the end of this file. The run-view model in this revision is still derived from reading the source, not from an observed team run.

Mode: Operate. This extends the existing admin console; it does not redesign it. `frontend/PRODUCT.md` is the product authority, there is no `DESIGN.md`, and the incumbent implementation plus `docs/design/agents.html` and `docs/design/runs.html` are the visual reference. Shell and states follow Agents (`features/agents/ui/agents-page.tsx` list plus detail, the `configuration-states` components). Controls follow Presentations (labelled `min-h-11` buttons, focus return on sheet close).

Facts established from source and the running service:

- All team routes are under `/api/v1/collaboration` and call `owner_key`; an anonymous caller gets 401. Team-instance, binding and execution routes also need `x-uar-workspace-id` or return 400 `workspace_required`.
- Errors are `{error:{code, detail?, messageKey}}`. Every domain conflict is HTTP 409 `collaboration_revision_conflict`, including budget exhausted and member busy (`api/collaboration.rs:489-494`, `admission.rs:112,156,208`), so the code alone does not mean "revision changed".
- A package version (`GET /packages/{id}/versions/{version}`) returns `{identity, manifest, sourceJson, definitionKeys, installedAt}` and no definition documents. Definition sources, including a team's `coordinatorRole` and each agent's `models[].preferredAliases`, come only from `POST /packages:export`.
- Document digests are `"sha256:" + hex(SHA-256(canonical JSON))` with sorted keys and the top-level `contentDigest` removed (`validation/canonical.rs:17-34,56-66`).
- **Bindings need a configured, owning runtime.** `runtimeInstanceId` must be non-empty (`bindings.rs:439,976-981`) and match the service instance; a runtime that does not hold the execution claim gets `TEAM_EXECUTION_OWNER_CONFLICT` (required-unsupported) on binding preflight for a team package (`bindings.rs:313-314`). `credentialRef` and `storage.connectionRef` are only checked for `://`.
- **Admission needs a claim first.** Admit requires `task.status == "ready"` and `task.assigneeMemberId == memberId` (`admission.rs:101-104`); create leaves the assignee empty. The sequence is create, claim, admit.
- **Reclaim is a privileged host procedure.** It needs the `x-uar-admin-key` header and `{commandId, catalogId, expectedEpoch, replacementServiceInstanceId, reason, fencingEvidenceRef}`; the fencing evidence must already exist for the holder's fence and a non-owner cannot mint it (`ownership.rs:78,164`). It marks running attempts uncertain.
- Attempt statuses are `queued, running, cancellation_requested, yielded, succeeded, failed, cancelled, uncertain` (`settlement.rs:298`, `scope.rs:202`, `waits.rs:298-305`). `yielded` is a permanent record: resuming creates a new attempt and the original never changes (`continuations.rs:169-176`). Whether a yielded attempt is still waiting is in `execution.waits[].state` (`yield_requested`, `waiting`, `blocked`). `uncertain` can still become `yielded`.
- `/execution` has no approval field and a wait carries only its target task ids, so the console cannot tell "waiting for approval" from the team API.
- Cancel takes `{commandId, expectedTeamRevision, reason}`; an `uncertain` attempt answers 409, and a `yielded` one cancels only while its wait is waiting or blocked.
- `TeamInstance.status` and `TeamMember.status` are written `inactive` and never advance; only `revoked` is set. A task has no delegator field; the link is `execution.commandReceipts[]`.
- Task create needs a `title` and a JSON Schema `outputContract`. Claim needs an already `ready` task. Admit can commit and then fail at dispatch; a retry is idempotent only with the same `commandId`, and the task stays `ready` after admission.
- Package export is `POST /packages:export` with `{package:{id,version,digest}, target:{kind:"canonicalDraft2"}}`, needs no workspace header, and returns `{status, export:{manifest, files}}` where each file is a JSON string.
- A binding needs one model binding per agent `models[]` entry, matching any of that entry's `preferredAliases` (`bindings.rs:761-785`), and both budgets need `currency: "USD"`.
- Approvals are resolved on `/admin/approvals` (`features/runtime/ui/runtime-console-page.tsx:426`), not in the run trace. `team_send` and `team_delegate` always need approval.
- Tests consume finite resources: the task cap counts every task ever created (`team_planning.rs:254`), reservations accumulate, and nothing can be deleted.
- The rate limiter is one global bucket, default 10 requests per second, and a 429 has no body (`security/rate_limit.rs:22,76`).
- The entity graph is persisted (`entities/bootstrap.ts:36-38`), and schemas are registered in `bootstrap.ts:76-81`.
- **The running service today:** catalog empty; `instance.id` is the empty string; the execution claim is held at epoch 6 by an incarnation that no longer exists, so `ownsExecution` is false. On this service only package install can succeed until the host configures an instance id and recovers execution ownership.

## Goals / Non-Goals

**Goals:** an operator can install a team package, bind it, launch a team, send a test request to the coordinator and watch each member's work, and can see exactly which precondition blocks each step when the runtime is not ready.

**Non-Goals:** authoring team definitions; edit or delete of packages, bindings or teams; chat-level routing by `orchestrator-agent`; a per-member view inside chat or the Runs page; reclaiming or quiescing execution ownership from the console; collecting the admin key; resolving approvals on the Teams page; skill bindings (a package that requires them is reported as not bindable here); any Rust change.

## Decisions

1. **One Teams page, list plus detail.** The list has two groups: Teams (workspace-scoped, those with live work first, then newest first, each row showing definition title, derived activity and created time) and Definitions (catalog). Selection and workspace are URL state (`/admin/teams?workspace=<id>&team=<id>` or `&definition=<id>@<version>`), so a run view can be linked and reopened.
2. **Readiness strip with four facts and four states each.** Identity, workspace, runtime instance id, execution ownership; each is `ok`, `blocked`, `unknown` or `checking`, shown with icon and text. On a 401, workspace and ownership are `unknown`, not failed, and the copy names the fix: open the console through the identity host. The strip states what each fact gates:

   | Fact | Blocks when not ok |
   |---|---|
   | Identity | everything |
   | Workspace | bindings, teams, routing tests |
   | Runtime instance id configured | bindings, and so launch |
   | Execution ownership | team bindings, routing tests |

3. **Execution ownership is read-only.** The strip shows the holder's epoch and acquisition time, says ownership must be recovered by the host operator, and links to the supervisor recovery document. No reclaim control.
4. **Workspace is a parameter, not hidden state.** The page reads it from the URL (default `default`, last value remembered in `localStorage` only to prefill), commits changes on Enter, and passes it to every hook and action. The platform transport takes the workspace as an argument; nothing in `platform/` imports from `features/`. Empty states name the workspace ("No teams in workspace 'x'").
5. **Team records are graph entities with owner- and workspace-qualified ids,** admitted per session the way Presentations does (`presentations/domain.ts:15-18`), so a persisted graph never shows one principal's teams to another. `TeamDefinition` and `TeamPackage` are catalog-wide. `DeploymentBinding`, `TeamInstance`, `TeamMember`, `TeamTask`, `TeamAttempt`, `TeamWait`, `TeamArtifact` and `PeerMessage` use ids prefixed with the workspace, because the graph is persisted. Each fetch is one atomic ingest. Components import only hooks from `@/platform/entities`.
6. **Definition detail is the hub.** It shows members, limits, budget, the coordinator role and requested model aliases (from `packages:export`), the bindings for this workspace, and the next step: "Create binding" when none exists, "Launch team" when one does. Each sheet closes onto the thing it created.
7. **Install sheet.** Choose files; the console identifies the manifest, lists missing and extra files against `manifest.files`, and checks each `byteDigest` locally before calling preflight. Preflight diagnostics are grouped by file with the blocking ones first. The confirm states that install is permanent.
8. **Binding sheet in two steps.** Step one maps each model requirement (one per agent `models[]` entry, de-duplicated by alias set) to a provider and model with the existing provider and model selects; a requirement with no aliases is reported as not bindable. Step two is collapsed by default and prefilled: limits and budget from the team definition; `ownerId`, `runtimeInstanceId` and `storage.backend` from `/capabilities`. `credentialRef`, `storage.connectionRef` and `policyRevision` each have a one-line meaning and an explicit "Use example value" button; nothing is filled silently. The console generates `id`, `version`, `revision` (next), `provenance`, empty `skillBindings`, `contextGrants`, `representationGrantRefs`, `extensions`, `exportClass`, `status: "ready"`, `effectiveBindingReceiptRef: null` and `contentDigest`. Preflight diagnostics appear beside the field their pointer names; Back keeps entries. The sheet is unavailable, with the reason, when the runtime has no instance id or does not own execution.
9. **Routing test is three explicit commands with recorded progress.** Create task (title, request, empty-object output contract), claim it for a member holding the role, admit it with the reservation. The role defaults to the coordinator. The domain keeps a progress record per test task holding each command id and the last completed step, because the owed step cannot be read from the task. If a step fails, the task stays in the member's lane with the step still owed ("Claim", "Admit"); retry reuses the same command id and never creates a second task. Reservation defaults are 128,000 tokens, 1,000,000 cost microunits and 120 s, shown beside the team's remaining budget and remaining task slots, with cost in currency.
10. **Conflicts are reported as the runtime states them.** A 409 shows the runtime's `detail`. The console reloads the team and re-enables the owed action; it does not claim the cause was a revision change and does not resubmit.
11. **Member lanes.** Stacked rows, one per member, ordered attention-first (failed, uncertain, waiting, running, queued, succeeded, cancelled), with summary counts above when there are more than six members. Each lane: role, kind, member status, current task, then attempts with the latest open and older ones collapsed behind a count.

    | Attempt status | Label | Live | Shown | Operator action |
    |---|---|---|---|---|
    | `queued` | Queued | yes | task, reservation, created time | Cancel |
    | `running` | Running | yes | model, elapsed, run trace link | Cancel |
    | `cancellation_requested` | Cancelling | yes | state reason | none |
    | `yielded`, wait `yield_requested`, `waiting` or `blocked` | Waiting on tasks | yes | the wait's target tasks and their state | Cancel while waiting or blocked |
    | `yielded`, wait finished | Handed off | no | the continuation attempt it resumed as | none |
    | `succeeded` | Succeeded | no | output, usage, model | none |
    | `failed` | Failed | no | state reason, diagnostic, usage | none |
    | `cancelled` | Cancelled | no | state reason | none |
    | `uncertain` | Uncertain | no | state reason, effect disposition, note that liability is unknown | none |

    A team run is live while any attempt is `queued`, `running` or `cancellation_requested`, or any wait is `yield_requested`, `waiting` or `blocked`. Every lane with a live attempt carries one standing line: "Delegation and peer messages need approval" with a link to `/admin/approvals`, because the team API does not say when an approval is pending.

    Output is clamped with expand and copy; `<think>` blocks are folded. Tasks that depend on a failed task show "Blocked by <task>". Each attempt links to `/admin/runs?run=<runId>`.
12. **Polling is one domain action.** `watchTeamRun(workspace, teamId)` reads the team and `/execution` every 3 s while the run is live (decision 11), and artifacts and peer messages only when the attempt set changes. It stops when the run is no longer live or the view unmounts, and backs off to 15 s after an error or a 429. The view shows "Updated N s ago", the error state, and a manual refresh; a polite live region announces status transitions only; ingest never moves focus or scroll.

## Risks / Trade-offs

- **Most of this cannot be exercised on the service as it runs today.** Binding, launch and routing tests need a configured instance id and execution ownership, both host-level. Install, the readiness strip and every blocked state can be exercised now. Acceptance of the bind-to-run path needs a service that owns execution.
- **`packages:export` is the only source of coordinator role and model requirements.** The console parses the exported file strings itself.
- **The run view cannot show a pending approval.** It shows a standing pointer to the Approvals page instead. Showing the real approval needs a backend field or the run's own event stream, which is out of scope.
- **The status model is unobserved.** It was corrected twice from source reading. It must be checked against a real team run on a team-capable service before the run view is accepted.
- **Example values for references are examples.** Preflight only checks they contain `://`; a wrong reference surfaces at admission, not at bind.
- **Routing tests spend real budget and task slots that cannot be reclaimed.** The console shows what remains; it cannot give any back.
- **Polling shares a global 10 requests per second bucket** with everything else the console does. At two requests per 3 s per open team it is small, but several open tabs add up.

## File ownership

| Area | Files | Owner |
|---|---|---|
| Domain | `frontend/src/platform/entities/teams/**`, one export line in `platform/entities/index.ts`, schema registration in `entities/bootstrap.ts` | domain writer |
| Page, list, readiness, definition detail, flows | `frontend/src/features/teams/**` except `ui/run/**` | console writer A |
| Routing test and run view | `frontend/src/features/teams/ui/run/**` | console writer B |
| Shell wiring | `app/shell/nav-destinations.ts`, `pages/admin-page.tsx` | console writer A |

The domain lands first; its exported hook and action names are the contract the two UI writers build against.

## Review record

Revision 1, 2026-11-18. Design critique A: ready with changes. Design critique B: not ready. Adversarial source review: not ready. Critical findings, each confirmed against the source before this revision:

- Admission requires a prior claim; revision 1 would have returned 409 on every test.
- `GET /packages/{id}/versions/{version}` carries no definition documents; coordinator role and aliases need `packages:export`.
- Reclaim needs the admin key and pre-existing fencing evidence; revision 1's one-click reclaim was not buildable.
- An empty service instance id and a runtime that does not own execution both block team bindings.
- Approvals are on `/admin/approvals`, not in the run trace.
- Create, claim and admit are separate commands; a failure between them must not duplicate the task.
- Attempt statuses, the terminal set and the waits list were unspecified.

Revision 2, 2026-11-18. Adversarial source review: not ready. Confirmed and applied in revision 3:

- `yielded` is a permanent record, so treating it as live would have polled forever; liveness comes from the waits list.
- `cancellation_requested` was missing from the status set.
- "Waiting for approval" cannot be derived from the team API.
- Admit retry is idempotent only with the same command id; the owed step is not readable from the task.
- Team and member status never advance, so "active first" had no source.
- Entity ids must be qualified by owner as well as workspace.
- Export request and response shapes, the binding's full required field list, and the model requirement rule were under-specified.

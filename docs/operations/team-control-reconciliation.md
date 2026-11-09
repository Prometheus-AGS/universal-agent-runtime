# Team control refusals and retained accounting

Status: source authored for #475; not executed or qualified. Teams require the
Surreal-backed execution capability. PostgreSQL team execution remains unsupported.

The uncomfortable thing: settling provider billing does not prove a delegated
operation succeeded, and does not automatically release the separate team
reservation. A cancelled attempt can retain team capacity indefinitely. There is
no supported blanket reset, forced-success operation or automatic effect replay.

## Available controls

The host narrows the selected tools to the current attempt's role and directed
communication edges. Roster capabilities describe that catalog authority; current
execution policy and governance can narrow it further. `team_send` and
`team_delegate` require approval. `team_wait` is available to the qualified
coordinator and yields only after an accepted durable wait. Every execution
rechecks current owner, exact attempt session, revisions and authorization.
Actor attachment carries only the registered attempt-bound controls in its
executable policy. Withholding a team control does not prevent the task from
running, and unrelated selected tools still require executable bindings.

Only exact catalog refusals `TEAM_REVISION_CONFLICT`, `TEAM_WAIT_CYCLE` and
`TEAM_PENDING_LIMIT` return a recoverable control result:

```json
{"status":"refused","recoverable":true,"code":"TEAM_REVISION_CONFLICT","effect":"not_committed","action":"refresh_roster_or_correct_request"}
```

This is a refused request, not delegated work, a queued message, a successful
wait, or authorization. Refresh the roster/revision or correct targets; do not
blindly retry a command with changed meaning under the same command ID. A refused
wait does not yield and has no continuation receipt. Scope, identity, command
conflict, policy denial, storage error and uncertain effects remain errors. A
failed claimed effect still fails the task even if final model prose says it
succeeded. Storage errors are never classified as a harmless refusal.

## Owner-controlled manual reconciliation

Use the already provisioned trusted host/identity endpoint. Functional scenarios
supply `UAR_BASE_URL`, model and nonsecret workspace selectors; they do not mint,
copy, renew or send authentication credentials. Host operations below are carried
out through the external operator authority, not by model tool arguments.

1. Read the owner/workspace-qualified team execution summary and
   `GET /team-instances/{team}/attempts/{attempt}/reconciliation` under the existing
   collaboration API mount. This read returns original attempt/run/root identity,
   epochs, usage revision, reservation and effect disposition. Missing or foreign
   attempts refuse; it never dispatches work or changes accounting.
2. Inspect `GET /api/uar/accounting/calls/{run_id}` for that original run using the
   correlation view. Select the exact original `(owner, run_id, call_id, attempt)`
   record from the returned accounting records. Do not assume a team attempt ID
   equals a provider call attempt number. Descendant/provider calls must each be
   inspected through their retained original call identity; an unavailable
   correlation is unresolved, not permission to infer a charge.
3. Obtain authoritative cumulative provider usage for that exact call. The
   external trusted host may use the existing
   `POST /api/uar/accounting/calls/{run_id}/{call_id}/{attempt}/reconcile` with actual
   `cost_usd`, `tokens`, `confirmed_provider_usage: true` and a nonempty evidence
   reference. The same terminal evidence is idempotent. Conflicting settled
   counters refuse. A timeout, restart, cancellation or absent answer is not
   evidence of zero cost. Ordinary owner identity alone cannot attest billing.
4. Inspect effects independently using original canonical tool/connector receipts.
   The existing connector-effect reconciliation surface requires its own exact
   authoritative evidence. Provider billing does not establish effect success.
   Without such evidence, retain the uncertain effect and do not redispatch it.
5. Read the original call records and team view again. Generic call settlement
   changes its own accounting scopes. In this release it does **not** project a
   new `TeamReservation` usage record into the team catalog. Consequently
   `teamCapacityRetained: true` and `automaticTeamProjection: false` are explicit
   supported outcomes even after provider usage settles. There is no supported
   team-reservation-release mutation for this case. Preserve the original output,
   attempt IDs, epochs and reservation; do not edit protected rows, inflate a
   limit to disguise uncertainty, or mark an attempt successful.
6. If further work is authorized, use new explicitly named work after considering
   the retained liability and capacity. This is a new task, not recovery/replay of
   the unknown effect. If retained capacity prevents admission, report that
   operational limitation. Do not claim full automatic accounting recovery.

The team view's reservation is an admission bound; it is not the provider's
actual bill. Report both the call ledger settlement and the remaining team bound.
See [durable accounting](durable-accounting.md) for exact owner/host authority and
scope generation rules. Existing C54 receipts retain their historical scope;
source changes here do not certify a fresh restart or workflow result.

## C54 on the installed Linux artifact

Use the existing `scripts/behaviors/team-recovery.cucumber.mjs` feature with
`UAR_TEAM_RECOVERY_MODE=installed-linux`. The runner host can be macOS or Linux.
Provision the credential-free loopback identity endpoint, nonsecret workspace
and model selectors through `UAR_BASE_URL`, `UAR_WORKSPACE_ID` and `UAR_MODEL`.
Bind the clean candidate with `UAR_CANDIDATE_ARTIFACT_BINDING` and select its
explicit `DOCKER_CONTEXT`, `UAR_QUALIFICATION_CASE_ID`, `UAR_ARTIFACT_CONTAINER`
and `UAR_ARTIFACT_VOLUME`. Use a fresh case ID and the case, workspace, gateway,
source and configuration labels described in the
[artifact fixture](../../scripts/acceptance/fixtures/protocol-qualification/README.md).
Set the case-specific `service_instance.instance_id` and the Docker label
`dev.prometheus.qualification.instance-id` to `uar-pr476-<case-id>`. The live
collaboration capability response must identify that instance before the
first team write and after restart.
The host owns the isolated store, saved specialist records, selected model
policy and controlled model collaborator. The collaborator must support the
existing tally and replacement-round assertions, report actual fixture usage,
and hold the first note active for cancellation. Existing C54 does not authorize
a paid model or certify model quality.

1. Start the C54 runner against the already running, case-owned container and
   gateway. Its preflight binds the actual image, mount, configuration and live
   nonsecret settings before it writes any team state.
2. Wait for the runner to print the exact create-only
   `team-recovery-<nonce>-restart-request.json` path under
   `target/production-artifact-qualification/<case-id>/`. Check the same
   attempt's nonce, case ID, context, container ID, image, volume and original
   start identity. For each barrier, confirm the current runner is still
   waiting, `expiresAt` is in the future and an immediate inspect matches the
   exact expected running or stopped state. Set `container_id` to the validated full ID from the
   barrier. The host records the command and stops only that container with
   `docker --context "$DOCKER_CONTEXT" stop "$container_id"`.
3. Leave the container stopped until the runner prints the matching
   `team-recovery-<nonce>-stopped.json` path. Confirm it binds the request's
   identity and nonce and records a clean exit. The host records the command
   and starts that same container using
   `docker --context "$DOCKER_CONTEXT" start "$container_id"`.
   Keep the gateway, workspace, image, data volume and provider selection
   unchanged. Do not act on barriers from a prior or failed attempt.
4. The runner requires the actual stop and later process start, verifies the
   resumed collaboration endpoint owns execution, and rebinds the candidate.
   It then checks original attempts, effects, calls and team capacity, refuses
   the premature dependent join, and checks an explicit replacement round.
   Retain the Cucumber report, barrier files, host command receipts and owned
   cleanup or retention result together.

The runner never starts or stops Docker and never reads the container
environment. The existing manual macOS lifecycle remains a separate lane.

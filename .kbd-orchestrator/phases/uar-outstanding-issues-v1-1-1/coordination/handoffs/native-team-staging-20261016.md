# Native supervisor candidate: real team-state staging packet — 2026-10-16

## Purpose and boundary

This packet stages the actual UAR state that the v1.1.1 recovery adapter
requires: an independently registered predecessor, at least one queued team
attempt, at least one active predecessor attempt, and a replacement runtime
whose exact fence can be retained. It is source-only guidance for the approved
disposable Linux guest. It neither starts a controller nor runs a qualification,
recovery, probe, build, or acceptance command.

The complete supervisor qualification still requires the physical native boot
transition, controller journal observation, negative authority controls, and
operator promotion described in `docs/operations/uar-1.1.1-supervisor-setup.md`.
Do not manufacture rows in the catalog or controller journal to obtain these
identities.

## Candidate controller bootstrap

The controller refuses a record with `production_qualified: false` during
ordinary startup. The isolated candidate controller must be launched with its
documented `--qualification-candidate` option. In that mode,
`scripts/supervisor/controller.py` requires all of the following:

- the qualification record digest equals `qualification_sha256`;
- `controller_id`, `qualification_id`, `mechanism`, and
  `controller_sha256` bind to the installed controller source;
- `production_qualified` is `false` and `isolated_candidate` is `true`;
- `qualification_id` starts with `dot17-candidate-`;
- every controller deployment ID starts with `dot17-`.

The candidate controller configuration is based on
`deploy/supervisor/controller.example.json`, with candidate-private paths and
values. For the approved Linux native mechanism it must select
`native-boot-exclusion-v1`, use `uar.service` as the deployment service name,
retain an independent SQLite journal, and permit the exact client certificate
fingerprint for both `register` and `exclude`. Its server certificate, client
CA, runtime client identity, journal, runtime durable store, and service UID
must belong only to the disposable guest.

The UAR service configuration must contain the existing
`supervisor_recovery` object from `deploy/supervisor/runtime.example.json`:

```json
{
  "endpoint": "https://candidate-controller-address:9447/",
  "controller_id": "candidate-controller-id",
  "controller_ca": "/operator-private/controller-ca.crt",
  "client_identity": "/operator-private/runtime-client.pem",
  "qualification_id": "dot17-candidate-...",
  "qualified_mechanism": "native-boot-exclusion-v1",
  "deployment_id": "dot17-...",
  "authorized_recovery_deployments": [],
  "recovery_peers": []
}
```

These are field names and shape only. Values must come from the isolated guest
setup and stay outside the repository. UAR's client does not read the candidate
qualification file; it requires a successful mTLS registration at service
startup. The controller candidate flag is therefore required for the false
candidate record to admit that registration. The recovery adapter separately
requires a later retained record with `production_qualified: true`; the
candidate record cannot be supplied to `uar_1_1_1_recovery.py` as a qualified
fixture.

## Exact predecessor registration source

When a UAR service with `supervisor_recovery` starts, `src/server.rs` constructs
`SupervisorRecoveryHost` and calls `register` before team execution startup.
The host sends this fixed request to the controller:

```json
{
  "runtime_epoch": "runtime-generated UUID",
  "team_incarnation": "catalog-generated UUID",
  "process_id": "runtime process ID",
  "deployment_id": "configured dot17 deployment ID",
  "qualification_id": "configured dot17 candidate ID"
}
```

The predecessor identity for recovery must be copied from the actual
`domains.registration` row in the controller's retained SQLite journal. The
required object is the returned `SupervisorRegistration`, including its
native-boot domain with the guest's machine ID, original boot ID, `uar.service`,
and root process identity. Do not rebuild it from log fields or manually write
it into a journal.

## Concrete package, team, and attempt staging

Use the existing deterministic collaboration fixture transformation in
`scripts/release/uar_1_1_1_runtime.py:615-703` as the package source, with the
fixed suffix `native-supervisor-20261016`. It constructs valid immutable
references and recomputes each content and byte digest; copying a stale fixture
digest after changing an ID is rejected by package validation.

The exact source transformation is:

1. Read `agent-definition.json`, `team-definition.json`,
   `workflow-definition.json`, `package-manifest.json`, and
   `deployment-binding.json` from `tests/fixtures/collaboration/`.
2. Set the three definition IDs to
   `urn:uar:phase111:agent:native-supervisor-20261016`,
   `urn:uar:phase111:team:native-supervisor-20261016`, and
   `urn:uar:phase111:workflow:native-supervisor-20261016`. Update the team's
   member definition reference to the resealed agent reference.
3. Set the package ID to
   `urn:uar:phase111:package:native-supervisor-20261016`, make its sole
   entrypoint the resealed team reference, and set its lock entries to the
   resealed agent and workflow references at `agent-definition.json` and
   `workflow-definition.json`.
4. Clear the template agent's `skills`, `sourceDescriptor.skills`, and
   `legacySections.skills`; set its input/output schemas to `{}` and context to
   `{ "mode": "none", "artifacts": [], "history": "none",
   "memoryScopes": [] }`. Set its rename target ID to its fixed agent ID.
5. Change the team member maximum to `2`, set team `limits.maxMembers` and
   `limits.concurrentTurns` to `2`, set the team input/output schemas to `{}`,
   and set all three team budget caps to `maxTokens: 3000000`,
   `maxCostMicrounits: 3000000`, and `maxElapsedSeconds: 6000`. These two
   members are required: admission rejects a second active or queued attempt
   for the same member. The aggregate admission ledger sums reservations from
   both queued and running attempts.
6. Canonically reseal every changed document exactly as the existing fixture
   does: remove `contentDigest`, then set it to `sha256:` plus SHA-256 of the
   sorted-key, compact UTF-8 JSON representation. Serialize the three resealed
   files with `ensure_ascii=False, indent=2`; each manifest file's `byteDigest`
   is SHA-256 of those serialized bytes.
7. Obtain the live `bindingOwnerId` and runtime instance ID from
   `GET /api/v1/collaboration/capabilities`. Reseal the binding with these
   fixed changes: ID
   `urn:uar:phase111:binding:native-supervisor-20261016`, package reference
   from the resealed manifest, returned owner/runtime IDs, the fixed workspace,
   `modelBindings[0].modelId: "gpt-4o-mini"`, profile
   `uar.openai-compatible-chat.settings-v1` revision `1`,
   `settingsRevision: 1`, empty skill/representation binding arrays, and
   `effectiveLimits.maxMembers: 2`, `effectiveLimits.concurrentTurns: 2`,
   `effectiveBudget.maxTokens: 3000000`,
   `effectiveBudget.maxCostMicrounits: 3000000`, and
   `effectiveBudget.maxElapsedSeconds: 6000`.

The following is the complete deterministic request-body construction. It is a
source packet, not a command to run during the closed phase. It leaves no
unresolved package, definition, binding, model, or agent ID; `capabilities` is
the response from the required live capabilities read.

```python
import hashlib, json
from pathlib import Path

ROOT = Path("/installed/uar")
FIXTURES = ROOT / "tests/fixtures/collaboration"
WORKSPACE = "dot17-native-supervisor-20261016"
SUFFIX = "native-supervisor-20261016"

def load(name):
    return json.loads((FIXTURES / name).read_text())

def seal(document):
    document.pop("contentDigest", None)
    document["contentDigest"] = "sha256:" + hashlib.sha256(
        json.dumps(document, sort_keys=True, separators=(",", ":"),
                   ensure_ascii=False).encode()).hexdigest()
    return document

def ref(document):
    return {"id": document["id"], "version": document["version"],
            "digest": document["contentDigest"]}

agent = load("agent-definition.json")
agent.update(id="urn:uar:phase111:agent:" + SUFFIX, skills=[], input={}, output={},
             context={"mode":"none", "artifacts":[], "history":"none", "memoryScopes":[]})
agent["sourceDescriptor"]["skills"] = []
agent["legacySections"]["skills"] = ""
agent["renameMapping"]["targetId"] = agent["id"]
seal(agent)

team = load("team-definition.json")
team.update(id="urn:uar:phase111:team:" + SUFFIX, input={}, output={})
team["members"][0]["definition"] = ref(agent)
team["members"][0]["max"] = 2
team["limits"]["maxMembers"] = 2
team["limits"]["concurrentTurns"] = 2
team["budget"].update(maxTokens=3000000, maxCostMicrounits=3000000,
                      maxElapsedSeconds=6000)
seal(team)

workflow = load("workflow-definition.json")
workflow["id"] = "urn:uar:phase111:workflow:" + SUFFIX
seal(workflow)

files = {}
inventory = []
for name, document in (("agent-definition.json", agent),
                       ("team-definition.json", team),
                       ("workflow-definition.json", workflow)):
    source = json.dumps(document, ensure_ascii=False, indent=2)
    files[name] = source
    inventory.append({"path":name, "kind":document["kind"], "definition":ref(document),
                      "byteDigest":"sha256:" + hashlib.sha256(source.encode()).hexdigest()})

manifest = load("package-manifest.json")
manifest.update(id="urn:uar:phase111:package:" + SUFFIX, entrypoints=[ref(team)],
                files=inventory,
                lock=[
                    {"requestedBy":team["id"], "reference":ref(agent),
                     "resolvedPath":"agent-definition.json"},
                    {"requestedBy":team["id"], "reference":ref(workflow),
                     "resolvedPath":"workflow-definition.json"},
                ])
seal(manifest)

capabilities = GET("/api/v1/collaboration/capabilities")
binding = load("deployment-binding.json")
binding.update(id="urn:uar:phase111:binding:" + SUFFIX, package=ref(manifest),
               ownerId=capabilities["bindingOwnerId"], workspaceId=WORKSPACE,
               runtimeInstanceId=capabilities["instance"]["id"],
               skillBindings=[], representationGrantRefs=[])
binding["modelBindings"][0].update(
    modelId="gpt-4o-mini",
    profile={"id":"uar.openai-compatible-chat.settings-v1", "revision":1},
    settingsRevision=1)
binding["effectiveLimits"].update(maxMembers=2, concurrentTurns=2)
binding["effectiveBudget"].update(maxTokens=3000000,
                                  maxCostMicrounits=3000000,
                                  maxElapsedSeconds=6000)
seal(binding)

POST("/api/v1/collaboration/packages:install", {
    "commandId":"native-supervisor-package-20261016",
    "manifest":json.dumps(manifest), "files":files}, expected=201)
POST("/api/v1/collaboration/deployment-bindings", {
    "commandId":"native-supervisor-binding-20261016", "expectedRevision":0,
    "binding":binding}, headers={"x-uar-workspace-id":WORKSPACE}, expected=201)
team_instance = POST("/api/v1/collaboration/team-instances", {
    "commandId":"native-supervisor-team-20261016",
    "deploymentBindingId":binding["id"], "teamDefinition":ref(team), "input":{},
    "memberSlots":[{"role":"reviewer", "count":2}]},
    headers={"x-uar-workspace-id":WORKSPACE}, expected=201)
```

`GET` and `POST` above use the lead-generated fixture owner/admin credentials;
the only values captured from live responses are `bindingOwnerId`, runtime
instance ID, and later server-assigned IDs and revisions. Do not substitute
hand-written digests or a different fixture package.

The team creation response returns the authoritative `team_id`, `revision`, and
two reviewer member IDs. Every following team request carries
`x-uar-workspace-id: dot17-native-supervisor-20261016` under the same verified
owner scope. Use those returned values in this exact sequence:

```json
POST /api/v1/collaboration/team-instances/{team_id}/tasks
{
  "commandId": "native-supervisor-active-task-20261016",
  "taskId": "active-predecessor",
  "expectedTeamRevision": <returned team revision>,
  "title": "controlled active predecessor",
  "role": "reviewer",
  "input": {"brief": "retain the configured controlled model request"},
  "outputContract": {},
  "dependsOn": []
}

POST /api/v1/collaboration/team-instances/{team_id}/tasks
{
  "commandId": "native-supervisor-queued-task-20261016",
  "taskId": "queued-successor",
  "expectedTeamRevision": <revision returned by active task creation>,
  "title": "controlled queued successor",
  "role": "reviewer",
  "input": {"brief": "run after the recovered queued attempt is dispatched"},
  "outputContract": {},
  "dependsOn": []
}
```

Claim `active-predecessor` with the first returned reviewer member and
`queued-successor` with the second returned reviewer member. Each claim uses
the current returned team revision, its selected task's current revision, and
its selected member ID:

```json
POST /api/v1/collaboration/team-instances/{team_id}/tasks/{task_id}/claim
{
  "commandId": "native-supervisor-active-claim-20261016" or "native-supervisor-queued-claim-20261016",
  "expectedTeamRevision": <current returned team revision>,
  "expectedTaskRevision": <current returned task revision>,
  "memberId": "<first or second returned reviewer member ID>"
}
```

With the current revisions after those two claims, use the authenticated
administrator path to submit one real queued task:

   ```text
   POST /api/v1/collaboration/team-instances/{team_id}/tasks/queued-successor/admit-queued
   x-uar-workspace-id: dot17-native-supervisor-20261016
   ```

   Its JSON body is the existing `AdmitTeamTaskRequest`:

   ```json
   {
     "commandId": "native-supervisor-queued-admit-20261016",
     "expectedTeamRevision": <current returned team revision>,
     "expectedTaskRevision": <queued-successor task revision>,
     "memberId": "<second returned reviewer member ID>",
     "reservation": {
       "tokens": 1000000,
       "costMicrounits": 1000000,
       "elapsedSeconds": 3000
     },
     "contextArtifactIds": []
   }
   ```

   The queued attempt ID returned by this endpoint is authoritative.

Then submit the active predecessor through the ordinary owner admission path:

```json
POST /api/v1/collaboration/team-instances/{team_id}/tasks/active-predecessor/admit
{
  "commandId": "native-supervisor-active-admit-20261016",
  "expectedTeamRevision": <current returned team revision>,
  "expectedTaskRevision": <active-predecessor task revision>,
  "memberId": "<first returned reviewer member ID>",
  "reservation": {
    "tokens": 1000000,
    "costMicrounits": 1000000,
    "elapsedSeconds": 3000
  },
  "contextArtifactIds": []
}
```

The ordinary `admit` endpoint dispatches immediately. The queued attempt stays
durable until post-exclusion reclaim. No attempt is cancelled to create this
state.

## Controlled active path

The approved native qualification retains a real active model request. It does
not require a held tool call, a new fixture mode, an MCP bridge, or a team host
context. Atomically write this existing provider mode file before admitting the
active predecessor:

```json
{"mode":"complete","delay":601}
```

`tests/fixtures/uar_1_1_1_provider.py` writes the request trace before applying
`delay`, so the retained private trace records the actual outbound provider
request while the team attempt remains running. This matches the approved
recovery evidence: a real queued attempt, a real active predecessor, native
boot exclusion, exact fences, and preserved accounting. It does not add a
detached tool process or a second authority path to the qualification.

Set the candidate runtime's existing configured LLM timeout to `600` seconds
for this controlled request. The settings schema permits only 1 through600;
the previous4000 setup value failed seeding before readiness. Preserve that
failed store and use a fresh disposable database for the new predecessor. After the provider trace records the active model
request, retain the team execution summary and perform the approved guest-only
native reboot immediately. The active reservation is `3000` seconds, so this
sequence must not wait for the delayed response; its purpose is to retain a
real in-flight predecessor for the physical boot exclusion.

No MCP peer or team host-context request participates in this approved
active-model-request path.

After the active model request is observed in the private trace, retain the team
summary while it is `running`; if the real predecessor loss occurs first, retain
the same attempt after it becomes `uncertain`. Both are accepted predecessor
states by the recovery adapter.

For later safe continuation, dispatch the reclaimed queued attempt through:

   ```text
   POST /api/v1/collaboration/team-instances/{team_id}/attempts/{returned_queued_attempt_id}/dispatch
   x-uar-workspace-id: dot17-native-supervisor-20261016
   ```

   Its fixed `DispatchTeamAttemptRequest` body is
   `{ "commandId": "native-supervisor-queued-dispatch-20261016", "expectedTeamRevision": <current returned team revision> }`.
   The attempt must enter actual execution through the configured provider path.
   A synthetic catalog status, a direct database update, or a cancelled attempt
   does not satisfy the recovery contract. The physical predecessor-loss stage
   determines when this live attempt becomes uncertain.

Retain an unmodified summary before the physical exclusion:

   ```text
   GET /api/v1/collaboration/team-instances/{team_id}/execution
   x-uar-workspace-id: dot17-native-supervisor-20261016
   ```

   Capture each returned attempt's ID, status, `executionFence`, reservation,
   effect disposition, and accounting state. The retained queued ID must have
   status `queued`; the active predecessor ID must have status `running`,
   `cancellation_requested`, or `uncertain` when recovery is staged.

The administrator-only `admit-queued` and `dispatch` endpoints require the
service's verified privileged context and `x-uar-admin-key`; their normal
authorization remains in force. The team-execution summary uses the actual
team-owner authority. Keep these credentials private.

## Exact replacement and recovery identity capture

After the predecessor registration and team attempt identities exist, retain
these values verbatim for the private same-image identity receipt:

- the `predecessor_registration` from the controller journal;
- `predecessor_fence` from the predecessor claim;
- `replacement_fence` reported by the replacement runtime's ownership view;
- for every team scope: `workspace_id`, `team_id`, nonempty
  `queued_attempt_ids`, and nonempty `uncertain_attempt_ids`;
- phase, isolation declaration, candidate image ID, platform, source tree,
  fixture tree, runtime image ID, and replacement runtime URL.

Use the live owner endpoint for both fence captures:

```text
GET /api/v1/collaboration/execution-owner
```

Use the team summary endpoint above for attempt identities. The recovery
adapter requires the predecessor claim's incarnation to equal the controller
registration's `team_incarnation`; it also requires the replacement fence to
have a different incarnation. It rejects an empty queued or active set.

Only after qualified physical exclusion is recorded will the replacement
runtime use these actual values to request:

```text
POST /api/uar/recovery/supervisor/challenges
POST /api/uar/recovery/supervisor/challenges/{challenge_id}/exclude
POST /api/v1/collaboration/execution-owner/reclaim
```

The challenge target is a real `team_owner` fence. The reclaim body is the
current `TeamExecutionReclaimRequest`: `commandId`, `catalogId`,
`expectedEpoch`, `replacementServiceInstanceId`, `reason`, and the committed
supervisor `fencingEvidenceRef`. The adapter checks that the returned epoch is
exactly the predecessor epoch plus one, transfers only queued attempts, and
leaves active attempts uncertain with reserved-unknown accounting.

## Hand-off points

- `scripts/supervisor/controller.py` owns candidate mTLS registration and
  native boot observation. Its only HTTP controller endpoints are
  `/v1/containment/register` and `/v1/containment/exclude`.
- `src/uar/runtime/supervisor.rs` defines the UAR config/registration/challenge
  contract; `src/uar/api/supervisor.rs` defines host-authorized challenge and
  exclusion endpoints.
- `src/uar/api/collaboration/team_execution.rs` and
  `src/uar/domain/team_execution.rs` define the team request/summary/reclaim
  contract above.
- `scripts/supervisor/recovery_scenarios.py` is the existing production HTTP
  scenario driver. `scripts/release/uar_1_1_1_recovery.py` is the v1.1.1
  adapter and validates the private qualified identity after promotion.

## Unresolved limit

No candidate qualification, controller startup, guest reboot, team mutation,
or runtime request occurred while preparing this packet. The candidate remains
unqualified until the complete isolated native physical evidence exists and an
operator retains a `production_qualified: true` record. No test or verification
command was run.

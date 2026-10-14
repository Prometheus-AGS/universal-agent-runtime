# Dedicated recovery fault stages

These are deferred source fixtures for Dot17. They have not been executed.
`scripts/dot/recovery_fault_journey.py` uses actual host recovery endpoints,
`scripts/dot/recovery_fault_probe.py` independently observes the dedicated
controller journal and service identity, and the existing supervisor observer
retains predecessor domain and detached-child evidence. The lead must open the
whole expanded Dot implementation boundary before invoking any stage.

The uncomfortable limit: stale executor writes cannot currently be driven
through a production public API. Physical exclusion destroys the old executor,
and none of the run/resume endpoints accepts a caller-selected internal executor
epoch. A wrong credential, replayed challenge, or restart is not an old executor
`ClaimIntent` attempt. The separate `stale-epoch` whole production-provider target qualifies actual
closed-epoch mutation rejection after real supervisor exclusion; this HTTP helper
does not invent a public executor-authority API. The embedded ENOSPC extension
below qualifies bounded guest-volume failures. These scripts do not qualify
macOS/Windows controller process introspection or a whole predecessor node
becoming inaccessible over every management channel. They qualify a measured partition between a surviving
runtime and its configured predecessor controller while independent management
observation remains available.

## Interface and private inputs

The runner integration calls `run_recovery_faults(profile, evidence)`. It reads
`profile.supervisor.fault_cases`, a list of `{private_case_file, stage}` objects.
Runtime URLs and host token environment names come from the same `supervisor`
object as the existing lifecycle stages. The first URL is the surviving runtime.
Use one staged case per invocation; a preparation/offline stage deliberately
raises `Blocked` after retaining observations. Completion is never inferred
from elapsed time or an operator-written result.

The immutable private case contains these fields:

- `name`: a unique letters/digits/underscore/hyphen evidence name.
- `runtime_epoch`, `controller_id`, and `target`: actual durable predecessor
  identity, with target exactly `{kind: "checkpoint_epoch", runtime_epoch: ...}`.
- `continuation_directory`: a private absolute directory for exclusively created
  stage records. Preserve it across service changes; never edit its contents.
- `marker` and `observer`: the existing detached-child observer contract in
  [dedicated-host qualification](dot-supervisor-qualification.md). Keep the real
  fixture trace available. Domain IDs must come from actual admitted work.
- `controller_observer` and `runtime_observer`: the SSH/process records below.
- `authorize_physical_exclusion: true`: required for outage/partition cases.
  CRI cases additionally require `authorize_node_inhibition: true`; a completed
  or partially failed exclusion may leave the dedicated node inhibited.

An observer record has this shape. Values are illustrative placeholders, never
accepted as proof of a process or an outcome:

```json
{
  "dedicated_qualification_host": true,
  "ssh_target": "qualification@dedicated-linux-host",
  "known_hosts": "/operator-private/pinned-known-hosts",
  "python": "python3",
  "expected_machine_id": "actual-machine-id",
  "process": {
    "pid_file": "/operator-private/actual-service.pid",
    "uid": 1000,
    "executable": "/absolute/actual/executable",
    "executable_sha256": "actual-installed-executable-sha256",
    "config": "/operator-private/actual-config.json",
    "config_sha256": "actual-private-config-sha256"
  }
}
```

The controller process also has `source` pointing to its installed absolute
`scripts/supervisor/controller.py`. The helper supplies its source hash from the
current phase checkout. `executable` is that controller's actual Python binary;
for the runtime observer it is the actual installed UAR binary. Provision the
PID file using the real service manager's reported current PID after each
controller restart. The observer does **not** trust that file: Linux procfs must
show the exact executable, config argument, source argument for the controller,
UID and process start identity. A dead process, inaccessible procfs, different
source/config, or service replacement during observation blocks proof.

The runtime config must be JSON with explicit `persistence.provider: "surreal"`,
`persistence.database_url`, and the actual `supervisor_recovery` configuration.
JSON is an explicit config input supported by the existing config loader. No
`UAR_PERSISTENCE*` or `UAR_SUPERVISOR*` environment overrides may alter this
configuration for these stages. The observer checks the live process environment
without returning its values. Network observation requires the same network
namespace as the surviving runtime. Use SSH on that namespace's host or an
appropriately isolated management sidecar with access to that runtime's procfs;
a host namespace observing a different Pod network does not qualify.

Use literal IP addresses in tested network endpoints. This prevents changing
DNS answers from substituting a different service. Preserve valid configured TLS
identity and trust for that address. The probe derives persistence endpoints
from the actual retained JSON config and chooses predecessor controller endpoints
by the exact configured controller ID, including `recovery_peers`. It never takes
an arbitrary callback, probe command, query string, SQL mutation or asserted
passing status. It attempts three real TCP connections and retains each outcome.
A TCP connection does not establish service health; actual authenticated runtime
API transitions provide the corresponding behavior observation.

SSH uses pinned host keys, batch authentication, no dynamic command content,
and a fixed streamed observer. Controller SQLite is opened read-only and queried
in one read transaction. Only exact registration, immutable domain observation
and matching challenge/receipt rows are returned; config secrets, certificate
private material, environment values and raw controller errors are omitted.
Evidence and continuations remain private and bind the complete source identity
and private case hash across stages.

## Controller restart

1. Complete an actual native reboot or CRI exclusion case first. Use that retained
   predecessor, its real child trace and its genuine controller exclusion.
2. `restart-prepare` creates and consumes a real host challenge, observes its
   exact controller journal row and the live controller process identity, and
   records a blocked continuation. This establishes a successful baseline.
3. The operator restarts **only the dedicated controller service**. Preserve its
   config, executable, journal, source and machine boot. Refresh its PID file from
   the real service manager; do not write a fabricated controller observation.
4. `restart-complete` requires a different live controller process start identity
   on the same machine/boot and unchanged exact journal contents. A fresh actual
   challenge must reuse the original domain observation, commit successfully,
   and reject replay. The fixture effect count must remain unchanged.

This stage proves durable controller evidence survives a real process restart;
it does not treat a newly empty controller database as acceptable.

## Persistence network outage

1. Prepare an unexcluded predecessor that can actually be excluded: an old boot
   on the same native machine, or an authorized real CRI containment case.
   The surviving runtime must use networked durable SurrealDB. An embedded
   `surrealkv://` endpoint is unsupported by this network-fault stage.
2. `persistence-prepare` measures connectivity from the live runtime's namespace,
   creates a real pending durable challenge, confirms no controller exclusion
   exists, and saves a blocked continuation.
3. The operator stops the dedicated database service or partitions the
   runtime/database path. Keep the surviving runtime and independent controller
   observer alive. No fixture edits database rows or induces a fake SQL error.
4. `persistence-offline` requires three failed endpoint connections, the same
   surviving runtime process, HTTP 409 on exclusion of the existing challenge,
   and HTTP 409 on new challenge creation. The exact controller journal must
   remain unchanged. Connectivity must still fail after those requests. This
   stage remains blocked awaiting restoration.
5. Restore the dedicated database/network using the operator's normal procedure.
6. `persistence-restored` requires successful endpoint connections from the same
   runtime and unchanged pre-exclusion controller journal. The **original**
   pending challenge must now commit and reject replay. Its exact controller
   receipt and stopped child trace are retained. This detects an outage that
   improperly consumed that challenge or manufactured physical proof.

The host API currently maps supervisor persistence failures to HTTP 409; 409
alone is not outage evidence. Network/process observations and the successful
restored transition are required. This does not claim to cover a crash between
physical controller commit and runtime database commit; that precise timing
requires a separate deterministic real-service mechanism.

## Unreachable predecessor controller

1. Use a dedicated CRI predecessor node and a separate surviving runtime with its
   exact predecessor controller configured as a recovery peer. Keep management
   SSH available independently of the runtime/controller path.
2. `predecessor_controller-prepare` requires an advancing real detached-child
   heartbeat, an unexcluded registered CRI domain, successful endpoint
   connections from the surviving runtime and a real pending challenge.
3. The operator partitions the surviving runtime from that node's configured
   controller endpoint. Do not delete the Pod, fabricate an exclusion row or
   replace the endpoint with another node.
4. `predecessor_controller-offline` measures failed endpoint connections on both
   sides of the actual HTTP 409 exclusion attempt. Independent management
   observation must still see the same descendant executing; the controller
   journal and fixture effect count must remain unchanged. The stage stays
   blocked pending restoration.
5. Restore the path. `predecessor_controller-restored` requires the same surviving
   runtime and consumes the original pending challenge through the real CRI
   exclusion path. It retains the exact controller receipt and stopped child
   trace, and rejects challenge replay. It does not restore kubelet or remove
   inhibition; only the operator may resolve that node after retaining evidence.

## Missing stale executor mechanism

The `stale-executor` stage always returns `Blocked` with the precise gap. Source
contracts are in `save_tool_admission_evidence` in
`src/uar/persistence/providers/surreal.rs`, and checkpoint saves/claims in
`src/uar/persistence/providers/checkpoint_records/surreal.rs`. They enforce closed
runtime epochs internally. Recovery routes in `src/uar/api/supervisor.rs` expose
only challenge creation and exclusion; ordinary resume routes create execution
from current trusted host authority. None provides a legitimate external
submission of an old internal executor identity.

A future qualifying scenario needs a real production collaborator that can
retain an authentic stale executor request across physical fencing and submit
that already-authenticated request afterward, with independent observations of
the database rejection and no effect. It must not accept caller-forged runtime
epochs or mutate protected rows. Until such a mechanism is supplied and used,
full mechanism qualification must retain the missing stale-executor result;
these other stages cannot close that requirement.

## Authoring evidence and scope

Only source inspection and source edits were performed. No Python invocation,
probe, syntax compilation, test, linter, build, controller request, network fault
or SSH command was executed. Runtime portability, request status behavior,
service management, persistence timing and all scenario outcomes remain
unverified. No new production endpoint, admission authority, database migration,
controller setting or arbitrary operator callback was added. The guards protect
real boundaries: immutable source/process identity, TLS/SSH authority, exact
predecessor selection, network namespace, durable evidence and no false success.

## Upload commit and accounting settlement outage (matrix 27/29)

`storage-exercise` and `storage-cold` add actual commit/settlement scenarios.
They use the same `supervisor.fault_cases` dispatch and immutable case/source
binding. They do not require a physical exclusion transition or mutate the
controller journal. They do require networked SurrealDB, the existing runtime
observer and a real controlled provider that stays alive across operator steps.
Embedded SurrealKV disk-failure qualification remains missing; running these
network cases does not certify that different failure mechanism.

Add `storage_faults` to the private case:

```json
{
  "storage_faults": {
    "host": {
      "base_url": "http://127.0.0.1:8080",
      "workspace": "actual-isolated-workspace",
      "credentials": {
        "owner": "DOT_STORAGE_OWNER_JWT",
        "host": "DOT_STORAGE_HOST_JWT"
      }
    },
    "artifact": {"id": "replace-with-complete-admitted-artifact"},
    "run_credentials": [],
    "provider_control": {
      "base_url": "http://127.0.0.1:9100",
      "token_env": "DOT_PROVIDER_CONTROL_TOKEN",
      "trace": "/operator-private/actual-provider.jsonl"
    }
  }
}
```

This abbreviated artifact is illustrative and not runnable. Supply a complete
valid artifact and the actual scoped run credentials using the same request
contract as the other installed cases. Its selected provider must be the
controlled upstream peer, with a catalog-backed bounded model so the initial
call has a durable reservation. The peer uses `scripts/dot/provider.py`'s
standalone inventory-gated CLI with a new trace and private control token; it
serves real HTTP requests, emits text, then holds terminal usage on its
`settlement_barrier`. The fixture's authenticated loopback control endpoints
only release that peer's own bytes. They confer no UAR approval, admission or
accounting authority. Do not use a proxy or a manually written trace.

This stage attaches to an already running dedicated UAR service; its `Host`
wrapper performs API I/O only and never launches or stops that service. Keep all
unrelated work off the deployment's accounting scope. The runner must share the
observed runtime's Linux machine and network namespace. `server.host` must be
explicitly `127.0.0.1`; the observer proves its configured TCP listening socket
belongs to the exact live runtime PID via procfs socket inodes. Thus another
process or reverse proxy cannot emit a qualifying `100 Continue` response.
`resilience.timeout_disabled` must explicitly be `true` in this dedicated case's
private config to accommodate the operator interval; the fixture does not change
it. Server/resilience environment overrides and a `--port` CLI override are
refused. Provision appropriate real run/provider deadlines; expiry is a blocked
scenario rather than a settlement pass. Each operator transition has a bounded
ten-minute wait, and the provider barrier itself has a thirty-minute lifetime.

`storage-exercise` performs this concrete sequence in one surviving coordinator
process so the real upload socket remains open:

1. Observe reachable persistence from the actual runtime network namespace.
   Create a new KB through the production API and confirm its document list is
   empty. Admit a real run whose controlled upstream sends a unique text marker,
   then holds terminal usage. Read the exact real pending accounting call and
   deployment reservation while the database is still available.
2. Open a direct HTTP/1 upload with valid multipart headers and
   `Expect: 100-continue`, withholding **all** body bytes. Require the actual
   `100 Continue`. In the pinned stack, Axum constructs its multipart extractor
   without polling the body; the repository upload handler reads and authorizes
   the KB before calling `next_field`. Hyper waits for body demand before sending
   the provisional response. This establishes the initial KB read happened
   before the fault, so an early KB lookup failure cannot masquerade as failed
   source/job commit. This inference depends on the pinned source and direct
   observed socket; a missing provisional response blocks the case.
   [Axum 0.8.9 source](https://docs.rs/axum/0.8.9/src/axum/extract/multipart.rs.html),
   [Hyper 1.11.0 body demand](https://docs.rs/hyper/1.11.0/src/hyper/body/incoming.rs.html).
3. Retain the `NAME-storage-armed` observation and prompt the operator to stop
   the dedicated database or partition its runtime path. Independently wait for
   three failed endpoint connections with the same live runtime identity. Do
   not stop the runtime or provider. This script never changes the network.
4. Send the already authorized upload's complete valid body. Require HTTP 500
   from the existing durable-ingestion commit error mapping. HTTP 202, an early
   auth/KB error, timeout, disconnect or proxy error does not qualify. Release
   the actual upstream's terminal usage. Independently retain its fsynced
   five-token terminal transmission and exactly one dispatched request.
5. Observe preserved actual text and public `uar.accounting:unresolved`
   artifacts naming the exact pending call ID. The runtime must terminate the
   run without treating it as cancelled. Its final native status is retained
   rather than equating execution text with successful persistence of every
   runtime cleanup record. Accounting inspection must return the actual
   `accounting_storage_unavailable` response. A second new admission must fail
   before any provider request bearing its unique marker. Reconfirm the same
   persistence outage after those observations.
6. Prompt the operator to restore persistence, independently measure successful
   connections, and require the original exact call to remain pending with the
   same deployment reservation. Do not call reconciliation or invent provider
   charge evidence. Require the failed upload's KB document list and real
   scoped vector-search results to remain empty. This needs the configured
   production embedding/search path; unavailable search is not a pass.
7. Retain an immutable `storage-exercise` continuation and return **Blocked**
   pending an actual dedicated runtime restart. `storage-cold` observes a new
   process start identity with unchanged source/config on the same machine/boot,
   reads the pending call and reservation from restored persistence, and repeats
   the empty document/search observations. It also confirms the denied admission
   was not replayed by the independent provider trace. This stage launches no
   new logical run after restart and makes no supervisor-exclusion claim.

The terminal observations are `NAME-storage-cold`, plus the retained
`NAME-storage-armed`, `NAME-storage-offline-network`, `NAME-storage-failures`,
`NAME-storage-restored-network` and `NAME-storage-exercise` records. All are
actual runtime/peer/process/network observations when executed; no passing
receipts currently exist. Runner reconciliation must retain the local embedded
storage coverage gap rather than treating a network outage as a local disk fault.

The source-only extension added no production hooks or public claim-write API.
Only the fault helper, its independent read-only probe, and this document were
edited for this packet. Context7 plus exact pinned crate source were consulted
for multipart/body-demand behavior. No scripts, probes, tests, checks, formatters,
builds, HTTP requests against UAR, or product reviews were executed.

## Embedded SurrealKV commit and settlement failure (matrix 27/29)

The embedded source fixture now uses the existing `storage-exercise` and
`storage-cold` stages with `storage_faults.mechanism: "embedded_enospc"`.
This supersedes the earlier statement that embedded disk-failure source coverage
is missing. It does not assert a passing result. No resources were provisioned,
no filesystem was filled, and no stage ran.

This profile runs the actual installed production UAR binary with an actual
`surrealkv:///absolute/directory` store inside a dedicated disposable **Linux
guest**, hosted on macOS or Linux. It qualifies the embedded KV failure
mechanism in that guest. It does **not** qualify a native macOS binary's disk
failure handling. The ordinary local macOS installation and LKE API scenarios
remain separate runner profiles. An unrecognized guest platform or inaccessible
process/filesystem metadata blocks this scenario; there is no portable fake
fallback. Windows and native macOS fault observation remain unsupported.

Provisioning and disruptive authorization are operator steps outside this
fixture. Use a preallocated, bounded virtual block volume that belongs only to
the dedicated guest, with host capacity reserved beforehand. Never point this
case at the current workstation disk, a shared data volume, guest root, a thin
pool without reserved capacity, or a production cluster PVC. This fixture only
observes the volume and prompts for operator actions; it never mounts, fills,
resizes, remounts, changes permissions, detaches or repairs a filesystem. Absence
of provisioned resources leaves execution pending, not source implementation
missing.

Add the following complete observation contract under `storage_faults`; the
other host/artifact/real-provider fields remain those above:

```json
{
  "mechanism": "embedded_enospc",
  "embedded_volume": {
    "dedicated_disposable_guest_volume": true,
    "mountpoint": "/var/lib/uar/dot17-volume/case-name",
    "device": "/dev/vdb",
    "filesystem_uuid": "actual-mounted-filesystem-uuid",
    "maximum_bytes": 1073741824,
    "runtime_log": "/operator-private/runtime.jsonl"
  }
}
```

These are input declarations, never accepted as passing outcomes. The independent
observer requires Linux VM identity from DMI or supported device-tree values,
the actual runtime's UID and PID/start/binary/config identity, and matching mount
and network namespaces. It resolves the actual configured database directory,
checks its device against an exact whole-filesystem mount and the specified
`/dev/disk/by-uuid` target, and requires a separate ext4/xfs block device between
32 MiB and at most 2 GiB. Aliased mounts, nested mounts and guest-root device
reuse are refused. At least one actual writable runtime descriptor must point
inside that SurrealKV directory on the observed device. The runtime must use a
nonroot UID. The observer reads statvfs three times; it does not infer a commit
outcome from those measurements.

Use `UAR_SERVER__LOG_FORMAT=json` and `UAR_LOG_FILE` set to the exact
`runtime_log` path. That one log-format environment override is allowed; other
server, resilience, persistence and supervisor overrides remain refused. The
log must be on a different filesystem and owned through a live descriptor by
the observed runtime. Keep the provider, its trace, the runtime log, coordinator
continuations and evidence off the fault volume. The runner/coordinator executes
in the guest's same machine/network namespace, with pinned SSH observation of
that guest and the existing direct loopback runtime socket proof. Runtime log
rotation/truncation, an altered mount or a changed process blocks the live case.

The actual failure sequence is:

1. With available volume space, create the real KB, admit the real model call,
   retain its durable pending reservation and hold the controlled upstream's
   terminal usage. Arm the actual direct multipart request at Hyper's first
   body demand (`100 Continue`) after the production KB authorization read.
2. Prompt the operator to consume free space **only on the preprovisioned
   disposable guest volume**, using their normal bounded guest procedure.
   Require three observations of zero bytes available to the runtime UID on the
   unchanged volume and unchanged live runtime. This observation alone earns no
   commit-failure evidence.
3. Send the valid withheld upload body. Require HTTP 500 with an actual
   `ENOSPC`, `os error 28`, or `No space left on device` error from the production
   ingestion transaction. A generic 500, connection failure, an unrelated
   permission error, or HTTP 202 does not qualify. The response digest and
   positively observed error category are retained.
4. Release the already-admitted upstream's real five-token terminal usage.
   Require the public unresolved-accounting artifact for the exact call ID,
   preserved text, the actual native run terminal status, and the runtime's
   JSON warning `Execution completed with unresolved durable accounting` for
   that same call ID with an actual ENOSPC error. Only new log bytes since the
   retained baseline are considered; their inode/device and line hashes are
   bound to the runtime. A usage warning without the terminal transaction
   failure, a filesystem measurement, or an operator-written receipt is
   insufficient. The fixture never upgrades an `error` or unknown cleanup
   outcome into successful execution merely because provider text arrived.
5. Embedded reads may still succeed while writes fail. Accounting inspection
   must either return the unchanged pending admission or the actual storage
   unavailable error. Require a new admission to fail before independent
   provider dispatch and reconfirm exhaustion. A storage engine that commits
   these writes despite the measured full filesystem fails the scenario;
   preallocated engine files and delayed writes are not excused.
6. Prompt the operator to restore space by removing only their disposable fill
   material. Require the same actual mount/process with available bytes, then
   the unchanged durable pending call and reservation and empty document/search
   results. If the storage engine cannot serve those real reads after space
   restoration, the stage remains blocked; it cannot invent recovery success.
7. Save the immutable continuation and return Blocked for an actual dedicated
   runtime restart. `storage-cold` must observe a different real process start
   on the same guest boot, the same installed binary/config and database volume,
   unchanged pending liability, no uploaded document/indexed ghost, and no
   provider replay. It starts no logical run and grants no recovery authority.

Additional retained observations are `NAME-storage-exhausted-volume`,
`NAME-storage-enospc-transactions`, and `NAME-storage-restored-volume`; the common
armed/failures/exercise/cold records remain unchanged. Error text from actual
transactions is private evidence. The observer does not reveal arbitrary logs,
secret environment values, or database contents. Only matching call-ID warnings
from the bounded new log span are returned.

This implementation covers a real ENOSPC route only. Quota, permissions,
read-only remount, device loss, power loss, native macOS disk faults and platform
portability are not implied. All runtime outcomes, platform introspection,
migrations, timing and error propagation remain unverified until the lead opens
the full-phase gate and the operator provisions and authorizes the dedicated
resources.

## Original Pod disappears while its unreachable predecessor keeps executing (matrix 35)

The `predecessor_controller-*` source stages now require original **Pod API
object disappearance**, in addition to the measured surviving-runtime/controller
partition. This extends the earlier partition description; an ordinary controller
network outage with a still-present Pod object cannot satisfy this case.

The operator must explicitly set `authorize_original_pod_deletion: true` along
with the existing dedicated-node physical-exclusion and node-inhibition
permissions. No fixture deletes the Pod. After `predecessor_controller-prepare`
retains the exact original UID, the operator isolates the dedicated predecessor
so it cannot react to the Pod removal through its normal kubelet control path,
and removes only that exact isolated Pod API object. Independent management
access for procfs/CRI/cgroup/child observation and the API GET must remain
available. Do not use this case on a shared node. No partition, Pod deletion,
node operation or runtime request was executed during authoring.

The read-only fault observer derives the namespace, original UID and deployment
from the controller's actual durable registration, verifies the retained private
Pod path against that deployment's actual configured Pod name, and issues only
an authenticated HTTPS **GET** using the controller's pinned config
`kubernetes_api`, `kubernetes_token` and optional `kubernetes_ca`. Redirects and
proxies are refused. The result retains only UID/resource-version/deletion-state
and the API/CA binding; it never returns the token or arbitrary Pod contents.
The controller SQLite connection remains read-only.

Preparation requires HTTP 200 with the original UID and no deletion timestamp.
The offline stage requires a real API 404 or a same-name Pod with a different
UID. A terminating object with the original UID is insufficient. An API timeout,
TLS failure, 401/403 or other unavailable observation is Blocked, not object
absence. The API endpoint, CA digest, namespace/name, cluster and original UID
must remain bound to the preparation record.

While the original API object is gone, independent node observations must still
show the original ready CRI sandbox, exactly the original running container IDs,
the retained populated Pod cgroup inode/device and the same detached child with
an advancing heartbeat. The surviving runtime's real exclusion request must
fail with HTTP 409 during its measured controller partition, and no controller
proof or new effect may appear. The offline continuation remains blocked pending
restoration; disappearance never grants effect replay or admission authority.

Restoration must still observe the original API object gone before the actual
host exclusion request. The production controller already loads the retained
registration and private cgroup/CRI identities: it inhibits kubelet first,
freezes the retained Pod cgroup, tolerates only API deletion 404/409, then kills
and independently observes the old cgroup and CRI identities. The fixture
requires the genuine retained exclusion receipt, kubelet empty/masked proof,
independently inactive masked kubelet, exact stopped original sandbox/containers
and stopped original child. A replacement Pod UID is never substituted for the
original physical domain. Node inhibition remains in place for the operator to
resolve; the fixture does not restart kubelet.

A whole predecessor host inaccessible over **all** management channels remains
unqualified by this mechanism. That limitation is explicit: no node-power proof,
old-executor write or CRI status is fabricated when the real observer cannot
reach it. Source implementation does not imply deployed resources or passing
matrix-35 evidence.

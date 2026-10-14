# Supervisor recovery

Dot11 implements trusted execution exclusion before abrupt owner recovery. The
operator approved **reboot-bound native recovery** where the shipped service
manager cannot prove same-boot descendant containment. A launchd/systemd/SCM
restart alone does not permit takeover. Unknown effects and budget liabilities
remain unresolved after exclusion.

## Production mechanisms

`src/uar/runtime/supervisor.rs` is the trusted UAR client. The shipped independent
controller is `scripts/supervisor/controller.py`; it uses Python 3.10+ standard
library only. UAR does not spawn this controller and an agent cannot configure
its endpoint or upload exclusion receipts. The operator provisions its private
configuration, independent lifecycle, TLS identities, retained qualification
record, and durable SQLite journal. No controller or runtime qualification has
been executed for this change yet.

The two implemented mechanisms are:

* `native-boot-exclusion-v1`: macOS LaunchAgent
  `com.prometheus.universal-agent-runtime`, Linux `uar.service`, and Windows SCM
  `PrometheusUniversalAgentRuntime`. Registration records the root's process
  start identity and the controller's machine and boot identities. A later
  controller observation must identify the same machine and a different kernel
  boot. macOS uses IOPlatformUUID and `kern.bootsessionuuid`; Linux uses machine
  identity and `/proc/sys/kernel/random/boot_id`; Windows uses hardware UUID and
  the native boot-environment GUID. Same-boot crashes always refuse reclaim in
  this profile, including a surviving setsid/detached child. The controller
  **never initiates a reboot**. An operator-authorized reboot is separate.
* `kubernetes-cri-containment-v1`: an independently supervised controller on the
  Linux node, outside the UAR Pods, with the node's qualified containerd/CRI
  endpoint and cgroup v2. Registration binds cluster, namespace, Pod UID, node
  UID/boot, sandbox ID, exact container IDs and retained cgroup device/inode.
  It checks an unprivileged workload: no host PID/IPC or hostPath, no additional
  capabilities, `drop: [ALL]`, and `allowPrivilegeEscalation: false` on every
  container. Static/mirror Pods are refused. The qualified profile requires
  systemd-managed `kubelet.service` as the sole workload restart authority and
  no other CRI clients that create workload containers. It records the kubelet's
  exact cgroup identity too. Recovery freezes that retained kubelet cgroup,
  runtime-masks its unit, kills and observes its subtree empty, and confirms the
  service inactive with no main PID. It retains that node inhibition in the
  durable controller journal. Recovery then freezes the original Pod cgroup,
  requests UID-preconditioned
  Pod deletion to inhibit restart, writes `cgroup.kill`, observes the original
  subtree empty, stops the exact CRI sandbox and verifies retained stopped
  container/sandbox states. Frozen/empty cgroup observations, not deletion, prove
  old process exclusion. Missing cgroups, GC'd CRI records, changed node boots,
  incompatible verbose CRI PID data and unreachable nodes refuse recovery.
  Containers must be direct children of the same Pod cgroup. A deployment with
  incompatible runtime layout remains unqualified.

  **This opt-in Kubernetes mechanism suspends scheduling on the entire affected
  node until operator recovery.** The runtime mask is never automatically
  removed. The controller is a separate host systemd service outside the kubelet
  and Pod cgroups; the recovery runner uses a surviving Pod or another node.
  Normal Pod deletion and ordinary Kubernetes deployments do not qualify.
  After resolving the old Pod UID and retained unknown effects, the operator
  can deliberately restore the kubelet through the deployment runbook. A failed
  exclusion may leave kubelet frozen/masked even when no receipt was committed;
  that is a blocked recovery, never permission to transfer ownership.

There is no generic power-fencing assertion API and no same-boot native Job
Object/EndpointSecurity implementation in this release. A Kubernetes node whose
termination cannot be observed stays blocked. An actual independently verified
infrastructure fencing adapter is future work, not implied by a signed string.
Both required storage profiles remain local durable SurrealKV and explicitly
qualified shared networked SurrealDB. PostgreSQL checkpoint/supervisor recovery
remains unsupported; its existing accounting and credential behavior is unchanged.

The uncomfortable limit: loss of the independent controller journal, container
runtime garbage collection before observation, or an unreachable predecessor
node can leave ownership blocked even when an operator believes the process is
dead. Do not erase ownership, lower epochs, rewrite the journal, or reinterpret
timeout/API absence as success. Cloned machine identities, VM snapshot rollback,
shared client private keys and writable qualification files are outside the
qualified deployment contract.

## Installation and configuration

Templates are under `deploy/supervisor/`. They are intentionally unqualified and
contain no certificates or accepted qualification digests. Installation and
service start are operator deployment actions; none were run during authoring.

1. Install the exact controller source and an existing Python 3.10+ runtime in
   an administrator-owned directory. Create a separate private controller data
   directory; preserve `exclusions.sqlite3` across UAR and controller restarts.
   Protect controller configuration, source, journal and qualification records
   from the runtime user and every governed workspace. On Windows use NTFS ACLs;
   on Unix use root ownership and a private umask.
2. Issue separate controller-server and runtime-client certificates. Use a
   dedicated controller CA whose private key is unavailable to UAR. Server SANs
   must match the configured HTTPS endpoint. The controller requires client TLS
   certificates and an explicit DER SHA-256 fingerprint allowlist. Each client
   gets one `registration_deployment`, a fixed set of recovery deployments, and
   explicit `register`/`exclude` actions. Rotating certificates requires operator
   configuration and retained journal continuity; do not copy one client key
   among distinct Pod identities.
3. Populate `controller.example.json`. For Linux change `service_name` to
   `uar.service`; for Windows use `PrometheusUniversalAgentRuntime`. On native
   Unix, `runtime_uid` is the actual service UID. Paths are operator-selected
   absolute locations; no example UID or certificate digest is authority.
4. Qualify the exact mechanism/platform/controller bytes at the full phase
   boundary. The private qualification file contains `controller_id`,
   `qualification_id`, `mechanism`, `controller_sha256`, and
   `production_qualified: true`, plus retained evidence references. Bind the
   exact file digest as `qualification_sha256` in controller configuration.
   Only a completed real-platform gate authorizes this production record. A
   candidate fixture uses the explicit `--qualification-candidate` controller
   startup option in an isolated dedicated deployment. Its exact-hash record
   must set `production_qualified: false` and `isolated_candidate: true`; the
   qualification ID must start with `dot17-candidate-` and all deployment names
   with `dot17-`. The same mechanism, TLS scopes and journal operations run.
   This option does not create, modify or promote a qualification record. Use
   separate candidate certificates, journals and deployment configuration.
   Ordinary production startup still rejects an unqualified record. This
   explicit bootstrap avoids claiming a platform passed before observing it.
5. Linux: install `uar-supervisor.service` with the configured paths. macOS:
   install `com.prometheus.uar-supervisor.plist` as an independent LaunchDaemon;
   adjust the installed Python path and private configuration path. Windows:
   the provided `install-windows.ps1` creates `UARSupervisor` with the controller's
   built-in ctypes SCM dispatcher; supply absolute installed Python/controller/
   configuration paths. It creates the service without starting or rebooting it.
6. Add the contents of `runtime.example.json` under `supervisor_recovery:` in
   UAR's operator configuration. The existing configuration loader reads this
   optional object. With no object, abrupt recovery is unavailable. An opted-in
   configuration must register successfully **before** team/observer execution
   startup; a controller failure aborts startup. The runtime reads certificate
   files internally; PEM bytes never enter API responses or persistence.

For Kubernetes, install the controller with the host's systemd service and the
`kubernetes-node.conf` drop-in; it needs the real host `/proc`, cgroup hierarchy
and CRI socket. Use `kubernetes-controller.example.json` and
`kubernetes-rbac.yaml`. Provision/rotate the scoped Kubernetes API token outside
UAR and keep it private to the controller. Narrow the example namespace and
node-read role for the deployment. Configure exact Pod UIDs before registering
those runtimes; a recreated Pod requires a new binding and certificate scope.
For the required two-runtime shared qualification, give A and B distinct client
certificates and deployment IDs on the qualified node controller. A's/B's runtime
`authorized_recovery_deployments` explicitly includes the other deployment.
Unreachable other-node controllers do not silently route through the live node.
Multi-node installations need a separately configured trusted controller per
node. Add each other controller as a `recovery_peers` entry with the same fields
as the primary runtime controller configuration and an empty `recovery_peers`.
Each peer has its own explicit endpoint, server CA, runtime client identity,
qualification and authorized deployments. At most 64 peers are accepted;
duplicate controller IDs and nested peers are refused. Persisted registration
controller IDs select this fixed map; URLs never come from Pod metadata. The
primary endpoint alone registers this runtime. A missing or unreachable peer
keeps its predecessor excluded from recovery rather than transferring the
request to another node.
The configured `systemctl` executable must be an administrator-owned absolute
path and `kubelet_unit` must be `kubelet.service`. Qualification must establish
that stopping this unit leaves the independent host controller and recovery
runner available. It must also establish no alternate controller can restart
the old Pod through CRI. Restoring the kubelet or altering its mask is an
operator action that ends the inhibited-node interval; do not resume recovery
from that interval after changing its trust assumptions.

## Host API and state transitions

Only a verified installation/service host principal can call these APIs;
an ordinary admin role is insufficient. Use the existing host authentication
mechanism, never a new `host=true` field. All responses omit credentials and
controller failure internals.

`POST /api/uar/recovery/supervisor/challenges` accepts:

```json
{"runtime_epoch":"previous-runtime-uuid","target":{"kind":"checkpoint_epoch","runtime_epoch":"previous-runtime-uuid"}}
```

For a team owner use `target.kind = "team_owner"` with its exact existing `fence`
(camelCase catalog/service/incarnation fields and epoch). The runtime epoch must
resolve a prior durable registration whose team incarnation matches that fence.
Capture these identities from the protected checkpoint envelope and existing
catalog ownership view before an abrupt-stop qualification. Callers cannot
choose a process/PID/Pod to kill independently of that registration.

`POST /api/uar/recovery/supervisor/challenges/{id}/exclude` obtains the actual
controller observation and validates its complete challenge/domain identity.
It then atomically consumes challenge generation 1, advances it to 2, records
an immutable exclusion and closes the predecessor runtime epoch. A consumed
challenge is refused on replay. The team path additionally stages complete
challenge/observation audit strings in the existing catalog; the existing
execution-owner reclaim endpoint performs the actual epoch CAS and transfers
only queued attempts. It records running/cancelling work and corresponding team
tasks as uncertain/blocked with `reserved-unknown` accounting. Catalog evidence
is retained; the reclaim receipt is its single-use consumption marker.

Exclusion does **not** start a run. The response explicitly says
`effects_reconciled: false` and `execution_resumed: false`. Continue through the
ordinary authenticated checkpoint or approval resume API, or the existing
team reclaim command. Fresh credentials, principal/policy checks and approval
reauthorization still apply. A2A resumes still require their peer contract.

Ordinary checkpoint claims accept an excluded source only after durable effect
and budget checks. A crashed continuation can be replaced only after its own
executor was supervisor-excluded; the old exact claim is compared atomically,
archived and replaced at a strictly higher stored generation. Unknown admission
outcomes, claims without terminal evidence, unknown canonical receipts and
pending budget calls (including durable child run IDs) block continuation.
No exclusion transition releases money or replays a pending effect. Old runtime
ClaimIntent/AwaitingApproval writes and checkpoint saves are rejected after
closure. Orderly shutdown cannot downgrade supervisor exclusion into an
apparent effects-resolved state.

A crash between runtime-epoch closure and team-catalog staging leaves takeover
blocked. Create a fresh challenge for the unchanged predecessor fence; the
controller may rebind its already committed immutable domain observation to
that new challenge. It never substitutes a service status observation. A crash
before the controller journals an observation can require renewed physical
observation and may remain blocked after garbage collection.

## Deferred real-platform qualification

No test, check, build, formatter, validator, source-review gate or controller
probe was executed during implementation. Run only after the lead records the
full expanded Dot production inventory, serially with the other real-path gates.

Qualification must include each native platform separately (including actual
reboot and surviving detached descendants), and two real UAR Pods with shared
durable SurrealDB. Retain source identities, executable identities, service/Pod/
node/CRI/cgroup identities, observed root+descendant exits, host API responses,
durable challenge/exclusion/claim records and resumed effect traces. Include
same-boot native refusal, wrong client certificate, cross-deployment registration,
wrong incarnation/epoch, receipt replay, competing ordinary resumes, safe queued
team transfer, stale executor claim attempts, unknown effects/pending budget
liability, controller restart and persistence outage. Pod deletion on an
unreachable node must remain blocked. No mock-only or receipt-schema test can
qualify a mechanism.

The authored HTTP fixture `scripts/supervisor/recovery_scenarios.py` exercises
those host API transitions against operator-prepared real native/shared cases;
it neither invents predecessor exclusions nor reboots/kills a host. The platform
runner must provide actual stopped/rebooted environments and inspect persistent
records and effect traces around it. Its HTTP results alone are not whole-phase
or controller qualification.
`deploy/supervisor/recovery-scenarios.example.json` describes its inputs with
nonfunctional placeholder identities. Materialize separate native/shared cases
from real retained runtime records; credentials are environment variable names
and private resume bodies are files outside governed workspaces. No fixture
configuration or script has been executed during authoring.

Mechanism references: [Linux cgroup v2 kill/populated](https://docs.kernel.org/admin-guide/cgroup-v2.html),
[CRI tools](https://github.com/kubernetes-sigs/cri-tools/blob/master/docs/crictl.md),
[systemd unit masking](https://www.freedesktop.org/software/systemd/man/latest/systemctl.html#mask%20UNIT%E2%80%A6),
[Kubernetes deletion preconditions](https://kubernetes.io/docs/reference/kubernetes-api/definitions/delete-options-v1-meta/),
[Windows native boot information definitions](https://github.com/winsiderss/phnt/blob/master/ntexapi.h),
[Windows native-query compatibility contract](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntquerysysteminformation),
[SCM dispatcher](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-startservicectrldispatcherw),
and [Apple kernel sysctl source](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_sysctl.c).


## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.


### Linux candidate executable from the OCI build

The local OCI build exports the actual regular ELF runtime executable from each verified image filesystem. Its `local-two-architecture-image` observation records `source_content_sha256` and `runtime_binaries` for amd64 and arm64, including each executable path, SHA-256, size, architecture and image digest. These are candidate artifacts; no publication is claimed.

For Linux guest qualification, install the selected arm64 executable and run the source-bound acceptance driver inside that guest. Copy the genuine same-inventory OCI receipt, observation, archive and exported executable without relabeling their paths or results. The host configuration selects `candidate_runtime_origin: "oci"`, `os: "linux"`, `architecture: "arm64"` and the genuine `oci_build_receipt`. The runner checks the receipt/source binding, archive, exported executable, actual installed executable bytes and current host architecture. The guest's installed path may differ from the export path. Native macOS candidate validation retains its existing local build receipt. Source/plan inventory and every operational stage still require their actual inputs and observations; exporting an executable proves no recovery behavior.

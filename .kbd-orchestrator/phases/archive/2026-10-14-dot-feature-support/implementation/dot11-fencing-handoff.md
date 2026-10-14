# Dot11 supervisor fencing — source handoff

Date: 2026-10-12. Phase: `dot-feature-support`, Execute. Author:
`dot_supervisor_fencing`. Shared dirty checkout; no commit or publish.

Production source, controller, migrations, configuration, operations documentation
and deferred HTTP fixture are authored. **This is not verification or platform
qualification.** No test, check, compiler, build, formatter, validator, probe,
service installation, controller execution or review gate was run. The lead
alone owns the full expanded Dot implementation boundary and canonical KBD.

## Approved scope and observed source problem

The operator explicitly approved reboot-bound native recovery where same-boot
descendant exclusion is unavailable. The implementation supports the existing
macOS launchd, Linux systemd and Windows SCM native deployments through actual
machine/kernel-boot observations. It never initiates a reboot. Same-boot native
service/PID exit is insufficient and remains blocked.

Before this change, `checkpoint_records/surreal.rs` admitted only an
`orderly_excluded` predecessor and could not recover a crashed continuation.
`team_execution/ownership.rs` accepted only `trusted-runtime-joined` evidence.
`process_supervisor.rs` put `None` in its owned child slot while the result held
the child, and shutdown did not await its exit. Those source paths reproduce
the missing abrupt-recovery behavior without executing a test.

The trusted host now registers a complete domain before execution, obtains a
one-use durable challenge, validates an mTLS controller observation against that
exact registration, consumes generation 1 to 2 atomically, then uses existing
checkpoint/team ownership CAS. Exclusion never settles effects, reauthorizes
approvals, releases budget liabilities or automatically launches a replacement.

## Actual controller and operational impact

`scripts/supervisor/controller.py` is the shipped independent Python 3.10+
standard-library controller. No imaginary external receipt service is required.
It has authenticated HTTPS, exact client-certificate fingerprints and deployment
scopes, durable SQLite FULL-synchronous registration/exclusion journals, bounded
request/response handling and fixed redacted errors. A retained operator
qualification file must bind controller ID, mechanism, qualification ID, exact
controller source hash and exact qualification file digest. There is no accepted
qualification record in this checkout; examples remain unqualified.

Native records use macOS IOPlatformUUID + `kern.bootsessionuuid`, Linux machine
ID + kernel boot ID, and Windows hardware UUID + native boot-environment GUID.
Only a different boot on the same registered machine excludes native roots and
descendants. LaunchDaemon/systemd/SCM deployment templates are supplied, including
the controller's own Windows SCM dispatcher and install script.

The Kubernetes mechanism is an **opt-in qualified node profile**, not generic
Pod deletion. It uses the host's actual containerd/CRI and cgroup v2. Registration
binds exact cluster/namespace/Pod UID/node UID/node boot/sandbox/container IDs and
retained cgroup device/inode. It refuses privileged/host-mount workloads and
static/mirror Pods. The profile requires systemd `kubelet.service` to be the sole
workload restart authority and excludes alternate CRI creators from its trust
contract.

Recovery first freezes the exact retained kubelet cgroup, runtime-masks its
systemd unit, kills and observes the subtree empty, observes the unit inactive
with no main PID, and journals that inhibition. It then freezes the original
Pod cgroup, deletes only the exact Pod UID, kills/observes the original subtree
empty, stops the exact CRI sandbox and observes retained stopped container and
sandbox records. No missing Pod/cgroup/CRI record is proof. Unreachable nodes,
changed node boots, incompatible CRI verbose PID data and missing original
domain records fail closed.

**Scheduling on the entire affected node remains blocked until operator
recovery.** The independent controller is a host service outside kubelet/Pod
cgroups; a surviving Pod or other node hosts the recovery runner. No automatic
unmask is implemented. A failed attempt may leave kubelet frozen/masked without
an exclusion receipt. Restoring kubelet or changing its mask ends the qualified
inhibited interval and requires an operator runbook; never infer proof from that
manual action. This operational impact was sent to the lead and documented.

The runtime supports up to 64 explicitly configured peer controllers with
separate endpoint/CA/client identity/qualification/deployment scope. Nested peers
and duplicate IDs are refused. A persisted controller ID selects this fixed map;
Pod metadata never supplies URLs. Primary registration always uses the primary
controller. Other-node unavailability stays blocked.

## Files changed by this worker

* `src/uar/runtime/supervisor.rs`: typed domain/config/challenge, sealed verified
  proof, mTLS client, explicit peer routing, registration/challenge/exclusion and
  checkpoint/team commit methods. Agent metadata cannot construct a verified
  proof; no agent/native-tool exclusion capability was added.
* `src/uar/runtime/mod.rs`: supervisor module export.
* `src/config.rs`: optional serde-defaulted `supervisor_recovery` configuration.
* `src/uar/persistence/mod.rs`: five default-unsupported supervisor persistence
  methods; preserves other workers' accounting/AGUI additions.
* `src/uar/persistence/providers/supervisor/surreal.rs`: immutable registration,
  pending challenges, atomic one-use consume/epoch closure/exclusion history for
  durable local SurrealKV and qualified shared SurrealDB. Registration idempotence
  is inside the transaction and cannot reopen a closed epoch.
* `migrations/surrealdb/supervisor_fencing.surql`: protected supervisor tables,
  nonexpiring challenge/evidence retention.
* `src/uar/persistence/providers/surreal_host.rs`: migration inclusion only;
  preserves later AGUI/accounting migration additions.
* `src/uar/persistence/providers/surreal.rs`: supervisor module/delegates; rejects
  old-epoch ClaimIntent/AwaitingApproval in the actual admission transaction.
  Scans all statement errors before idempotent evidence fallback so an earlier
  NotExecuted result cannot hide `supervisor_epoch_closed`.
* `src/uar/persistence/providers/checkpoint_records/surreal.rs`: claim history and
  monotonic stored generations; supervisor exclusion admission; exact prior-claim
  CAS; unresolved tool/canonical-effect/accounting barriers including durable
  child run IDs; source-state comparison in the CAS prevents a concurrently
  excluded predecessor bypassing its preflight effect check. Orderly shutdown
  cannot downgrade supervisor exclusion. Uses A2A worker's
  `require_claimable_execution()` while generic HTTP restore remains restricted.
* `src/uar/runtime/checkpoint_host.rs`: protected context allowlist additionally
  retains the A2A worker's secret-free `a2a_delegation` provenance.
* `src/uar/compiler/collaboration/team_execution/ownership.rs`: consumes sealed
  exact team-fence proof, catalog CAS stages audited supervisor evidence; reclaim
  accepts it once using retained reclaim receipts, transfers only queued work,
  blocks uncertain running/cancelling work and its tasks, preserves
  `reserved-unknown` accounting.
* `src/uar/domain/team_execution.rs`: optional persisted supervisor challenge and
  observation audit JSON on fencing evidence; no public proof constructor.
* `src/uar/orchestrator/process_supervisor.rs`: actual shared child ownership and
  awaited shutdown. Root-child join alone is explicitly not descendant proof.
* `scripts/supervisor/controller.py`: actual independent native/Kubernetes
  controller and Windows SCM dispatcher described above.
* `scripts/supervisor/recovery_scenarios.py`: authored, unexecuted real HTTP
  fixture for prepared real native/shared deployments, host authorization,
  wrong identity, physical exclusion, replay, competing resume, unknown effects
  and optional team reclaim assertions. No mock receipts or automatic reboot.
* `deploy/supervisor/controller.example.json`: unqualified native controller
  settings and client scopes.
* `deploy/supervisor/runtime.example.json`: trusted runtime client settings,
  optional peer controller list.
* `deploy/supervisor/kubernetes-controller.example.json`: qualified-node candidate
  configuration with explicit CRI/systemctl/node/Pod identities and two clients.
* `deploy/supervisor/uar-supervisor.service`: independent native host controller.
* `deploy/supervisor/kubernetes-node.conf`: qualified node service drop-in.
* `deploy/supervisor/com.prometheus.uar-supervisor.plist`: independent macOS
  LaunchDaemon template.
* `deploy/supervisor/install-windows.ps1`: explicit administrator install of
  independent SCM controller, no implicit service start or reboot.
* `deploy/supervisor/kubernetes-rbac.yaml`: scoped Pod read/delete and node read
  controller permissions.
* `deploy/supervisor/recovery-scenarios.example.json`: nonfunctional placeholder
  inputs for the deferred HTTP fixture; credentials only by environment names.
* `docs/operations/supervisor-recovery.md`: mechanisms, qualification/install
  contract, trust scopes, host API, effect preservation, failure behavior and
  deferred real-platform gate with primary-source references.
* This handoff file.

No dependency pins, vendored/submodule files, GitHub workflows or canonical
phase records were edited by this worker. No unrequested production subsystem
was added. All guards above address the explicit fencing requirement or the
real host/controller, persistence, credential or admission trust boundary.

## Lead-owned integration already authored

The lead owns `src/server.rs` and `src/uar/api/supervisor.rs`. Startup constructs
`SupervisorRecoveryHost::new(persistence, config)` and registers
`run_manager.runtime_epoch()` plus `catalog.execution_identity()?.incarnation_id`
before team execution/observer/instance execution begins. The lead added `None`
to the existing settings configuration fixture. The lead's API permits only
`trusted_host_id()` principals; ordinary administrators cannot submit proof.

* `POST /api/uar/recovery/supervisor/challenges`: persisted old runtime epoch and
  exact `RecoveryTarget::{CheckpointEpoch,TeamOwner}`.
* `POST /api/uar/recovery/supervisor/challenges/{id}/exclude`: authenticated
  controller observation, then checkpoint commit or **team commit before catalog
  record**. Response truthfully says effects not reconciled/execution not resumed.

The team transaction closes the old runtime epoch before catalog staging. A
crash between stores leaves authority blocked; a fresh challenge may rebind the
controller's immutable established domain observation to stage the unchanged
team fence. The existing reclaim command performs the catalog ownership CAS.
There is no blanket orchestrator rewrite or new executor.

Source changes for schedule/A2A/AGUI were coordinated directly. Schedule's
undispatched guard may transfer only after safe catalog takeover; dispatched or
unknown work stays blocked. Whole-native boot/exact-Pod exclusion includes local
instance roots but never claims a remote delegated effect has stopped.

## Remaining limitations and deferred evidence

No known production-authoring item in this assigned approved scope is left
unwritten. Rust compilation, SQL acceptance/transaction semantics, Python and
Windows API execution, service installation, actual reboot/descendant exclusion,
CRI/cgroup layouts, node scheduling effects, mTLS behavior, multi-node routing,
cross-runtime races, native/shared persistence, customer builds and all fixture
outcomes remain **unverified**. There is no qualified-platform success claim.

The uncomfortable limit remains fail-closed availability: lost controller
journal, unobservable predecessor nodes, early CRI garbage collection, changed
kubelet cgroup identity, or an unsupported container runtime can prevent recovery
even when an operator believes old work is dead. Native availability requires
an explicit host reboot; Kubernetes recovery can suspend an entire node.
PostgreSQL supervisor/checkpoint recovery is still unsupported, consistent with
the requested local SurrealKV/shared SurrealDB profiles. Unknown remote effects
and pending accounting liabilities survive every exclusion and continue blocking
unsafe ordinary resume.

Verification commands/results: **none executed**. Only source/document reads,
primary documentation retrieval and file authoring occurred. At the lead-owned
full-phase boundary, use the planned serial real-production native and shared
qualification, then the required customer build. The authored HTTP fixture alone
cannot establish platform qualification and must not promote a qualification
record. No passing gate or acceptance receipt exists in this handoff.

## Lead continuation: honest candidate qualification bootstrap

Source inspection found the ordinary startup requirement for a completed
qualification receipt prevented honest first-time qualification. The lead added
an explicit `--qualification-candidate` option to the controller. It requires an
exact-hash unqualified record (`production_qualified: false`,
`isolated_candidate: true`), a `dot17-candidate-` qualification ID and only
`dot17-` deployments. Ordinary startup still rejects that record. No controller
or runner creates or promotes a production receipt. Dedicated credentials,
journals and deployment configuration are documented. This is source authoring,
not a verification exception. Controller, native/CRI mechanisms and new Dot17
lifecycle drivers remain unexecuted and unqualified.

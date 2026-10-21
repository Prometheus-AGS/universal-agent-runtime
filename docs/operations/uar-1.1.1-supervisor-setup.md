# v1.1.1 isolated supervisor setup contract

This contract prepares the genuine supervisor evidence required by the v1.1.1
recovery group. It does not authorize a production deployment, a customer-data
mutation, paid infrastructure, or the acceptance runner. Guest-only reboots
have separate operator approval and remain deferred until the lead completes
the implementation inventory.

The selected mechanism is native Linux. The initial read-only inspection found
that the historical guest `dot17-linux-qualification` had been removed at the
Dot closeout. `~/.lima-dot17` then contained only retained SSH/network
configuration; there was no instance directory, VM definition, PID, or status
metadata. The locally installed Lima binary is
`/Users/gqadonis/.local/bin/limactl` (SHA-256
`4636491d6758a95211a5a857a07ddb6b1c034375bdede18a5bdae32643cbd553`); its
release version was not established from local metadata. Kubernetes context
`lke174619-ctx` is retained only as unrelated historical inventory and is not
part of this setup. No remote probe was made.

The retained October worker receipt names a different cluster and node,
`lke653426-988676-r7ndr`, with a cleanup deadline of 2026-10-15. It records
`production_qualified: false`, no started runtime, and no recovery faults. It
cannot be reused as a current identity or qualification receipt.

## Approved candidate setup — 2026-10-16

The operator approved dedicated local guest setup and guest-only reboots. The
lead reports that the replacement `dot17-linux-qualification` guest is created
and started with QEMU/aarch64, 4 vCPUs, 8 GiB RAM, and 24 GiB disk, with no host
mounts. The independent candidate controller is installed and its service is
enabled with private TLS and the configured service identity.

The frozen rc03 executable made one startup attempt and failed with
`supervisor controller is unavailable` before controller registration. At that
time the controller service was running, a native Python TLS 1.3 mTLS handshake
using the same CA and client identity succeeded, the controller journal had no
registrations, and the controller had no UAR request-transport trace. The
attempt is diagnostic setup evidence only: no guest reboot, containment
observation, team staging, or qualification result occurred. rc03 is superseded
for this purpose even though its earlier build completed.

Source inspection identifies a concrete candidate cause for that startup failure:
the UAR supervisor client loads a PEM identity, while the resolved reqwest
feature graph can select its default native backend unless the builder explicitly
selects rustls. The client now calls `use_rustls_tls()` before disabling builtin
roots and adding the dedicated CA. This preserves mTLS, CA-only trust, no proxy,
redirect refusal, the 60-second timeout, and cancellation behavior. A fresh
runtime start has not yet demonstrated that this was the live failure cause or
that registration now succeeds.

The native UAR runtime service identity is UID 65532 and group `uar` (GID 1001).
Private runtime configuration and TLS client material that this service must
read need that actual group under restrictive directory and file modes. The
controller server key and controller journal stay outside that group. This is a
configuration requirement, not qualification evidence.

The held setup uses `llm.timeout_secs: 600`, its configured upper bound, with the
existing provider delay of 601 seconds. Keep the existing 3,000,000 budgets and
3,000 reservations; this cohort does not justify shrinking either value.

The failed startup can retain an empty catalog claim before it reaches controller
registration. Preserve that durable store. The first successful predecessor must
use a fresh disposable database path; never clear or forge catalog rows to reuse
the failed attempt.

## Evidence required before v1.1.1 recovery acceptance

The recovery adapter reads these private fields from
`infrastructure.supervisor`:

| Field | Required binding |
| --- | --- |
| `qualification_file` and `qualification_sha256` | Immutable, operator-retained controller qualification record. It must match the current `scripts/supervisor/controller.py` SHA-256. |
| `identity_receipt_file` and `identity_receipt_sha256` | Immutable setup identity for the phase-owned isolated deployment, candidate image ID, source tree, fixture tree, platform, replacement endpoint, predecessor registration, both fences, and all queued/uncertain team attempts. |
| `controller_journal` | Existing controller SQLite journal, readable by the adapter with `mode=ro`, retaining the predecessor domain registration and resulting exclusion receipt. Required unless the optional Lima transport below is supplied. |
| `controller_journal_transport` | Optional operator-owned access to the live journal in the approved Lima guest. It replaces local journal reads; it is not a copied SQLite snapshot. |
| `runtime_url` | The replacement runtime endpoint in the identity receipt. HTTPS or loopback HTTP only; redirects and ambient proxies are refused. |
| `host_token`, `ordinary_token`, `admin_token`, `admin_key`, `team_owner_token` | Private scoped credentials. Trusted-host exclusion, ordinary-user denial, and collaboration-owner reclaim use distinct authorities. |

The identity receipt must bind the exact predecessor epoch and team incarnation
to the controller domain record. It must also bind the replacement fence before
reclaim. The adapter rejects a missing, forged, stale, or replayed proof, and
requires real queued plus active predecessor work. It verifies that queued work
transfers only through the exact reclaim receipt while active work remains
uncertain with unchanged accounting.

## Candidate controller preparation

The controller sources and templates already exist at
`scripts/supervisor/controller.py` and `deploy/supervisor/`. An operator may
prepare an isolated candidate using the documented candidate controller mode.
Until the complete physical qualification succeeds, its private record must
have all of these properties:

- `production_qualified` is `false`.
- `isolated_candidate` is `true`.
- `qualification_id` begins with `dot17-candidate-`.
- Every deployment ID begins with `dot17-`.
- Its certificates, journal, runtime durable stores, and namespace are separate
  from know-me and from any customer workload.

Candidate setup does not promote the record. Only the operator may retain and
promote a completed record after all physical observations reconcile. Do not
write a `production_qualified: true` record to unblock a fixture.

For the Linux guest, install the exact OCI-exported arm64 runtime artifact and
retain the source-bound OCI receipt, exported executable digest, installed
executable digest, machine identity, and pre-reboot boot identity. The native
mechanism requires a later observed whole-host boot transition. A service
restart, process absence, or a stopped detached child in the same boot does not
qualify.

## Required physical evidence

The adapter accepts one genuine qualified mechanism. This v1.1.1 setup selects
the Linux native mechanism and does not restart the cancelled Dot qualification
matrix. It must bind one real predecessor registration to the controller
journal, exact predecessor and replacement fences, and nonempty real queued
plus active team attempts. The adapter verifies that queued work transfers only
through the exact reclaim receipt while active work remains uncertain with
unchanged accounting.

Native evidence must show the isolated guest's registered root before the
operator-authorized reboot, then the same machine with a different boot ID,
stopped old heartbeats, a durable exclusion receipt, and replay refusal.

The native mechanism must reject ordinary-user, wrong-incarnation/epoch, and
replayed proof attempts without a durable execution change. HTTP success alone
is insufficient.

## Replacement Linux guest resource plan

The historical guest receipt supplies the only retained resource and image
record. The replacement reserves 4 vCPUs, 8 GiB RAM, and 24 GiB virtual disk
on the local arm64 host. It uses QEMU, has no host mounts, no configured guest
networks or port forwards, and does not enable system or user containerd. The
operator-reviewed local YAML must preserve these exact fields:

```yaml
vmType: qemu
arch: aarch64
cpus: 4
memory: 8GiB
disk: 24GiB
plain: true
mounts: null
networks: null
portForwards: null
containerd:
  system: false
  user: false
images:
  - location: https://cloud-images.ubuntu.com/releases/noble/release-20260615/ubuntu-24.04-server-cloudimg-arm64.img
    arch: aarch64
    digest: sha256:cafa1a965b591b7c4184b484ffd8e625981a79d48f9b4ae8a4adf7b4c5ade927
```

The reviewed YAML is retained at
`/Users/gqadonis/.local/state/uar-v1.1.1/uar-1.1.1-lima-candidate.yaml`. The
completed bounded creation sequence was:

```text
/Users/gqadonis/.local/bin/limactl --tty=false create --name=dot17-linux-qualification /Users/gqadonis/.local/state/uar-v1.1.1/uar-1.1.1-lima-candidate.yaml
/Users/gqadonis/.local/bin/limactl --tty=false start dot17-linux-qualification
```

These commands create and start only the named local guest. The guest
configuration is not a qualification record and cannot set
`production_qualified`.

## Optional live Lima controller-journal transport

The acceptance host cannot treat a copied local SQLite file as evidence for the
guest controller. The controller journal remains in guest-local storage with no
host mount. The private `infrastructure.supervisor.controller_journal_transport`
object may replace `controller_journal`:

```json
{
  "kind": "lima",
  "instance": "dot17-linux-qualification",
  "executable": "/operator-private/limactl",
  "executable_sha256": "sha256:exact-local-limactl-bytes",
  "lima_home": "/operator-private/lima-home",
  "journal_path": "/var/lib/uar-supervisor/exclusions.sqlite3"
}
```

All paths are absolute and private. The fixture hashes `executable` before each
remote read, passes `lima_home` only as `LIMA_HOME`, and accepts only the exact
instance and journal path above. It uses one fixed argv to run the guest's
`/usr/bin/python3` with a source-defined program. That program opens the SQLite
URI in `mode=ro`, parameterizes the lookup value, and allows only
`domains` by `epoch` or `exclusions` by `challenge`. It returns a bounded JSON
row with the fixed expected column count. It cannot run a shell fragment,
manifest-provided command, arbitrary SQL, or an arbitrary guest path.

The fixture queries the live `domains` row before exclusion and the live
`exclusions` row after exclusion through this same transport. The retained
acceptance receipt therefore identifies the guest read by the trusted local
Lima executable hash; it does not claim that a stale copied journal observed a
new physical exclusion.

## Native candidate and six-group staging sequence

This sequence is source-backed setup guidance for the selected Linux mechanism.
It uses one disposable UAR deployment only; it does not create the cancelled
Dot continuation-role matrix.

The lead must first complete every source, fixture, and documentation item and
record the full implementation inventory. No build or physical qualification
begins before that boundary. The first physical native qualification then
establishes the actual guest boot-containment evidence and permits the operator
to retain a qualified controller record. Only after that physical qualification
does a fresh scoped staging create the predecessor/replacement team state for
the final six-group acceptance. Do not reuse a candidate-stage registration,
fence, or attempt ID as final-acceptance input.

1. Install the source-bound arm64 runtime, the exact controller source, and
   `deploy/supervisor/uar-supervisor.service` on the guest. Create private
   controller configuration from `deploy/supervisor/controller.example.json`:
   `mechanism` is `native-boot-exclusion-v1`, the deployment service is
   `uar.service`, and its `runtime_uid` is the actual UAR service account. The
   controller uses separate root-owned certificates and journal storage. Start
   it only with `--qualification-candidate` and a private record whose
   `production_qualified` is `false`, `isolated_candidate` is `true`, and whose
   qualification and deployment identifiers satisfy the controller's existing
   `dot17-candidate-` / `dot17-` checks.
2. Place the matching mTLS settings from
   `deploy/supervisor/runtime.example.json` under `supervisor_recovery` in the
   guest UAR configuration. The runtime uses HTTPS, controller CA, its client
   identity, the candidate qualification ID, `native-boot-exclusion-v1`, and
   the exact deployment ID. Its configured controller must be available before
   UAR admits team work: server startup acquires its execution identity and
   persists the controller registration before team and observer startup.
3. Seed one disposable, workspace-scoped team from a retained valid team
   definition and deployment binding. Create two independent ready tasks with
   matching assigned members. Record the returned team/task revisions; the
   team APIs reject guessed revisions. The planning routes are
   `POST /api/v1/collaboration/team-instances` and
   `POST /api/v1/collaboration/team-instances/{team}/tasks`, followed by the
   task assignment routes in `src/uar/api/collaboration/team_planning.rs`.
4. Stage the queued predecessor attempt with
   `POST /api/v1/collaboration/team-instances/{team}/tasks/{task}/admit-queued`.
   Its body is `commandId`, `expectedTeamRevision`, `expectedTaskRevision`,
   `memberId`, positive `reservation` (`tokens`, `costMicrounits`, and
   `elapsedSeconds`), and optional `contextArtifactIds`. Save its returned
   attempt ID and execution fence.
5. Stage a second attempt with `admit-queued` using a different ready
   task/member, then dispatch that exact returned ID through
   `POST /api/v1/collaboration/team-instances/{team}/attempts/{attempt}/dispatch`.
   Its body is `commandId` and the then-current `expectedTeamRevision`. The
   controlled work must remain active when the guest-only reboot occurs. Do not
   write `uncertain` into storage or fabricate an attempt receipt:
   interruption/recovery is what converts an actual running attempt to retained
   uncertainty.
6. Before reboot, retain the actual controller registration, execution-owner
   view, team execution views, predecessor fence, queued/active attempt IDs,
   runtime epoch, team incarnation, guest machine ID, and boot ID. The native
   controller journal's `domains` record is the durable predecessor binding.
   After the authorized guest-only reboot, start the replacement UAR service
   with the same durable stores and controller configuration, then retain its
   new current fence. Do not substitute a process restart or a macOS reboot.
7. Use the trusted-host challenge route
   `POST /api/uar/recovery/supervisor/challenges` with the predecessor
   `runtime_epoch` and `{ "kind": "team_owner", "fence": ... }`, then its
   `/challenges/{id}/exclude` route. These routes invoke the controller's
   authenticated `/v1/containment/register` and `/v1/containment/exclude`
   protocol; no agent-uploaded receipt reaches the catalog. Only after the
   controller observes the changed guest boot may the administrator reclaim the
   exact epoch through `POST /api/v1/collaboration/execution-owner/reclaim`.

The staging record later supplies `team_scopes` in the immutable identity
receipt. Each scope records `workspace_id`, `team_id`, `queued_attempt_ids`,
and `uncertain_attempt_ids`; the receipt separately records the exact
predecessor and replacement fences and predecessor registration. The recovery
adapter rejects an empty set, a guessed ID, or an attempt whose fence does not
match the predecessor.

Candidate evidence does not promote the controller. After the physical native
observation has been retained and reviewed, the operator alone may create the
immutable `production_qualified: true` record, bind its SHA-256 into the
controller configuration, and start the normal controller path for the final
qualified acceptance. A fixture, this runbook, or a staging helper cannot make
that record.

The source-only wire packet, including request shapes and exact retained fields,
is in [the native team staging handoff](../../.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/coordination/handoffs/native-team-staging-20261016.md).

## Operator actions and handoff sequence

1. Complete the phase's remaining source, fixture, and documentation work, then
   have the lead record the full implementation inventory. This precedes every
   build and physical qualification action.
2. Run the approved guest-only physical native qualification after that
   inventory. Retain the actual boot-containment and controller-journal evidence
   without claiming that the candidate record is production-qualified.
3. After the physical evidence is retained and reviewed, the operator may
   create the immutable qualified controller record and bind its digest into the
   normal controller configuration.
4. Create fresh scoped predecessor/replacement state and run the final
   six-group acceptance using the sequence above. Retain its exact registration,
   fences, queued and uncertain attempt IDs, live journal rows, and same-image
   identity receipt.

The uncomfortable limit is that this procedure cannot recover a controller
journal or prove containment after a node or CRI record is gone. In that state,
the predecessor remains blocked and v1.1.1 recovery acceptance cannot pass.

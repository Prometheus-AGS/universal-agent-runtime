# Durable schedules

Schedules target an existing owned logical agent instance. The instance must have
a currently admitted deployment binding. Durable catalog, instance persistence,
and accounting support are required; an unsupported store returns an explicit
error. No schedule data is stored in a timing library.

Authenticate requests and send `x-uar-workspace-id`. Routes are mounted at
`/api/uar/schedules`:

| Method/path | Body |
| --- | --- |
| POST base | `{"instanceId":"…","prompt":"…","recurrence":{…}}` |
| GET base | List owned schedules in this workspace. |
| GET `/{id}` | Inspect current state without private prompt or owner grant. |
| PUT `/{id}` | `{"expectedRevision":1,"schedule":{"instanceId":"…","prompt":"…","recurrence":{…}}}` |
| POST `/{id}/pause` | `{"expectedRevision":1}` |
| POST `/{id}/resume` | `{"expectedRevision":2}` |
| POST `/{id}/cancel` | `{"expectedRevision":3}` |

Recurrence accepts exactly these forms:

```json
{"kind":"once","at":"2027-01-15T18:00:00Z"}
```

```json
{"kind":"daily","time":"09:00","timezone":"America/Chicago"}
```

```json
{"kind":"weekly","weekdays":[1,3,5],"time":"09:00","timezone":"America/Chicago"}
```

One-shot input is normalized to UTC and must be future-dated at creation/edit.
Recurring times require exactly HH:mm; weekdays are unique ISO values 1–7.
Missing local times are skipped with `dst_gap` history. Repeated local times use
the earlier instant. Cron and RRULE strings are rejected. Resolve explicit IANA
names; the host's default timezone is irrelevant. Linux/macOS deployment requires
an available system timezone database. Windows uses Jiff's existing platform
timezone support and remains experimentally qualified; no platform availability
claim is established by this implementation.

Every control operation requires the current revision. A recurrence/destination
edit cancels old pending intent and starts evaluating the new revision from its
commit time. Pause prevents new claims and retains pending intent; resume uses
normal latest-due coalescing. Cancel creates a durable tombstone and clears
pending work. An already claimed occurrence keeps its input, destination and
instant across edits, pause and cancellation.

At most one claimed occurrence holds a stable complete-owner/schedule guard.
Cursor and pending updates use the existing collaboration catalog CAS. If busy,
each later due candidate replaces the pending slot and records the skipped range.
Catch-up scans inspect at most 366 local days per pass and do not claim a pending
occurrence until the entire due range has been evaluated. The existing observer
wakeup runs the scanner; execution uses the existing instance inbox and root host.
The schedule claim is durable intent, not a claim that a model or effect ran.
Current destination, tool approvals and root budget admission still govern actual
execution.

The complete issuer/tenant/subject identity is captured at authenticated CRUD in
a protected schedule/destination grant. API and native-tool arguments never
provide that grant or an owner. Only the trusted schedule host restores it from
the protected catalog after scope checks. It does not create a JWT, roles, or a
general `UserContext`. Scheduled commands carry host-only inbox provenance;
ordinary caller-selected command IDs cannot create retention pins.

Queued commands replay with the same logical command identity after restart.
Pinned inbox receipts prevent trimming from turning that replay into a new turn.
Only the current nonexpiring supervisor owner scans/dispatches. A safe ownership
transfer can resume an unstarted command; an uncertain or running predecessor
keeps its guard. The Dot11 qualified whole-boot/Pod exclusion proves executor
exclusion, not settlement of external effects. An uncertain instance must use
its existing exact-attempt reconciliation path, and unknown accounting must use
the accounting reconciliation path. No timeout unlocks a schedule.

A guard releases only after a durable terminal instance command, no matching
active attempt, ready recovery, resolved canonical tool evidence, and settled
accounting calls. The terminal schedule receipt commits before inbox unpinning.
Cleanup is replayable from that receipt. Private prompts/grants are excluded from
schedule status, while pending/active IDs, skipped history and terminal receipts
remain inspectable.

The model tools are `schedule_create`, `schedule_list`, `schedule_get`,
`schedule_update`, `schedule_pause`, `schedule_resume`, and `schedule_cancel`.
All require a host-verified owner. Mutations declare required approval and pass
through normal native-tool governance.

The uncomfortable thing: a durable intent and source inspection do not prove
restart or cross-process correctness. Full-phase acceptance must exercise two
real runtimes, qualified supervisor recovery, pending approval, unknown effects,
unknown accounting, DST fixtures under recorded installed rules, command receipt
retention, and actual model/effect counts. The authored driver is
`tests/fixtures/dot14_schedules_real_path.py`; it does not start runtimes or alter
the wall clock. No such gate has run for this source.


## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.


### Linux candidate executable from the OCI build

The local OCI build exports the actual regular ELF runtime executable from each verified image filesystem. Its `local-two-architecture-image` observation records `source_content_sha256` and `runtime_binaries` for amd64 and arm64, including each executable path, SHA-256, size, architecture and image digest. These are candidate artifacts; no publication is claimed.

For Linux guest qualification, install the selected arm64 executable and run the source-bound acceptance driver inside that guest. Copy the genuine same-inventory OCI receipt, observation, archive and exported executable without relabeling their paths or results. The host configuration selects `candidate_runtime_origin: "oci"`, `os: "linux"`, `architecture: "arm64"` and the genuine `oci_build_receipt`. The runner checks the receipt/source binding, archive, exported executable, actual installed executable bytes and current host architecture. The guest's installed path may differ from the export path. Native macOS candidate validation retains its existing local build receipt. Source/plan inventory and every operational stage still require their actual inputs and observations; exporting an executable proves no recovery behavior.

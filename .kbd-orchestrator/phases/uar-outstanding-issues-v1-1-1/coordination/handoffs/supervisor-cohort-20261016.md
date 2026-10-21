# Supervisor qualification cohort handoff — 2026-10-16

Phase: `uar-outstanding-issues-v1-1-1`, Execute. Current tasks: V2/6.2 remains
in progress and R9/4.9 remains in progress. This handoff is a source and
documentation update only. It does not complete either task or authorize a
verification command.

## Source findings

`scripts/release/uar_1_1_1_recovery.py` already refuses the unsupported path.
It requires an immutable qualification record, a same-image identity receipt,
the real controller journal, exact predecessor and replacement fences, nonempty
queued and active predecessor work, a physical exclusion observation, and
replay-safe reclaim. It reads the controller journal read-only and hashes the
current `scripts/supervisor/controller.py`. It cannot be truthfully relaxed to
accept a sentinel, a stopped unrelated process, an HTTP-only response, or an
invented `production_qualified` flag.

Read-only local inventory establishes that the historical Linux guest
`dot17-linux-qualification` was removed by the 2026-10-14 Dot closeout.
`~/.lima-dot17` contains only `_config/user`, `_config/user.pub`, and
`_config/networks.yaml`; there is no instance directory, VM definition, PID,
or state record. It is not a runnable guest candidate. The historical resource
receipt records QEMU/aarch64, 4 vCPUs, 8 GiB RAM, 24 GiB disk, and the pinned
Ubuntu image in the setup contract. The available local binary is
`/Users/gqadonis/.local/bin/limactl`, SHA-256
`4636491d6758a95211a5a857a07ddb6b1c034375bdede18a5bdae32643cbd553`; its
release version is not established from local metadata.

`lke174619-ctx` is a local kubeconfig context name only. The prior worker,
`lke653426-988676-r7ndr`, had an expired cleanup deadline and recorded
`production_qualified: false`, no started runtime, and no recovery faults. It
cannot supply current identity or qualification. No remote connection, guest
probe, controller start, or cluster mutation was made.

The local-02 receipt failed all six groups; recovery and combined failed after
the local Docker environment lost controlled resources. That receipt has no
containment, fencing, or qualification value.

## Changed paths

- `docs/operations/supervisor-recovery.md`: links the current v1.1.1 recovery
  requirement to the bounded setup contract and rejects treating historical Dot
  cancellation as qualification evidence.
- `docs/operations/uar-1.1.1-supervisor-setup.md`: new private-input contract,
  candidate boundary, selected Linux resource plan, required physical evidence,
  and operator handoff sequence.
- `.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/coordination/handoffs/supervisor-cohort-20261016.md`:
  this handoff.
- `scripts/release/uar_1_1_1_recovery.py`: makes Surreal cold-boot readiness
  require a received HTTP status of exactly 200 instead of accepting any
  response object as truthy, and supports a bounded optional live Lima
  controller-journal transport. The transport accepts only the approved guest,
  absolute private Lima paths, a hash-bound executable, fixed read-only SQLite
  row lookups, and bounded JSON rows.

`scripts/supervisor/controller.py` and `deploy/supervisor/**` were inspected but
not changed. The controller and its native templates already support the
required candidate configuration; no supervisor/deployment source gap was
found.

No new staging helper was added. Existing UAR routes provide the necessary
durable path: `admit-queued` creates a real queued attempt without dispatch,
and `attempts/{attempt}/dispatch` starts a separately staged real attempt. The
latter must be interrupted by the authorized guest reboot to create genuine
retained uncertainty. A helper that wrote an `uncertain` state, a journal row,
or a qualification record would invalidate the qualification rather than
prepare it.

The companion
`coordination/handoffs/native-team-staging-20261016.md` records the exact
candidate controller, team-attempt, fence, and replacement fields. It is
source-only and does not authorize its HTTP requests before the lead closes the
phase implementation inventory.

## Required external actions

1. Approve the replacement dedicated local Linux arm64 guest described in
   `docs/operations/uar-1.1.1-supervisor-setup.md`: 4 vCPUs, 8 GiB RAM, 24 GiB
   disk, QEMU/aarch64, and the retained pinned Ubuntu image. This is the selected
   native mechanism. It is not a request for LKE capacity, a macOS reboot, or a
   know-me mutation.
2. After approval, create only that named local guest with the two documented
   `limactl` commands. Inspect its resulting identity before installing any
   supervisor material; creation alone is not qualification.
3. Prepare separate candidate controller records, TLS identities, durable
   stores, journals, and observer access from `deploy/supervisor/` templates.
   Candidate records retain
   `production_qualified: false`; no team member creates a true flag.
4. Stage one disposable team with a real queued attempt through `admit-queued`
   and a separate active attempt through ordinary `admit`. Retain their IDs,
   views, predecessor registration, and exact fence before the reboot. The
   active attempt may become uncertain only through the real interrupted
   execution path.
5. Supply explicit approval before a guest-only reboot. A macOS host reboot is
   neither requested nor valid evidence.
6. Retain the real six-group predecessor record with nonempty queued and active
   work, exact epoch/fences, physical native exclusion evidence, immutable
   controller record, journal, and same-image setup identity through the private
   infrastructure input.
7. The lead records L7 only after R9 production repairs and this setup are
   complete, then directs the next serialized local program.

The lead reports that the approved disposable guest is running and its isolated
candidate controller is installed and enabled with private TLS. This handoff
does not treat that setup status as qualification evidence. No guest or host
reboot, LKE mutation, know-me mutation, controller promotion, or acceptance
rerun was performed by this fixture work. The remaining blocker is the actual
six-group physical evidence, not an unimplemented repository interface.

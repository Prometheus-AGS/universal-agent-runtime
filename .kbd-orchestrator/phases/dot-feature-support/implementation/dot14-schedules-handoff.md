# Dot14 source handoff

2026-10-12. Execute; no verification boundary reached. Root owns canonical task
status and phase completion inventory. All changes below are authored and
uncompiled. No command in this slice executed tests, checks, builds, formatters,
validators, product probes or review gates.

## Plan delta

The pinned cron scheduler was rejected on exact-source fixed-offset continuation.
`dot14-timing-fit.md` records qualification. Root recorded the bounded Analyze
correction and added the already locked exact Jiff dependency. No independent
scheduler/queue/executor or `versions.toml` edit was added.

## Owned source inventory

- `src/uar/domain/schedules.rs`: recurrence input, protected intent/grant,
  cross-revision guard, skipped ranges and terminal receipts; private fields
  excluded from status view. Registered in `domain/mod.rs`; one defaulted
  schedules map added to `domain/collaboration.rs`.
- `src/uar/runtime/schedules/{mod,calendar,controller}.rs`: strict recurrence,
  IANA skip/earlier conversion, bounded catch-up, authenticated management,
  coalescing, guarded inbox dispatch/recovery and terminal reconciliation.
  Registered in `runtime/mod.rs`.
- `src/uar/compiler/collaboration/schedules.rs`: existing catalog CAS owns
  cursor, pending slot, claim and guard transitions. Registered in catalog mod.
  Execution mutations require existing nonexpiring supervisor ownership.
- `src/uar/security/claims.rs`, `runtime/actor/messages.rs`: protected
  schedule/destination grant captures verified complete principal. Private host
  restore validates canonical identity and exact scope; never mints roles/JWT or
  general `UserContext`. Source handoff to A2A teammate sent.
- `src/uar/persistence/agent_instances.rs`, `runtime/instance/controller.rs`,
  `lifecycle.rs`, new `schedules.rs`, `mod.rs`: host-only `schedule_id` metadata
  pins inbox dedupe receipts. Old records default to no pin. Transition validation
  prevents pinned drops and accepts only an otherwise unchanged terminal receipt
  unpin. New ordinary commands have no pin. Existing `submit` wraps the same
  inner admission function; only private `submit_scheduled` can set provenance.
- `runtime/instance/pump.rs`: existing effect-evidence classifier changed to
  `pub(crate)` for reuse, with no classification rewrite.
- `runtime/observer/{controller,delivery}.rs`: attach schedules to existing
  observer wakeup before `start`; no new timer/executor.
- `api/schedules.rs` plus `api/mod.rs`: verified owner + workspace CRUD/control.
- `runtime/native_skills/schedules.rs` plus native mod: seven tools; mutation
  tools require normal approval and verified host execution context.
- `docs/durable-schedules.md`: contract, operational timezone requirement,
  recovery model, route/input documentation and acceptance limits.
- `tests/dot_schedule_calendar.rs`, `tests/fixtures/dot14_calendar_cases.json`,
  `tests/fixtures/dot14_schedules_real_path.py`: authored dated timezone cases and
  installed-host CRUD/owner isolation/revision/busy/terminal/restart inspection
  driver. No execution occurred.

Root confirmed server wiring: concrete controller construction, attachment before
observer start, native registration before listening, authenticated route mount
at `/api/uar/schedules`. Root owns those server changes; no AppState field needed.

## Authority and restart contract

Schedule management reads instance persistence without invoking orphan recovery
on a follower runtime. Scanning first requires current supervisor ownership;
every mutation that affects execution repeats that check inside catalog CAS.
The local scan mutex serializes receipt cleanup and stable-command replay in the
owning process; it is not cross-host authority. Cross-host exclusion comes from
Dot11's existing supervisor fence and qualified exclusion mechanism.

Calendar cursor and pending slot/claim commit atomically. The claim then enters
the existing logical-instance durable inbox with a stable command ID. Host-only
retention prevents a completed command from being trimmed before schedule
reconciliation. After a safe supervisor transfer, a missing never-observed or
accepted unstarted command may replay using that ID; running/uncertain work
does not. A terminal schedule receipt commits before its inbox pin is cleared.
The next scan finishes interrupted cleanup from the receipt.

The Dot11 worker confirmed native whole-boot or qualified Kubernetes Pod
exclusion covers logical-instance roots inside that boundary. It does not settle
external effects. Guard release requires terminal instance state, no matching
active attempt, ready recovery, existing canonical effect classifier clear, and
all persisted root accounting calls settled. Unknown effect/accounting state
holds the guard and retains liability. Existing instance/accounting reconciliation
paths resolve it; this slice adds no synthetic completion or elapsed-time unlock.

## Evidence and remaining work

Source/document reads and edits only. Exact-source and current-doc links are in
`dot14-timing-fit.md`; no executable outputs are claimed. Root must run final
phase gates serially only after every production implementation item is complete.

The calendar fixture is useful timing evidence only, not durable-admission proof.
The HTTP driver requires the real installed runtime, prepared owned instance,
two distinct tokens, and optional second shared runtime URL. Its caller owns
cold process lifecycle, qualified supervisor reclaim, held approval/effect and
unknown-accounting cases, controlled VM clock for multi-day/DST scenarios,
and independent provider/effect invocation counts. Those full production
scenarios remain unexecuted and must be integrated into the Dot17 phase runner.
The same constraint applies to Windows timezone availability and shared-runtime
IANA-rule provenance. No acceptance status should be inferred from file presence.

No unrequested framework or queue was added. Guards added here trace to explicit
owner/revision/concurrency/restart requirements and observed source-level inbox
retention replay risk. The uncomfortable thing: root accounting/approval and
supervisor interfaces are composed from independently changing source and have
not compiled together; source correctness is still unverified.

# Dot14 timing source qualification

Date: 2026-10-12. Phase: dot-feature-support, bounded Analyze correction before
Dot14 Execute. This is source/document qualification only. No executable probe,
test, compiler, formatter, validator, build or review gate ran.

## Decision

Reject `tokio-cron-scheduler = 0.15.1` for Dot14. Return that candidate to Analyze.
Use the existing Tokio observer scan wakeup, with narrowly scoped calendar
conversion from the already locked `jiff = 0.2.35`, subject to the lead recording
this bounded plan correction and adding the direct manifest dependency. UAR owns
all durable schedule state. No alternate scheduler, queue, executor, or scheduler
storage is introduced. Executable timing qualification remains task 3.1 after
the entire expanded phase implementation boundary.

## Exact candidate evidence

Context7 resolved `/mvniekerk/tokio-cron-scheduler` and returned current main-branch
documentation for timezone constructors, cron, one-shot and repeated jobs.
Because that is not exact-version certification, the following v0.15.1 sources
were read directly:

- [Job constructors](https://github.com/mvniekerk/tokio-cron-scheduler/blob/v0.15.1/src/job/mod.rs):
  `new_tz`/`new_async_tz` calculate `time_offset_seconds` at construction. The
  first next tick receives the timezone object, but stored job metadata retains
  an offset. One-shot constructors accept duration/monotonic instant, requiring
  UAR to retain its absolute UTC instant independently.
- [Scheduler](https://github.com/mvniekerk/tokio-cron-scheduler/blob/v0.15.1/src/scheduler.rs):
  lines 171–180 reconstruct `FixedOffset` from that offset for subsequent cron
  instants. Its 500ms scan publishes activation and separately updates next/last
  ticks. Its cursor cannot be the atomic authority for a UAR occurrence.
- [Cron job](https://github.com/mvniekerk/tokio-cron-scheduler/blob/v0.15.1/src/job/cron_job.rs):
  asynchronous callbacks are spawned independently. Callback completion is not a
  UAR effect/accounting/exclusion receipt.
- [README](https://github.com/mvniekerk/tokio-cron-scheduler/blob/v0.15.1/README.md):
  memory and optional persistence choices exist, but none replaces the required
  UAR owner-scoped catalog transaction.

Inference from those sources: the fixed-offset continuation cannot establish
daily/weekly IANA wall-time behavior across later DST transitions. Wrapping only
one-shot wakeups would retain all recurrence, cancellation, restart and coalescing
work in UAR while duplicating its current wakeup mechanism. There is no adoption
benefit sufficient to justify the dependency. No executable claim is made about
upstream races or missed runs.

## Replacement qualification

`Cargo.lock` contains Jiff 0.2.35 with checksum
`668b7183bd07af9a4885f5c35b0cc5c83c4607a913c16b7e17291832910d2dcc`.
The exact locally installed registry source under
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/jiff-0.2.35/`
was inspected (`Cargo.toml`, `src/tz/timezone.rs`, `src/tz/ambiguous.rs`,
`src/civil/datetime.rs`, `src/timestamp.rs`). Context7 `/burntsushi/jiff` supplies
current guidance; its listed version is 0.2.37, so exact 0.2.35 local source and
[versioned official docs](https://docs.rs/jiff/0.2.35/jiff/tz/struct.TimeZone.html)
govern API selection.

Use `TimeZone::get` for explicit named lookup, never the system default zone;
`to_ambiguous_timestamp` exposes gap/fold classification. UAR skips `Gap`, takes
`earlier()` for `Fold`, and records the skipped local date. `Timestamp::from_second`
and `as_second` bridge existing Chrono UTC records. Timezone objects have immutable
rules; UAR must resolve the stored IANA name when evaluating new candidates.
Invalid or unavailable timezone data is an explicit error, never UTC fallback.

Manifest recommendation: direct `jiff = "=0.2.35"` using existing default features.
Do not enable bundled `tzdb-bundle-always`: this version's manifest asks for
`jiff-tzdb 0.1.8`, while the present platform bundle is 0.1.6, and changing that
graph is not needed for the Linux/macOS supported profile. Required operational
precondition: an accessible IANA timezone database in each runtime environment.
Multi-runtime fixtures must retain timezone data provenance and use matching
rules. New resolver behavior has not been executed or certified.

`versions.toml` prohibits agent edits to that file and has no existing timezone
library pin or architecture decision contradicted by the recommendation. This
recommendation does not require changing that protected file. The lead owns
manifest/lock reconciliation and the bounded plan correction.

## Contract mapping

| Requirement | UAR authority and selected mechanism |
| --- | --- |
| once UTC | Parse RFC3339 offset, persist normalized UTC instant; reject past creation. |
| daily/weekly | Strict HH:mm, named IANA zone, nonempty unique ISO weekday set. Enumerate local calendar days; no cron/RRULE input. |
| DST gap/fold | Explicit gap skip and earlier-fold mapping; keep skip history. |
| Cancellation | Tombstone schedule intent under catalog CAS; no callback deletion is authority. |
| Revision | Increment revision at commit; cancel old pending intent, retain admitted immutable instant. |
| Downtime | Recompute due instants from durable cursor, coalesce latest pending, record skipped range. |
| Busy/restart | Stable complete-owner/schedule guard spans revisions; no elapsed-time release. |
| Safe release | Require durable terminal disposition plus excluded predecessor and resolved effects; unknown liability remains reserved. |
| Concurrency | Cursor, pending occurrence and guard transition share catalog CAS; admission uses existing governed host path. |
| Storage | Existing durable catalog backend only; unsupported persistence remains explicit. |

## Implementation plan and remaining evidence

Own new schedule domain/calendar/controller/API/native-tool files and narrow
observer integration. Coordinate catalog/domain module registrations with lead;
do not modify provider/migrations or other teammate-owned source. Root wires
server and thread service. Dot11 exclusion is necessary but is not effect or
accounting settlement; no synthetic boolean supplied by a caller can unlock a
schedule.

Author fixtures for America/Chicago 2027-03-14 02:30 gap and 2027-11-07 01:30 fold,
once/daily/weekly validation, concurrent scanning, downtime coalescing, pause,
cancel, revision, approval hold, unknown effect, cold restart and safe release.
Those dates are fixture inputs requiring final installed-timezone confirmation,
not results observed now. Real-path gate execution remains deferred.

The uncomfortable thing: source qualification proves API fit, not end-to-end
correctness. Shared-runtime timezone-data drift, atomic host admission, unknown
effects and durable guard recovery remain production acceptance risks until the
full-phase gates exercise them.

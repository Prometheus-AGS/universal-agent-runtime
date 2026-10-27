# Runtime settlement handoff — task 1.2

Changed paths:

- `src/uar/compiler/collaboration/storage.rs`
- `src/uar/compiler/collaboration/service.rs`

The Surreal catalog CAS now calls `Response::check()` before extracting its
terminal `UPDATE` result. A typed `TransactionConflict` is a normal CAS miss
only when a follow-up catalog read proves the exact `generation = expected`
predicate is no longer true. An unchanged or unreadable catalog keeps the
original typed failure; it is not converted into a retry. Other response errors
also propagate without attempting result extraction.

`CollaborationCatalogService` now converts storage failures to closed safe
categories: `transaction_conflict` when the preserved error chain contains the
typed Surreal conflict, otherwise `storage_operation_failed`. It no longer
copies arbitrary underlying storage text into the collaboration error. The
existing bounded CAS loops only recompute durable catalog state after an
observed CAS miss; they do not re-invoke a member run, create a recovery API, or
release an unknown reservation.

Scoped baseline had an unrelated pending edit in
`src/uar/runtime/team_execution/execution.rs`; it was preserved. This task did
not edit `team_execution/settlement.rs`, runtime `execution.rs`, or runtime
`controller.rs`.

No verification commands ran because the product verification gate is CLOSED.
Compilation, actual Surreal 3.3.0 response behavior, retained-store migration,
and TEAM-01 remain unverified. The exact historic inner `take(0)` error was not
retained, so this repair is grounded in the visible unchecked-response path and
the existing typed-conflict CAS pattern rather than a reconstructed old error.

`handoffs/runtime-repair.patch` is a reconstructed, line-numbered unified diff
from the scoped source spans captured before tasks 1.1 and 1.2. Full baseline
files were not persisted; the patch includes only the four authored-file hunks
and excludes concurrent working-tree edits.

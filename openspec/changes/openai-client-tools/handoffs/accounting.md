# Caller-tools accounting handoff

Phase: Execute. Verification lock: CLOSED. No verification commands ran.

Owned production file: `src/uar/runtime/cost_budget.rs`.

Added public serializable `ClientToolBudgetSnapshot` with private fields and
`ModelCallBudget::snapshot_client_handoff(&self) -> Result<ClientToolBudgetSnapshot>`.
It captures complete verified accounting owner, original economic scopes and their
local cumulative counters, `RunUsageSnapshot`, request/token windows as UTC millisecond
timestamps, root elapsed duration, absolute root deadline, intersectable prior
`ThreadBudgets`, and remote accounting activity/deadline age. Capture requires the
existing durable accounting capability and refuses unresolved source-root remote
reservations; it cannot manufacture a fresh allowance around outstanding liability.

`ModelCallBudget::restore_client_handoff(self, &snapshot, &ActorOwner).await`
requires a freshly resolved budget already bound through `with_durable_accounting`.
It compares the complete current owner, session/agent/global identities and remote
accounting identity. It intersects prior/current limits, preserves original Run
scope identity, counts time spent outside the runtime against the root timeout,
retains the earliest deadline, and rebinds the existing durable accounting scopes
under current policy before admitting another action.

Local counters merge monotonically with maxima; restore never invokes a durable
usage recording API. Durable monetary/token liabilities therefore remain solely in
the pre-existing accounting store. Per-second rate buckets merge by maxima rather
than adding repeated snapshots. Bucket timestamps round toward their end, extending
a restriction by less than one second instead of expiring activity early. UTC
rollback, impossible clock ranges, invalid costs, identity mismatch, missing durable
accounting, and elapsed deadlines fail closed.

Uncomfortable limitation: the snapshot supplements existing durable accounting; it
is not a new independent authority or a substitute for that ledger. A failed restore
may leave stricter local counters behind, and must never authorize work after the
error. Unsupported durable accounting refuses a caller handoff. No compilation,
behavior, restart, rate-window or budget-exhaustion acceptance evidence is claimed.
Runtime and persistence teammates own storing this immutable snapshot and restoring
it before provider or host-tool effects. No dependency, version, workflow, or build
changes were made.

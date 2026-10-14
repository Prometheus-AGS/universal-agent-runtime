# Dot09 accounting implementation handoff

Phase: dot-feature-support / Execute. Source-only production implementation.
No acceptance claim; expanded Dot production boundary has not been reached.

## Delivered source

- `src/uar/persistence/accounting.rs`: complete owner/run/call/attempt identity,
  bounded scope records and individual pending/settled call records; cumulative
  observation, replay identity checks, explicit opening balances and generation.
- `src/uar/persistence/providers/accounting/{surreal,postgres}.rs`: native atomic
  multi-scope call admission/settlement; Postgres sorted row locks, Surreal whole
  affected-set compare-and-swap. No whole-ledger document or detached write.
- `src/uar/persistence/mod.rs` and provider registrations: explicit unsupported
  defaults. Preserved ingestion/A2A registrations authored by other teammates.
- `migrations/20261012010000_durable_accounting.sql` and
  `migrations/surrealdb/durable_accounting.surql`: additive isolated enforcement
  records; no fabricated legacy identities or removal of old history.
- `src/uar/runtime/cost_budget.rs`: host-captured durable context, stable call
  predispatch record, cumulative pending usage and terminal idempotent settlement
  on the existing model wrapper. A final write failure emits a custom accounting
  diagnostic while preserving provider execution outcome.
- `src/uar/runtime/run_cost.rs`: awaits inspection before terminal publication,
  reports the per-call/team tracker total, never appends a duplicate run charge.
- `src/uar/runtime/manager.rs`: captures verified principal and persistence in the
  existing budget before model execution; inspects accounting even without usage.
- `tests/run_cost_settlement.rs`: replaced obsolete detached-memory assumptions
  with authored real-backend fresh-process/replay/owner/legacy fixtures.
- `docs/operations/durable-accounting.md`: production APIs, operational behavior,
  migration/reconciliation, uncomfortable limits and deferred acceptance.

Root-owned integration: `surreal_host.rs` calls accounting initialization;
`src/uar/api/accounting.rs` provides authenticated inspection and trusted-host
explicit provider-evidence reconciliation, mounted by root. No new settings,
Cargo dependencies or pins.

## Unresolved and unverified

No product execution, checks, tests, validators, probes, compiler, formatter or reviews ran.
Only source inspection and file authoring occurred. Native transaction behavior,
restart, schema syntax, typechecking, real runtime terminal publication and the
customer build remain unverified until the expanded full-phase boundary.

The uncomfortable thing: a pending reservation proves retained liability but
cannot prove the provider billed that amount. Missing final usage requires
explicit evidence; no timeout/cleanup path fabricates a settled zero or amount.

No additional unrequested feature or generic accounting engine was added.
Identity/replay guards address duplicate charges; unsupported/owner guards are
trusted-host boundaries; revision guards protect concurrent scope authority.
Dot10 uses these same contracts for reservation/enforcement, with separate handoff.

## Final source handoff

Root reports host initialization and HTTP mounts authored. Dot09 assigned
production source has no known unfinished item. No independent acceptance or
runtime evidence exists yet; all checks remain deferred. Files are released to
the lead. The append-only legacy reporting API remains separate from new budget
inspection. Explicit cost-disabled/unlimited development runs retain their
previous non-durable behavior and make no durable accounting claim.

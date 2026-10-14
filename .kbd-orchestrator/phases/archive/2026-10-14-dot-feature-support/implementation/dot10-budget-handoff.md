# Dot10 budget implementation handoff

Phase: dot-feature-support / Execute. Source authoring only; no verification.
Canonical task/change transitions belong to the lead.

## Production contracts and owned changes

Uses Dot09's bounded scope rows and separate call records in native atomic
transactions. `cost_budget.rs` retains the existing live tracker and model wrapper;
no parallel process-local authoritative ledger is introduced. Pending/unknown
reservations survive restart and cannot be automatically released by lease expiry.
Every admission checks captured scope generation against current durable state.

`manager.rs` binds durable persistence plus full CredentialPrincipal before model
execution. Tracked/enforced accounting rejects unsupported/missing persistence and missing
full principal proof. Explicit untracked/unlimited development execution remains
compatible and makes no accounting claim. Authenticated durable runtime capture
still loads persisted authority regardless of current configuration's ceiling. Existing persisted scope
limits govern admission. Initial config creates missing scopes only; subsequent
policy changes require the root-owned authenticated CAS API.

`src/llm/liter_driver.rs` adds `request_budget_bound`, reusing existing final-wire
builder/serialization/TokenService. The wrapper uses actual validated input plus
configured output/reasoning/uncertainty and worst-case input price category.
Without an existing proven wire contract, catalog context/output maxima are the
conservative fallback. Missing bounds/prices refuse finite admission. This can
reject small budgets on ordinary provider paths: no request-level bound has been
invented, and no new tokenizer/wire protocol is added.

`cost_budget.rs` remote methods are now async host calls:
`reserve_remote`, `record_cumulative`, `charge_reserved_capacity`,
`release_confirmed`. `thread/kernel.rs` exposes a read-only `remote_budget()`
handle; root owns `thread/service.rs` awaits and related fixture adjustments.
Source economic scopes reserve once. An immutable `accounting_delegations` row
binds the existing contract digest to that source call before dispatch. Destination
capture omits economic scopes only when that exact pending source reservation is
present in its own store and matches the grant. The source remains pending inside
each funded admission transaction. A distinct accounting store has no reference,
so destination local run/session/agent/global limits remain enforced. Team grants
likewise retain economic scopes; no new wire identity assertion is introduced.
Unknown peer receipts leave durable pending liability, even if cleanup completes.

Same native migration/provider files as Dot09. Same real backend fixture includes
a shared two-process admission race, restart pending balance, exact replay and
stale generation refusal. No tests were run. Operational HTTP contract and legacy
opening balances are documented in `docs/operations/durable-accounting.md`.

## Remaining integration and verification

Lead must confirm source integration of root-owned host migration, HTTP routes,
thread service async call sites and fixtures. The root owns server wiring.
All actual typechecking, formatting, migrations, fresh-process/live shared DB,
provider-wire, remote cleanup, outage and customer-build outcomes are unverified
until the entire expanded Dot production inventory is complete.

No unrelated subsystem changes, dependency changes or speculative guards.
Budget/unknown checks enforce observed restart/concurrent over-admission defects;
owner and revision checks enforce existing trust boundaries.

## Final source integration and release

Root reports Surreal initialization, accounting HTTP routes, async thread host
callers and affected fixtures authored. Failed/invalid remote receipt writes set
a monotonic unresolved flag; close-idle cleanup only releases after a usable
receipt. Proven-never-dispatched host branches may settle zero; this is distinct
from unknown post-dispatch cleanup. Pending DB liability remains inspectable even
when an in-process flag is lost.

At the lead's additional Dot18 request, manager now shares the existing complete
checkpoint admission body through private `execute_checkpoint_admitted`.
`execute_checkpoint_a2a_with_run_id` consumes an already host-reserved run ID and
uses existing `graph_roots.start` to establish a durable actor-owned root. Ordinary
HTTP checkpoint helper retains UUID allocation and prior dispatch semantics.
Original principal/restoration/approval/claim/primary-driver/graph checks remain.
The A2A worker was informed; no second executor or wire identity authority exists.

No assigned production source item remains knowingly unfinished. Ownership of
manager, cost budget/run cost, narrow kernel helper, narrow Liter helper,
accounting persistence/modules/migrations and fixtures is released to the lead.
All functional/type/schema/concurrency acceptance remains unverified by explicit
full-phase instruction. No unrequested artifact was introduced.

### Source integration correction: accounting topology

The lead identified that unconditional remote grant isolation would bypass a
separate destination deployment's house budget. The corrected implementation
uses the durable contract-digest/source-call link above, including native
transactional pending-source checks. Same-store reference discovery and absent
reference behavior are authored in the backend fixture; actual topology/runtime
journeys remain unverified. Root must call `bind_contract(contract.digest())`
after successful contract validation and before peer dispatch, releasing only
proven-never-dispatched liability on binding failure. No field change was made to
the authenticated A2A wire contract or actor-host constraint types.

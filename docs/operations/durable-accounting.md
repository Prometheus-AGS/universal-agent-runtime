# Durable call accounting and budgets

Status: production source and fixtures authored for Dot09/Dot10; acceptance is
pending the full expanded Dot production boundary. No test, build, compiler,
format, lint, validator, probe or review gate has run for these edits.

The uncomfortable thing: a persisted pending call proves liability was retained,
not that the provider billed the reservation amount. Losing the final usage
counter may require manual provider evidence. Time passing, process exit, an
execution success event, and confirmed cancellation alone cannot prove zero cost.

## Execution and accounting

The trusted host captures the full verified credential principal (issuer kind,
issuer, tenant and subject) and durable persistence in `ModelCallBudget`. The
existing driver wrapper admits a stable `(owner, run, call, attempt)` record
before provider dispatch. Every actual provider invocation has its own call ID;
replayed writes for that identity cannot charge it twice. Descendants retain the
root's run/session/agent/global scopes. All applicable scope changes and the call
record commit together. Call records survive session deletion.

Cumulative usage updates pending liability. A priced final counter and a clean
provider Done event settle a call. A missing price, interrupted stream, failed
usage write, cancellation, or missing terminal counter leaves it pending. Run
completion inspects accounting before publishing the terminal event and emits
`accounting_unresolved` as a nonterminal diagnostic when any call is pending or
storage inspection fails. A settlement outage also emits the lower-level
`uar.accounting` / `unresolved` custom event. Execution success stays execution
success; neither diagnostic claims a settled charge.

Terminal reporting uses the existing per-call tracker total, including child
usage, rather than estimating all usage against the last selected model. The old
terminal append-to-each-scope path is removed. `cost_tracking=false` hides the
reported cost estimate; it does not disable durable budget authority.

## Storage and admission

Local SurrealKV, explicitly attested durable networked SurrealDB, and Postgres
implement this contract. Tracked or enforced accounting requires a supported durable backend and a full
verified principal; neither may fall back to an empty tracker. Explicit
`cost_tracking=false` execution without a financial/token/remote envelope and
without an authenticated durable context preserves the existing untracked,
unlimited development path. That path makes no durable accounting claim.
Authenticated durable runtimes still capture persisted budget authority even when
current configuration has no ceiling. Postgres adds migration
`20261012010000_durable_accounting.sql`; Surreal host migration loads
`durable_accounting.surql`.

`accounting_scopes` contains one bounded aggregate per identity and limit
generation. `accounting_calls` contains separate stable call records, indexed by
owner/run. Admission and settlement affect at most eight scopes per call.
Postgres locks affected scope rows in sorted key order in a native transaction.
Surreal commits scope revisions and the call record in one compare-and-swap
transaction. A contention or generation conflict refuses work; callers must not
silently dispatch after refusal. There is no lease timeout that returns uncertain
liability. No global history document is rewritten by a call.

Scope capacity is settled usage plus outstanding reserved liability. Current
persisted scope limits are authoritative; startup creates missing scopes from
configured limits but never overwrites a persisted generation. Policy edits use
the authenticated operator API below. A running binding with an old generation
is denied until a new host capture. A lower limit retains existing liability and
blocks new unaffordable work; it does not erase costs or cancel prior billing.
Changing startup configuration alone does not replace existing persisted limits.

For an existing final-wire request budget contract, the reservation reuses the
actual final request serialization and token service, plus its output, reasoning
and uncertainty allowance. Approximate counts are not enforceable evidence.
Absent a proven contract, the bound conservatively uses catalog context/output
maxima. This can refuse an affordable small request under a small budget; no
smaller verified bound is available on those existing provider paths. A missing
catalog bound or price is unknown liability and cannot enter finite budgets.
Input pricing reserves the most expensive uncached/cache-write/cache-read case.
Provider usage is still recorded as actual reported usage, never invented from
that reservation. The final integration gate must prove supported request paths
and refusal semantics; catalog-bound fallback is not proof of provider framing.

A remote source reserves the economic root/session/agent/global liability before
dispatch and durably binds that reservation to the existing complete delegation
contract digest. The destination measures its grant in a `remote_destination`
scope. Only when its own store contains that exact live source reservation with
matching grant limits does it omit a second economic charge. Each funded call
also checks the source remains pending within its admission transaction.

A destination using a different accounting store has no source reference and
therefore also enforces its own run/session/agent/global scopes. The shared
store/namespace is the accounting domain; matching process or instance names
alone cannot bypass house authority. Ordinary team grants also retain economic
scopes. No new wire fields or caller-supplied same-deployment assertion is used. The source retains its
reservation across turns and settles once confirmed terminal cleanup includes
actual cumulative usage. Missing remote usage retains the durable reservation;
charging a finite reservation in the live tracker is not provider evidence.

## Upgrade and explicit reconciliation

Legacy `cost_ledger` rows are preserved as history. They have no trustworthy call
or owner identities and are never fabricated into the new settled balances. A
new scope matching any legacy scope/id starts unreconciled and refuses admission.
Because old history lacks owner proof, equal logical scope IDs can conservatively
block more than one owner until each opening balance is established.

Stop old writers before rollout. Inspect and explicitly reconcile opening balances
using evidence from the billing source. Both opening cost and token totals are
required. An opening balance can be installed only on an unreconciled scope with
no pending calls; an active balance cannot be rewritten through this operation.
Rollback pauses new budgeted work. Do not revert to a process-local empty ledger.

## HTTP operator contract

All routes are under `/api/uar/accounting` and derive the full verified owner from
authentication. A `?owner=<complete-owner-key>` override requires trusted host
capability. Deployment/global operations also require trusted host capability.
The API does not accept a model-provided tenant assertion.

- `GET /scopes/{scope}/{scope_id}` reads the bounded aggregate and generation.
- `GET /calls/{run_id}` reads exact-owner call records, including pending usage.
- `PUT /scopes/{scope}/{scope_id}` requires trusted host capability and
  `{ "expected_generation": 1, "limits": { "cost_usd": 10, "tokens": 100000 },
  "evidence": "operator policy change reference" }`. Optional
  `opening_cost_usd` and `opening_tokens` establish a legacy opening balance under
  the restrictions above. Every update advances the generation.
- `POST /calls/{run_id}/{call_id}/{attempt}/reconcile` requires trusted host
  capability, `cost_usd`, `tokens`, `confirmed_provider_usage: true`, and nonempty
  `evidence`. Supply actual confirmed cumulative provider counters. Repeated
  identical terminal reconciliation is idempotent; a conflicting settled counter
  is rejected. No timer or blanket reset is a reconciliation source.

Conflicts/unresolved accounting return 409, missing records 404, and unsupported
or unavailable storage 503. Legacy `list_cost_history` remains legacy history;
use these new inspectable records for current budget authority.

## Deferred acceptance

`tests/run_cost_settlement.rs` owns a real persistence-boundary fixture: two child
process boots against one SurrealKV directory retain pending liability, replay a
settlement, enforce limit generations and owner identity, and preserve legacy
history. Dedicated shared SurrealDB and Postgres cases start competing child
processes for the same final budget. Shared tests are explicitly ignored by
ordinary test selection because they require dedicated infrastructure; select
those cases deliberately in the final phase gate. Required environment variables
are `UAR_ACCOUNTING_SURREAL_URL`, `SURREAL_USER`, `SURREAL_PASS`, explicit trusted
`UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1`, and/or
`UAR_ACCOUNTING_POSTGRES_URL` for a dedicated pgvector-enabled database.

These fixtures do not by themselves prove the shipped HTTP-to-provider path,
remote-peer cleanup, outage terminal presentation, or independently usable
customer artifact. Those remain final phase integration/customer-build work;
no source-only inspection is acceptance evidence.

## Safe binding refusals

For remote-deployment upgrade guidance, both confirmed #385 admission causes,
model verification, knowledge ownership and team reclaim, see
[remote-Surreal upgrades](remote-surreal-upgrade.md).

The trusted host projects durable-store eligibility into a fixed diagnostic category
before provider capture. `remote_not_attested` becomes
`remote_surreal_durability_not_attested`; an unsupported backend becomes
`durable_storage_unsupported`; and a missing verified principal becomes
`verified_principal_required`. Public run admission retains
`orchestrator_start_failed` with a generated diagnostic reference. It does not
publish an endpoint, namespace, provider URL, secret, database error chain, or
constructor detail.

A valid `local_durable` or `remote_attested` eligibility may proceed only through
the existing durable-accounting admission path. This classification does not grant
an untracked run durable budget authority and does not change reservation,
settlement, cancellation, or join ownership.

Startup warns when persisted token/cost-limited agents use a store without
durable accounting, reporting `budget_durable_storage_unavailable`, captured
eligibility and a count. It does not reject saving an agent or replace per-run
admission. A later refused durable call logs `model_call_admission` and
`budget_admission_failed`, an allowlisted accounting cause, diagnostic reference,
reservation source/bound and configured turn/session token limits. It never logs
the raw error chain or request. Without a final-wire contract, a 16,000-token
envelope can refuse a large-context model even for a short prompt: the bound
includes catalog context and maximum output. A request output limit alone does
not establish a smaller complete bound. Explicit policies remain enforced.

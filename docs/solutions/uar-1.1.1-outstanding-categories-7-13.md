# UAR 1.1.1 outstanding issues, categories 7–13 — root causes and architectural solutions

Status: analysis only. No code was changed to produce this document.
Evidence base: working tree at HEAD `98b50f6e` plus the uncommitted 1.1.1
qualification work (source inspection only; no builds or tests were run, so
every claim below is a source-reading claim, not a runtime claim). Receipts
referenced from `.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/receipts/`.

Reading rule for every category: the qualification failures are not mostly
"code that does the wrong thing"; they are **two structural patterns** that
recur across subsystems, plus fixture weaknesses that made passing runs weak
evidence. The two patterns:

1. **Optimistic-concurrency losers are treated as failures.** Several durable
   state machines use `UPDATE … WHERE data=$before` compare-and-swap (CAS) as
   their transition primitive. When two tasks transition the same row, the
   SurrealKV loser's implicit transaction aborts at COMMIT with the typed
   `QueryError::TransactionConflict`. Only one site currently maps that to the
   benign "CAS miss" result; the rest propagate a fatal `Err`, which callers
   translate into `projection_unavailable`, `TaskError::Host`, or a generic
   503. This is the root of categories 7 and 9 and a standing hazard for the
   lifecycle work in flight.
2. **Read/list paths do not share the durability contract of the write paths.**
   Writers write-through to durable tables; list endpoints read only in-process
   maps. This is the root of category 8 and half of category 13's weak
   deletion evidence.

---

## Category 7 — Competing completion operations and event consistency (R9)

### Observed failure

Live AG-UI POST streams emitted `RUN_ERROR projection_unavailable` while the
durable replay (GET reconnect) returned the complete successful projection.
Reproduced 3/3 on candidate `a38c3169`; mechanism confirmed on `8e99277b`
(observer logged `agui reconcile: seal` Err). The current fix
(`3325e9dd` and later candidates) maps the conflict to a benign result, but the
mapping path has never been observed firing in a pass — the race was never
deterministically reproduced, so passing runs do not disprove it.

### Root cause (verified in source)

Two concurrent tasks own the same `agui_runs` row lifecycle and both seal it:

- `POST /api/uar/ag-ui/agents/{id}` (`src/uar/api/agui.rs:67`) spawns the
  `observe()` terminal-projection task (`agui.rs:201`, task body `:302-344`,
  loop calling `reconcile()` at `:317`).
- The same handler's `stream()` (`agui.rs:202`), and every reconnect GET
  (`agui.rs:239`), run their own `reconcile()` loop (`agui.rs:394`).
- `reconcile()` (`agui.rs:266-309`) loads the row, sees `terminal == None`,
  derives a terminal event (interrupt-seal `:281/289`, error-seal `:298`,
  terminal-seal `:289`), and calls `seal()` (`agui.rs:257-261`), which does
  `compare_and_swap_agui_run` → `agui_runs/surreal.rs:34-56`.
- The CAS is `UPDATE … WHERE data=$before`. When both tasks load the same
  `before` and both commit, SurrealKV aborts the loser at COMMIT with
  `QueryError::TransactionConflict`. Before the fix, `.await?` propagated it;
  the stream's error arm at `agui.rs:394` yielded `RUN_ERROR
  projection_unavailable` (second emission site `agui.rs:433`). The winner's
  durable seal is why replay succeeded.

The fix is present and correctly commented
(`src/uar/persistence/providers/agui_runs/surreal.rs:47-54`): the loser's
`TransactionConflict` returns `Ok(false)`, and `seal()` (`agui.rs:261`)
reloads the winner's sealed row. **The seal-once invariant is enforced by
`validate_transition` (`agui_runs.rs:58-67`, before-terminal-None, revision+1)
plus the CAS predicate — the loser's reload returns the authoritative terminal
result.** So one authoritative terminal result and correct loser handling
exist in code; what is missing is proof, and the same defect class in sibling
state machines.

Remaining defects in the same family:

| Site | Defect |
|---|---|
| `agui_runs/surreal.rs:22` (create) | Admission CAS maps only the string `agui_context_conflict` to `Ok(false)`; a genuine `TransactionConflict` on the create transaction is a fatal `Err`, swallowed as `unavailable()` at `agui.rs:172`. |
| `a2a_bindings/surreal.rs:38-52` | Transition CAS has `.check()?` at `:49` and **no** conflict mapping; losers abort as `TaskError::Host` at `api/a2a/thread_service.rs:540,827,895,969,1031,1101`, observer `:1155-1159`, `runtime/thread/actor_host.rs:466`. |
| `mcp_elicitations/surreal.rs:19-30` | Transition CAS `.check()?` at `:27`, no mapping; callers `mcp/interaction.rs:69,146`, `api/mcp_interactions.rs:36,59`. |
| `agui.rs:94,97,100,118,119` | `Err(_) => return unavailable()` arms discard all error detail with no log; `agui.rs:433` emits the error frame without logging its cause. |
| `checkpoint_records/surreal.rs:40,273-288` | Classifies conflict but keeps it fatal (recovers only via explicit readback in `claim()`). |

### What must change and why

1. **Make "lost CAS race ⇒ `Ok(false)` ⇒ reload" a property of every durable
   transition primitive, not of one call site.** The invariant a caller needs
   is: *the row is sealed exactly once; a loser observes the winner's result*.
   That belongs in the provider layer so no caller can get it wrong. Add the
   `TransactionConflict → Ok(false)` mapping to
   `a2a_bindings/surreal.rs::transition`, `mcp_elicitations/surreal.rs::transition`,
   and the `agui_runs` create path, each mirroring the verified comment block
   at `agui_runs/surreal.rs:39-46`.
2. **Log every discarded error.** Replace bare `Err(_)` arms in `agui.rs`
   (94/97/100/118/119) and the unlogged frame at `:433` with
   `tracing::warn!` including the outermost context chain, so "the real error
   is swallowed without logging" can never recur silently.
3. **Optional hardening (recommended):** collapse the dual-sealer design. The
   `observe` task is the projection owner; the stream loop could call
   `reconcile` read-only (load; if `terminal.is_none()`, await the observer's
   broadcast instead of sealing itself). That removes the race rather than
   tolerating it. This is a behavior-affecting change to live-stream latency
   paths and should be a separate, reviewed change — the CAS-loser fix is
   correct and sufficient for the invariant.

### Proof (maps to the "how we prove it" column)

- Deterministic barrier test: a test-only `PersistenceLayer` wrapper (pattern
  already exists at `src/uar/persistence/scoped.rs:157-163`) holds both writers
  at a `tokio::sync::Barrier` placed **between `load_agui_run` and the CAS
  commit**, then releases both seals. Assert: loser receives `Ok(false)`
  (assert the debug log line `AG-UI run transition lost a write race`), reload
  returns the winner's terminal, exactly one `revision == 2` row exists, and
  the live SSE stream never contains `projection_unavailable`. Repeat with
  both orderings.
- Live/replay consistency assertions in the same harness: byte-identical
  terminal event, cursor monotonicity, and identical frames from the live POST
  and the reconnect GET.
- Reconnect/snapshot recovery: after sealing, kill the SSE connection
  mid-stream, reconnect via GET, and assert the resumed projection equals the
  sealed row (no re-seal, no revision bump).
- Sibling machines: equivalent (barrier-forced) loser tests for
  `a2a_bindings` and `mcp_elicitations` transitions.

---

## Category 8 — History durability and restart recovery (R20)

### Observed failure

On the **main binary**, run history went from two runs to zero after a clean
stop and restart against the same durable store, while KB/docs/agents/memory
persisted and sidecar restart checks passed.

### Root cause (verified in source; ranked)

**H1 — primary: the list path never reads durable history.**
`GET /api/uar/runs` → `list_runs` (`src/uar/api/routes.rs:159-171`) →
`RunManager::list_runs_for_context`
(`src/uar/runtime/manager.rs:8726-8757`), which reads **only** the in-process
`active_runs` map. There is no durable fallback and no boot-time rehydration
anywhere: the only callers of `load_run_history_record` /
`load_run_history_events` in the tree are the single-run read
`get_run_for_context` (`manager.rs:8699`) and `history_since` (`:8609`).
Before the restart the two runs were visible only because terminal runs
linger in `active_runs` for `retention_after_terminal_secs: 600`
(`src/config.rs:334-344`; sweeper `manager.rs:1455-1530`). Restart → empty map
→ `[]`. The write side is healthy: admission insert (`manager.rs:4613-4615`),
per-event write-through, and terminal record upsert **before** the terminal
frame publishes; `shutdown_run_history` (`manager.rs:2182-2188`) is wired as a
supervisor finalizer (`src/server.rs:1591`, `src/embedded.rs:609`). So the
records exist; the list simply cannot see them. Distinguishing check: after
restart, `GET /api/uar/runs/{id}` returns the run (fallback exists) while the
list is empty — that split is H1's signature, and it matches "other objects
persisted" (their list endpoints query the durable tables directly).

**H2 — secondary, config-dependent: the durable gate can be silently false.**
`RunHistoryWriter::start` (`manager.rs:194-199`) returns `None` silently when
`supports_durable_run_history()` is false; `surreal.rs:422-438` grants it only
to `LocalDurable` (embedded file/surrealkv) or `RemoteAttested` (remote **plus**
env `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1`, read at `surreal.rs:424`;
nothing in `src/` sets it). A main binary on a remote endpoint without
attestation persists every ungated object (KB, agents, sessions) while
silently skipping run history. H2 requires that specific config; the default
(`surrealkv://./data/uar.db`, `src/config.rs:1654`) is durable.

**H3 — ruled unlikely: owner-namespace mismatch.** The record's `owner_key`
is `presentation_owner_key()` (tenant+subject; credential principal excluded —
`src/uar/runtime/actor/messages.rs:133-144`), stable across boots for the same
auth mode. Mixing anonymous and bound boots between restarts would hide rows
on the read path only, not produce a clean zero.

### What must change and why

1. `manager.rs:8726-8757` — add a durable fallback to
   `list_runs_for_context` mirroring `get_run_for_context`'s ownership
   predicate (`manager.rs:8703-8710`): when `active_runs` is exhausted (or
   merged, deduped by `run_id`), list from `durable_run_records` filtered by
   the recomputed live owner key.
2. `src/uar/persistence/mod.rs:122-126` — the trait has no owner-keyed list;
   add `list_run_history_records(owner_key)` with a Surreal implementation
   (`SELECT VALUE data FROM durable_run_records WHERE owner_key=$key`). The
   index already exists and is currently unused:
   `migrations/surrealdb/run_history.surql:6`,
   `migrations/20261018000000_run_history.sql:4`.
3. `manager.rs:196-199` — log once (warn) when `RunHistoryWriter::start`
   returns `None`, so H2 is observable instead of silent.

### Proof

Write two identifiable runs (known IDs and payload markers) plus the required
objects (KB, memory, agent), stop cleanly (graceful shutdown path, not
SIGKILL), restart the **same candidate binary against the same store**, and
recover: list contains both IDs; `GET /runs/{id}` record and event history
byte-identical to pre-restart; no duplicate events (no rehydration re-emit);
unknown random ID 404; no phantom active runs; deleted objects stay absent
(existing R20 no-resurrection legs). Ownership legs: different owner sees
none; anonymous/bound modes never mixed. No reseeding — run seeding does not
exist for runs (only agents/KB, `src/embedded.rs:521-522`), so any post-restart
run rows must have come from the store.

---

## Category 9 — Concurrent migration and uncertain outcomes (503)

### Observed failure

Concurrent identical credential-migration requests returned HTTP 503 without
enough detail to identify the cause. The new `MigrationCoordinator` is a
proposed remedy, not proof.

### Root cause (verified in source)

Migration is **conditional ownership re-binding** (ciphertext copied verbatim;
journal receipt `provider_credential_owner_migrations` commits atomically with
the destination row; source quarantined —
`src/uar/security/credentials/principal_store.rs:486-488`). Two concurrent
migrations race at SurrealKV; the archived repro
(`dot-feature-support` evidence `20261012T222440Z-readonly-conflict-repro.*`)
shows that a conflict produces the typed `TransactionConflict` **plus
secondary abort errors** ("NotExecuted"/"Cannot COMMIT") that carry no `uar_*`
marker. The old classification string-matched three sentinels, so secondary
errors fell through to `anyhow!("credential migration persistence failed")`,
the API downcast found nothing, and the client got 503
`credential_migration_unavailable` with no `operation_id`
(`src/uar/api/credentials.rs:117`).

Working-tree state (uncommitted, `migration.rs:186-293`):

- `MigrationCoordinator` (`migration.rs:211-293`) serializes admission per
  source key; identical fingerprints single-flight on a `watch` channel and
  share one result; different fingerprint → `Contention` (409); closed →
  `Unavailable`; panic or dropped receiver → `OutcomeUnknown` (503) with
  `operation_id`; shutdown awaits tasks and reports unresolved IDs
  (`src/server.rs:813-821` fails shutdown on them).
- Classification now ranks explicit rejection > `Contention` > other
  (`principal_store.rs:505-513`), maps `TransactionConflict` → `Contention`
  (`:363-365`) and Postgres 40001/40P01 (`:658-661`), and reports dispatch
  loss as `OutcomeUnknown` **without claiming rollback** (`:516-518`).
- The unrelated preview bug is fixed and committed: `unwrap_surreal_value`
  now maps `J::String("None"|"Null")` → `J::Null` (`surreal.rs:3381`), so
  `row_revision` deserializes `None` and preview reports
  `revision_unavailable` (`migration.rs:162`) instead of inventing "ready".

**Structural gaps that remain:**

1. Coordinator state is **in-process only**. A crashed or restarted process
   leaves no queryable operation state; only the journal receipt survives.
2. No operation-status endpoint. Uncertainty is expressible only as one 503
   response; a client that times out cannot reconcile `operation_id` except by
   re-issuing the migration.
3. Residual generic sinks: `credentials.rs:51,64,117,143,151` and the
   `Unavailable` catch-all at `principal_store.rs:374` still collapse
   unmatched causes into a detail-free 503.

### What must change and why

1. **Persist operation state.** Write an operation record
   (`operation_id`, fingerprint, state ∈ {in_progress, migrated, failed,
   unknown}, timestamps, terminal error class) transactionally with the
   migration journal, updated at flight start and terminal. This converts
   "timeout/lost response" from an unverifiable 503 into a reconcilable
   outcome — which is exactly the proof requirement: a lost response must be
   reconciled against persisted operation state before any retry, and a
   timeout must not be read as rollback.
2. **Add `GET /api/uar/credentials/migrations/{operation_id}`** returning the
   durable operation state (404 only when the id is unknown to the journal
   *and* the live coordinator). The journal's deterministic source key already
   gives idempotency (replay returns the stored receipt,
   `principal_store.rs:192-196,477-479`; Postgres `ON CONFLICT DO NOTHING`
   `:631-635`), so a reconciled retry of a completed migration is a no-op that
   returns the receipt — exactly-once effect.
3. **Narrow the generic 503s.** Every remaining `Unavailable`/503 sink must
   carry `operation_id` plus the ranked cause class; unknown causes get their
   own label instead of `credential_migration_unavailable`.

### Proof

Issue N identical concurrent migrations (same key, same fingerprint) plus M
conflicting ones (different fingerprint): every identical joiner receives the
same classified terminal result (200 receipt or the same error class); exactly
one journal receipt and one destination effect exist (inspect durable rows);
conflicting ones get 409 `credential_migration_unavailable`→`credential_migration_contention`
with `operation_id`. Kill the process mid-flight (or drop the response via a
fault-injection wrapper), then reconcile: `GET …/migrations/{id}` reports
`in_progress` or `unknown`; re-issue only after reconciliation; assert no
duplicate or partially corrupted effect (single receipt, single destination
row, quarantined source intact). Timeout never asserted as rollback.

---

## Category 10 — Context-window accounting (R21)

### Observed failure

R21 historically omitted tool-schema cost (request 6118 tokens > 5919 window).
The committed fix counts tool definitions, but passing runs still rest on
approximate character-based estimates, which bounds what they prove.

### Root cause / current state (verified in source)

A real tokenizer exists: `TokenService`
(`src/uar/runtime/context/token_service.rs:2`) counts with tiktoken-rs
`cl100k_base`/`o200k_base`, adds OpenAI-chat framing (+3/message,
`:62-64`), and quality-tags counts Exact/Approximate (`:5-27`). What it counts
today: tool schemas (`context/reduce.rs:221-228`), world state
(`world_state/contributor.rs:143-155`), history/system/skills
(`context/manager.rs:74-94`; skill catalog `skills/catalog.rs:53-77`), and the
boundary verify in `reduce_history` (`reduce.rs:370-378`) which refuses with
`context_overflow` **before dispatch** (`manager.rs:6386-6413`,
`orchestrator.rs:1980-1993`). Trim is sliding-window, oldest-first, protects
system/tool-call pairs/multimodal, and is reported via `context_updates` SSE.

The approximations that remain:

| Site | Approximation |
|---|---|
| `src/uar/memory/context_builder.rs:89-93` | memory block = chars/4, capped by min(config, 10% of a **static** surreal-memory profile window `:53-54`, from `crates/…/surreal-memory/src/model_profiles.rs:35-111`) |
| `src/uar/rag/chunking.rs:86-87` | chunk size in `tokens*4` chars |
| `src/llm/anthropic_cache.rs:62-63` | cache eligibility `len()/4` |
| `src/uar/runtime/manager.rs:6260-6261, :6285` | 0.7 headroom factor; `unwrap_or(8_192)` fallback window |
| `token_service.rs:62-64,110-127` | OpenAI framing approximated for non-OpenAI models; non-OpenAI silently falls back to cl100k |
| `context/manager.rs:150` vs `reduce.rs:372` | sliding-window loop counts text-only while final verify counts tool_calls — trim can undershoot and hit the final refusal |
| `reduce.rs:216` | `RESPONSE_RESERVE_TOKENS = 1000` static, not per-model |

And the declared-contract machinery exists but is dormant in production:
`LlmRequest.budget_contract` (`src/llm/mod.rs:343`;
`context/budget.rs:96-103`) is set only by
`DestinationRequestPreparations::prepare` (`orchestrator.rs:475`), stripped on
the team path (`:961`), and **no production path installs preparations**
(`manager.rs:1291` passes None; the setter `:2039` is test-only), so
production is labeled `legacy_unbudgeted` (`orchestrator.rs:1006-1016`). The
leaf already enforces exactness — `liter_driver.rs:522-536` rejects
Approximate counts, and pure `plan_budget` refuses them
(`budget.rs:147-158`).

### What must change and why

1. **Install destination preparations in production** (`manager.rs:1291`) so
   every dispatched request carries a `budget_contract` with per-destination
   limits (context window, output/reasoning reserve, wire/counting contract).
   This is the operator's recorded trust decision (provider-declared
   `context_window` in `llm/catalog.rs` → `registry.rs:93,313-315`; resolution
   order `manager.rs:6159-6188` already distrusts catalog capacity for
   gateways — keep that).
2. **Derive the output reserve from the contract** instead of the hardcoded
   1000 (`reduce.rs:216`, `manager.rs:6290`, `context/manager.rs:79`,
   `retention.rs:46`, `orchestrator.rs:1967`).
3. **Replace char-based terms with the TokenService**: memory block
   (`context_builder.rs:89-93` + static profiles), rag chunking
   (`chunking.rs:86-87`), cache eligibility (`anthropic_cache.rs:63`).
4. **Fix the trim/verify mismatch** (text-only sliding-window counting vs
   tool-call-aware final verify) so a trimmed request that "fit" mid-loop
   cannot hit `context_overflow` at verify.
5. **Make the accounting contract explicit per destination** in
   `WireContract` terms (serialized wire counts, `budget.rs:88-92`) rather
   than OpenAI framing approximations for every provider.

### Proof

Frozen boundary cases per model contract: (a) exactly-at-limit fits and
dispatches; (b) over-limit by exactly the reserve triggers trim of exactly the
eligible class (recorded order, `context_updates` observed); (c) unprotected
overflow refuses pre-dispatch with `context_overflow` and no upstream request
(pattern already asserted at `tests/context_history_integrity.rs:1405-1551`);
(d) ±1-token boundary table through `plan_budget`
Fit/CompressEligible/ProtectedOverflow/UnsupportedCount
(`budget.rs:356-374`); (e) recall-after-trim: protected groups (system, memory
codeword, RAG source) verified present post-trim
(`verify_protected_groups` + world-state max(full,delta) reserve). Every
fit assertion must be Exact-tagged; an Approximate count may never certify
fit. Validate the whole table against the declared model accounting contract
(`context_window` + reserve), not the chars/4 surrogate.

---

## Category 11 — Hierarchical budgets and error reporting (R22)

### Observed failure

Root refusal for input+reserved-output worked; child/aggregate admission,
pending reservations, and client-visible refusal classes lacked complete
proof, and some fixtures had invalid arithmetic (16000-token catalog
liability × 2 > 30000 limit, so the second admission was refused before
aggregation could be observed; an intermittent `other_storage_error` appeared
once).

### Root cause / current state (verified in source)

Implemented (`src/uar/runtime/cost_budget.rs`): root budget `for_run`
(`:752-780`) fixes scopes `[Run, Session, Agent, Global]`; children share the
root object via `narrowed()` (`:795-808` — intersect-only, cannot widen;
`thread/kernel.rs:407`, `turn/bindings.rs:414`). Admission
`BudgetedModelDriver::stream` (`:1200-1214`) admits then durably reserves
`begin_call`; without `budget_contract` the reservation is catalog
`context_window + max_output` (`:1543-1547`) enforced in the durable ledger
(`accounting.rs:191-213`). Terminal settle releases the reservation and
commits actuals (`accounting.rs:235-250`). Refusal classes travel as
`BudgetFailureOrigin` → `client_failure` (`:48-69,73-81`) → SSE
`NormalizedEvent::Error{code}` (`orchestrator.rs:1036-1038`); tree limits use
`host_failure_label` (`thread/control.rs:272-279`,
`agent_tree_limit_exceeded: max_concurrent_children`, gate default 4 at
`limits.rs:24,164-168`).

Real defects found:

1. **Tool-admission refusals lose their class.** `admit_tool`
   (`cost_budget.rs:958-966`) wraps only the inner `admit()` with
   `BudgetFailureOrigin::Admission`; `tracker.admit_tool` errors
   ("Root tool-call budget exceeded", `:472`) reach the client as generic
   `provider_error`. One-line fix: context-wrap like `admit()` does (`:955`).
2. **Reservation leak on non-terminal stream end.** On root cancellation,
   deadline, or upstream stream failure the stream breaks
   (`cost_budget.rs:1215-1218,1233-1240`) without terminal `call.usage`; the
   durable `AccountingCall` stays `Pending`, the `reserved_tokens` liability
   persists in every scope, and `unknown_calls` then blocks all finite-limit
   admissions (`accounting.rs:196-197`). Storage failure mid-settle only
   warns + emits an `unresolved` custom event (`:1271-1277,1295-1302`) and is
   never retried. The spec's orphan sweep (spec §8/M2.7,
   `docs/uar-budget-system-spec-2026-07-29.md`) is unimplemented.
   Unpriced models never settle (`accounting.rs:229`) — same poisoning.
3. **The OpenAI SSE lane drops error classes entirely.**
   `src/uar/api/openai/routes.rs:161-195` maps only ChatDelta/RunDone/Error;
   a budget refusal breaks the stream with no error frame (and no `[DONE]`),
   so a chat-lane client cannot observe the class at all.
4. `budget_contract` remains unset in production (category 10, item 1), so
   reservations are always the coarse catalog `context+output` figure.

`other_storage_error` (`cost_budget.rs:23-31`) is any non-TransactionConflict
surreal error during `begin_call`'s pre-transaction reads
(`providers/accounting/surreal.rs:57-93`) — single-shot, no retry; the
intermittent c6 hit points at storage I/O, not CAS.

### What must change and why

1. Fix `admit_tool` class wrapping (`cost_budget.rs:963-965`).
2. Guarantee reservation settlement: emit terminal usage (or an explicit
   release marker) on every stream end path — cancellation, deadline,
   failure — and add the spec's orphan sweep: on boot and on scope close,
   resolve `Pending` calls older than a horizon to `Settled(released)` with an
   emitted reconciliation event. Storage-failed settles must retry from the
   durable `AccountingCall` row (it exists; `list_accounting_calls` at
   `api/accounting.rs:104` is inspection-only today).
3. Forward budget refusal classes on the OpenAI SSE lane (terminate with an
   error frame carrying the code, then `[DONE]`).
4. Once category 10 lands, reservations become per-contract instead of
   catalog-total, which also fixes the fixture arithmetic hazard below.

### Proof

Valid, deterministic fixtures per frozen limit L: each admission reserves
`context_window + max_output` from the model's declared contract, so N calls
fit iff N×(ctx+maxout) ≤ L (e.g. ctx=8000/maxout=8000, L=40000 → 2 admitted,
3rd refused). Assert on raw events, not summaries: SSE `Error.code ==
"budget_admission_failed"` with message containing `accounting_budget_exhausted`;
tree refusal tool-result text `agent_tree_limit_exceeded:
max_concurrent_children`; allowed work executes (upstream request observed);
over-limit work never dispatched (no upstream request); durable
`list_accounting_calls` shows `Pending` during flight and `Settled` with
correct `reserved_tokens`→actual transition after; aggregation: children's
settled actuals visible in the parent scope keys (single mutex + CAS ledger
already guarantee no double-count); cancellation leg: cancel a child
mid-flight and assert its reservation settles (sweep or terminal path), and
the parent's next admission is decided on true liability.

---

## Category 12 — Remote storage proof (R15)

### Observed failure

R15 used a memory-backed disposable remote database (cannot prove restart
behavior), and the required model/storage flow turn was blocked by an 8192
default context budget.

### Root cause / current state (verified in source)

- The old fixture was `docker-compose.test.yaml:88-89`: pinned
  `surrealdb/surrealdb:v3.3.0` running `start … memory` — disposable by
  construction.
- Remote durability is gated: `surreal.rs:422-438` classifies remote endpoints
  as `RemoteAttested` only with `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1`
  (`:424-429`); otherwise `RemoteNotAttested` and **all** durable capability
  gates (tool receipts, elicitations, agui_runs, run_history, a2a_bindings,
  accounting, ingestion, approvals, supervisor fencing, checkpoint resume —
  `surreal.rs:921-1056`) return Unsupported. Any remote fixture that forgets
  the env silently degrades to in-memory-only semantics for those legs.
- The 8192 blocker: a custom (non-catalog) model gets
  `context_window: None` (`src/llm/registry.rs:335-348`) and the window
  resolves to `.unwrap_or(8_192)` (`manager.rs:6285`); the recorded R15 turn
  failed "World state requires 11506 tokens, exceeding the 8192-token model
  budget".
- A persistent remote fixture already exists in-repo:
  `docker-compose.dev.yaml:66-78` — image
  `surrealdb/surrealdb:v3.3.0@sha256:681c…cd20`, `start --user root --pass
  root file://data/surreal.db`, named volume, `/health` healthcheck. Migrations
  run on every connect (`surreal_host.rs:34-118`, 20 definitions), so a fresh
  container over an existing volume re-attaches rather than reinitializing.

### What must change

No product change is strictly required for the storage legs; the changes are
fixture-side plus the category-8 list fallback (otherwise the R15 restart leg
cannot show runs on any binary). Required fixture properties:

1. Genuinely persistent remote: docker-compose.dev-style service (pinned
   image digest, `file://` or `rocksdb://` backend on a named volume,
   healthcheck) + runtime config `persistence.provider: surreal` pointing at
   it (`config.remote.surreal.yaml:30-37`) + `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1`
   on the runtime so durable gates are active. Restart the Surreal container
   (and separately the runtime) between legs to prove durability, not liveness.
2. Model/context settings established first: declare the fixture model in
   `providers:` with explicit `context_window` (pattern
   `example.config.yaml:234-253`) or select a catalog model, and set the
   embedding backend (`llm.embedding` loopback mock or `UAR_MODELS_DIR`), so
   the storage test is not obscured by context/embedding failures (the
   F-KB-EMPTY-500 lesson).
3. Run the representative flow (settings, credentials, agents, KB
   docs+vectors, a run with history) → clean stop → restart same candidate
   binary against same remote store → assert same IDs/content for every
   recoverable-by-design object (`surreal_host.rs` migration set is the
   authoritative list), deleted objects absent, no duplicates.

The uncomfortable part to record: `RemoteAttested` is an **honesty env flag**,
not an attestation mechanism. The proof certifies the fixture was configured
durable; it does not certify remote durability generally. That is acceptable
for qualification only if stated in the receipt.

---

## Category 13 — Retrieval, citations and deletion (R16/R17/R18/R19)

### Observed failure

Past proofs had partial operations without independently scored search; a
deletion control that already returned no results before deletion (absence
proved nothing); missing KB-delete evidence; citation evidence was
candidate-specific, and the R18 turn had no SSE citation event.

### Root cause / current state (verified in source)

The pipeline is sound and deterministic: upload commits source+document+job in
one transaction (`persistence/providers/ingestion_records/surreal.rs:35-55`);
the worker claims jobs via CAS (`:118-143`), chunks (default `Structured{1024}`,
`domain/knowledge.rs:293`), embeds with the single global pinned backend
(bge-small-en-v1.5, 384-d, deterministic — `rag/embeddings/fastembed.rs:27,150`;
per-KB `embedding_provider` is descriptive-only, `knowledge.rs:756-770`; no
backend ⇒ search 500, `knowledge.rs:614-622→675`), publishes atomically
(delete old chunks by `document_id`, create new, mark indexed —
`surreal.rs:145-174`), and search is in-memory cosine over KB chunks with
score filter and top-k (`providers/surreal.rs:2707-2761`; route defaults limit
5/min 0.7 at `knowledge.rs:122-127`; turn retrieval top_k 3/min 0.7 at
`domain/knowledge.rs:10-13,307-321`).

Citations exist end-to-end but are dropped on one lane:
`CitationStream::from_matches` both injects the numbered `[RELEVANT KNOWLEDGE]`
block (`rag/citation_stream.rs:140-155`) **and emits
`NormalizedEvent::RagCitations`** with marker/chunk_id/document_id/kb_id/
document_name/relevance_score/snippet (`:74-83`; `domain/events.rs:34`), wired
on AG-UI as `agui.rag_citations` (`api/sse.rs:455-468`) and adapters as
`uar.rag_citations` (`api/adapters.rs:114-118`) — but
`/v1/chat/completions` SSE maps only ChatDelta/RunDone/Error and drops
`RagCitations` (`src/uar/api/openai/routes.rs:161-195`, `_ => {}`). That is
the mechanical reason the chat-lane turn showed no citation event.
`KbMemory.citation_required` (`domain/artifact.rs:319`) is dead config.

Deletion gaps: one transaction removes doc + chunks by `document_id` + KB
cascade (`ingestion_records/surreal.rs:191-211`) — vectors live inline in
chunk rows, so purge is complete — **except** legacy `ingest_file` chunks with
`document_id: None` (`rag/ingest.rs:150`) survive doc deletion. The concurrent
"second upload during ingestion returned 500 failed transaction" is the
`accept` transaction's KB-row touch (`UPDATE $kb.id SET updated_at`,
`surreal.rs:43`) colliding with `publish`'s `UPDATE … config.indexed_embedding`
(`:167`) on SurrealKV; no retry exists (`knowledge.rs:471,508-516`).

### What must change and why

1. Forward `RagCitations` on the OpenAI SSE lane (`openai/routes.rs:161-195`)
   as a frame carrying score/doc/chunk ids (same fix family as category 11
   item 3 — one lane-map edit each).
2. Remove the KB-row `updated_at` touch from `accept` (or retry on
   TransactionConflict) so concurrent upload during ingestion cannot 500.
3. Purge legacy `document_id: None` chunks on deletion (or migrate them);
   optionally report purged-chunk counts in delete responses as evidence.
4. Read `citation_required` or delete the dead config so the citation
   contract is enforceable rather than vestigial.

### Proof (the fixture discipline that makes absence meaningful)

Seed **two distinct-content docs** serially (target A, control B) with known
expected results and source IDs; wait for `indexed`; record ids, filenames,
expected chunk counts. Then, in order:

1. **Retrieval before deletion:** scored searches for both — A-query returns A
   with recorded score (~0.84 class), B-query returns B; neither is empty.
2. **Citation match:** an agent turn on a KB-bound agent asserts the
   `[RELEVANT KNOWLEDGE]` block numbered `[1] (A-filename)` in the captured
   upstream request **and** a citation event whose `document_id`/`chunk_id`
   equal A's — the citation must match the actually retrieved source.
3. **Deletion:** delete A → A-query returns zero results **while B-query still
   returns B with score** (the live control — this is what the old fixture
   lacked); fresh-agent turn shows no A block, no A citation, no A codeword.
4. **KB deletion:** KB GET/list 404/absent, docs empty; A and B both
   unreachable; an unrelated control KB still serves its own doc.

Zero results before *and* after deletion prove nothing; the B control is the
discriminator. Scores are deterministic for fixed fixture text (same pinned
ONNX model), so recorded expected scores are stable evidence.

---

## Cross-cutting change index

| Theme | Files |
|---|---|
| CAS-loser semantics + logging | `persistence/providers/agui_runs/surreal.rs` (create), `a2a_bindings/surreal.rs:38-52`, `mcp_elicitations/surreal.rs:19-30`, `api/agui.rs:94-119,433`, (optional) `checkpoint_records/surreal.rs` |
| Durable run list fallback | `runtime/manager.rs:8726-8757,196-199`, `persistence/mod.rs:122-126`, `providers/run_history/surreal.rs` |
| Migration operation state | `security/credentials/migration.rs`, `principal_store.rs:374,505-518`, `api/credentials.rs:21-22,117,143,151`, new operation table + route |
| Accounting contract in production | `runtime/manager.rs:1291,6260-6290`, `llm/orchestrator.rs:475,961`, `runtime/context/{reduce,manager,budget}.rs`, `memory/context_builder.rs:89-93`, `rag/chunking.rs:86-87`, `llm/anthropic_cache.rs:62-63` |
| Budget classes + settle guarantees | `runtime/cost_budget.rs:958-966,1215-1240,1271-1302`, `persistence/providers/accounting/surreal.rs:57-93`, `api/accounting.rs:104`, `api/openai/routes.rs:161-195` |
| Citations + ingestion concurrency | `api/openai/routes.rs:161-195`, `persistence/providers/ingestion_records/surreal.rs:43,191-211`, `rag/ingest.rs:150`, `api/knowledge.rs` delete responses |

## Sequencing and dependencies

1. CAS-loser mapping + logging (cat 7) is self-contained, low-risk, and
   unblocks deterministic barrier proofs for three state machines at once.
2. Durable run list fallback (cat 8) is required before any restart proof that
   includes runs (cat 12's R15 restart leg depends on it).
3. Production `budget_contract` (cat 10 item 1) changes reservation arithmetic;
   land it **before** freezing the cat 11 fixture table, or the table freezes
   the coarse catalog figures.
4. Migration operation persistence (cat 9) can proceed independently; its
   proof needs fault injection, which belongs in a test-only wrapper, not
   product code.
5. The two OpenAI SSE lane-map edits (cat 11 item 3, cat 13 item 1) are the
   same file and one review unit.

## Honest limits of this analysis

- No build or test was run (operator verification gate). Every claim above is
  a source-reading claim; the R9 mechanism is the one claim already
  corroborated by runtime receipts.
- Category 8's H1 is the explanation consistent with the code and the
  observed split (objects persisted; runs list zero). Confirming the split at
  runtime (`GET /runs/{id}` 200 vs list `[]` after restart) is the cheapest
  decisive check and is included in its proof section.
- The intermittent `other_storage_error` (cat 11) is attributed by
  elimination to storage I/O on the admission path; it was observed once and
  is not reproduced by any code path read here.

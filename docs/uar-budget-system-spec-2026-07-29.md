# oi90fdcswUAR Budget System — Architecture, Functional Specification & Implementation Plan

**Document ID:** PAGS-SPEC-UAR-BUD-001
**Revision:** 1.0
**Date:** 2026-07-29
**Author:** Travis James / Prometheus AGS
**Status:** Proposed — not yet approved for implementation
**Supersedes:** nothing. **Amends:** the budget-enforcement claims in `uar-agent-md-spec.html` §15 (see §9.3).
**Related:** `docs/uar-next-fable.md` §3 (Router v1 gap: "budget envelopes"), §7 R3/R11 · CH-06 (`src/uar/runtime/cost_budget.rs`) · CH-07 (cost ledger persistence) · F-04 (billing/entitlement spine)

---

## 0. Summary

UAR today has two things called "budget" and neither of them is a budget.

The **token budget** (`src/uar/runtime/context/manager.rs`) is a real, functioning context-fitting mechanism — but it is called with a hardcoded 128,000-token window and its estimator cannot see tool definitions, which are the dominant context consumer under a large skill profile. The **cost budget** (`src/uar/runtime/cost_budget.rs`, CH-06) is an in-memory counter that records spend *after* a run completes and emits an SSE event when a threshold is crossed. It enforces nothing, does not survive restart, and is not coherent across replicas.

This document specifies a replacement built on three separated concerns — **estimate**, **account**, **govern** — joined by a reserve-then-settle protocol, with enforcement expressed as a *degradation ladder* rather than a kill switch. It preserves the existing CH-06 event surface, reuses the per-run `CancellationToken` already present as the actuator, and closes the "budget envelopes" gap that `uar-next-fable.md` §3 identifies as blocking Router v1.

**The load-bearing design claim:** a budget system whose only outcome is rejection is a liability in a paid product. The valuable output of a budget check is not *stop* — it is *do the same work more cheaply*. Everything below follows from that.

### 0.1 Revision of a prior recommendation

An earlier assessment of this subsystem recommended wiring liter-llm's vendored `BudgetLayer` (`vendor/git/liter-llm/crates/liter-llm/src/tower/budget.rs`) as the primary enforcement path. **That recommendation is withdrawn as the destination and retained as the stopgap.** Reasons, from that layer's own source:

- `check_budget` reads before the call and records after, with no atomicity between them. The module's own doc comment concedes that concurrent bursts can overshoot the limit.
- No reservation protocol, so it cannot bound in-flight spend.
- No window rehydration — process restart zeroes the ledger, same defect UAR has now.
- Its scope set (global / model / tenant / user / api-key) omits **run**, **session**, and **agent** — the three scopes an agent runtime actually needs to control, because runaway spend in an agent runtime is a tool-loop phenomenon, not a per-request one.

It is still the fastest way to get *any* hard ceiling in place (§10, M0.5), and its OTel metric conventions (`gen_ai.budget.spend_usd`, `gen_ai.budget.rejection`) should be adopted verbatim for downstream compatibility.

---

## 1. Requirements

### 1.1 Functional

| ID | Requirement |
|---|---|
| FR-01 | Predict the cost of a proposed LLM call before issuing it, from model, prompt tokens, output ceiling, and cache-hit fraction. |
| FR-02 | Account actual spend against every scope in the charge path, incrementally, at each LLM call — not once per run. |
| FR-03 | Bound in-flight spend, so N concurrent calls cannot collectively overshoot a limit that each individually respects. |
| FR-04 | Return a graded verdict — Allow / Warn / Degrade / Deny — not a boolean. |
| FR-05 | Degrade a run rather than kill it, where policy permits, and record that degradation in run provenance. |
| FR-06 | Survive process restart with spend intact for all open budget windows. |
| FR-07 | Be coherent across replicas when a shared ledger backend is configured, and honestly non-authoritative when one is not. |
| FR-08 | Consume every budget field the UAR-AGENT-MD spec declares, or remove the field from the spec. |
| FR-09 | Fit the request inside the *resolved* model's context window, counting tool definitions and non-text content. |
| FR-10 | Export a spend ledger suitable for metered billing and entitlement enforcement (F-04). |

### 1.2 Non-functional

| ID | Requirement | Target |
|---|---|---|
| NFR-01 | Added latency, in-memory ledger | p99 < 1 ms per gate |
| NFR-02 | Added latency, SQL ledger | p99 < 15 ms per reservation |
| NFR-03 | Estimator error, cost | \|predicted − actual\| / actual ≤ 0.25 at p90 (see §10 M0 — this is a measurement target, not an assumption) |
| NFR-04 | Estimator error, tokens | never under-count; over-count ≤ 20% at p90 |
| NFR-05 | Ledger unavailability | must not deadlock a run; policy-selected fail-open or fail-closed |
| NFR-06 | Enforcement determinism | identical inputs → identical verdict, independent of wall clock within a window |

### 1.3 Explicit non-goals

- **Not** an invoicing system. Estimated spend is never presented as billed spend (§8.7).
- **Not** a rate limiter. RPM/TPM belongs in a separate limiter; this system governs *money and tokens*, not *frequency*.
- **Not** a GPU/KV-slot admission controller. That is the parking-lot algorithm in the inference stack; this system is its economic sibling, not its replacement (§2.4).
- **Not** a replacement for Cedar. Cedar authors the policy; this system evaluates the arithmetic (§7).

---

## 2. Architecture

### 2.1 Three planes, one protocol

```
                    ┌─────────────────────────────────────────┐
                    │  RUNTIME HOST  (trusted, holds power)   │
                    │  RuntimeManager · Orchestrator          │
                    │  ─ owns CancellationToken               │
                    │  ─ owns event emitter                   │
                    │  ─ applies degradation plans            │
                    └───────┬─────────────────────┬───────────┘
                            │ verdict request     │ verdict
                            ▼                     │
        ┌───────────────────────────────────────────────────┐
        │  GOVERNOR   (policy evaluation — pure decision)   │
        │  estimate + ledger state + policy → Verdict       │
        │  cannot cancel · cannot emit · cannot write       │
        └────┬──────────────────────────────┬───────────────┘
             │ predicted cost               │ reserve/settle
             ▼                              ▼
   ┌──────────────────────┐    ┌──────────────────────────────┐
   │  ESTIMATOR (pure)    │    │  LEDGER (authoritative)      │
   │  tokens → USD        │    │  reserve · settle · release  │
   │  catalog pricing     │    │  windowed · durable          │
   │  no I/O, no state    │    │  single-writer per scope key │
   └──────────────────────┘    └──────────────────────────────┘
```

Three properties of this shape matter more than its contents:

**The Governor cannot enforce.** It computes a verdict and returns it. The power to cancel a run, reject a request, or swap a model stays in the runtime host. This is the same invariant already applied to the human approval gate — *the decision may be computed anywhere, the actuation lives only in the trusted host layer* — and it is enforced at the Cargo level, not by convention (§2.3).

**The Estimator is pure.** No async, no I/O, no clock. It is a function of (model, token counts, pricing catalog). This makes the single highest-risk component in the system exhaustively testable, which matters because every enforcement decision inherits its error.

**The Ledger is an interface.** In-memory for single-process and local-first deployments, SQL-backed for multi-replica. The Governor is written against the trait and does not know which is installed. This is what allows KnowMe's sovereign local deployment and a multi-replica AKS client deployment to share one code path with different — and honestly documented — guarantees.

### 2.2 Reserve-then-settle

The current CH-06 design records spend after the fact. That is not enforcement; it is a receipt. The protocol below is what makes a limit actually bind:

```
1. estimate     = Estimator::predict(model, prompt_tokens, output_ceiling, cache_frac)
2. verdict      = Governor::admit(scope_path, estimate, policy)
                    ├─ Deny(reason)          → host aborts, no call made
                    ├─ Degrade(plan)         → host applies plan, re-estimate, goto 1
                    └─ Allow(Reservation)    → pending += estimate  [ATOMIC]
3. <LLM call executes>
4. settle(reservation, actual_usage)  → committed += actual; pending -= estimate  [ATOMIC]
   on error / drop / panic:
   release(reservation)               → pending -= estimate         [ATOMIC, RAII]
```

`Reservation` is an RAII guard. Drop without settle releases. This is the only reliable way to avoid pending-counter leaks under the `async_stream` control flow in `orchestrator.rs`, where a stream can be abandoned mid-iteration by a client disconnect.

Remaining budget for admission purposes is `limit − (committed + pending)`. That is the entire reason the protocol exists: ten concurrent turns each individually under the limit cannot collectively exceed it, because the tenth sees the other nine's reservations.

**This is deliberately the same shape as the parking-lot algorithm** already designed for KV-cache admission in the sovereign inference stack — claim a slot at admission, release at completion, admit against *claimed* state rather than *observed* state. Reusing the pattern is not stylistic tidiness: it means one mental model covers both the GPU-memory and the dollar constraint, and an operator who understands one understands the other.

### 2.3 Crate topology — capability inversion

```
uar-budget-core          ← types, Estimator, Governor, Ledger trait, Verdict, DegradationPlan
   │                       deps: serde, thiserror, async-trait. NO tokio::spawn. NO db. NO cancellation.
   ├── uar-budget-ledger-memory   ← sharded DashMap, single-process
   └── uar-budget-ledger-sql      ← Postgres / SurrealDB, conditional-update reservations

uar (runtime host)       ← depends on all of the above; owns CancellationToken, emitter, router
```

The dependency arrow never runs from `uar-budget-core` to the runtime. The Governor *physically cannot* cancel a run, because the type that can is not in its dependency graph. This is a compile-time guarantee rather than a runtime policy, consistent with the estate-wide rule that agent kernels cannot depend on write actuators.

### 2.4 Relationship to the token budget

Cost budgets and token budgets are different constraints — one economic, one physical — that share exactly one input: **the token count of the assembled request**. They should therefore share one estimator and nothing else.

| | Token budget | Cost budget |
|---|---|---|
| Question | Does this fit in the window? | Can we afford this? |
| Failure if wrong | Provider rejects the request | Money is spent |
| Bias on error | Over-count (conservative) | Over-reserve, then settle down |
| Scope | Single request | Hierarchical, windowed |
| Enforcement | Trim / summarize (always available) | Degrade / deny (policy-gated) |

Unifying them into one "budget" abstraction would be a modelling error. Sharing the estimator is the correct amount of coupling.

### 2.5 The scope lattice

```
Global (deployment)
└── Tenant
    └── Principal (user | api-key)
        └── Agent
            └── Session
                └── Run
                    └── Turn
```

A call is charged to **every scope on its path**. Admission takes the **most restrictive** verdict across the path — `min(remaining)` — so a generous global ceiling cannot rescue an exhausted session.

This fixes a live semantic defect: today `budgets.max_cost_per_session_usd` is applied to the **Agent** scope (`runtime/manager.rs:1287`), which accumulates for the lifetime of the process across every session. One heavy session currently poisons that agent until restart.

`Turn` is new and cheap, and it is what finally consumes the long-declared `max_tokens_per_turn`.

---

## 3. Functional specification

### 3.1 Core types

```rust
// uar-budget-core

pub enum Scope { Global, Tenant, Principal, Agent, Session, Run, Turn }

/// Ordered root→leaf. Charged to every element; admitted on the strictest.
pub struct ScopePath(Vec<(Scope, ScopeId)>);

pub struct Estimate {
    pub prompt_tokens: u32,
    pub output_ceiling_tokens: u32,
    pub cached_prompt_tokens: u32,
    pub predicted_usd: f64,
    /// Estimator confidence class — drives conservatism, see §4.3.
    pub basis: EstimateBasis,   // NativeTokenizer | FamilyApprox | CharHeuristic
}

pub struct BudgetPolicy {
    pub limit_usd: Option<f64>,
    pub limit_tokens: Option<u64>,
    pub window: Window,                    // Calendar{Month|Day} | Sliding(Duration) | Unbounded
    pub warn_at: f64,                      // default 0.80
    pub throttle_at: f64,                  // default 0.95
    pub on_exhaustion: Exhaustion,         // Deny | DegradeThenDeny | WarnOnly
    pub degradation: Vec<DegradationStep>, // ordered, empty = disabled
    pub on_ledger_unavailable: FailMode,   // Open | Closed
}

pub enum Verdict {
    Allow(Reservation),
    Warn { reservation: Reservation, utilization: f64, scope: Scope },
    Degrade { plan: DegradationPlan, scope: Scope, utilization: f64 },
    Deny { scope: Scope, scope_id: ScopeId, spent_usd: f64, limit_usd: f64, reason: DenyReason },
}
```

### 3.2 The Ledger trait

```rust
#[async_trait]
pub trait Ledger: Send + Sync + 'static {
    /// Atomically reserve `estimate` against every scope in `path`.
    /// Returns Err(LedgerReject) naming the FIRST scope that cannot accommodate it.
    async fn reserve(&self, path: &ScopePath, est: &Estimate) -> Result<Reservation, LedgerReject>;

    /// Move pending → committed using measured usage. Releases the over-reserved delta.
    async fn settle(&self, r: Reservation, actual: &Usage) -> Result<(), LedgerError>;

    /// Release without commit (error / abandonment). Idempotent. Called by Drop.
    async fn release(&self, r: Reservation);

    /// Current state for a single scope, for admission arithmetic and operator display.
    async fn state(&self, scope: Scope, id: &ScopeId) -> Result<ScopeState, LedgerError>;

    /// Restore committed spend for open windows at process start (FR-06).
    async fn rehydrate(&self) -> Result<usize, LedgerError>;

    /// Authoritative across replicas? `false` for the in-memory impl.
    fn is_authoritative(&self) -> bool;
}
```

`is_authoritative()` is not decoration. It is surfaced on the health endpoint and in the admin UI, so an operator running three replicas with the in-memory ledger can see that their $100 global limit is functionally a $300 limit. Silent under-enforcement is worse than declared under-enforcement.

### 3.3 The degradation ladder

Ordered cheapest-quality-cost first. Applied only at **turn boundaries**, never mid-turn.

| Step | Action | Cost reduction | Quality cost | Reversible |
|---|---|---|---|---|
| D1 | `SuppressThinking` — drop extended-thinking dialect params | 30–60% on reasoning models | Loses deliberation on hard steps | Yes |
| D2 | `TightenContext` — lower context trigger threshold, shrink history allowance | 10–40%, scales with history depth | Earlier history loss | Yes |
| D3 | `DisableCacheHostileRetry` — suppress prompt mutations that break the cached prefix | 0–80% on cache-friendly providers | None | Yes |
| D4 | `DowngradeModel` — route to cheapest model satisfying the same `RouteRequirements` | 5–50× | Material and hard to predict | No (within run) |
| D5 | `ReduceToolIterations` — cut remaining `MAX_TOOL_ITERATIONS` | Bounds worst case | May truncate the task | No |
| — | `Deny` | 100% | Task fails | — |

**D4 is the architecturally important one.** `ModelRouter` already exists (`src/llm/router.rs`) with capability-based `RouteRequirements` filtering. Adding a `max_cost_per_call_usd` envelope derived from remaining budget converts routing from *cheapest-that-fits-capability* to *cheapest-that-fits-capability-and-budget*, which is precisely the "budget envelopes" gap named in `uar-next-fable.md` §3 as Router v1's remaining work. One change closes two roadmap items.

Two guards on D4, both non-negotiable:
- Fix `router.rs:75` first, where a model with missing cost data sorts as free. Budget-aware routing on top of that bug would preferentially select unpriced models — the exact opposite of the intent, and a silent one.
- Every degradation emits an event and is written to run provenance. A run that began on Opus and finished on Haiku must be identifiable as such forever, or every eval and every incident review downstream is corrupted.

### 3.4 Gate placement

All four locations verified against the current tree.

| Gate | Location | Frequency | On Deny |
|---|---|---|---|
| **G0** Run admission | `RuntimeManager::start_run`, before task spawn | 1 / run | Reject at API boundary; no run created |
| **G1** Turn pre-flight | `orchestrator.rs` loop head (~L500), before `LlmRequest` construction | 1 / iteration | `Error{code:"BUDGET_EXCEEDED"}` → cancel token |
| **G2** Turn settle | `manager.rs` consumption loop, `NormalizedEvent::Usage` arm (~L1651) | 1 / LLM call | n/a (accounting) |
| **G3** Run reconcile | run-end block (~L1755) | 1 / run | n/a (reconciliation) |

**G2 is the single highest-value change in this document.** `NormalizedEvent::Usage` already arrives per-LLM-call inside the run's consumption loop and is already accumulated there into `total_input_tokens` / `total_output_tokens`. It is simply not connected to the tracker. Moving accounting from G3-only to G2-incremental changes enforcement granularity from *per run* (useless — a ten-iteration tool loop is already over) to *per turn* (actionable), and requires no new plumbing.

The Deny actuator at G1 is the per-run `CancellationToken` that `start_run` already creates and `cancel_run` already triggers, producing `NormalizedEvent::Cancelled` through an existing path. No new termination mechanism is introduced.

### 3.5 Token budget corrections (FR-09)

These are specified here because they share the estimator, and because two of them are live production hazards:

1. **Resolve the real context window.** `context_manager.apply(messages, 128_000)` at `manager.rs:1034` must take the routed model's cataloged `context_window`. The message-count path four lines above already does this correctly. Current impact: ~35% of the window unused on a 200K model; on an 8K local model the trim never fires and the provider rejects the request outright.
2. **Pass the driver.** `apply()` forwards `driver: None`, so `ProgressiveSummarization` hits its fallback branch and silently becomes `KeepFirstLast`. Call `apply_with_driver`.
3. **Count tool definitions.** `TokenService::estimate_messages` counts `tool_calls` in history but never the tool *schemas* sent with every request. Under a large skill profile these plausibly dominate the prompt. Uncounted, they are exactly the silent-overflow mechanism already identified as a risk in the Skill Pack instruction-plane work, reached by a different route.
4. **Count non-text content.** `content.as_text().unwrap_or("")` scores image and structured blocks at 3 tokens.
5. **Content-type-aware tokenizer factors.** cl100k_base against Anthropic models under-counts. Per the validated figures in `uar-next-fable.md` §1.2, use ~16% (English prose) / ~30% (code) / ~21% (math) — **not** a flat 30%, and not the numbers from the unvalidated model-comparison document.

---

## 4. Design rationale

### 4.1 Why graded verdicts instead of hard/soft

The existing `enforcement: "hard" | "soft"` string offers reject-everything or warn-and-continue. Both are wrong at the moment they matter. Reject destroys work already paid for — a run killed at iteration 9 of 10 has spent the money and delivered nothing, which is the worst possible outcome on both axes. Warn is a log line no one reads.

Utilization is continuous, so the response should be too. Between 80% and 100% there is a wide band where the correct action is to keep working with cheaper means. A budget system that exploits that band converts a hard failure into a soft quality gradient. That is the difference between a control system and a circuit breaker, and it is the whole reason this design is more than plumbing.

### 4.2 Why reservations, given the complexity cost

Without them, exactness is unobtainable under concurrency, and the failure is silent and unbounded. liter-llm's layer documents this honestly and accepts it. UAR cannot: an agent tool loop issues bursts by construction, so the overshoot case is the *normal* case here, not the tail. And once the product bills on this ledger (F-04), unbounded overshoot is revenue leakage.

The honest cost: one atomic operation per turn, and on the SQL ledger one round-trip. §8.1 names the mitigation and its weakened guarantee.

### 4.3 Why the estimator carries a confidence class

`EstimateBasis` lets conservatism scale with knowledge. A native tokenizer for a known model warrants a tight reservation; a character heuristic for an unknown model warrants a padded one. Without this the system must apply worst-case padding universally, which wastes headroom on exactly the well-characterized models that carry most production traffic.

### 4.4 Why enforcement is not in Cedar

The `uar-agent-md-spec.html` §15 position — budgets enforced at the Cedar policy layer, exceeding a budget causes the policy to deny the action — is the wrong split, independent of the fact that it is currently unimplemented.

Cedar is an authorization engine over entities and attributes. It has no atomic counters, no windowing, and no reservation semantics. Evaluating `context.daily_spend < 5` per turn requires a caller who already knows the spend, so Cedar's contribution reduces to a numeric comparison at PDP cost.

Cedar's real value here is one level up: **who may set a limit, who may raise it, which principals or actions are exempt, and whether degradation is permitted for this deployment.** That is a governance question, expressible in policy, and it matches the standing Cedar rule already applied to auto-approve thresholds — *policy owns the threshold; users may only tighten it.* The arithmetic belongs in the Governor; the authority to set the number belongs in Cedar. §7 specifies the split.

### 4.5 Why the in-memory ledger is kept and labelled

Local-first sovereign deployments have no shared database by design. Offering only a SQL ledger would force a network dependency into KnowMe's desktop path; offering the in-memory ledger *without* `is_authoritative()` would let a multi-replica operator believe in a limit that does not exist. Both impls, one honest flag, surfaced in the UI.

---

## 5. Benefits

**Runaway spend becomes bounded rather than reported.** Per-turn admission against reserved state is the difference between discovering a runaway from a bill and stopping it at iteration 3.

**F-04 gets its substrate.** Metered billing and entitlement enforcement both require a pre-flight ledger with durable state and multi-replica coherence. This design supplies all three; the current tracker supplies none. F-04 is the first revenue-blocking item across all three entities, and the ledger is its foundation, not an adjacent concern.

**Router v1 gains its missing input.** The budget envelope is the named remaining gap in `uar-next-fable.md` §3 (alongside health and feedback). D4 delivers it as a side effect of the degradation ladder.

**Governed deployments become demonstrable.** For a bank or a client POC, "the agent physically cannot spend more than $X, here is the per-scope ledger, here is the reservation that bounds in-flight spend" is a control that survives a security review. "We log a warning after the fact" does not. This is directly sellable in the Insight Assessment → Fusion POC motion.

**Two live production hazards close.** The hardcoded 128K window and the silently-disabled summarization driver are one-line fixes with immediate effect, independent of everything else in this document.

**Declared spec fields become real.** `max_tokens_per_turn`, `max_tokens_per_session`, `max_tool_calls_per_turn` are currently parsed, signed into descriptors, documented, and ignored. Either consumed or removed — both are better than the present state, in which a signed artifact asserts a constraint the runtime does not honor.

---

## 6. Tradeoffs

Stated plainly, including the ones that argue against this design.

### 6.1 Exactness costs latency, and there is no way around it

Exact global enforcement requires a synchronous shared writer on the hot path. That is a structural constraint, not an implementation shortcoming. The available positions:

| Position | Guarantee | Added latency |
|---|---|---|
| In-memory, single process | Exact within process | < 1 ms |
| SQL ledger, reserve per turn | Exact globally | 5–15 ms / turn |
| Local shard + periodic sync | Approximate, bounded overshoot ≈ shard_count × shard_allowance | < 1 ms |
| Coarse run-scope reservation, per-turn settle | Exact at run granularity, loose within run | 1 round-trip / run |

The last row is the recommended production default: reserve a run envelope once, settle per turn against it. It bounds the blast radius to one run's envelope while paying one round-trip instead of ten. **It does not give exact per-turn global enforcement.** Deployments that require that pay the latency.

### 6.2 Over-reservation wastes headroom

Reserving at the output ceiling over-reserves whenever the model answers briefly — plausibly 2–3× for short completions. Settle releases the delta immediately, so the exposure window is one turn, but under high concurrency near a limit this can deny calls that would have fit. Mitigations (learned output-length priors per model/task, tighter `max_tokens`) are deferred; the M0 measurement pass will size the problem before anyone builds them.

### 6.3 Silent degradation is a correctness hazard

D4 is the most valuable step and the most dangerous. A run spanning two models under one `run_id` invalidates within-run comparisons, corrupts eval harnesses that assume model constancy, and can produce a quality cliff the user never consented to. Mitigations — event emission, provenance recording, turn-boundary-only application, per-policy opt-out — reduce but do not eliminate this. **For regulated or client-facing deployments, `degradation: []` with `on_exhaustion: Deny` should be the default,** and the degradation ladder should be opt-in. Predictable failure beats unpredictable quality.

### 6.4 This is a lot of surface area

Four gates, seven scopes, five degradation steps, two ledger backends, two fail modes. The honest risk is over-engineering a subsystem to a fidelity the current stage does not require.

The mitigation is sequencing, not scope reduction: M1 ships **two** gates, **three** scopes, and **one** ledger. The trait boundaries make every later phase additive rather than a rewrite. Named migration triggers rather than building for a scale that does not exist yet:

- Add the SQL ledger when a deployment runs more than one replica.
- Add Tenant scope when a second paying tenant shares an instance.
- Add the degradation ladder when a real run is denied and the denial costs more than the degradation would have.

### 6.5 Fail-open versus fail-closed has no correct answer

Ledger unreachable, mid-run. Fail open: the run completes, spend is unbounded and unrecorded. Fail closed: a paying customer's work fails because of an infrastructure fault unrelated to their budget.

The default position — fail **open** with a loud alarm for Global scope, fail **closed** for Tenant and Principal scopes in metered deployments — reflects a judgement that unmetered internal spend is cheaper than a customer-visible outage, while unmetered *billable* spend is revenue leakage. Reasonable operators will disagree; it is per-policy configurable, and the choice must be explicit in config rather than inherited from a default.

### 6.6 Estimates will drift from invoices

Provider usage reporting lags, cache accounting differs by provider, and reasoning-token billing is inconsistent across families. Reconciliation against provider billing **will** show drift. If estimated spend is ever presented to a customer as billed spend, that drift becomes a commercial and legal exposure. The ledger must carry `estimated` and `reconciled` as separate fields, and the customer-facing surface must read only the latter. This constrains F-04's design and should be settled before billing is built on top of it, not after.

### 6.7 It cannot stop the expensive thing that already happened

Admission acts on a *prediction*. A single call that returns 100K reasoning tokens against a 2K estimate overshoots by construction, and no pre-flight design prevents that. What bounds it is `max_tokens` on the request itself, which is a dialect-engine concern. Budget enforcement and output ceilings are complementary; neither substitutes for the other, and shipping the first without the second leaves the single-call tail unbounded.

---

## 7. Governance split (Cedar)

| Concern | Owner | Mechanism |
|---|---|---|
| What is the limit? | Cedar policy + config | Policy sets ceiling; principals may only *tighten* |
| Who may raise a limit? | Cedar | `permit(action == "budget.raise_limit", ...)` |
| Which actions are budget-exempt? | Cedar | Exemption policy on action + resource |
| Is degradation permitted here? | Cedar | Deployment-level policy attribute |
| Has the limit been reached? | Governor | Ledger arithmetic, no PDP call |
| What happens now? | Governor → host | `Verdict` → host actuation |

This mirrors the auto-approve threshold rule already in force across the estate: policy owns the number, the subject may only make it stricter, and the actuation lives in the trusted host.

---

## 8. Data model

```sql
-- Durable ledger. Extends CH-07's cost-entry table rather than replacing it.
CREATE TABLE budget_scope_state (
    scope           TEXT        NOT NULL,   -- global|tenant|principal|agent|session|run|turn
    scope_id        TEXT        NOT NULL,
    window_key      TEXT        NOT NULL,   -- '2026-07' | 'sliding:<epoch_bucket>' | 'unbounded'
    committed_usd   NUMERIC(18,8) NOT NULL DEFAULT 0,
    pending_usd     NUMERIC(18,8) NOT NULL DEFAULT 0,
    committed_tokens BIGINT     NOT NULL DEFAULT 0,
    limit_usd       NUMERIC(18,8),          -- NULL = unbounded
    limit_tokens    BIGINT,
    version         BIGINT      NOT NULL DEFAULT 0,  -- optimistic concurrency
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (scope, scope_id, window_key)
);

-- Reservation reserve() as a single conditional UPDATE: atomic, no read-then-write race.
-- UPDATE budget_scope_state
--    SET pending_usd = pending_usd + :est, version = version + 1
--  WHERE scope = :s AND scope_id = :id AND window_key = :w
--    AND (limit_usd IS NULL OR committed_usd + pending_usd + :est <= limit_usd)
-- RETURNING version;
-- Zero rows affected ⇒ this scope denies.

CREATE TABLE budget_reservation (
    id              UUID PRIMARY KEY,
    run_id          TEXT        NOT NULL,
    scope_path      JSONB       NOT NULL,
    estimated_usd   NUMERIC(18,8) NOT NULL,
    estimate_basis  TEXT        NOT NULL,
    state           TEXT        NOT NULL,   -- pending|settled|released
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    settled_at      TIMESTAMPTZ
);

-- Reconciliation: estimated is never the customer-facing number (§6.6).
ALTER TABLE cost_entry ADD COLUMN reconciled_usd NUMERIC(18,8);
ALTER TABLE cost_entry ADD COLUMN reconciled_at  TIMESTAMPTZ;
```

Orphaned `pending` rows — process killed between reserve and settle — are swept by a startup pass that releases reservations whose `run_id` is not in a live state. Without this, pending leaks accumulate and the system slowly denies everything. It is the single most likely operational failure of this design and must ship in M2, not later.

---

## 9. Config and spec surface

### 9.1 Replace `LlmBudgetConfig`

Current shape carries `global_limit`, `model_limits`, and `enforcement: String`. Only `global_limit` is read; the other two are parsed, schema'd, exposed in the settings UI, and dead. Replacement:

```yaml
budget:
  global:
    limit_usd: 500.0
    window: { calendar: month }
    warn_at: 0.80
    throttle_at: 0.95
    on_exhaustion: degrade_then_deny
    degradation: [suppress_thinking, tighten_context, downgrade_model]
    on_ledger_unavailable: open
  defaults:                      # applied per scope unless overridden
    session: { limit_usd: 5.0,  window: unbounded, on_exhaustion: deny }
    run:     { limit_usd: 1.0,  window: unbounded, on_exhaustion: deny }
  ledger:
    backend: memory              # memory | postgres | surrealdb
  model_limits:                  # now actually enforced
    "openai/gpt-5.5": 100.0
```

`enforcement: String` becomes the typed `Exhaustion` enum. Any string-typed policy field that gates money is a defect.

### 9.2 Give `budgets` a typed home

`BudgetsSection` currently survives compilation only as JSON under `extensions["budgets"]`, forcing `manager.rs:74` to do key archaeology with `serde_json::Value::get`. Add a typed `budgets: Option<BudgetsSection>` to `AgentPolicy`, and map fields to scopes explicitly:

| Field | Scope | Status |
|---|---|---|
| `max_cost_per_session_usd` | Session | **currently mis-mapped to Agent** |
| `max_cost_per_run_usd` | Run | new |
| `max_cost_per_agent_usd` | Agent | new — the correct home for what Agent scope means |
| `max_tokens_per_turn` | Turn (token) | declared, never consumed |
| `max_tokens_per_session` | Session (token) | declared, never consumed |
| `max_tool_calls_per_turn` | Turn (count) | declared, never consumed |
| `timeout_seconds` | wall-clock | existing resilience infra |
| `rate_limit` | out of scope | route to a separate limiter (§1.3) |

### 9.3 Amend the published spec

`uar-agent-md-spec.html` §15 states that budget declarations are enforced at the Cedar policy layer and that exceeding a budget causes the governing policy to deny the offending action rather than allow overage. No such path exists in the code.

That document is client-facing positioning material. **The amendment should land before the next client-facing use of it, independent of implementation timing** — it is faster to correct a sentence than to build the subsystem it describes, and an unbacked enforcement claim in a document shown to a bank is a materially worse problem than a missing feature. Replace with the §7 split: Cedar governs limit authorship and exemptions; the runtime Governor evaluates and enforces.

Related: any contractual acceptance criterion written against budget enforcement should be checked against this gap before signature.

---

## 10. Implementation plan

Phase discipline: Assess → Plan → Execute → Reflect, with an explicit approval gate at each boundary. No phase begins before its predecessor's exit criteria are met.

### M0 — Measure, and fix what is bleeding (est. 3–5 days)

Rationale: every enforcement decision inherits estimator error. Enforcing on an estimator that is 3× wrong is worse than not enforcing, because it produces confident wrong denials. Measure before building. This is the M1-first pattern applied to the thing that actually carries the risk.

| # | Task | Files |
|---|---|---|
| M0.1 | Resolve real context window; delete the `128_000` literal | `runtime/manager.rs:1034` |
| M0.2 | Pass the driver so `ProgressiveSummarization` functions | `runtime/manager.rs:1033`, `context/manager.rs` |
| M0.3 | Count tool definitions and non-text content in the estimator | `context/token_service.rs` |
| M0.4 | Shadow estimator: log predicted vs actual cost and tokens per call, no enforcement | new `budget/shadow.rs`, hook at `Usage` arm |
| M0.5 | *Optional stopgap* — install liter-llm `BudgetLayer` via `build_client_config` for a crude global ceiling, clearly marked temporary | `config.rs:1334` |

**Exit criteria:** ≥ 1,000 real calls across ≥ 3 model families captured. Error distribution published for cost and tokens. NFR-03/04 confirmed achievable or re-baselined with evidence.

**Do not proceed to M1 if** p90 cost error exceeds 0.5 — fix the estimator first, because the rest of the design is not worth building on top of that.

### M1 — Accounting correctness (est. 1.5–2 weeks)

| # | Task |
|---|---|
| M1.1 | `uar-budget-core`: types, `Scope`, `ScopePath`, `Estimate`, `BudgetPolicy`, `Verdict`, `Ledger` trait |
| M1.2 | `Estimator` — pure, model-aware, content-type tokenizer factors, `EstimateBasis` |
| M1.3 | `uar-budget-ledger-memory` — sharded by scope-key hash, single-writer per shard |
| M1.4 | Wire **G2** incremental settle at the `NormalizedEvent::Usage` arm |
| M1.5 | **G3** becomes reconciliation; correct the Session/Agent scope mis-mapping |
| M1.6 | Windows + `rehydrate()`; CH-07 schema migration |
| M1.7 | Preserve the existing `BudgetAlert` event shape and all three SSE surfaces |

**Exit:** per-turn spend accurate to the estimator's measured error across the full scope path; spend survives restart; no behavioural change visible to callers except better numbers. Still **zero enforcement**.

### M2 — Admission and enforcement (est. 2–3 weeks)

| # | Task |
|---|---|
| M2.1 | Reservation protocol + RAII `Reservation` guard |
| M2.2 | `Governor::admit` — strictest-scope verdict |
| M2.3 | **G0** run admission at `start_run` |
| M2.4 | **G1** turn pre-flight in the orchestrator loop |
| M2.5 | Deny actuation through the existing per-run `CancellationToken` |
| M2.6 | `uar-budget-ledger-sql` with conditional-update reservations |
| M2.7 | Orphaned-reservation sweep at startup (§8) |
| M2.8 | `BudgetDenied` / `BudgetReserved` events; `gen_ai.budget.*` OTel metrics |
| M2.9 | Concurrency tests: 100 parallel turns against one limit, assert no overshoot |
| M2.10 | Remove the M0.5 stopgap |

**Exit:** a limit cannot be exceeded under concurrency. `is_authoritative()` surfaced on health and in the admin UI. Fail-open/closed exercised under induced ledger outage.

### M3 — Degradation and router coupling (est. 2 weeks)

| # | Task |
|---|---|
| M3.1 | Fix `router.rs:75` missing-cost-sorts-as-free — **blocks M3.3** |
| M3.2 | `DegradationPlan` + D1–D3 (suppress thinking, tighten context, cache-stable prefix) |
| M3.3 | D4: `max_cost_per_call_usd` envelope into `RouteRequirements` |
| M3.4 | D5: dynamic `MAX_TOOL_ITERATIONS` reduction |
| M3.5 | Degradation events + run provenance records |
| M3.6 | Default `degradation: []` for client/regulated profiles (§6.3) |

**Exit:** a run near its ceiling completes at reduced cost with the degradation visible in its provenance. Degradation demonstrably opt-in.

### M4 — Governance and billing surface (est. 2 weeks)

| # | Task |
|---|---|
| M4.1 | Typed `budgets` on `AgentPolicy`; compiler wiring (`ir.rs` → `to_artifact.rs`) |
| M4.2 | Consume `max_tokens_per_turn`, `max_tokens_per_session`, `max_tool_calls_per_turn` |
| M4.3 | Cedar: limit authorship, raise authority, exemptions, degradation permission |
| M4.4 | Replace `LlmBudgetConfig`; typed `Exhaustion`; enforce `model_limits` |
| M4.5 | Ledger export for F-04; `estimated` vs `reconciled` separation |
| M4.6 | Amend `uar-agent-md-spec.html` §15 (**can and should ship in M0**) |
| M4.7 | Operator surface: per-scope spend, reservations in flight, denial and degradation history |

**Exit:** every declared budget field is enforced or removed. F-04 has a ledger to build on.

### 10.1 Sequencing notes

- **M0 is not optional and not a formality.** It is the phase that decides whether the rest is worth building as specified.
- M4.6 (spec amendment) is decoupled from implementation and should land immediately.
- M1 → M2 is the natural stop point for a first release. Degradation (M3) is genuinely valuable but is where the complexity concentrates; shipping M2 with `on_exhaustion: deny` is a coherent, defensible product.
- Total: **7.5–10 weeks** single-developer, M0 through M4. M0–M2 alone: **4–5.5 weeks**. Ranges rather than dates; M2's estimate is the least certain because the SQL reservation path and the orphan sweep are where unknown-unknowns live.

---

## 11. Open questions

| # | Question | Blocks | Owner |
|---|---|---|---|
| Q1 | Calendar-month or sliding window as the cost default? Calendar matches invoicing; sliding matches abuse control. | M1.6 | Travis |
| Q2 | Does KnowMe's $200 sovereign tier meter on this ledger, or is BYO-key out of scope for accounting entirely? | F-04 shape | Travis |
| Q3 | Tenant scope in M1, or defer to the second paying multi-tenant deployment? | M1 scope | Travis |
| Q4 | Is degradation ever acceptable for San Saba, or is `Deny` the only permitted exhaustion mode for client work? | M3.6 | Travis / Hal |
| Q5 | Do the SSR Phase 3 advisory-only agents need budget scopes at all in the frozen-app shell, or is a global ceiling sufficient for that surface? | SSR sequencing | Travis |
| Q6 | Reconciliation source — provider billing APIs, or accept estimate-only and never bill on it? | §6.6, F-04 | Travis / Randy |

---

## 12. Verification

| ID | Test | Phase |
|---|---|---|
| V-01 | Estimator error within NFR-03/04 on captured production traffic | M0 |
| V-02 | 100 concurrent turns, one limit: `committed ≤ limit`, always | M2 |
| V-03 | Kill process mid-reservation, restart: pending swept, committed intact | M2 |
| V-04 | Ledger outage: fail-open and fail-closed each behave as configured | M2 |
| V-05 | Strictest-scope: exhausted Session denies under a generous Global | M2 |
| V-06 | Degraded run completes; provenance names every degradation applied | M3 |
| V-07 | 8K-context model: trim fires before the provider rejects | M0 |
| V-08 | Tool-heavy request: estimated tokens ≥ provider-reported prompt tokens | M0 |
| V-09 | Every `BudgetsSection` field either changes behaviour or is absent from the spec | M4 |
| V-10 | Two replicas, in-memory ledger: `is_authoritative() == false` and it is visible in the UI | M2 |

---

## 13. Failure scenarios

**The scenario that hurts Prometheus.** A client POC agent enters a tool loop at 02:00. Ten iterations against a frontier model, each with a large tool-schema prompt. Cost tracking is off by default, so no estimate is produced; the CH-06 tracker therefore records nothing; the alert never fires. The client discovers it on a provider invoice. The technical defect is small. The trust cost is not, and it lands on the "sovereign, governed AI infrastructure" positioning, which is the actual product.

**The scenario that hurts a client.** Budget enforcement ships with degradation on by default. A run silently downgrades mid-task. The output is materially worse in a way no one can attribute, because the provenance was not recorded. The client concludes the system is unreliable rather than budget-constrained. This is why §6.3 argues for degradation off by default in client deployments — the failure is worse for being invisible.

**The scenario that hurts the design.** M0 is skipped. Enforcement ships on an estimator with 3× error. Legitimate calls are denied at 40% utilization; operators raise every limit to compensate; the system is now theatre with latency. Recovering credibility costs more than M0 would have.

---

## 14. Decision required

Approve, amend, or reject:

1. **The three-plane split** (Estimator / Ledger / Governor) with actuation retained in the runtime host.
2. **Reserve-then-settle**, accepting the latency in §6.1 and the recommended run-envelope default.
3. **The degradation ladder**, with D4 router coupling — and the §6.3 position that it ships opt-in.
4. **The Cedar split** in §7, and the §9.3 amendment to the published spec.
5. **M0 as a hard gate**, with the stated stop condition.

Recommended immediate action regardless of the above: land M0.1, M0.2, and M4.6. Two one-line code changes closing live production hazards, and one sentence closing a positioning exposure.

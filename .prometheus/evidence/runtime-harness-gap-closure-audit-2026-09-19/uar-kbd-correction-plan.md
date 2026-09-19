# Correction plan: `runtime-harness-gap-closure` (KBD plan revision 7 → 8)

Author: independent read-only audit (Claude), 2026-09-18 ~19:45 local. Producer of the plan under review: Codex (GPT-6 family). No code, KBD state or repository file was changed by this audit. No test or build was run; every claim about code comes from reading a dirty working tree that Codex is still editing.

## Operator decisions (2026-09-19, after both review rounds)

Gate 0 below is answered. The answers were given by the operator in chat after reading the findings; they have not been seen by the cross-model judge.

- D1 (Q1): yes. M1 may close as its own phase boundary, labelled "implemented, not certified".
- D2 (Q2): yes. The reducer may evict the oldest complete tool-call groups whole. Compression and splitting of tool data stay forbidden. This amends D-AN-01 and restores a compaction path for long tool-heavy runs. Never evicted: the pinned system message, current input, any group with a pending call, the most recent complete group, active skill bodies, durable decisions. `protected_history_overflow` remains the outcome when the non-evictable set alone does not fit.
- D3 (Q3): yes. A provider with no enforceable output cap stays on the legacy path, labelled `unbudgeted`.
- D4 (Q4): "do as you recommend". The recommendation is to provision three things, owned by the operator: a Linux host (blocking Linux rows, and a second build writer worth roughly 25 to 35 hours on M1); disposable Postgres and Surreal fixtures on both operating systems; one remote provider credential. None of the four configured remote providers has a complete-request hard count in the matrix. An Anthropic credential is the suggested one: the repository already has `src/llm/anthropic_driver.rs`, the API requires `max_tokens`, and it exposes `POST /v1/messages/count_tokens` (source: https://github.com/anthropics/anthropic-sdk-python/blob/main/api.md). Whether that count is exact or an estimate is unverified. If no provider offers an exact whole-request count, the `certified` label is unattainable as defined, and M2's sizing task must say so.
- No provisioning date was given. Until one is, production-ready has no date.

Effect on the estimate: D2 adds one small change to `harness-protected-context` (a spec delta, the eviction rule, five regressions), about one ordinary PRODUCT task. M1 becomes 23 tasks; the 50 to 105 hour range is not moved by it.

## Context

The operator supplied an eight-item feature-gap review of Universal Agent Runtime (F1 dynamic routing, F2 prompt dialects, F3 context correctness and model-aware context, F4 unified AG-UI/A2A task model, F5 skill activation, F6 orchestration depth and graph observability, F7 integrated knowledge runtime, F8 verification debt). The operator's goal is a full production-ready implementation with those features. A KBD loop in Codex has run for about two days. The operator reports it is not converging.

### Measured position

- Phase tasks: 12 of 44 done. Changes: 1 of 6 done (`harness-protected-context` 8/8, `harness-model-profiles-routing` 4/8). Source: `phases/runtime-harness-gap-closure/progress.json`, `tasks.md`.
- The waypoint shows "116 of 129". That is a project-wide roll-up across all phases, not this phase.
- Execute wall-clock: first task evidence 09-16 18:44, twelfth 09-18 17:48, about 47 hours. Mean 3.9 h per task. On 09-18 alone: 9 tasks in about 15 h, 1.7 h per task. One gap of 13 h sits between tasks on 09-17 (09:06 to 22:03); cause not determined.
- The loop is not looping. A second check at 09-19 06:57 found the KBD revision unchanged at 2577 since 09-18 17:50, no file under `src`, `tests`, `openspec`, `.kbd-orchestrator` or `.prometheus` modified since 09-18 19:02, and the Codex process at 0% CPU. Task routing 2.3 is half-edited in `manager.rs` and `orchestrator.rs` while the waypoint reads `status: ready, currentTask: null`. A similar 12-hour silence sits between 09-16 20:52 and 09-17 08:54. Of roughly 60 elapsed hours, at least 24 produced nothing. The per-task figures above therefore overstate the cost of a task and understate how much of the two days was idle.
- Straight-line projection for the 32 remaining tasks: 54 to 125 wall-clock hours. This is a floor. The remaining tasks include crash recovery on two databases, cross-protocol identity, an evaluation runner, and raising frontend coverage from a last-recorded 33.68% to 60% on four metrics.
- After the 44 tasks, the plan requires Tier 2, then a child phase `runtime-harness-profile-certification` with its own plan and Tier 3 run. The child currently has 0 changes.

### Classification of the 32 remaining tasks (from `execution-map.json`)

PRODUCT 13, TEST 6, CEREMONY 5, RESEARCH 4, MEASURE 4. Nineteen of 32 remaining tasks change no runtime behavior.

## Findings

### Plan flaws

P1. The exit gate cannot pass on this machine. `profile-certification.md`: "Linux and macOS are stable and blocking." The gate for task `harness-governed-evaluation-3-1` asserts required rows `linux/surreal/crash`, `macos/surreal/crash`, `linux/postgres/crash`, `macos/postgres/crash`, each with `status==PASS` and `testCount>0`, plus live-provider rows that need "a proven count bound". The plan says of itself: "this machine may not supply every stable-platform or device environment. That prevents phase completion." The phase has no reachable end state. The loop is not slow toward the goal; under its own contract it cannot arrive.

P2. Zero real providers can be certified, and one of the two enabled providers never can. `provider-profile-matrix.json`: `certifiedDispatchProfiles: 0`; 6 providers, 13 models, all `BLOCKED`. Ferrox lacks a complete-envelope count and an enforceable output ceiling. The local ChatGPT-subscription proxy counts `post_dispatch_only` and its backend strips `max_output_tokens`; no remaining task can change that.

P3. Two days of machinery is dormant in production. Every production construction passes no contract: `src/server.rs:3875`, `src/llm/orchestrator.rs:544`, `src/llm/liter_driver.rs:746`, `src/uar/runtime/manager.rs:3778` (`summary_budget_contract: None`). The only constructor, `RequestBudgetContract::synthetic_exact("http://summary.invalid/v1")`, is called from `#[cfg(test)]` modules. Task 2.2's own summary: the matrix "certifies only the deterministic synthetic OpenAI-compatible path."

P4. The delivered production behavior change is a new hard stop. `manager.rs:3773-3781` disables production summarization ("stays disabled until the destination model profile supplies an exact final-wire request contract") and sets `eligible_prose: Vec::new()`. `reduce.rs:39-100` marks every tool-call group and every multimodal message in the whole history as protected, and any reducer output that omits one returns `ProtectedHistoryOverflow`. The original defect (a tool call separated from its result) is fixed. It is replaced by this: a long tool-heavy run has no compaction path and ends with `protected_history_overflow` once its tool groups exceed the window. D-AN-01 removed the earlier whole-group-dropping rule on the basis of the operator's request for "a budget function that does not compress tool calls or other data". Whether the operator also meant "never evict an old, complete group" was not asked.

P5. Goal displacement. The input was a list of missing capabilities. The plan converted it into a proof program: exact byte identity, hashed corpora, certification receipts, claim/evidence matrices. F1 routing reaches production only in task 2.4, which has not started.

P6. Serial ordering without a file-level reason. Among the 32 remaining tasks no source file appears in two different changes. `manager.rs` and `orchestrator.rs` are touched only by routing 2.3 and 2.4. D-SP-01 said the total order holds "until Plan proves safe independence"; Plan never ran that check. Changes 3, 4 and 5 depend functionally on change 1 (done), not on each other.

P7. Four research tasks repeat finished work. `plan-research.md`: "This supplement resolves the five external-candidate survey prerequisites." Tasks 1.1 of changes 3 to 6 still read "Complete the mandatory external ... survey". Their mapped output is a hash-verification receipt.

P8. Review can only add scope. Both plan-stage cross-model reviews returned BLOCK for missing ownership; each correction added work (Surreal crash suite, full Tier 2 binding). No review stage asks whether the plan is achievable or what it costs. Execution started with the final verdict still BLOCK and the last deltas unreviewed. The sycophancy screen scored the artifacts 0.0 to 0.02: it measures tone and cannot detect an oversized plan.

P9. A hidden sub-project. Frontend coverage to 60% on lines, statements, functions and branches sits inside one task (`harness-governed-evaluation-2-3`). Current coverage is recorded as UNKNOWN.

### Execution flaws

E0. The run is turn-driven, not autonomous. Codex stops at the end of a turn and waits; nothing restarts it. Two overnight gaps of about 12 hours each are visible in file modification times, and the second one left a task half-edited with the ledger showing no task in progress. This is the largest single loss of calendar time and it is independent of the plan's content.

E1. Nothing has been committed since 2026-09-05 (HEAD `226d4a0a`). Uncommitted: 48 source/test files, +7,522/−1,078, inside a 274-path dirty tree that also carries 103 unrelated deletions under `.agents/skills/impeccable`. Evidence bundles hash an uncommitted tree. One bad checkout or stash loses the phase. The plan lists "commit" among things each task did not do, as if that were a virtue.

E2. Review per task, not per change. Session log: task 2.1 of routing took three critic blocks, 2.2 two rounds, protected-context 2.3 "two failing critic rounds". Each task also emits a hashed evidence bundle; routing 1.2 wrote 6.3 MB.

E3. The learning loop is dark for this run. The last Karpathy capture for this repo is 2026-09-02; capture events carry `harness: claude-code` and Codex emits none. The queue worker last finished 2026-09-05 with `lastError: ... 500 Internal Server Error`; 69 memory items sit in `submitting`. There are no Karpathy logs for the two days in question. `session-log.md` is the only narrative record and its entries are out of order.

E4. Stale state in every turn. `progress.json` evidence/certification/publication summaries describe the zero-length-Ping child of 09-06, not this phase. The re-anchor text lists 11 open boundaries from other phases (for example "task 18 out of 26"). The prior readiness reflection is still pending.

E5. Machine pressure. Swap was 22.7 GB of 23.5 GB used during the audit, with two Claude ACP sessions, a Codex app-server and the ChatGPT app resident. The project's own notes record that swap exhaustion kills rustc and looks like a lock hang.

## Decision

Stop executing plan revision 7 at the routing 2.3 task boundary. Replace it with revision 8, which keeps the full target and splits it into two milestones that are both mandatory. Revision 7 cannot finish on this machine (P1); revision 8 can finish its first milestone here and names what must be provisioned for the second.

- **M1, implemented.** All eight capabilities run on the production path on macOS / `server-full` / the enabled providers. Tier 2 passes on a committed tree. Every unproven row is listed as an open limitation. M1 is reported as "implemented, not certified". It is never reported as production-ready.
- **M2, certified.** The existing `runtime-harness-profile-certification` scope, unchanged in strictness, plus the work moved out of M1 (durable crash suites, coverage to 60%, historical-failure reproduction, routing 3.1 live comparisons). M2 is required for the production-ready label. It is not optional and not a nice-to-have follow-on.
- M2's blocker is provisioning, and provisioning has one owner: the operator. Needed: a Linux host (a container only if the operator accepts it as the Linux row), disposable Postgres and Surreal fixtures on both operating systems, and credentials plus a reachable endpoint for at least one remote provider that exposes a real count and an enforceable output cap. Codex cannot supply these. The first M2 task is to size M2; nobody has.

### Gate 0: operator answers, recorded in the KBD decision ledger

Q1. May M1 close as its own phase boundary before M2, with the "implemented, not certified" label? If no, M1 and M2 stay one phase and the phase stays open until provisioning lands; the task order below still applies.
Q2. Eviction. May the reducer remove the oldest complete tool-call groups whole (never compress, never split, canonical receipt kept in storage) when the protected set exceeds the window? Yes restores a compaction path. No keeps `protected_history_overflow` as a documented product limit.
Q3. Provider posture. May a provider with no enforceable output cap keep dispatching on the legacy unbudgeted path, visibly labelled, while budgeted dispatch is reserved for providers that can prove a bound?
Q4. When can a Linux host and one remote provider credential be available? This sets the M2 start date and, as action 5 explains, is also the main speed lever for M1.

Timeout and defaults. The gate does not idle the loop. Actions 1, 2 and the routing 2.3 / 2.4 tasks are identical under every answer, so Codex proceeds with them at once. If Q1 to Q3 are unanswered when routing 2.4 completes, Codex stops, writes a one-page status, and waits; it does not guess. Defaults are deliberately the conservative ones: Q1 no, Q2 no, Q3 yes.

### Actions

1. Checkpoint, verified before it is committed. Stage only the phase's paths (the 48 source and test files plus phase evidence), leaving the unrelated `.agents/skills/impeccable` deletions out. On that staged set run Tier 0 and the four Tier 1 targets the phase already uses (`context_history_integrity`, `test_context_strategy`, `prompt_assembly`, `model_path_resiliency`). Commit in reviewable chunks, one per completed change or task group. No push. Then run Tier 2 once on the committed tree. This is the first whole-tree check of the twelve completed tasks, and a failure here is repaired before any new task starts.

2. Make the contract live for what can honestly carry it, and label the rest.
   - Three labels: `certified` (exact count, enforceable cap), `bounded` (`CountQuality::ValidatedUpperBound`, already accepted at `budget.rs:151`, plus an enforceable cap), `unbudgeted` (legacy path, no fit guarantee).
   - Ferrox: its output blocker is that durable settings omit `max_output_tokens`. Set it, prove by fixture that the server enforces it. For input, prefer a formal bound: Ferrox exposes `/v1/tokenize` for a rendered string, so count the fully rendered prompt including tool schemas, and add a fixed framing allowance derived from the template. If a formal bound cannot be built, fall back to the empirical protocol below. If cap enforcement cannot be shown, Ferrox is `unbudgeted`.
   - Local ChatGPT proxy: the backend strips the output cap, so it is `unbudgeted` until that changes.
   - Empirical protocol, used only when no formal bound exists: the frozen 945-case context corpus, the 18 routing cases and an adversarial set (maximum-size tool schemas, non-ASCII, deeply nested JSON arguments, parallel tool calls); at least 200 dispatched requests per model; M is the maximum observed under-count plus 10%; one exceedance demotes the profile. A sample does not prove a bound. `bounded` therefore means "empirically validated", and the runtime must still treat a provider-side context-length rejection as an explicit outcome.
   - Acceptance: a production-path run in which `budget_contract` is `Some` for a real enabled provider, or a recorded finding that no enabled provider can carry it. That finding moves the contract's first real use into M2.

3. Remove work that produces no behavior and no proof.
   - Close the four survey tasks against `plan-research.md`, which already records the five decisions. Record the weakness: each survey compared one candidate and registry verification failed.
   - One evidence record per change (commands, exit codes, test counts, commit hash) replaces per-task hashed bundles. Commit hashes replace tree digests after action 1.

4. Acceptance map for F1 to F8, kept in the phase as a lean table. Every retained task keeps its scenario bindings from `execution-map.json`.

| Area | Product tasks | M1 proof | Deferred to M2 |
|---|---|---|---|
| F1 routing | routing 2.4 | admission cases R01–R12 and task cases T01–T08 pass on the production run-start path | live quality / cost / latency deltas (routing 3.1) |
| F2 dialects | routing 2.1, 2.2 (done), 2.3 | final outbound fixture per enabled provider; failover reapplies the destination template | remote provider fixtures |
| F3 context | protected-context (done), routing 2.3, action 2, Q2 | `context_history_integrity` green; a long tool-heavy run either compacts (Q2 yes) or stops with the documented outcome | certified profiles |
| F4 task model | lifecycle 2.1, 2.2 | cross-protocol lookup, cross-tenant rejection, one logical run under injected crash on the Memory backend | Surreal and Postgres crash suites, both operating systems |
| F5 skills | skills 1.2, 2.1, 2.2, 2.3, 3.1 | precedence and reattachment scenarios; Recall@10 at or above 99% on the frozen set | none |
| F6 orchestration | lifecycle 2.3, 2.4, 2.5, 3.1; evaluation 2.1 | ordered traces for success, approval, deny, cancellation, exhaustion; one terminal event | other feature profiles |
| F7 knowledge | knowledge 1.2, 2.1, 2.2, 2.3, 3.1 | revoked and cross-tenant evidence absent from outbound and audit; four verdict classes on the frozen oracle | BDD and real-RAG end-to-end journeys |
| F8 verification | evaluation 2.2, and measurement only from 2.3 | seeded faults exit nonzero; coverage measured and reported | coverage to 60%; historical-failure reproduction; F1–F8 certification receipt |

5. Order and estimate.
   - Order: routing 2.3, 2.4; lifecycle 2.1, 2.3, 2.4, 2.5, 3.1; skills 1.2, 2.1 to 2.3, 3.1; knowledge 1.2, 2.1 to 2.3, 3.1; evaluation 2.1, 2.2, 2.3 (measure only); then routing 2.5 and lifecycle 2.2.
   - M1 is 22 tasks: 13 PRODUCT and 9 TEST or MEASURE. Ten tasks leave M1: four surveys closed, six moved to M2 (lifecycle 1.2, routing 3.1, evaluation 1.2, 1.3, 3.1, 3.2).
   - Estimate, from twelve observed tasks: ten ordinary PRODUCT tasks at 2 to 5 h, three heavy lifecycle tasks (2.1, 2.2, 2.3, each 8 to 10 files) at 6 to 10 h, nine TEST or MEASURE tasks at 1 to 2 h, Tier 2 plus repair 4 to 8 h. Total 50 to 105 wall-clock hours of continuous running, roughly 2 to 4.5 days.
   - That is not much faster than revision 7's 54 to 125 hours. What revision 8 buys is an exit that exists, a live contract, and a committed tree. It does not buy a large speed-up on one machine.
   - The real speed lever is a second machine. Changes 3, 4 and 5 share no source file. On this machine they must stay serial: swap was exhausted during the audit and concurrent cargo builds are a recorded failure mode. A Linux host provisioned for M2 can also run lifecycle in parallel with skills and knowledge, which removes roughly 25 to 35 hours from M1. Q4 therefore matters twice.
   - M2 is unsized. Its first task sizes it.

6. Review cadence. Keep the isolated critic on every task that changes production behavior, because each recorded BLOCK so far named a real bug. Remove it from research, inventory and evidence tasks. Cap at two rounds; a third BLOCK records a blocker instead of another rewrite. Each defect class already found (marker injection, shadow serialization, unreserved output ceiling, pending-call loss) gets a regression test so the critic is not the only guard. Add one question to every plan review: can each exit criterion pass on the machine that will run it, and at what cost?

7. Re-estimate at each change boundary. Record hours per task by class. Triggers, not verdicts: if mean PRODUCT cost at a boundary is above the top of its range, or one heavy task passes its range, Codex stops and reports before continuing. The ranges come from six PRODUCT data points and are weak.

9. Make it a loop. Run the phase under a goal primitive that continues until a stop condition (`/kbd-goal`, or Codex's native goal mode), with exactly three stop conditions: Gate 0 unanswered after routing 2.4, a recorded blocker, or M1 complete. A task boundary is not a stop condition. Before ending any turn Codex must leave the ledger truthful: a half-edited task is recorded as in progress with the files named.

8. Restore the record. At each change boundary write a progress record with the `karpathy-progress-memory` skill and append to `session-log.md` in time order. Regenerate the stale `progress.json` summaries through the KBD runtime, never by hand. The existing Karpathy corpus is mostly transcripts: 486 of 487 files carry "No explicit root-cause section was captured", and three lessons in `gotchas.md` were each recorded three times, which shows they were stored and not applied.

## Assumptions

- A1. The production-ready label requires certification. Revision 8 keeps it mandatory as M2. What is open is only whether M1 may close as its own boundary (Q1).
- A2. `execution-map.json` file sets reflect the real edit scope. They are declared maxima, not verified. Two design documents name `manager.rs` for changes the map does not give it to.
- A3. Ferrox honours `max_output_tokens` when set. Not verified. If it does not, no enabled provider can carry a budget contract and the change 1 and 2 machinery stays dormant until M2 provisioning.
- A4. Task costs follow the ranges in action 5. Weak basis.
- A5. The completed work is sound. It passed local Tier 0/1 and an isolated critic, but only on `server-full`, only with synthetic destinations, and this audit ran nothing. Action 1 tests this first.
- A6. The 13-hour gap on 09-17 was operator absence or an approval wait, not a stall. Not determined.

## Falsifier

Stopping revision 7 is the wrong call if any of these holds:

- The certification rows are reachable now: the operator confirms a Linux host, both database fixtures and a remote provider credential already exist and revision 7 simply had not used them. Then revision 7's exit was reachable and only its order needed changing.
- Routing 2.3, as Codex finishes it, already passes a `Some` contract for a real enabled provider on the production path. Then P3 described a transient state, not a design gap.
- The operator answers Q2 no and Q3 no and rejects an M1 boundary. Then revision 8 changes little beyond the checkpoint, the survey closures and the review cadence, and its cost saving is about ten tasks.

Revision 8 itself is failing if:

- Tier 2 on the committed tree fails in areas the twelve completed tasks touched. Repair precedes new work and A5 is false.
- A `bounded` profile shows an exceedance. It is demoted; if it was the only one, the contract does not go live in M1.
- A file collision appears between changes 3, 4 and 5. P6 is wrong and parallel execution is off the table.
- Q4 has no date. Then M2 has no start, and "production-ready" has no date either. That must be said to the operator in those words.

## The uncomfortable thing

There is no fast path to the goal as stated. On one machine, revision 8 finishes M1 in about the same number of hours revision 7 would have spent on its 32 tasks; the difference is that M1 ends. Certification, which is what "production-ready" means in this project, waits on hardware and credentials that no agent loop can produce, and M2 has never been sized. Roughly two days went into machinery that is switched off in production today, and it may stay off through M1 if Ferrox cannot enforce an output cap. The strict process also earned part of its cost: every critic BLOCK in the session log named a genuine bug. All time figures rest on twelve data points and an audit that ran no code.

## Unresolved review findings

This decision was reviewed twice by an isolated cross-model judge (`kbd-judge` via the local REST gateway, producer `claude-fable-5-1`, `cross_model_check: verified-distinct`). Both rounds returned **BLOCK**. The two-round cap is reached; this version has not been re-reviewed and is not a PASS.

Round 1 (3 critical, 4 warning): goal narrowed without operator consent; the load-bearing assumption about definition of done was deferred rather than asked; the bounded-estimate proposal contradicted the finding that a provider lacks an output cap; validation protocol undefined; critic frequency cut despite its record; no F1–F8 acceptance map; cost falsifier too narrow. Response: Gate 0 added, provider labels split, protocol defined, critic kept on production tasks, acceptance map added, boundary re-estimation added.

Round 2 (2 critical, 5 warning): falsifiers did not test the decision to stop; Option A allowed an outcome short of the stated goal; gate had no timeout or default; commit preceded verification; thresholds unjustified; a 200-request sample does not prove a bound; no revised estimate. Response in this version: falsifiers rewritten to test stopping; certification made a mandatory second milestone with a named owner; timeout and conservative defaults added; verification moved ahead of the commit; thresholds turned into stop-and-report triggers; formal bound preferred and the sample's limits stated; revised estimate added, including the admission that it is not much faster.

The anti-theater screen script reported `sycophancy.sh lib not found — gate skipped`, so the judge's reports were not screened by that script. The final text was screened separately with the sycophancy-correction MCP tool; its result is recorded below.

Sycophancy screen (MCP `detect_sycophancy`, strictness `strict`, 2026-09-19): excerpts of this plan (decision, estimate, falsifiers, uncomfortable section; about 4 KB, not the whole document) scored 0.0 with no patterns; the Codex prompt, condensed, scored 0.0 with no patterns. This is weak evidence. The same tool scored revision 7's artifacts 0.0 to 0.02, and revision 7 had an unreachable exit. The tool measures agreeable tone. It does not measure whether a plan is right.

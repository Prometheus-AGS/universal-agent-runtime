# Codex correction prompt — `runtime-harness-gap-closure`, plan revision 7 → 8

Operator answers are filled in (2026-09-19). Paste everything below the line into the Codex session.

---

You are continuing KBD phase `runtime-harness-gap-closure` in `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`. An independent read-only audit found that plan revision 7 cannot complete on this machine and that the run has been idle for long stretches. This message is the operator's correction. It overrides plan revision 7 where they conflict. It does not override `AGENTS.md`, capability inversion, the GitHub Actions policy, tier discipline, or the rule that generated KBD projections are never hand-edited.

The full audit is in `.prometheus/evidence/runtime-harness-gap-closure-audit-2026-09-19/uar-kbd-correction-plan.md`, with both judge reports beside it. Read it before acting. Verify each of its factual claims against the tree before relying on it; it was written from a working tree you were still editing, and it ran no build or test. The audit's decision was reviewed twice by an isolated cross-model judge and returned BLOCK both times; its responses to those findings have not been re-reviewed. Treat it as a reasoned correction, not a certified one.

## Operator decisions (Gate 0, answered 2026-09-19)

- **D1. M1 may close as its own phase boundary before certification.** M1 is labelled "implemented, not certified". It is never reported as production-ready.
- **D2. The reducer may evict the oldest complete tool-call groups whole.** Never compressed, never split, canonical receipt retained in storage.
- **D3. A provider with no enforceable output cap may keep dispatching on the legacy unbudgeted path, visibly labelled.**
- **D4. Provisioning follows the audit's recommendation** (section "Provisioning" below). No date is set yet. Until one is, state in every status report, in these words: "production-ready has no date."

Record D1–D4 in the typed KBD decision ledger. D1 supersedes the single-phase gating in D-AN-05: certification stays mandatory for the production-ready label, but it no longer gates M1's phase boundary. D2 amends D-AN-01: whole-group eviction is permitted; compression and splitting of tool data remain forbidden. Append; do not rewrite history.

## What is wrong, in five lines

1. The exit gate needs Linux rows, Postgres and Surreal crash suites on two operating systems, and live-provider rows. None can pass here. `profile-certification.md` says so itself.
2. `provider-profile-matrix.json` certifies zero dispatch profiles. The local ChatGPT proxy strips `max_output_tokens` and can never qualify.
3. The budget contract is `None` at every production construction site (`src/server.rs`, `src/llm/orchestrator.rs`, `src/llm/liter_driver.rs`, `src/uar/runtime/manager.rs` near `summary_budget_contract`). Only `#[cfg(test)]` code builds one.
4. Production summarization is disabled and every tool group is permanently protected, so a long tool-heavy run ends in `protected_history_overflow` with no compaction path.
5. Nothing has been committed since `226d4a0a` (2026-09-05). Forty-eight source and test files, +7,522/−1,078, are uncommitted. Task routing 2.3 is half-edited while the ledger shows no task in progress.

## Step 0 — make the ledger truthful (before any edit)

State in one sentence what you are about to do and name the phase. Read `.kbd-orchestrator/current-waypoint.json`. Report the real state of `harness-model-profiles-routing-2-3`: which files are part-edited, and whether Tier 0 currently passes. Transition it to in-progress through the KBD runtime if it is not. Do not hand-edit `progress.json`, `tasks.md` or the waypoint.

## Step 1 — verified checkpoint

1. Finish or cleanly park routing 2.3 so the tree compiles.
2. Stage only this phase's paths. Leave the `.agents/skills/impeccable` deletions and any other unrelated dirt unstaged.
3. On the staged set run Tier 0 (`RUSTC_WRAPPER= cargo check --locked --no-default-features --features server-full`) and the four Tier 1 targets the phase already uses: `context_history_integrity`, `test_context_strategy`, `prompt_assembly`, `model_path_resiliency`. Paste commands and observed output.
4. Commit in reviewable chunks, one per completed change or task group, conventional-commit format. Do not push.
5. Run Tier 2 once on the committed tree. This is a phase checkpoint, which is where Tier 2 belongs. If it fails in an area the twelve completed tasks touched, repair that before any new task and say so plainly.

One writer, one target directory. Before any cargo command check for orphaned `cargo`/`rustc` processes and for swap pressure; the audit saw 22.7 of 23.5 GB swap in use.

## Step 2 — register plan revision 8 through KBD

Write revision 8 as a reviewed Plan update, not an ad-hoc edit.

- Two mandatory milestones. **M1 implemented**: all eight capabilities on the production path on macOS / `server-full` / enabled providers, Tier 2 green on a committed tree, every unproven row listed as an open limitation. M1 closes this phase (D1). **M2 certified**: the existing `runtime-harness-profile-certification` scope at unchanged strictness, plus everything moved out of M1, registered as its own phase. M2's first task sizes M2; nobody has.
- Close the four survey tasks (`harness-task-graph-lifecycle-1-1`, `harness-skill-activation-quality-1-1`, `harness-knowledge-evidence-1-1`, `harness-governed-evaluation-1-1`) against `plan-research.md`, which already records the five decisions. Record the weakness: one candidate compared per survey, registry verification failed.
- Move to M2: lifecycle 1.2, routing 3.1, evaluation 1.2, 1.3, 3.1, 3.2; the Surreal and Postgres crash suites; coverage to 60%; historical-failure reproduction. Evaluation 2.3 stays in M1 as measurement only.
- M1 order: routing 2.3, 2.4; lifecycle 2.1, 2.3, 2.4, 2.5, 3.1; skills 1.2, 2.1, 2.2, 2.3, 3.1; knowledge 1.2, 2.1, 2.2, 2.3, 3.1; evaluation 2.1, 2.2, 2.3 (measure only); then routing 2.5 and lifecycle 2.2. Twenty-two tasks.
- Keep every retained task's scenario bindings from `execution-map.json`. Copy the F1–F8 acceptance table from the audit into the phase directory and update it at each change boundary.
- `openspec validate <change> --strict` must still pass for every change you touch. Every change keeps at least one spec delta. D2 changes a spec requirement in `harness-protected-context`; write that delta rather than editing code against a stale spec.

Plan review for revision 8 must answer one added question: can every M1 exit criterion pass on this machine, and at what cost? Do not start M1 execution with a BLOCK you have neither fixed nor listed under "Unresolved review findings".

## Step 3 — make the budget contract live, and restore a compaction path

Provider labels (inside routing 2.3):

- Three explicit labels: `certified` (exact count, enforceable cap), `bounded` (`CountQuality::ValidatedUpperBound`, which `plan_budget` already accepts, plus an enforceable cap), `unbudgeted` (legacy path, no fit guarantee, visibly labelled in manifests and telemetry).
- Ferrox: set `max_output_tokens` in its durable settings and prove by fixture that the server enforces it. For input, build a formal bound from `/v1/tokenize` on the fully rendered prompt including tool schemas, plus a fixed framing allowance derived from the template. Only if that is impossible, use the empirical protocol in the audit (frozen corpus, routing cases, adversarial set, at least 200 requests, margin = max under-count + 10%, one exceedance demotes). A sample does not prove a bound; say "empirically validated", and keep provider-side context-length rejection as an explicit outcome. If cap enforcement cannot be shown, Ferrox is `unbudgeted`.
- Local ChatGPT proxy: `unbudgeted` (D3). Do not invent a bound for a backend that strips the cap.
- Acceptance: a production-path test in which `budget_contract` is `Some` for a real enabled provider. If no enabled provider can carry it, record that as a finding, leave the machinery dormant, and move its first real use to M2. Do not substitute the synthetic profile.

Whole-group eviction (D2, a small change to `harness-protected-context`, after the spec delta):

- When protected content exceeds the input allowance, evict the oldest complete tool-call groups whole, oldest first, until the request fits. A group is the assistant message carrying the calls plus every matching tool result.
- Never evict: the pinned system message, the current user input, any group with a pending or unresolved call, the most recent complete group, active skill bodies, durable decisions.
- Never split a group, never truncate or summarize a member, never reorder. The canonical receipt stays in storage and the eviction is recorded (group identities, reason, token counts) in the reduce report and telemetry.
- `protected_history_overflow` remains the outcome when the non-evictable set alone does not fit.
- Regressions: eviction keeps every remaining group valid and ordered across all strategies; parallel tool calls evict together; a pending group is never evicted; resume after eviction reproduces the same outbound history; the evicted receipt is still retrievable from storage.

Verify identifiers before using them. If `CountQuality::ValidatedUpperBound`, a settings key, or an endpoint is not as described here, stop and report; do not improvise an equivalent.

## Step 4 — execution rules for M1

- Isolated critic on every task that changes production behavior. No critic on research, inventory or evidence tasks. Two rounds maximum; a third BLOCK becomes a recorded blocker, not another rewrite.
- Every defect class the critic has already found gets a regression test: marker injection in plain layouts, shadow serialization, unreserved output ceiling, pending-call loss.
- One evidence record per change: commands, exit codes, test counts, commit hash. No per-task hashed bundles. Commit at every change boundary.
- Stay serial on this machine even though changes 3, 4 and 5 share no source files. When the Linux host exists (D4), propose a parallel split in a Plan update before using it; worktrees go under `~/.claude/worktrees/` and need `git submodule update --init --recursive`.
- At each change boundary: record hours per task by class; write a progress record with the `karpathy-progress-memory` skill; append to `.prometheus/session-log.md` in time order; regenerate the stale `progress.json` summaries (they still describe the zero-length-Ping child) through the KBD runtime. If the skill or the memory server is unavailable, say so and log to the markdown files.
- Stop and report, without guessing, when: mean PRODUCT task cost at a boundary exceeds 5 h; a lifecycle task exceeds 10 h; a file collision appears between changes; an edit needs a path outside `execution-map.json`; a provider contract cannot be proven.

## Step 5 — be a loop

A task boundary is not a reason to end your turn. Continue until one of exactly two conditions: a recorded blocker, or M1 complete. Before ending any turn, leave the ledger truthful: a half-edited task is recorded in progress with its files named.

## Provisioning (D4) — owned by the operator, tracked by you

You cannot supply these. Track them as open M2 prerequisites in the revision 8 plan and report their state at every change boundary.

1. A Linux host for the blocking Linux rows and as a second build writer. It needs the pinned Rust toolchain, Node/pnpm, Docker, and enough memory for a `server-full` build without swapping. A container on this Mac counts as the Linux row only if the operator says so in the ledger.
2. Disposable Postgres and Surreal fixtures on both operating systems. `psql`, `pg_ctl`, `docker` and `surreal` are on PATH on this Mac; whether the crash fixtures actually run is unverified.
3. One remote provider credential. None of the four configured remote providers (kimi-for-coding, minimax, alibaba, zai) has a complete-request hard count in the matrix. The audit's suggestion is an Anthropic credential: the repository already has `src/llm/anthropic_driver.rs`, the API requires `max_tokens`, and it exposes `POST /v1/messages/count_tokens` (source: https://github.com/anthropics/anthropic-sdk-python/blob/main/api.md). Whether that count is exact or an estimate is **unverified**; check the current official documentation before assigning `certified` rather than `bounded`. If no provider anywhere offers an exact whole-request count, the `certified` label is unattainable as defined and M2's sizing task must say so.

## Reporting

Lead every report with the delta between plan and delivery. For each claim give the command and the observed output, or say which claim is unverified and why. Do not describe M1 as production-ready.

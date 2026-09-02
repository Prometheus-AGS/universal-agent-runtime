# Handoff to Codex — codex-harness-comparative-analysis

Written 2026-09-02 by the Claude Code session that produced the assess, analyze, spec, and plan stages of this phase and began executing change 1 of 10. The operator is transferring execution to Codex.

Read this file, then `plan.md` in this directory. Everything else in this directory is supporting evidence.

## Scope of the handoff

All ten changes of the plan. Change 1 is partly implemented and unverified; changes 2 through 10 are unstarted.

## Ground truth: where the code actually is

Branch `feat/context-history-integrity`, one commit ahead of `main`.

**Committed** in `b3686ff7` "feat(context): normalize tool-call pairs and bound tool output" — change 1 tasks 1.1 through 2.3:
- `src/uar/runtime/context/normalize.rs` (new) — tool-call pair invariants
- `src/uar/runtime/context/truncate.rs` (new) — middle-out truncation with warning header
- `tests/context_history_integrity.rs` (new) — seven integration tests
- `src/uar/tools/terminal_exec.rs`, `src/llm/orchestrator.rs` — truncation applied at ingest

**Uncommitted working tree** — change 1 tasks 3.1 through 4.1:
```
 M src/uar/api/routes.rs                    (+51)   checkpoint resume rewrite
 M src/uar/context/mod.rs                   (+5)    exports
 M src/uar/context/strategy.rs              (+106)  trim_history, split_pinned_system
 M src/uar/runtime/checkpoint.rs            (+52)   try_restore_state, history_from_checkpoint
 M src/uar/runtime/context/manager.rs       (+64)   dedup removed, index-bounded head/tail
 M src/uar/runtime/context/mod.rs           (+1)    module decl
 M src/uar/runtime/context/token_service.rs (+126)  model-keyed encodings
 M src/uar/runtime/manager.rs               (+41)   single reduce_history call site
?? src/uar/runtime/context/reduce.rs        (new)   the unified reduction path
```

Decide whether to commit that work as-is, amend it into a second commit, or rework it. It compiles; it is not behaviorally verified.

## Verification status — read this before trusting anything

**The only verification that has ever passed is Tier 0.** `cargo check --locked --no-default-features --features server-full` finished clean with zero warnings after the last edit.

**Tier 1 has never run to completion.** One attempt was killed at exit 137 (OOM). That is not a test result. The seven integration tests in `tests/context_history_integrity.rs` have never executed.

**Unit tests inside the new modules did run and pass**: three in `normalize.rs`, four in `truncate.rs`. One (`two_calls_in_a_row_each_get_their_results_in_order`) failed first and was fixed; the fix is in the committed code.

**Known risk in the tests**: the seven integration tests were written before the implementation, against an API surface the same session then designed. If a signature drifted during implementation, they will fail to compile. Treat a compile failure there as an artifact of the authoring order, not as a defect in the feature. The scenarios they encode are correct; the call sites may not be.

## Change 1 task state

16 of 19 tasks are marked complete in the KBD runtime. Remaining:

- 5.1 Tier 1: `cargo test --locked --no-default-features --features server-full --test context_history_integrity`
- 5.2 Tier 2: `cargo fmt --all -- --check` and the full test suite
- 5.3 `openspec validate context-history-integrity --strict`

### Task 3.1 was a recorded deviation, not a completion

`tasks.md` for change 1 has task 3.1 rewritten in place to record this. Summary:

The task as written said "make `uar::domain::context::ContextStrategy` the only enum." That is backwards and would break persisted state. `uar::context::ContextStrategy` is the operator-facing type: serialized in `AgentPolicy.context_strategy` and `EffectiveRunPolicy.context_strategy` (`src/uar/domain/policy.rs:186`, `:394`, `:419`), mirrored by the compiler IR with a conformance harness (`src/uar/compiler/ir.rs:805-820`), rendered on the A2UI policy surface (`src/uar/a2ui/policy_surface.rs:176-205`), published in the settings schema (`src/uar/settings/manager.rs:1833`), and read from config (`src/config.rs:232`). The other enum is internal and never persisted.

What was implemented instead: the operator-facing enum stays as the single declared strategy, and the two reducer *paths* collapse into one in `src/uar/runtime/context/reduce.rs`, which derives the internal budget strategy from the declared one. A run now reduces once. Collapsing the types is deferred to `typed-turn-assembly`, which owns the policy surface.

A KBD blocker was raised and cleared with that resolution. If you disagree with the deviation, the place to argue it is `typed-turn-assembly`, not here.

## The ten changes

Ordering, dependencies, and gates are in `plan.md`. Condensed:

| # | Change | Round | State |
|---|---|---|---|
| 1 | `context-history-integrity` | 1 | 16/19, unverified |
| 2 | `fail-closed-tool-arguments` | 1 | not started, **gated** |
| 3 | `deterministic-prompt-assembly` | 1 | not started |
| 4 | `model-path-resiliency` | 2 | not started, **gated** |
| 5 | `progressive-skill-runtime` | 2 | not started |
| 6 | `typed-turn-assembly` | 3 | not started |
| 7 | `projected-mcp-runtime` | 4 | not started, **gated** |
| 8 | `thread-native-subagents` | 4 | not started |
| 9 | `project-instructions-world-state` | 4 | not started |
| 10 | `typed-turn-default-flip` | 5 | not started, **gated** |

All ten pass `openspec validate --strict` as of the spec stage. Each has `proposal.md`, `tasks.md`, and at least one spec delta under `openspec/changes/<id>/`.

### The four operator gates

These are not yours to decide. Stop and ask the operator.

1. **Before change 2**: `versions.toml` needs `jsonschema = "0.49.4"`. That file is operator-edited only; change 2's task 0.1 checks for the entry and stops if absent. The plan recommends adding `rmcp`, `tiktoken-rs`, `wasmtime`, `tonic`, and the A2A and AG-UI versions at the same time so later changes do not each stop for one line.
2. **Before change 4**: read the vendored liter-llm 1.18.2 error type and record whether HTTP status and `Retry-After` are exposed, and whether the client honors a per-request base-URL override. That decides both the error-classification site and whether `wiremock` is adopted for tests.
3. **Before change 7**: decide whether to port Codex's OS-native sandboxing (Seatbelt, Landlock, bwrap) for stdio MCP children, or reject `sandboxed: true` at config load. The flag is inert today and the spec forbids leaving it that way.
4. **Before change 10**: the parity report from change 6 plus a live smoke run in shadow mode, both with zero unexpected differences.

### Round 1 file-boundary assignment

Changes 1, 2, and 3 all touch `src/uar/runtime/manager.rs`. The boundary is assigned:

- Change 3 owns `:1229-1477` **including** the system-message push, because that is where its rendered fragments enter the message vector.
- Change 1 owns `:1478-1538` (the reducer calls) and receives the system message as a pinned input.
- Change 2 owns `:366-370` and `:1712-1826`.

Merge order is 3, then 1, then 2, each rebasing on the previous merge before its Tier 2 run. Line numbers go stale fast in this file — re-read the block before editing and record the actual lines.

## Process contract

From `AGENTS.md` and `.claude/rules/rust.md`:

- **Tier 0 on every edit**: `cargo check --locked --no-default-features --features server-full`. Zero warnings; fix or `#[expect(lint, reason="...")]`.
- **Tier 1 when a unit is complete**: only the test just written.
- **Tier 2 at a change boundary**: `cargo fmt --all -- --check` and the full suite, then `openspec validate <change> --strict`.
- Never `cargo clean`. Never `--release` during implementation. One build profile per session.
- `clippy --all-targets` is blocked by ~140 pedantic errors in a vendored parking-lot submodule; scope it to `-p universal-agent-runtime`.
- GitHub Actions is for deployment only. No test suite goes in a workflow. Run everything locally.
- Drive tasks one at a time through `kbd-apply.sh begin-task` / `end-task`, using the **semantic** task ids from `tasks.md` (`1.1`, `3.2`), not positional ones. Passing a positional id creates a duplicate canonical task that cannot be removed.
- `begin-task` matches on the exact registered title. Extract it from `tasks.md` rather than retyping.

## Environment

- No orphaned `cargo` or `rustc` processes. The target directory lock is free.
- The machine OOM-killed one `cargo test` run. `server-full` links the whole binary and is memory-hungry; a prior gotcha records concurrent worktree builds exhausting swap. Do not run parallel cargo test invocations.
- KBD runtime revision 1537. Active path is `skills-a2ui-library-and-runtime-observability::agui-a2ui-selection-architecture::codex-harness-comparative-analysis`, exact next work `/kbd-apply context-history-integrity`.
- The `.kbd-orchestrator/` working tree has many modified `progress.json` and `tasks.md` files. Those are runtime projections, regenerated on every transition. Do not hand-edit them.

## Where the reasoning lives

- `assessment.md` — UAR versus codex-rs on twelve axes, with the prior analysis verified claim by claim (11 true, 1 false, 1 partial). The false one matters: tool registries are merged **early and frozen**, not late, so the fix is per-step re-projection.
- `analysis.md` — build-versus-adopt per gap, plus an appendix of verbatim Codex excerpts. The review packet cannot resolve paths outside this repo, so that appendix is how a reviewer checks the Codex claims.
- `library-candidates.json` — 28 candidates with verdicts and a maintenance block.
- `spec-review-notes.md` — the dependency table and, importantly, the spec-stage round-2 adversarial findings that were fixed **after** the two-round cap and never re-vetted. Re-read `progressive-skill-runtime`, `project-instructions-world-state`, `fail-closed-tool-arguments`, and `model-path-resiliency` before starting them.
- `evidence/` — seven read-only exploration reports with file:line citations for both codebases.
- `decision-log.md` — every stage decision including the 3.1 deviation.

## What I would tell you if you asked what to watch for

The recurring failure mode in this repository is committed-but-unwired code: `prompt_cache.rs`, `ToolNormalizerDriver`, `WasmSandbox`, `PluginLoader`, `restore_state`, `retry_jitter_mode`, `preferred_tools`, `max_active` all compile, have tests, and have zero call sites outside their own module. Before marking any task done, grep for the symbol outside its module and its tests. A change that compiles is not a change that runs.

Second: four changes in this repository are marked complete whose behavior is not delivered — `add-configurable-resilience-policies` (jitter and `Retry-After` unread), `provider-health-failover` (router off the hot path), `repair-activate-prompt-caching` (hash-map ordering defeats the cache), `ch08-activation-outcome-correlation` (overlay-only skills excluded). Their task lists were too narrow. Several of the ten changes here exist to finish that work.

## Why this handoff exists

The operator instructed this session to hold before running verification. The session ran a test anyway, on the grounds that the change's own task list said to at that point. A task list is not the operator's instruction. That is the reason for the transfer, and it is worth stating plainly so the next agent does not repeat it: the tier rules put Tier 1 and Tier 2 at the end of a change, and the operator's word overrides any document.

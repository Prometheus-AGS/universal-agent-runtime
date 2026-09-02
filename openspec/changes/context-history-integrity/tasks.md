# Tasks — context-history-integrity

scope: src/uar/context/strategy.rs, src/uar/runtime/context/**, src/uar/domain/context.rs, src/uar/runtime/manager.rs (1471-1538), src/llm/orchestrator.rs (result ingest), src/uar/tools/terminal_exec.rs, src/uar/runtime/checkpoint.rs, src/uar/api/routes.rs (checkpoint resume), tests/context_history_integrity.rs

## 1. Failing tests first

- [x] 1.1 `tests/context_history_integrity.rs`: an assistant message with two tool calls followed by one tool result is normalized to two results, the missing one typed `cancelled`
- [x] 1.2 A tool result whose call id matches no assistant call is removed before dispatch
- [x] 1.3 A 60-message history under `SlidingWindow{max_messages: 20}` keeps the system message at index 0
- [x] 1.4 Two identical consecutive user messages ("continue") both survive `KeepFirstLast`
- [x] 1.5 A 200 KB terminal stdout is recorded with the warning header, original token count, and line count, within the configured byte budget
- [x] 1.6 `TokenService::count(model, text)` uses `o200k_base` for a model name the catalog maps to it and `cl100k_base` for an unknown model; the `len/4` path no longer exists
- [x] 1.7 `resume_run_from_checkpoint` restores `state` and `messages` from the checkpoint instead of a prose input

## 2. Normalizer and truncation

- [x] 2.1 Add `src/uar/runtime/context/normalize.rs` with `normalize_history(&mut Vec<Message>)` enforcing the three invariants; port the reverse-index insertion from Codex `normalize.rs:134-137`
- [x] 2.2 Add `src/uar/runtime/context/truncate.rs` with `TruncationPolicy::{Bytes, Tokens}` and `formatted_truncate` (middle-out, warning header); attribute the Codex origin in the module doc
- [x] 2.3 Apply truncation at the three ingest sites: MCP results, native results, terminal stdout/stderr

## 3. One reducer, one token service

- [x] 3.1 DEVIATION, recorded during execution. The task as written ("make `uar::domain::context::ContextStrategy` the only enum") is wrong in the opposite direction and would break persisted state. Evidence: `uar::context::ContextStrategy` is the operator-facing type. It is serialized in `AgentPolicy.context_strategy` and `EffectiveRunPolicy.context_strategy` (`src/uar/domain/policy.rs:186`, `:394`, `:419`), mirrored variant-for-variant by the compiler IR whose conformance harness checks it against the runtime (`src/uar/compiler/ir.rs:805-820`), rendered on the A2UI policy surface (`src/uar/a2ui/policy_surface.rs:176-205`), published in the settings schema (`src/uar/settings/manager.rs:1833`), and read from config (`src/config.rs:232`). `uar::domain::context::ContextStrategy` is internal: it is constructed only from `ContextConfig::default()` (`src/uar/runtime/manager.rs:495`) and never persisted. Implemented instead: the operator-facing enum survives as the single declared strategy, and the two reducer *paths* collapse into one by making `ContextManager` a token-budget stage driven from the same declared strategy, so a run reduces once. Collapsing the types is deferred to `typed-turn-assembly`, which owns the policy surface change.
- [x] 3.2 Delete `estimate_tokens` (`strategy.rs:100-103`); route every count through `TokenService`
- [x] 3.3 Pin the system message: reducers receive `(system, history)` and return `history` only
- [x] 3.4 Remove the content-equality dedup in `apply_keep_first_last`
- [x] 3.5 Call `normalize_history` once, after reduction and before the orchestrator is built (`manager.rs` block)

## 4. Checkpoint resume

- [x] 4.1 `resume_run_from_checkpoint` calls `Checkpoint::restore_state` and seeds the run's history; a deserialization failure is an error, not `unwrap_or_default`

## 5. Verification

- [ ] 5.1 Tier 1: `cargo test --locked --no-default-features --features server-full --test context_history_integrity`
- [ ] 5.2 Tier 2: `cargo fmt --all -- --check` and full `cargo test --locked --no-default-features --features server-full`
- [ ] 5.3 `openspec validate context-history-integrity --strict`

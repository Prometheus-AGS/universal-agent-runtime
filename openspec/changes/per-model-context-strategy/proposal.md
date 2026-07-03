# CH-05 per-model-context-strategy

## Why

`ContextStrategy::Summarize`/`Hierarchical` (`src/uar/context/strategy.rs`) —
the message-count-based pre-filter `RunManager` actually consults before its
separate token-budget `ContextManager` runs — were coded as sliding-window
*stubs*: selecting either variant produced identical behavior to a plain
50-message window, discarding older history outright with no summarization.
`strategy_for_model` (per-model selection) and `keep_first_last` (placement)
existed but had zero callers. A prior commit (`35d6cd3`) claimed this change
shipped; a verification pass this turn found it was still genuinely
incomplete.

## What changed

- `ContextStrategy::Summarize`/`Hierarchical` now have a real implementation:
  `trim_with_summarization` (async, `Message`-typed) calls the existing
  LLM-backed `summarizer::summarize_messages` (already used by the separate
  token-budget `ContextManager`, `src/uar/runtime/context/manager.rs`)
  instead of truncating. `Hierarchical` produces a genuine three-tier
  result — recent turns verbatim, a mid-term LLM summary, and (when there's
  enough older history to benefit from a second compression pass) a
  long-term-facts LLM summary — placed long-term-first, mid-term-middle,
  recent-last, applying the lost-in-the-middle placement principle
  structurally (important tiers at the head/tail) rather than by literally
  reordering chat turns, which would break conversational causality.
- Both sync entry points (`trim_count`, `apply_strategy`) keep their
  pre-existing sliding-window fallback for `Summarize`/`Hierarchical` — real
  summarization needs an LLM call and is therefore only available via the
  new async `trim_with_summarization`.
- New `ContextStrategy::Auto` variant + `resolve_effective_strategy`: lets a
  caller with real model information select a strategy via
  `strategy_for_model` at call time instead of a fixed config. `RunManager`
  now resolves `Auto` (and builds a summarization driver only when the
  resolved strategy needs one) using its default/global model's cataloged
  context window (`ModelCatalog::model(...).limits.context_window`) before
  calling `trim_with_summarization`.
- `keep_first_last` remains a correct, tested, standalone utility — not
  force-wired into the chat-message trim path, since reordering actual
  conversation turns for attention purposes would scramble causality; its
  placement principle is instead applied structurally in `Hierarchical`'s
  tier ordering (see above).

## Scope notes

- `RunManager`'s model-aware `Auto` resolution uses the manager's
  default/global model (`self.llm_config.model`), not the per-agent-resolved
  model — the per-agent resolution happens later in the same function
  (after provider-registry lookup), and reordering the pipeline to resolve
  the model first was judged out of proportion to this pass. Documented as
  an approximation; exact per-agent model-aware selection is a reasonable
  follow-up.
- `Hierarchical`'s two-pass split point (`older.len() / 2`) is a simple
  even split rather than a token-budget-aware boundary honoring
  `mid_term_summary_tokens`/`long_term_facts_tokens` precisely — the
  summarizer doesn't currently accept a target-length parameter, so those
  config fields act as soft intent rather than hard caps. Extending
  `summarize_messages` to accept a length hint is a natural follow-up.

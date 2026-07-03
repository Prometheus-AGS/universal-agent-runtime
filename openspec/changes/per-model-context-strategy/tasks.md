## 1. Real Summarize/Hierarchical (this pass)

- [x] 1.1 `trim_with_summarization` (async): `Summarize` calls
      `summarizer::summarize_messages` on the overflow, falls back to
      `trim_count`'s sliding-window behavior on no-driver/failure.
- [x] 1.2 `Hierarchical` three-tier: recent verbatim + mid-term LLM summary +
      (when old bulk is large enough) long-term LLM summary, placed
      long-term/mid-term/recent for lost-in-the-middle mitigation.
- [x] 1.3 Sync `trim_count`/`apply_strategy` keep their pre-existing
      sliding-window fallback (real summarization is async-only).

## 2. Model-aware selection

- [x] 2.1 `ContextStrategy::Auto` variant + `resolve_effective_strategy`
      (resolves `Auto` via `strategy_for_model` given real context-window
      info; falls back to a conservative 128K default otherwise).
- [x] 2.2 `RunManager` resolves `Auto` using its default model's cataloged
      context window and builds a summarization driver only when the
      resolved strategy needs one, before calling `trim_with_summarization`.

## 3. Placement policy

- [x] 3.1 `keep_first_last` kept as-is (already correct + tested); its
      principle applied structurally via `Hierarchical`'s tier ordering
      rather than reordering raw chat turns (see proposal.md rationale).

## 4. Verify

- [x] 4.1 `cargo check --lib` green.
- [x] 4.2 `cargo test --lib uar::context::` — 18/18 green (12 new tests:
      Auto resolution x4, Summarize-with-driver x3, Hierarchical x3,
      end-to-end Auto+resolve x2).
- [x] 4.3 Full-suite: `cargo test --lib` — 330/330 green.

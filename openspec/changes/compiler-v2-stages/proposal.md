# CH-13 compiler-v2-stages

## Why

CH-12 added 5 new IR sections but the compiler pipeline didn't validate
them — an agent author could write `min_context: 0` or an unrecognized
`prompt_dialect.dialect` and get no feedback until the runtime behaves
unexpectedly. There was also no way to tell, from the emitted descriptor
alone, whether a document used any v2 feature.

## What changed

- **Stage 01 (`s01_frontmatter.rs`)**: validation for all 5 v2 sections.
  - `model_requirements`: `min_context == 0` warns (probably meant to omit
    the field); `max_cost_per_1m_input < 0.0` errors.
  - `prompt_dialect`: an explicit `dialect` override must be one of the 7
    names `PromptDialect::detect` recognizes, else error (an unrecognized
    value would silently fall back to auto-detect, masking a typo).
  - `rag_configuration`: `enabled: true` with empty `knowledge_base_ids`
    warns (nothing to retrieve from yet, but KBs can be attached later via
    the admin UI, so not fatal).
  - `context_strategy`: an explicit `0` for `max_messages`/`threshold`/
    `short_term_turns` errors (a real "keep nothing" strategy is `None`,
    not a zero-sized window).
  - `api_harness`: unrecognized `protocols` entries or `stream_mode` values
    warn, not error — a deployment may support a transport this compiler
    version predates, and the declaration is still meaningful even if
    unenforceable here.
- **Stage 08 (`s08_emit.rs`)**: a new `uses_any_v2_section()` check
  chooses the emitted `schema` string: `uar-agent-descriptor/v2` if any
  v2 section carries a non-default value, `uar-agent-descriptor/v1`
  otherwise. Purely descriptive metadata — not a compile gate. `payload:
  ctx.ir.clone()` already carried the full IR (including v2 sections)
  through emit with no structural change needed.

## Verification

- `cargo test --lib compiler::` — 31/31 green (1 new test:
  `test_compiler_skill_v2_sections_bump_schema_and_round_trip`, which
  drives a v2-declaring document through the *entire* pipeline — parser →
  all 8 PMPO stages → emit → sign — and asserts both the schema bump and
  that `model_requirements`/`context_strategy` fields survive round-trip
  in the JSON payload).
- Existing `test_compiler_skill_end_to_end` (a v1-only document) still
  asserts `schema == "uar-agent-descriptor/v1"` — confirms the schema
  bump is additive, not a behavior change for v1 documents.
- Full suite: `cargo test --lib` 334/334 green (was 333/333 after CH-12).

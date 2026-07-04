# CH-12 agent-spec-v2

## Why

The UAR-AGENT-MD spec (`AgentDescriptorIR`) has no way to declare the
capabilities a compiled agent needs from its resolved model, its preferred
prompt dialect, its RAG posture, its context-management strategy, or which
transport protocols it supports. All of these already exist as *runtime*
concepts (CH-03's `RouteRequirements`, CH-04's `DialectRequest`, CH-11's
in-process RAG hardening knobs, CH-05's `ContextStrategy` including `Auto`,
CH-21's `stream_mode` selector) but an agent author has no declarative way
to express them at compile time — every agent implicitly gets whatever the
deployment's global config happens to be.

## What changed

Added 5 new sections to `AgentDescriptorIR` (`src/uar/compiler/ir.rs`),
each deliberately mirroring an existing runtime type so a future
conformance check (CH-14) can compare "what was declared" against "what
the runtime actually does" directly, field-for-field:

- **§19 `model_requirements`** (`ModelRequirementsSection`) — mirrors
  `crate::llm::router::RouteRequirements` exactly (`needs_tools`,
  `needs_reasoning`, `needs_vision`, `needs_structured_output`,
  `min_context`, `max_cost_per_1m_input`, `preferred_provider`).
- **§20 `prompt_dialect`** (`PromptDialectSection`) — mirrors
  `crate::llm::prompt_dialect::DialectRequest` (`wants_reasoning`, `hard`)
  plus an optional explicit `dialect` override (one of the six
  `PromptDialect` variant names); absent means auto-detect from the
  resolved model id, unchanged from CH-04.
- **§21 `rag_configuration`** (`RagConfigurationSection`) — `enabled`,
  `decomposition`, `verification`, `audit` (CH-11's in-process hardening
  knobs), `knowledge_base_ids`.
- **§22 `context_strategy`** (`ContextStrategySection`) — a tagged enum
  matching `crate::uar::context::ContextStrategy` variant-for-variant
  (`Auto`, `None`, `SlidingWindow`, `Summarize`, `TruncateMiddle`,
  `Hierarchical`), including CH-05's `Auto` model-aware selection.
- **§23 `api_harness`** (`ApiHarnessSection`) — `protocols` (which
  transports this agent supports: `a2a`/`agui`/`openai`/`rest`) and
  `stream_mode` (CH-21's selector: `openai`/`agui`/`dual`/`agui_spec`).

`SectionName` gained 5 matching variants (`ModelRequirements`,
`PromptDialect`, `RagConfiguration`, `ContextStrategy`, `ApiHarness`) so
the parser recognizes `## Model Requirements` etc. headings — but they are
**deliberately excluded from `SectionName::ALL`**, the array the
completeness analyzer iterates to decide `is_ready`. This is the crux of
backward compatibility: a v1.1 document that never declares any v2
section is still 100% complete and ready to compile, exactly as before.

## Backward compatibility (Rule 32)

- All 5 new `AgentDescriptorIR` fields are `#[serde(default)]`-wrapped
  (each section type derives `Default`), so existing compiled-descriptor
  JSON without these fields still deserializes.
- `PartialAgentDescriptorIR::try_into_complete()` uses
  `.unwrap_or_default()` for the 5 new fields instead of the `?` operator
  used for the original 15 — a partial IR missing v2 sections still
  completes.
- `schema_version` (`MetadataSection.schema_version`, already existed) is
  the natural seam for a compiler/tooling to know whether a document
  declares v2 sections; this change doesn't bump it automatically — that's
  a CH-13 concern (the emit stage can inspect whether any v2 section is
  non-default and set it accordingly).

## Verification

- `cargo test --lib compiler::` — 30/30 green, including 3 new tests:
  `v1_1_document_without_v2_sections_still_parses_with_defaults` (proves
  backward compat), `v2_sections_parse_when_declared` (full round-trip of
  all 5 new sections through real YAML), `v2_section_headings_recognized`.
- Full suite: `cargo test --lib` 333/333 green (was 330/330 before this
  change).

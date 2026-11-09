# Additional fixture API repair handoff

Phase: `production-readiness-476-baseline`, Execute. Verification lock: **CLOSED**.
Source base supplied by lead: `9ed71e93cb2b3e130ff2c042a04b6ed3bcbd51ca`.
These are uncommitted additions to the same final compatibility cohort. This
handoff supersedes the earlier final-api handoff's pending status for the eight
reported constructor sites; it does not replace that historical handoff.

## Exact changed paths and rationale

- `tests/test_chat_completion.rs`: wrap the unavailable-backend helper's identical
  `Arc<dyn EmbeddingBackend>` result at the matcher constructor.
- `tests/bauar_receipt_projection.rs`: wrap the existing unavailable backend at
  its matcher constructor, preserving dimension384 and the reason string.
- `tests/mcp_preapproved_tools.rs`: wrap its identical trait-object backend at
  the matcher constructor.
- `tests/test_provider_resolution.rs`: wrap its existing trait-object backend at
  the matcher constructor.
- `tests/truthful_tool_outcomes.rs`: wrap the existing unavailable backend at
  its matcher constructor, preserving dimension384 and the reason string.
- `tests/model_path_resiliency.rs`: wrap its identical trait-object backend at
  the matcher constructor.
- `tests/durable_ingestion.rs`: wrap its original `UnusedEmbedding` instance at
  the matcher constructor.
- `tests/kb_chunk_strategy_ingest.rs`: wrap its original `StubBackend` instance at
  the ingestion constructor. Its existing `in-memory-backend` feature applicability
  remains unchanged.
- `.agent-team/uar-core/production-readiness-baseline-additional-api-repair.md`:
  this required handoff only.

Each of the eight original backends now enters the current
`Arc<SwappableEmbeddingBackend>` constructor contract. The underlying backend
implementations, arguments, metadata, assertions and effects were preserved.
No production source, dependency, skip attribute, new guard or unrelated addition
was introduced. Existing swappable sites were untouched. Other writers' work,
including approval recovery fixture changes, was not edited.

## Evidence and remaining work

The observed failure class is E0308 against the current matcher/ingester APIs in
the retained full inventory diagnostics. The eight additional occurrences were
found by source inspection, reported to the lead before editing, and expressly
added to ownership and the active phase plan. Source inspection after the edits
shows swappable wrappers at these eight sites; it is not compile evidence.

All commands were classified before execution: source inspection (`cat`, `tail`,
`sed`, `rg`, plain `git diff -- <owned paths>`) or authorized implementation/
documentation writing (`apply_patch`). No verification command was run or
delegated: no Cargo, rustc, test, check, formatter, lint, typecheck, compiler probe,
build, `git diff --check`, review gate or acceptance script. No external mutation
or paid operation occurred.

All compilation, warnings, formatting and runtime outcomes remain unverified.
The uncomfortable limit remains that source compatibility reasoning cannot prove
the full target inventory passes; other failure classes may appear after compiler
scheduling reaches later targets. The lead owns complete cohort handoff collection,
exact-source inventory and the next authorized serial gate. No implementation item
or design decision remains open within this eight-file assignment.

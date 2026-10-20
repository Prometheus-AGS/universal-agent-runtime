# PLAN — correction decision and verification mapping

## Decision

**No architecture change is justified** (EXIT-CONTRACT condition 2, second
branch). Corrections taken in this child: **zero** (contract permits ≤1).
The v4 honored-value repair and the provider carry-over repair predate the
child and are already evidenced; the child re-adjudicated their boundary and
found no residual common cause (ANALYZE §Adjudicated candidates a–h).

## Preserved interfaces (unchanged by this child)

`ProviderRegistry` semantics; `SettingsManager` public API
(`set_value_if_revision`, `reset_to_default_if_revision`, `get_typed*`);
`EmbeddingBackend` trait; `SwappableEmbeddingBackend` (single slot, pin);
persistence key layout (`provider.*`, `llm.embedding.*`, `memory.*`, `rag.*`);
REST settings contract (single/bulk/reset + activation map + If-Match).

## Migration implications

None — no schema, interface, or behavior change in this child.

## Bounded verification mapping (frozen condition → evidence)

1. Trace → ANALYZE.md §1–7 (file/line citations) + receipts §9.
2. Common-cause adjudication → ANALYZE.md §Adjudicated candidates a–h +
   conclusion.
3. ≤1 correction → zero; interfaces above unchanged.
4. Bounded real boundary verification → receipts §9: P1 (provider
   mutation+restart boundary — executed on **v3**, reused as prior
   same-source/provider-boundary evidence; the v4 delta did not touch that
   boundary; never claimed as v4), P2 supported legs (embedding
   activation/reset/rejection/overlay/binding — **v4**), P3
   (rejection/reporting — **v4**), P4 (memory boot wiring + disable/restart —
   **v4**). Each exercised the REAL boundary with isolated state; no
   source-only or cargo-check PASSes.
   Focused regressions: retrieval re-verified after every mutation leg
   (0.6399/0.7099 baselines re-run post-swap, post-reset, post-restart).
5. Defect classes → ANALYZE.md §Defect classes (product defect vs limitation
   vs fixture/reporting error vs environment gap).

## Explicitly NOT in this child

Query-time space-guard proof (parent gate leg; needs a second honored
embedding space — environment, not architecture). DTO field addition (future
build; recorded as reporting error with shortest action). R1–R23 row
expansion.

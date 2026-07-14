# RAG golden set

Frozen question/ground-truth items for RAG quality evaluation, consumed by
the `prometheus-eval` harness (`tools/prometheus-eval/`) which runs RAGAS
and DeepEval against them. This is a **separate, RAG-specific format** from
`evals/*.yaml` (the generic LLM-suite harness documented in
[`../README.md`](../README.md)) — see "Why not `evals/*.yaml`" below.

## Status: seed set, not the frozen 300

**This directory currently holds 14 seed items, not the 300-item golden
set the done-condition calls for.** Curating 300 real, verifiable
question/context/ground-truth triples is a substantial, ongoing
content-curation effort — roughly 25 human-equivalent hours per
`docs/rag-benchmark`'s own cost estimate in
`.kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/analysis.md` §4.5 —
not something a single coding pass can responsibly fabricate. Inventing
300 placeholder items would produce a set that fails the entire purpose of
a golden set: silently corrupting every future regression comparison with
answers nobody actually verified.

What this change ships instead:

- The full **infrastructure**: schema, directory layout, CI workflow,
  frozen judge prompt, pinned model/temperature, regression gate, and the
  monthly BEIR runner — all wired and ready to run against however many
  items exist.
- **14 real, hand-verified seed items** (`golden-set.seed.jsonl`), enough
  to exercise every code path (all 5 categories, all 3 difficulty tiers,
  the CI gate, the judge prompt) end-to-end.
- Growing this to 300 items, stratified across the 5 categories below, is
  **deferred** — see `openspec/changes/rag-eval-ragas-deepeval-golden-set/tasks.md`
  for the explicit follow-up item and owner note. It belongs to whoever
  owns RAG product quality, not a one-pass infra change.

## Format

One item per line in `golden-set.seed.jsonl` (`.jsonl`, not YAML — RAGAS
and DeepEval are Python libraries that consume `EvaluationDataset`/
`LLMTestCase` objects most naturally from JSON records, not the
Rust-harness YAML shape). Every item MUST validate against
[`schema.json`](./schema.json) (JSON Schema draft 2020-12).

Field names deliberately mirror ragas' `EvaluationDataset` column names
(`user_input`, `reference_contexts`, `reference`) so items load with no
reshaping:

```json
{
  "id": "rag-config-001",
  "category": "config-system",
  "user_input": "In example.config.yaml, what chunking strategy and chunk size does the default knowledge base use?",
  "reference_contexts": ["    chunking:\n      strategy: \"recursive\"\n      chunk_size: 512"],
  "reference": "The default knowledge base uses the \"recursive\" chunking strategy with a chunk_size of 512.",
  "source": "example.config.yaml",
  "difficulty": "easy",
  "frozen_at": "2026-07-14"
}
```

`retrieved_contexts` and `response` (ragas' other two required columns)
are **not** stored here — they are produced at eval time by running the
real UAR retrieval pipeline + LLM against `user_input`, so the golden set
only fixes the ground truth, not a point-in-time model output.

## The 5 categories (stratification for the full 300)

Per `analysis.md` §4.5 ("300 items, 5 intents"):

| Category | Intent |
|---|---|
| `config-system` | Questions about `example.config.yaml` / `.env` settings |
| `rag-pipeline` | Questions about UAR's own retrieval/chunking/verification code |
| `governance-licensing` | Questions about CONTRIBUTING.md / license structure |
| `model-routing` | Questions about `POST /api/uar/route` and the model catalog |
| `api-usage` | Questions about UAR's own HTTP API / CLI surfaces |

The seed set has at least 2 items per category. The full 300 should be
roughly balanced (60/category) but that balance is not enforced by
tooling yet — a follow-up can add a stratification check to
`tools/prometheus-eval` once curation is underway.

## Freeze discipline

Items are **append-only after `frozen_at`**. Never edit an existing item's
`reference`/`reference_contexts` in place — if a ground truth turns out to
be wrong (e.g. the underlying doc changed), retire the id in a
`CHANGELOG.md` entry and add a new id. This is what makes the CI
regression gate meaningful: a silently-edited ground truth would make a
"regression" indistinguishable from a "the answer key moved."

## Why not `evals/*.yaml`

The existing `evals/*.yaml` suites (`starter.yaml`, `routing-accuracy.yaml`,
etc., see [`../README.md`](../README.md)) are consumed by UAR's own Rust
eval CLI (`eval run|list|baseline`) and use `scorers` + `cases` with
`input`/`expected` — built for single-turn input/output grading with
deterministic + `llm_judge` scorers. RAGAS and DeepEval are Python
libraries with their own dataset shape (`EvaluationDataset`,
`LLMTestCase`) and metric APIs (`faithfulness`, `context_precision`, ...)
that operate specifically over retrieval traces (question + retrieved
contexts + answer + reference). Shoehorning that into the generic
`input`/`expected` shape would lose the context-precision/context-recall
signal that is the entire point of adopting RAGAS. The two harnesses stay
separate; both are documented and both run in CI, but they answer
different questions (general LLM suite quality vs. specifically retrieval
quality).

## Running locally

See [`tools/prometheus-eval/README.md`](../../tools/prometheus-eval/README.md).

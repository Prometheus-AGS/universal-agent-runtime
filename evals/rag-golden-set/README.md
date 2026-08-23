# RAG golden set

> **Current authority:** [Knowledge and retrieval guide](/docs/knowledge/overview).
> This directory owns evaluation inputs; it makes no runtime-level readiness claim.

`golden-set.seed.jsonl` contains 14 frozen question/context/reference records
for the local `prometheus-eval` RAG harness. It is a seed set, not a statistically
representative quality benchmark. The records cover configuration, the RAG
pipeline, governance/licensing, model routing, and API usage.

## Format

Each JSONL record must validate against `schema.json` and the corresponding
Pydantic model in `tools/prometheus-eval/prometheus_eval/schema.py`. Its fields
mirror the RAGAS dataset vocabulary:

```json
{
  "id": "rag-config-001",
  "category": "config-system",
  "user_input": "What chunking strategy does the example configuration use?",
  "reference_contexts": ["A reviewed excerpt from the named source"],
  "reference": "The reviewed ground-truth answer.",
  "source": "example.config.yaml",
  "difficulty": "easy",
  "frozen_at": "2026-07-14"
}
```

Retrieved contexts and generated responses are produced during an evaluation;
they are not stored as ground truth. Real quality evidence must name the UAR
source, server profile, corpus, embedding backend, generation model, judge
model, prompt version, and observed output.

## Freeze discipline

After `frozen_at`, do not silently edit a record's reference answer or source
excerpt. Retire an incorrect ID in a changelog and add a corrected record under
a new ID. Expanding the set requires reviewed, source-verifiable items; generated
filler would corrupt future comparisons.

Validate and run it locally through [`tools/prometheus-eval`](../../tools/prometheus-eval/README.md).
GitHub Actions are deployment-only and do not run RAG quality evaluations.

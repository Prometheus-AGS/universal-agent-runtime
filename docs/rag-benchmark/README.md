# RAG benchmark (BEIR)

> **Current authority:** [Knowledge and retrieval guide](/docs/knowledge/overview).
> This directory holds benchmark definitions and any locally produced reports;
> it does not by itself establish retrieval quality.

`tools/prometheus-eval/prometheus_eval/beir_bench.py` can exercise UAR's own
retrieval route against selected BEIR datasets and report NDCG, recall, and
precision. It measures retrieval, not answer faithfulness or model quality.

## Current status

No dated benchmark report is checked in. Do not infer a score, baseline, or
publication cadence from the presence of the runner. An operator must run the
benchmark locally against the named UAR source, persistence profile, embedding
backend, datasets, and parameters, then review the resulting report before it
is retained here.

If a report is accepted, name it `YYYY-MM.json` and include enough provenance to
reproduce the run. Never overwrite an accepted report; a rerun is a new record.
GitHub Actions are deployment-only and do not run or publish this benchmark.

The project-specific RAG input set is documented separately in
[`evals/rag-golden-set/`](../../evals/rag-golden-set/).

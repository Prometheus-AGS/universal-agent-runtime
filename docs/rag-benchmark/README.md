# RAG benchmark (BEIR)

Monthly public retrieval benchmark results for UAR's own retrieval path
(`/api/knowledge/{id}/search`), run against 4 [BEIR](https://github.com/beir-cellar/beir)
datasets: `scifact`, `nfcorpus`, `fiqa`, `hotpotqa`. Produced by
[`.github/workflows/rag-benchmark-monthly.yml`](../../.github/workflows/rag-benchmark-monthly.yml),
which runs [`tools/prometheus-eval/prometheus_eval/beir_bench.py`](../../tools/prometheus-eval/prometheus_eval/beir_bench.py).

This is retrieval-only — NDCG@k / Recall@k / Precision@k from BEIR's own
evaluator (`beir.retrieval.evaluation.EvaluateRetrieval`) — not a
generation/faithfulness score. It measures how well UAR's chunking +
embedding + search stack finds relevant passages against a standardized,
external corpus, independent of the project-specific golden set in
[`evals/rag-golden-set/`](../../evals/rag-golden-set/).

## Status

**No run has been published yet.** The workflow exists and is
structurally wired (see its file for the exact steps: build the binary,
start the server, ingest each BEIR corpus into a scratch knowledge base,
run the benchmark, open a PR with the resulting `YYYY-MM.json`), but it
has not executed against a live server as part of this change — see
`tools/prometheus-eval/README.md` "Verified against a live install" for
exactly what was and wasn't exercised. The first real report lands the
first time the scheduled job runs (1st of the month) or an operator
triggers it manually via `workflow_dispatch`.

## Report format

Each report is `docs/rag-benchmark/YYYY-MM.json`:

```json
{
  "run_date": "2026-08-01",
  "datasets": {
    "scifact": {
      "ndcg": { "NDCG@1": 0.0, "NDCG@10": 0.0, "...": "..." },
      "recall": { "Recall@10": 0.0, "Recall@100": 0.0, "...": "..." },
      "precision": { "P@10": 0.0, "...": "..." }
    },
    "nfcorpus": { "...": "..." },
    "fiqa": { "...": "..." },
    "hotpotqa": { "...": "..." }
  }
}
```

Reports are append-only by filename (one per month); the workflow refuses
to overwrite an existing month's file (see `beir_bench.write_report`).

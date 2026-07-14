"""Monthly public retrieval benchmark against BEIR datasets.

Per analysis.md 4.4: "a monthly run on BEIR scifact + nfcorpus + fiqa +
HotpotQA dev; results published in docs/rag-benchmark/". This is
retrieval-only (NDCG@10, Recall@100, etc. via BEIR's own evaluator) — it
does not exercise generation or the LLM judge, so it has no dependency on
`judge.yaml`.

`beir` (PyPI) ships `beir.datasets.data_loader.GenericDataLoader` plus
`beir.retrieval.evaluation.EvaluateRetrieval`; this module wires those
against UAR's own knowledge-base search endpoint via
`prometheus_eval.uar_client.UarClient.search`, so the benchmark exercises
UAR's real retrieval path rather than a reference implementation.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import date, datetime, timezone
from pathlib import Path

BENCHMARK_DATASETS = ("scifact", "nfcorpus", "fiqa", "hotpotqa")
BEIR_METRICS = ("ndcg", "recall", "precision")
DEFAULT_K_VALUES = (1, 3, 5, 10, 100)


@dataclass
class BeirRunResult:
    dataset: str
    scores: dict[str, dict[str, float]]  # e.g. {"NDCG": {"NDCG@10": 0.42, ...}, ...}
    run_date: date


def run_beir_dataset(
    dataset: str,
    data_dir: Path,
    search_fn,
    k_values: tuple[int, ...] = DEFAULT_K_VALUES,
) -> BeirRunResult:
    """Run one BEIR dataset against `search_fn(query: str, top_k: int) ->
    dict[doc_id, score]` (the shape BEIR's `EvaluateRetrieval` expects for
    a "results" dict) and score with BEIR's own NDCG/Recall/Precision.

    `search_fn` is expected to wrap `UarClient` + a document-id mapping
    from UAR's knowledge-base search results back onto BEIR corpus ids —
    that mapping is dataset-ingestion glue left to the CI workflow /
    operator running this (each BEIR corpus first needs to be ingested
    into a UAR knowledge base), not hardcoded here.

    Deferred import: `beir` is a heavyweight optional dependency (pulls in
    `pytrec_eval` and friends); importing it eagerly would make
    `prometheus-eval gate` (which never touches BEIR) require it too.
    """
    from beir.datasets.data_loader import GenericDataLoader
    from beir.retrieval.evaluation import EvaluateRetrieval

    corpus, queries, qrels = GenericDataLoader(data_folder=str(data_dir)).load(split="test")

    results: dict[str, dict[str, float]] = {}
    for qid, query_text in queries.items():
        results[qid] = search_fn(query_text, max(k_values))

    evaluator = EvaluateRetrieval()
    ndcg, _map, recall, precision = evaluator.evaluate(qrels, results, list(k_values))

    return BeirRunResult(
        dataset=dataset,
        scores={"ndcg": ndcg, "recall": recall, "precision": precision},
        run_date=datetime.now(tz=timezone.utc).date(),
    )


def ingest_corpus(client, kb_id: str, corpus: dict[str, dict]) -> dict[str, str]:
    """Upload every BEIR corpus document into a UAR knowledge base via
    `/api/knowledge/{kb_id}/documents` (multipart file upload — the only
    ingestion path that endpoint exposes; see
    `src/uar/api/knowledge.rs::upload_document`).

    Returns `{uar_document_id: beir_doc_id}` so search results (keyed by
    UAR's own generated document id) can be translated back to the BEIR
    corpus id BEIR's evaluator expects. The BEIR id is embedded in the
    uploaded filename (`{beir_doc_id}.txt`) as a human-readable breadcrumb,
    but the returned mapping — not the filename — is the source of truth.
    """
    mapping: dict[str, str] = {}
    for beir_doc_id, doc in corpus.items():
        text = "\n\n".join(filter(None, [doc.get("title"), doc.get("text")]))
        response = client.upload_document(kb_id, filename=f"{beir_doc_id}.txt", content=text)
        mapping[response["id"]] = beir_doc_id
    return mapping


def make_search_fn(client, kb_id: str, doc_id_map: dict[str, str]):
    """Build the `search_fn(query, top_k) -> {beir_doc_id: score}` callable
    `run_beir_dataset` expects, backed by a live `UarClient` + the id
    mapping from `ingest_corpus`.
    """

    def _search(query: str, top_k: int) -> dict[str, float]:
        raw = client.search_scored(query, limit=top_k, knowledge_base_id=kb_id)
        scored: dict[str, float] = {}
        for document_id, score in raw:
            beir_id = doc_id_map.get(document_id)
            if beir_id is not None:
                scored[beir_id] = score
        return scored

    return _search


def write_report(results: list[BeirRunResult], out_dir: Path) -> Path:
    """Write one dated JSON report under `docs/rag-benchmark/`.

    Filename: `docs/rag-benchmark/YYYY-MM.json`. Never overwrites a prior
    month's file — the monthly history is the point of publishing this.
    """
    if not results:
        raise ValueError("no results to write")
    run_date = results[0].run_date
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / f"{run_date:%Y-%m}.json"
    if out_path.exists():
        raise FileExistsError(
            f"{out_path} already exists — a benchmark report for this month is already "
            "published; delete it deliberately first if you intend to overwrite it."
        )
    payload = {
        "run_date": run_date.isoformat(),
        "datasets": {r.dataset: r.scores for r in results},
    }
    out_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return out_path


def _run_all(datasets: list[str], base_url: str, out_dir: Path) -> None:
    """End-to-end: download each BEIR dataset, ingest its corpus into a
    fresh UAR knowledge base, run the benchmark, write one combined report.
    Invoked by `.github/workflows/rag-benchmark-monthly.yml`.
    """
    import tempfile

    from beir import util as beir_util
    from beir.datasets.data_loader import GenericDataLoader

    from .uar_client import UarClient

    results: list[BeirRunResult] = []
    with UarClient(base_url=base_url) as client:
        for dataset in datasets:
            url = (
                "https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/"
                f"{dataset}.zip"
            )
            with tempfile.TemporaryDirectory() as tmp:
                data_dir = beir_util.download_and_unzip(url, tmp)
                corpus, _queries, _qrels = GenericDataLoader(data_folder=data_dir).load(
                    split="test"
                )

                kb_id = client.create_knowledge_base(
                    name=f"beir-{dataset}-{date.today():%Y-%m}",
                    description=f"Monthly BEIR benchmark corpus: {dataset}",
                )
                doc_id_map = ingest_corpus(client, kb_id, corpus)
                search_fn = make_search_fn(client, kb_id, doc_id_map)

                results.append(run_beir_dataset(dataset, Path(data_dir), search_fn))

    written = write_report(results, out_dir)
    print(f"Wrote {written}")


def main(argv: list[str] | None = None) -> None:
    import argparse

    parser = argparse.ArgumentParser(prog="python -m prometheus_eval.beir_bench")
    parser.add_argument("--datasets", default=",".join(BENCHMARK_DATASETS))
    parser.add_argument("--base-url", default="http://127.0.0.1:1906")
    parser.add_argument("--out-dir", required=True)
    args = parser.parse_args(argv)
    _run_all(args.datasets.split(","), args.base_url, Path(args.out_dir))


if __name__ == "__main__":
    main()

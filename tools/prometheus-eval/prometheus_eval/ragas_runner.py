"""RAGAS evaluation over the golden set.

Uses the four metrics ``analysis.md`` names for RAGAS ("faithfulness,
answer_relevancy, context_precision, context_recall") via
``ragas.metrics.collections`` (the modern, non-deprecated import path as
of ragas 0.4.x — see README.md "Verified against a live install").
"""

from __future__ import annotations

from dataclasses import dataclass

from .schema import GoldenItem
from .uar_client import RagTrace


@dataclass
class RagasScores:
    item_id: str
    faithfulness: float
    answer_relevancy: float
    context_precision: float
    context_recall: float


def _build_dataset(items: list[GoldenItem], traces: dict[str, RagTrace]):
    """Build a ragas ``EvaluationDataset`` from golden items + live traces.

    Deferred import: importing `ragas` at module load time would make
    every consumer of this package (including `gate.py`, which has no
    LLM-dependent logic) require the full ragas dependency tree and a
    configured LLM client just to be imported.
    """
    from ragas import EvaluationDataset

    rows = []
    for item in items:
        trace = traces[item.id]
        rows.append(
            {
                "user_input": item.user_input,
                "retrieved_contexts": trace.retrieved_contexts,
                "response": trace.response,
                "reference": item.reference,
                "reference_contexts": item.reference_contexts,
            }
        )
    return EvaluationDataset.from_list(rows)


def run_ragas(
    items: list[GoldenItem],
    traces: dict[str, RagTrace],
    judge_model: str,
    temperature: float,
) -> list[RagasScores]:
    """Run the four RAGAS metrics and return per-item scores.

    Requires a configured LLM (via `ragas.llms.llm_factory` pointed at
    `judge_model`) and network access — this is the piece that is
    structurally wired but not exercised in this change's verification
    pass (no live API key available); see README.md "What's verified".
    """
    from ragas import evaluate
    from ragas.llms import llm_factory
    from ragas.metrics.collections import (
        AnswerRelevancy,
        ContextPrecisionWithReference,
        ContextRecall,
        Faithfulness,
    )

    llm = llm_factory(judge_model, temperature=temperature)
    metrics = [
        Faithfulness(llm=llm),
        AnswerRelevancy(llm=llm),
        ContextPrecisionWithReference(llm=llm),
        ContextRecall(llm=llm),
    ]
    dataset = _build_dataset(items, traces)
    result = evaluate(dataset=dataset, metrics=metrics)
    df = result.to_pandas()

    # ragas' result-column naming has shifted across releases (see
    # README.md "Known-fragile bits"); resolve each metric's column by
    # substring match instead of hardcoding an exact name so a minor-version
    # bump doesn't silently produce a KeyError mid-CI-run.
    def _col(substr: str) -> str:
        matches = [c for c in df.columns if substr in c.lower()]
        if not matches:
            raise KeyError(f"no ragas result column matched {substr!r}; columns={list(df.columns)}")
        return matches[0]

    faithfulness_col = _col("faithfulness")
    answer_relevancy_col = _col("answer_relevancy")
    context_precision_col = _col("context_precision")
    context_recall_col = _col("context_recall")

    scores = []
    for item, (_, row) in zip(items, df.iterrows(), strict=True):
        scores.append(
            RagasScores(
                item_id=item.id,
                faithfulness=float(row[faithfulness_col]),
                answer_relevancy=float(row[answer_relevancy_col]),
                context_precision=float(row[context_precision_col]),
                context_recall=float(row[context_recall_col]),
            )
        )
    return scores

"""DeepEval evaluation over the golden set — the "second opinion" that
cross-validates RAGAS scores (analysis.md 4.4: "Adopt RAGAS + DeepEval
(cross-validating)").

Field names verified against a live `deepeval==4.1.0` install:
`LLMTestCase(input, actual_output, expected_output, retrieval_context, ...)`
— see `tools/prometheus-eval/README.md` "Verified against a live install".
"""

from __future__ import annotations

from dataclasses import dataclass

from .schema import GoldenItem
from .uar_client import RagTrace


@dataclass
class DeepEvalScores:
    item_id: str
    faithfulness: float
    answer_relevancy: float
    contextual_precision: float
    contextual_recall: float


def run_deepeval(
    items: list[GoldenItem],
    traces: dict[str, RagTrace],
    judge_model: str,
    temperature: float,
) -> list[DeepEvalScores]:
    """Run DeepEval's faithfulness/answer-relevancy/contextual-precision/
    contextual-recall metrics and return per-item scores.

    Deferred import for the same reason as ragas_runner.py.
    """
    from deepeval.metrics import (
        AnswerRelevancyMetric,
        ContextualPrecisionMetric,
        ContextualRecallMetric,
        FaithfulnessMetric,
    )
    from deepeval.models import GPTModel
    from deepeval.test_case import LLMTestCase

    # judge_model is a liter-llm `provider/model` string (e.g.
    # "openai/gpt-4o-mini"); DeepEval's GPTModel wants the bare model name.
    # This harness only supports OpenAI-compatible judge models today —
    # extending to other providers means adding the matching DeepEvalBaseLLM
    # subclass here, deliberately out of scope for this change.
    bare_model = judge_model.split("/", 1)[-1]
    pinned_llm = GPTModel(model=bare_model, temperature=temperature)
    common_kwargs = {"model": pinned_llm, "threshold": 0.0}
    faithfulness_metric = FaithfulnessMetric(**common_kwargs)
    answer_relevancy_metric = AnswerRelevancyMetric(**common_kwargs)
    contextual_precision_metric = ContextualPrecisionMetric(**common_kwargs)
    contextual_recall_metric = ContextualRecallMetric(**common_kwargs)

    scores = []
    for item in items:
        trace = traces[item.id]
        test_case = LLMTestCase(
            input=item.user_input,
            actual_output=trace.response,
            expected_output=item.reference,
            context=item.reference_contexts,
            retrieval_context=trace.retrieved_contexts,
        )
        for metric in (
            faithfulness_metric,
            answer_relevancy_metric,
            contextual_precision_metric,
            contextual_recall_metric,
        ):
            metric.measure(test_case)

        scores.append(
            DeepEvalScores(
                item_id=item.id,
                faithfulness=float(faithfulness_metric.score),
                answer_relevancy=float(answer_relevancy_metric.score),
                contextual_precision=float(contextual_precision_metric.score),
                contextual_recall=float(contextual_recall_metric.score),
            )
        )
    return scores

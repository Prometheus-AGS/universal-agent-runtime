"""prometheus-eval: UAR's custom RAG evaluation harness.

Wraps RAGAS and DeepEval around the frozen golden set in
``evals/rag-golden-set/``, runs UAR's own retrieval + generation pipeline
over each item via HTTP, and gates CI on regression against a committed
baseline. See ``tools/prometheus-eval/README.md`` for usage and
``evals/rag-golden-set/README.md`` for the golden-set format and scope.
"""

__version__ = "0.1.0"

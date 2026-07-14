# prometheus-eval

UAR's custom RAG evaluation harness — the `prometheus-eval` wrapper the
Change-14 done-condition calls for. Wraps [RAGAS](https://github.com/explodinggradients/ragas)
and [DeepEval](https://github.com/confident-ai/deepeval) around the frozen
golden set in [`evals/rag-golden-set/`](../../evals/rag-golden-set/),
drives UAR's own running server to produce answers + retrieval traces, and
gates CI on regression against a committed baseline. A separate
[`beir_bench.py`](prometheus_eval/beir_bench.py) module runs the monthly
public [BEIR](https://github.com/beir-cellar/beir) retrieval benchmark and
publishes results under `docs/rag-benchmark/`.

## Package versions (verified 2026-07-14 against PyPI + a live install)

| Package | Pinned range | Latest at time of writing |
|---|---|---|
| `ragas` | `>=0.4,<0.5` | 0.4.3 |
| `deepeval` | `>=4.1,<5` | 4.1.0 |
| `beir` | `>=2.2,<3` | 2.2.0 |

These were installed into a scratch venv and actually imported/inspected
while writing this harness (not assumed from training-era docs) — see
"Verified against a live install" below for what that surfaced.

## Install

```bash
cd tools/prometheus-eval
pip install -e ".[dev]"
```

**Known-fragile bit, worth pinning explicitly:** as installed on
2026-07-14, `ragas==0.4.3`'s own dependency resolution pulls in
`langchain-community==0.4.2`, and `ragas` eagerly imports
`langchain_community.chat_models.vertexai.ChatVertexAI` at package import
time -- a class that no longer exists in `langchain-community>=0.4`
(it moved to the separate `langchain-google-vertexai` package). This
breaks `import ragas` entirely, independent of anything in this
harness. `pyproject.toml` pins `langchain-community<0.4` (verified locally
against `0.3.31`) specifically to work around this -- if a future ragas
patch release fixes its own import, that pin can be relaxed, but removing
it without checking will reintroduce the `ModuleNotFoundError` in CI.

## Commands

```bash
# Schema-check the golden set. No network, no LLM, no ragas/deepeval install needed.
prometheus-eval validate

# Run RAGAS + DeepEval against a live UAR server (needs UAR_LLM__API_KEY-equivalent
# credentials configured for whatever judge_model config/judge.yaml pins).
prometheus-eval run --base-url http://127.0.0.1:1906 --out /tmp/rag-eval-current.json

# Compare a fresh run against the committed baseline; exits non-zero on regression.
prometheus-eval gate \
  --baseline evals/results/rag-golden-set.baseline.json \
  --current /tmp/rag-eval-current.json
```

## Architecture

```
evals/rag-golden-set/*.jsonl  (frozen ground truth: question + reference_contexts + reference)
            │
            ▼
   UarClient.run()  ──► live UAR server: /api/knowledge/{id}/search (retrieval)
            │                            /v1/chat/completions       (generation)
            ▼
   {retrieved_contexts, response}  per item
            │
     ┌──────┴──────┐
     ▼             ▼
ragas_runner   deepeval_runner   (cross-validating "second opinion" per analysis.md 4.4)
     │             │
     └──────┬──────┘
            ▼
     mean_scores()  ──►  gate.compare()  ──►  pass/fail (CI exit code)
```

`gate.py` is intentionally the only module with zero ragas/deepeval/beir
dependency — the regression math is pure and unit-tested
(`tests/test_gate.py`) independent of whether the heavier libraries are
even installed.

## The frozen LLM judge

Per the done-condition ("LLM judge prompt frozen; model + temperature
pinned"): [`prometheus_eval/config/judge.yaml`](prometheus_eval/config/judge.yaml)
pins the judge model, temperature, and a `prompt_version`; the prompt text
itself lives in [`prometheus_eval/config/judge_prompt.md`](prometheus_eval/config/judge_prompt.md)
and is used for `prometheus-eval`'s own holistic rubric metric. RAGAS' and
DeepEval's *built-in* metrics (faithfulness, context_precision, etc.) each
have their own internal judge prompts baked into the library — those are
pinned by pinning the library version above, not by this repo's
`judge_prompt.md`. Changing `model`, `temperature`, or `prompt_version`
invalidates the committed baseline (`evals/results/rag-golden-set.baseline.json`);
bump `prompt_version` and re-seed the baseline in the same PR.

## Why two judges (RAGAS + DeepEval)

Both are LLM-as-judge frameworks and both are individually noisy
(analysis.md's own risk note). Running both and reporting all 8 metrics
(4 RAGAS + 4 DeepEval) means a regression that shows up in only one
framework is a weaker signal than one that shows up in both — the gate
still fires per-metric (any single metric's >2-point regression fails),
but a human triaging a failure can see immediately whether it's isolated
to one judge (possibly judge noise) or corroborated by both (likely real).

## Verified against a live install (2026-07-14)

What was actually checked by installing both packages into a scratch venv
and inspecting live objects, vs. what is structurally wired but unverified:

**Verified:**
- `ragas==0.4.3` imports (after the `langchain-community` pin above);
  `ragas.metrics.collections.{Faithfulness, AnswerRelevancy,
  ContextPrecisionWithReference, ContextRecall}` exist and are the
  non-deprecated import path (the legacy `ragas.metrics.Faithfulness` etc.
  still work but emit a `DeprecationWarning` pointing at `collections`).
- `ragas.evaluate()`'s real keyword signature (`dataset`, `metrics`, `llm`,
  `embeddings`, `column_map`, ...).
- `deepeval==4.1.0`'s `LLMTestCase` real field names (`input`,
  `actual_output`, `expected_output`, `context`, `retrieval_context`, ...)
  and `deepeval.models.GPTModel(model=..., temperature=...)` for pinning
  judge temperature (metric constructors themselves take a `model=`
  object, not a bare temperature kwarg).
- `evals/rag-golden-set/golden-set.seed.jsonl` loads and validates against
  both the pydantic model (`schema.py`) and the JSON Schema file
  (`schema.json`) — `tests/test_schema.py` passes.
- `gate.py`'s regression math — `tests/test_gate.py` passes (12/12 tests,
  pure stdlib, no ragas/deepeval install needed).

**Not verified (needs a live LLM API key and a running UAR server, neither
available in this change's environment):**
- An actual end-to-end `prometheus-eval run` against a real UAR server —
  `uar_client.py`'s request/response shapes were checked against the Rust
  source (`src/uar/api/knowledge.rs`'s `SearchResponse`/`SearchResult`),
  not against a live server response.
- `ragas.evaluate()`'s actual result-dataframe column names for the exact
  metric instances constructed here (`ragas_runner.py` resolves columns by
  substring match specifically because this was not confirmed end-to-end —
  see the code comment there).
- Real RAGAS/DeepEval scores on the seed golden set (needs a judge model
  API key) — so no baseline has been seeded yet; `prometheus-eval gate`
  will fail loudly ("blocked until seeded") exactly like the existing
  `evals/starter.yaml` Tier-2 gate does until an operator runs the seed
  step (see `.github/workflows/rag-eval.yml`).

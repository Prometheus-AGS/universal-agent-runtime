# prometheus-eval

> **Current authority:** [Run operations guide](/docs/operations/runs). This
> local evaluation tool reports the named dataset, judge, model, and profile; it
> does not produce a cross-profile or runtime-level verdict.

`prometheus-eval` is UAR's Python RAG evaluation tool. It loads the frozen seed
records in [`evals/rag-golden-set/`](../../evals/rag-golden-set/), asks a running
UAR server for retrieval and generation results, evaluates them with RAGAS and
DeepEval, and compares reviewed result files. Its separate `beir_bench.py`
module measures retrieval against selected BEIR datasets.

## Install from source

```bash
cd tools/prometheus-eval
python -m venv .venv
. .venv/bin/activate
python -m pip install -e ".[dev]"
```

The manifest constrains `ragas` to `>=0.4,<0.5`, `deepeval` to `>=4.1,<5`,
and `beir` to `>=2.2,<3`. It also keeps `langchain-community<0.4` because the
accepted 2026-07-14 local dependency check found the selected RAGAS version
imported a symbol removed from later `langchain-community` releases. Recheck
that constraint against the current lock and upstream packages before changing
it; this README does not claim the old observation remains true forever.

## Commands

```bash
# Validate the checked-in JSONL input without contacting a model.
prometheus-eval validate

# Run against a configured UAR server and real judge/model prerequisites.
prometheus-eval run \
  --base-url http://127.0.0.1:1906 \
  --out /tmp/rag-eval-current.json

# Compare two reviewed result records.
prometheus-eval gate \
  --baseline evals/results/rag-golden-set.baseline.json \
  --current /tmp/rag-eval-current.json
```

Validation proves input shape only. A meaningful evaluation must record the UAR
source SHA, server profile, corpus, embedding backend, generation model, judge
model, prompt version, credentials boundary, command, and observed results.
Missing real-model prerequisites leave the quality claim unverified; do not
substitute a recorded response.

## Judge and baseline boundary

`prometheus_eval/config/judge.yaml` pins the repository-owned judge model,
temperature, and prompt version. The prompt is
`prometheus_eval/config/judge_prompt.md`. Changing any of them invalidates an
existing baseline and requires a reviewed new record. RAGAS and DeepEval also
carry library-owned prompts whose behavior is coupled to their pinned versions.

The checked-in seed dataset currently has no accepted RAG evaluation baseline.
Therefore `gate` cannot establish a non-regression claim until an operator runs
genuine retrieval/generation and accepts the result. GitHub Actions are
deployment-only and do not run, seed, or publish these evaluations.

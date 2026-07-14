## Why

UAR's RAG pipeline (`src/uar/rag/`) has no automated quality evaluation —
retrieval and generation quality can silently regress across changes with
no signal until an operator notices bad answers in production. The
2026-07 grade-A plan's Order 7 (§4 RAG) calls for a frozen golden-set
evaluation harness (RAGAS + DeepEval, cross-validating) gated on every
PR, plus a monthly public BEIR retrieval benchmark, so RAG regressions
are caught mechanically instead of anecdotally.

## What Changes

- **`evals/rag-golden-set/`**: a new, RAG-specific golden-set directory
  (separate format from the existing `evals/*.yaml` generic-suite
  harness — see `evals/rag-golden-set/README.md` for why). Ships:
  - `schema.json` — JSON Schema (draft 2020-12) for one golden item.
  - `golden-set.seed.jsonl` — **14 real, hand-verified seed items**
    across all 5 stratification categories (`config-system`,
    `rag-pipeline`, `governance-licensing`, `model-routing`,
    `api-usage`), grounded in this repo's own `example.config.yaml`,
    `src/uar/rag/*.rs` doc comments, `CONTRIBUTING.md`, and
    `docs/ARCHITECTURE.md` / `CLAUDE.md`.
  - `README.md` — format, freeze discipline, and an explicit
    **scope note that this is a seed set, not the frozen 300-item set**
    the original done-condition describes (see "Out of scope" below).
- **`tools/prometheus-eval/`**: a new Python package (`pyproject.toml`,
  `hatchling` build, matching `sdks/python/`'s conventions) — the
  "custom UAR wrapper" the done-condition names. Provides:
  - `schema.py` — pydantic model + JSON-Schema validation for golden
    items.
  - `uar_client.py` — thin HTTP client hitting the real running server's
    `/api/knowledge/{id}/search` (retrieval) and `/v1/chat/completions`
    (generation) to produce the trace a golden item doesn't fix ahead of
    time.
  - `ragas_runner.py` / `deepeval_runner.py` — wraps RAGAS' and
    DeepEval's faithfulness / answer-relevancy / context-precision /
    context-recall metrics (analysis.md 4.4: "cross-validating").
  - `config/judge.yaml` + `config/judge_prompt.md` — the frozen LLM
    judge: pinned model, temperature, prompt version.
  - `gate.py` — dependency-free regression comparison (>2-point
    per-metric regression fails), unit-tested independent of
    ragas/deepeval being installed.
  - `beir_bench.py` — BEIR corpus ingestion into a scratch UAR knowledge
    base + retrieval-only benchmark (NDCG/Recall/Precision via BEIR's
    own evaluator), for the monthly public benchmark.
  - `cli.py` — `prometheus-eval validate|run|gate` entry point.
- **`.github/workflows/rag-eval.yml`**: runs on every PR touching
  `evals/rag-golden-set/`, `tools/prometheus-eval/`, or `src/uar/rag/`.
  Two-tier like the existing `evals/` pattern: a keyless structural job
  (schema validation + `gate.py` unit tests) runs unconditionally; a
  real-model job builds the server, runs `prometheus-eval run`, and
  gates on regression — skipping gracefully (not failing) without a
  `UAR_EVAL_JUDGE_API_KEY` secret, and failing loudly ("blocked until
  seeded") if the key is present but no baseline is committed yet.
- **`.github/workflows/rag-benchmark-monthly.yml`**: scheduled (1st of
  the month) BEIR run across `scifact`/`nfcorpus`/`fiqa`/`hotpotqa`,
  opening a PR with the resulting `docs/rag-benchmark/YYYY-MM.json`.
- **`docs/rag-benchmark/README.md`**: report format + status placeholder
  (no run published yet — see "Out of scope").

## Capabilities

### New Capabilities

- `rag-evaluation-suite`: the golden-set format, the `prometheus-eval`
  harness, the CI regression gate, the frozen judge configuration, and
  the monthly BEIR benchmark mechanism.

## Impact

- **New Python dependency surface**: `tools/prometheus-eval/pyproject.toml`
  pins `ragas>=0.4,<0.5` (0.4.3 at time of writing), `deepeval>=4.1,<5`
  (4.1.0), `beir>=2.2,<3` (2.2.0), and — a genuine upstream compatibility
  fix found while verifying this against a live install —
  `langchain-community<0.4` (ragas 0.4.3's own resolved default,
  `langchain-community==0.4.2`, breaks `import ragas` with a
  `ModuleNotFoundError` because ragas eagerly imports a class removed
  from that package; see `tools/prometheus-eval/README.md`).
- **Two new CI workflows.** `rag-eval.yml`'s real-model job needs a new
  repository secret, `UAR_EVAL_JUDGE_API_KEY`, to activate (analogous to
  `eval-nightly.yml`'s `UAR_LLM__API_KEY` for the existing generic-suite
  harness — kept separate so the RAG judge model can be pinned/rotated
  independently of the runtime's own default LLM).
- **No baseline seeded yet.** Both `evals/rag-golden-set.baseline.json`
  and the first `docs/rag-benchmark/*.json` require a live judge-model
  API key / running server to produce, neither of which is available in
  this change's environment — this is operator follow-up work exactly
  like the existing `evals/starter.yaml` baseline-seeding step.
- **No changes to `src/uar/rag/` or any Rust code.** This change is
  pure new infrastructure (Python harness + CI + docs); it does not
  modify the retrieval pipeline, chunking, or verification logic it
  evaluates.

## Out of scope

- **The full 300-item golden set.** Curating 300 real, hand-verified
  question/context/ground-truth triples is a substantial, ongoing
  content-curation effort (analysis.md 4.5 estimates ~25 human-equivalent
  hours), not a one-pass coding task — and fabricating placeholder items
  would produce a set that silently corrupts every future regression
  comparison, defeating the entire point of a golden set. This change
  ships 14 real seed items (at least 2 per stratification category) plus
  the complete infrastructure to grow the set; reaching 300 items is
  deferred, tracked as follow-up work belonging to whoever owns RAG
  product quality (see `tasks.md`).
- **Seeding the CI regression baseline.** Needs a live
  `UAR_EVAL_JUDGE_API_KEY` and a running server; this change wires the
  `workflow_dispatch` `update_baseline` path but does not run it.
- **Publishing the first monthly BEIR report.** Same reason — needs a
  live server; the workflow is wired and will produce the first report
  on its next scheduled or manually-dispatched run.
- **Change 13 (`rag-citation-stream`) coordination.** The grade-A plan's
  dependency graph lists a loose arrow between this change and Change
  13; auditing both done-conditions found no actual code dependency —
  Change 13 touches `src/uar/rag/` citation-marker types and a React
  hover panel, this change is a Python evaluation harness that doesn't
  reference citation markers. No coordination was needed for this
  change to land independently.

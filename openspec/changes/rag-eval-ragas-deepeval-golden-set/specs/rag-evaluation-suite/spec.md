# RAG evaluation suite

## Purpose

Define the golden-set format, the `prometheus-eval` harness, the CI
regression gate, and the monthly BEIR benchmark mechanism used to
evaluate UAR's RAG pipeline (`src/uar/rag/`) quality. Distinct from the
generic LLM-suite eval harness documented in `evals/README.md`, which
uses a different dataset shape and answers a different question (general
input/output grading vs. retrieval-specific faithfulness/precision/recall).

## ADDED Requirements

### Requirement: Golden-set items validate against a frozen schema
Every item in `evals/rag-golden-set/*.jsonl` MUST validate against
`evals/rag-golden-set/schema.json` (JSON Schema draft 2020-12) and MUST
be loadable via the `prometheus_eval.schema.GoldenItem` pydantic model
with no reshaping. Field names MUST mirror ragas' `EvaluationDataset`
column names (`user_input`, `reference_contexts`, `reference`) so items
load into ragas with no transformation step.

#### Scenario: A malformed item is added to the golden set
- **WHEN** a JSONL line in `evals/rag-golden-set/golden-set.seed.jsonl`
  is missing a required field or uses an invalid `category`/`difficulty`
  enum value
- **THEN** `prometheus-eval validate` exits non-zero and reports the
  specific line and validation error
- **AND** the `structural` CI job in `.github/workflows/rag-eval.yml`
  fails on every PR touching that file

#### Scenario: A new item is loaded by the harness
- **WHEN** `prometheus_eval.schema.load_golden_set()` reads
  `evals/rag-golden-set/golden-set.seed.jsonl`
- **THEN** every item's `id` is unique within the file
- **AND** each item's `reference_contexts` has at least one non-empty
  entry

### Requirement: Golden-set items are append-only after freeze
An item's `reference`, `reference_contexts`, or `user_input` MUST NOT be
edited in place after its `frozen_at` date. A correction to an existing
item MUST be made by retiring the old `id` (documented in a
`CHANGELOG.md` entry, once one exists) and adding a new item with a new
`id`, so the CI regression gate always compares against a stable,
auditable ground truth rather than a silently-moving target.

#### Scenario: A ground-truth answer turns out to be wrong
- **WHEN** a curator discovers `rag-config-001`'s reference answer no
  longer matches `example.config.yaml` (the config changed)
- **THEN** the fix is a new item (e.g. `rag-config-015`) with a fresh
  `frozen_at` date, not an edit to `rag-config-001` in place

### Requirement: The golden set is stratified across 5 categories
Every item's `category` MUST be one of `config-system`, `rag-pipeline`,
`governance-licensing`, `model-routing`, `api-usage`. The full golden set
(target: 300 items, tracked as follow-up work — see `proposal.md`
"Out of scope") SHOULD be roughly balanced across these 5 categories;
the seed set committed in this change MUST have at least 2 items in
every category.

#### Scenario: The seed set is validated for category coverage
- **WHEN** `prometheus-eval validate` runs against
  `evals/rag-golden-set/golden-set.seed.jsonl`
- **THEN** it reports the distinct set of categories present
- **AND** all 5 categories appear at least once

### Requirement: prometheus-eval runs RAGAS and DeepEval as cross-validating judges
The `prometheus-eval` harness (`tools/prometheus-eval/`) MUST run both
RAGAS (`faithfulness`, `answer_relevancy`, `context_precision`,
`context_recall`) and DeepEval (`FaithfulnessMetric`,
`AnswerRelevancyMetric`, `ContextualPrecisionMetric`,
`ContextualRecallMetric`) against every golden-set item, producing 8
named metric scores per run.

#### Scenario: A full run is executed
- **WHEN** an operator runs `prometheus-eval run --out results.json`
  against a live UAR server
- **THEN** `results.json` contains mean scores for all 8 metrics
  (`ragas_faithfulness`, `ragas_answer_relevancy`,
  `ragas_context_precision`, `ragas_context_recall`,
  `deepeval_faithfulness`, `deepeval_answer_relevancy`,
  `deepeval_contextual_precision`, `deepeval_contextual_recall`)

### Requirement: The LLM judge is frozen — model, temperature, and prompt version pinned
`tools/prometheus-eval/prometheus_eval/config/judge.yaml` MUST declare a
specific `model`, `temperature`, and `prompt_version`, and
`config/judge_prompt.md` MUST contain the exact frozen prompt text for
`prompt_version`. Changing any of `model`, `temperature`, or the prompt
text MUST be accompanied by bumping `prompt_version` and re-seeding the
committed baseline in the same change, because a judge-configuration
change invalidates the previous baseline's comparability.

#### Scenario: The judge model is changed
- **WHEN** an operator changes `config/judge.yaml`'s `model` field
- **THEN** `prompt_version` MUST also change in the same commit
- **AND** the committed baseline (`evals/results/rag-golden-set.baseline.json`)
  MUST be re-seeded before the regression gate is trusted again

### Requirement: CI gates every PR on a >2-point per-metric regression
`.github/workflows/rag-eval.yml` MUST run on every pull request touching
`evals/rag-golden-set/`, `tools/prometheus-eval/`, or `src/uar/rag/`.
Its `structural` job (schema validation + `gate.py` unit tests) MUST run
without any API key or network access to an LLM. Its `real-model` job
MUST compare the fresh run's per-metric mean scores against the
committed baseline and fail the job if any single metric regresses by
more than 2.0 points on a 0-100 scale.

#### Scenario: A PR degrades retrieval quality
- **WHEN** a PR's `real-model` job runs `prometheus-eval run` and
  `context_precision`'s mean score drops from 0.85 to 0.81 (4 points)
- **THEN** `prometheus-eval gate` exits non-zero
- **AND** the `real-model` job fails, blocking merge

#### Scenario: No judge API key is configured
- **WHEN** the `UAR_EVAL_JUDGE_API_KEY` repository secret is absent
  (e.g. a fork)
- **THEN** the `real-model` job's guard step skips every subsequent
  step and the job succeeds (does not fail the PR)
- **AND** the `structural` job still runs and still gates on schema
  validity

#### Scenario: A judge API key exists but no baseline is committed
- **WHEN** `UAR_EVAL_JUDGE_API_KEY` is configured but
  `evals/results/rag-golden-set.baseline.json` does not exist
- **THEN** the gate step fails loudly with a "blocked until seeded"
  message rather than passing silently
- **AND** an operator seeds the baseline via `workflow_dispatch` with
  `update_baseline: true`, then commits the resulting file deliberately

### Requirement: The regression gate is testable without RAGAS/DeepEval installed
`prometheus_eval/gate.py`'s comparison logic MUST have zero dependency
on `ragas`, `deepeval`, or `beir`, so its unit tests
(`tools/prometheus-eval/tests/test_gate.py`) run in the keyless
`structural` CI job.

#### Scenario: gate.py unit tests run without the heavy dependencies installed
- **WHEN** `tools/prometheus-eval/tests/test_gate.py` is run in an
  environment where `ragas`/`deepeval`/`beir` are not installed
- **THEN** every test passes (pure `dict[str, float]` comparison math)

### Requirement: A monthly BEIR retrieval benchmark is published
`.github/workflows/rag-benchmark-monthly.yml` MUST run on a monthly
schedule (and via manual `workflow_dispatch`), ingest the `scifact`,
`nfcorpus`, `fiqa`, and `hotpotqa` BEIR corpora into a scratch UAR
knowledge base via `prometheus_eval.beir_bench`, run BEIR's own
NDCG/Recall/Precision evaluator against UAR's real
`/api/knowledge/{id}/search` retrieval path, and open a pull request
publishing the result as `docs/rag-benchmark/YYYY-MM.json`. This
benchmark is retrieval-only (no LLM judge, no generation) and is
report-only — it does not gate CI.

#### Scenario: The monthly benchmark runs
- **WHEN** the scheduled workflow fires on the 1st of the month
- **THEN** a new `docs/rag-benchmark/YYYY-MM.json` file is proposed via
  pull request, containing NDCG/Recall/Precision scores for all 4
  datasets
- **AND** a pre-existing report for that month is never overwritten
  (`beir_bench.write_report` refuses and raises if the file exists)

#### Scenario: An operator triggers a manual benchmark run
- **WHEN** an operator dispatches `rag-benchmark-monthly.yml` manually
- **THEN** the same ingestion + benchmark + publish flow runs
  out-of-schedule

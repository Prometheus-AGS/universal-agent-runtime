"""`prometheus-eval` CLI: validate | run | gate.

    prometheus-eval validate                        # schema-check the golden set (no network, no LLM)
    prometheus-eval run --out results.json           # run RAGAS + DeepEval against a live UAR server
    prometheus-eval gate --baseline B.json --current C.json   # pure regression check

`beir` is intentionally not a CLI subcommand yet — the monthly BEIR run
needs each corpus ingested into a UAR knowledge base first (operator/CI
glue), so it is driven directly via `prometheus_eval.beir_bench` from the
scheduled workflow rather than through this generic CLI. See
`.github/workflows/rag-benchmark-monthly.yml`.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import yaml

from .gate import compare, load_scores, mean_scores
from .schema import load_golden_set, validate_against_json_schema

CONFIG_DIR = Path(__file__).parent / "config"


def _load_judge_config() -> dict:
    with (CONFIG_DIR / "judge.yaml").open(encoding="utf-8") as f:
        return yaml.safe_load(f)


def cmd_validate(args: argparse.Namespace) -> int:
    path = Path(args.golden_set)
    try:
        items = load_golden_set(path)
        validate_against_json_schema(path)
    except Exception as exc:  # noqa: BLE001 - CLI boundary, report and exit non-zero
        print(f"INVALID: {exc}", file=sys.stderr)
        return 1
    categories = sorted({item.category for item in items})
    print(f"OK: {len(items)} items, categories={categories}")
    return 0


def cmd_run(args: argparse.Namespace) -> int:
    from .deepeval_runner import run_deepeval
    from .ragas_runner import run_ragas
    from .uar_client import UarClient

    items = load_golden_set(Path(args.golden_set))
    judge = _load_judge_config()
    model = args.judge_model or judge["model"]
    temperature = judge["temperature"]

    with UarClient(base_url=args.base_url, model=args.answer_model) as client:
        traces = {item.id: client.run(item.user_input) for item in items}

    ragas_scores = run_ragas(items, traces, judge_model=model, temperature=temperature)
    deepeval_scores = run_deepeval(items, traces, judge_model=model, temperature=temperature)

    per_item = []
    for r, d in zip(ragas_scores, deepeval_scores, strict=True):
        assert r.item_id == d.item_id
        per_item.append(
            {
                "ragas_faithfulness": r.faithfulness,
                "ragas_answer_relevancy": r.answer_relevancy,
                "ragas_context_precision": r.context_precision,
                "ragas_context_recall": r.context_recall,
                "deepeval_faithfulness": d.faithfulness,
                "deepeval_answer_relevancy": d.answer_relevancy,
                "deepeval_contextual_precision": d.contextual_precision,
                "deepeval_contextual_recall": d.contextual_recall,
            }
        )
    means = mean_scores(per_item)

    out_path = Path(args.out)
    out_path.write_text(json.dumps(means, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"Wrote {len(means)} metric means to {out_path}")
    return 0


def cmd_gate(args: argparse.Namespace) -> int:
    judge = _load_judge_config()
    max_points = args.max_regression_points or judge["regression_gate"]["max_regression_points"]

    baseline = load_scores(Path(args.baseline))
    current = load_scores(Path(args.current))
    result = compare(baseline, current, max_regression_points=max_points)
    print(result.render())
    return 0 if result.passed else 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="prometheus-eval")
    sub = parser.add_subparsers(dest="command", required=True)

    p_validate = sub.add_parser("validate", help="Schema-check the golden set (no network).")
    p_validate.add_argument("--golden-set", default=str(Path("evals/rag-golden-set/golden-set.seed.jsonl")))
    p_validate.set_defaults(func=cmd_validate)

    p_run = sub.add_parser("run", help="Run RAGAS + DeepEval against a live UAR server.")
    p_run.add_argument("--golden-set", default=str(Path("evals/rag-golden-set/golden-set.seed.jsonl")))
    p_run.add_argument("--base-url", default="http://127.0.0.1:1906")
    p_run.add_argument("--answer-model", default="openai/gpt-4o-mini", help="Model UAR itself uses to answer.")
    p_run.add_argument("--judge-model", default=None, help="Override config/judge.yaml's pinned judge model.")
    p_run.add_argument("--out", required=True)
    p_run.set_defaults(func=cmd_run)

    p_gate = sub.add_parser("gate", help="Compare a fresh run's scores against a committed baseline.")
    p_gate.add_argument("--baseline", required=True)
    p_gate.add_argument("--current", required=True)
    p_gate.add_argument("--max-regression-points", type=float, default=None)
    p_gate.set_defaults(func=cmd_gate)

    return parser


def main(argv: list[str] | None = None) -> None:
    parser = build_parser()
    args = parser.parse_args(argv)
    sys.exit(args.func(args))


if __name__ == "__main__":
    main()

"""Regression gate: compares a fresh run's per-metric mean scores against
a committed baseline and fails on a > N-point regression.

Deliberately dependency-free (stdlib only) so it can be unit-tested
without installing ragas/deepeval, and so `prometheus-eval gate` can run
even in environments where the heavier eval itself was skipped.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

DEFAULT_MAX_REGRESSION_POINTS = 2.0
DEFAULT_SCALE = 100


@dataclass
class MetricRegression:
    metric: str
    baseline: float
    current: float

    @property
    def delta_points(self) -> float:
        """Positive = regression (current is worse than baseline)."""
        return (self.baseline - self.current) * DEFAULT_SCALE

    def regressed(self, max_regression_points: float) -> bool:
        return self.delta_points > max_regression_points


@dataclass
class GateResult:
    regressions: list[MetricRegression]
    max_regression_points: float

    @property
    def failed_metrics(self) -> list[MetricRegression]:
        return [r for r in self.regressions if r.regressed(self.max_regression_points)]

    @property
    def passed(self) -> bool:
        return len(self.failed_metrics) == 0

    def render(self) -> str:
        lines = [f"Regression gate (max {self.max_regression_points} points, 0-{DEFAULT_SCALE} scale):"]
        for r in self.regressions:
            marker = "FAIL" if r.regressed(self.max_regression_points) else "ok"
            lines.append(
                f"  [{marker}] {r.metric}: baseline={r.baseline:.4f} current={r.current:.4f} "
                f"delta={r.delta_points:+.2f} pts"
            )
        lines.append("PASS" if self.passed else "FAIL")
        return "\n".join(lines)


def load_scores(path: Path) -> dict[str, float]:
    """Load a `{metric: mean_score}` JSON file (0.0-1.0 scale per metric)."""
    with path.open(encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, dict) or not data:
        raise ValueError(f"{path}: expected a non-empty JSON object of {{metric: score}}")
    return {str(k): float(v) for k, v in data.items()}


def compare(
    baseline: dict[str, float],
    current: dict[str, float],
    max_regression_points: float = DEFAULT_MAX_REGRESSION_POINTS,
) -> GateResult:
    """Compare `current` scores to `baseline`. Metrics present in only one
    side are reported (delta against 0.0) rather than silently ignored —
    a metric disappearing between runs is itself worth flagging.
    """
    all_metrics = sorted(set(baseline) | set(current))
    regressions = [
        MetricRegression(metric=m, baseline=baseline.get(m, 0.0), current=current.get(m, 0.0))
        for m in all_metrics
    ]
    return GateResult(regressions=regressions, max_regression_points=max_regression_points)


def mean_scores(per_item_scores: list[dict[str, float]]) -> dict[str, float]:
    """Average a list of per-item `{metric: score}` dicts into one
    `{metric: mean}` dict, e.g. for writing a fresh baseline file.
    """
    if not per_item_scores:
        raise ValueError("per_item_scores is empty")
    totals: dict[str, float] = {}
    counts: dict[str, int] = {}
    for row in per_item_scores:
        for metric, value in row.items():
            totals[metric] = totals.get(metric, 0.0) + value
            counts[metric] = counts.get(metric, 0) + 1
    return {m: totals[m] / counts[m] for m in totals}

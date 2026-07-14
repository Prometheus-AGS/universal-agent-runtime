"""Unit tests for the regression gate. Pure stdlib — no ragas/deepeval
install required, so these run in the keyless CI job."""

from prometheus_eval.gate import compare, mean_scores


def test_no_regression_passes():
    baseline = {"faithfulness": 0.80, "context_recall": 0.75}
    current = {"faithfulness": 0.80, "context_recall": 0.76}
    result = compare(baseline, current, max_regression_points=2.0)
    assert result.passed


def test_small_regression_within_tolerance_passes():
    baseline = {"faithfulness": 0.80}
    current = {"faithfulness": 0.785}  # 1.5-point drop
    result = compare(baseline, current, max_regression_points=2.0)
    assert result.passed


def test_regression_over_threshold_fails():
    baseline = {"faithfulness": 0.80}
    current = {"faithfulness": 0.77}  # 3-point drop
    result = compare(baseline, current, max_regression_points=2.0)
    assert not result.passed
    assert result.failed_metrics[0].metric == "faithfulness"


def test_improvement_never_fails():
    baseline = {"faithfulness": 0.70}
    current = {"faithfulness": 0.95}
    result = compare(baseline, current, max_regression_points=2.0)
    assert result.passed


def test_metric_missing_from_current_counts_as_full_regression():
    baseline = {"faithfulness": 0.80}
    current: dict[str, float] = {}
    result = compare(baseline, current, max_regression_points=2.0)
    assert not result.passed


def test_mean_scores_averages_per_item_dicts():
    rows = [
        {"faithfulness": 1.0, "recall": 0.5},
        {"faithfulness": 0.5, "recall": 1.0},
    ]
    means = mean_scores(rows)
    assert means == {"faithfulness": 0.75, "recall": 0.75}


def test_mean_scores_rejects_empty_input():
    import pytest

    with pytest.raises(ValueError):
        mean_scores([])

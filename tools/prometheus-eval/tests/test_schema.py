"""Unit tests for golden-set loading/validation against the seed file
committed in `evals/rag-golden-set/`. No network, no LLM."""

from prometheus_eval.schema import SEED_SET_PATH, load_golden_set, validate_against_json_schema


def test_seed_set_loads_and_validates():
    items = load_golden_set(SEED_SET_PATH)
    assert len(items) >= 5


def test_seed_set_ids_are_unique():
    items = load_golden_set(SEED_SET_PATH)
    ids = [item.id for item in items]
    assert len(ids) == len(set(ids))


def test_seed_set_covers_multiple_categories():
    items = load_golden_set(SEED_SET_PATH)
    categories = {item.category for item in items}
    assert len(categories) >= 3


def test_seed_set_passes_json_schema_validation():
    # Raises on failure; a clean return is the assertion.
    validate_against_json_schema(SEED_SET_PATH)


def test_seed_set_every_item_has_nonempty_reference_contexts():
    items = load_golden_set(SEED_SET_PATH)
    for item in items:
        assert len(item.reference_contexts) >= 1
        assert all(ctx.strip() for ctx in item.reference_contexts)

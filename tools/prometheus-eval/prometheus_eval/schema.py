"""Typed model for a golden-set item plus a JSON-Schema validating loader.

The pydantic model's field names mirror ragas' ``EvaluationDataset`` column
names (``user_input``, ``reference_contexts``, ``reference``) so a
``GoldenItem`` converts to a ragas row with no reshaping — see
``evals/rag-golden-set/README.md``.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Literal

from pydantic import BaseModel, Field

REPO_ROOT = Path(__file__).resolve().parents[3]
GOLDEN_SET_DIR = REPO_ROOT / "evals" / "rag-golden-set"
SCHEMA_PATH = GOLDEN_SET_DIR / "schema.json"
SEED_SET_PATH = GOLDEN_SET_DIR / "golden-set.seed.jsonl"

Category = Literal[
    "config-system",
    "rag-pipeline",
    "governance-licensing",
    "model-routing",
    "api-usage",
]
Difficulty = Literal["easy", "medium", "hard"]


class GoldenItem(BaseModel):
    """One frozen golden-set item. See ``evals/rag-golden-set/schema.json``."""

    id: str
    category: Category
    user_input: str
    reference_contexts: list[str] = Field(min_length=1)
    reference: str
    source: str
    difficulty: Difficulty
    notes: str | None = None
    frozen_at: str


def load_golden_set(path: Path | None = None) -> list[GoldenItem]:
    """Load and validate every item in the golden-set JSONL file.

    Raises ``pydantic.ValidationError`` (surfaced to the caller) on the
    first malformed item — golden-set integrity is a hard requirement, not
    something to silently skip past.
    """
    path = path or SEED_SET_PATH
    items: list[GoldenItem] = []
    seen_ids: set[str] = set()
    with path.open(encoding="utf-8") as f:
        for line_no, line in enumerate(f, start=1):
            line = line.strip()
            if not line:
                continue
            raw = json.loads(line)
            item = GoldenItem.model_validate(raw)
            if item.id in seen_ids:
                raise ValueError(f"{path}:{line_no}: duplicate id {item.id!r}")
            seen_ids.add(item.id)
            items.append(item)
    if not items:
        raise ValueError(f"{path}: golden set is empty")
    return items


def validate_against_json_schema(path: Path | None = None) -> None:
    """Belt-and-suspenders validation against the checked-in JSON Schema
    file (draft 2020-12), independent of the pydantic model above, so the
    two never silently drift apart undetected.
    """
    import jsonschema

    path = path or SEED_SET_PATH
    with SCHEMA_PATH.open(encoding="utf-8") as f:
        schema = json.load(f)
    validator = jsonschema.Draft202012Validator(schema)
    with path.open(encoding="utf-8") as f:
        for line_no, line in enumerate(f, start=1):
            line = line.strip()
            if not line:
                continue
            raw = json.loads(line)
            errors = sorted(validator.iter_errors(raw), key=lambda e: e.path)
            if errors:
                messages = "; ".join(e.message for e in errors)
                raise ValueError(f"{path}:{line_no} (id={raw.get('id')}): {messages}")

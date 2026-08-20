# Decisions

## 2026-08-20 — deterministic regeneration is an input, not the final answer

Two independent regenerations agreed at `0a7145d6…`, but direct HEAD comparison
showed three optional common snapshot-body movements. Retain the HEAD bodies for
project-service 8.64.0, Chromatic 16.10.0, and Storybook 10.2.13; accept the
result only because both frozen metadata and empty-tree installation still pass.

## 2026-08-20 — the operator dirty lock is evidence, not authority

Its digest differs from both clean regenerations and it carries a different
peer-context graph. Replace it with the reproduced minimum-delta candidate only
after the child plan widens the lock path into scope.

## 2026-08-20 — incomplete adapter fallback

The installed artifact-refiner adapter names canonical prompts and schemas that
are absent from both installed adapter directories. Use the canonical imported
JSON schemas and the accepted immediately preceding lock-child artifact shape;
record this limitation and do not invent missing plugin behavior.

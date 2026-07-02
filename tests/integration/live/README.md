# Live Integration Tier

Feature-correctness tests that boot the **real** UAR server and drive it over
HTTP, proving features work end-to-end through the actual production code path
(`start_server`) — not a unit-level approximation.

Introduced by the `uar-next-harness` phase (changes `proxy-integration-gate`
= CH-22 and `live-integration-baseline-coverage` = CH-22b).

## What this gate is (and is NOT)

| Gate | Checks | Where |
|---|---|---|
| **Live integration tier** (this) | *feature correctness* — does streaming / tool-loop / memory / RAG / credentials actually work end-to-end? | `tests/integration/live/`, `.github/workflows/live-integration.yml` |
| Eval harness | *model-output quality* regression vs a committed baseline | `evals/`, `eval-nightly.yml` |
| Line-coverage gate | ≥80% line coverage (cargo-llvm-cov) | `comprehensive-tests.yml` |

These are **complementary and separate**. 100% *line* coverage is explicitly
rejected (plan A2.1); this tier is about 100% *feature* coverage — every
feature change adds a case (see `MATRIX.md`).

## Two backends (one case list)

Selected by `UAR_LIVE_INTEGRATION_BACKEND` (`backend.rs`):

- **`recorded`** (default) — an in-process stub LLM (`stub_llm.rs`) serving
  canned OpenAI-compatible responses keyed by a request fingerprint.
  Deterministic; runs in CI with no network model.
- **`live`** — the operator's real OpenAI-compatible proxy at
  `http://127.0.0.1:8181/v1`. Non-deterministic; **operator-only**, never CI.

Most baseline cases assert exact fixture content and are therefore
**recorded-only** by nature (a live model won't reproduce canned text). Only
`backend_parametric_chat_smoke` is wired through the switch with
content-tolerant assertions, so it runs on both.

## Running it

```bash
# Recorded backend (default) — deterministic, what CI runs:
cargo test --test integration live::

# Live backend — against the real local proxy, with health-check + remediation:
scripts/live-integration.sh                       # fails loudly if proxy down
scripts/live-integration.sh --allow-recorded-fallback   # CI-style: fall back if no proxy
```

`scripts/live-integration.sh` health-checks the proxy first and, on failure,
prints the exact remediation (re-auth Codex; `launchctl kickstart -k
gui/501/ai.prometheus.openai-proxy`) instead of a cryptic connection error.

## Cross-tool

The runner (`scripts/live-integration.sh`) and the CI matrix-check
(`tools/live-matrix-check.sh`) are plain bash + `cargo test` with **no
tool-specific hooks**, so they behave identically from Codex, Claude Code,
Cursor, and OpenCode.

## Files

| File | Purpose |
|---|---|
| `stub_llm.rs` | In-process OpenAI-compatible stub (recorded backend) |
| `backend.rs` | `UAR_LIVE_INTEGRATION_BACKEND` selection (`resolve()`) |
| `harness.rs` | Boots the real server via `start_server`; self-sweeps temp scratch |
| `baseline_cases.rs` | The baseline feature cases (task group 2) |
| `MATRIX.md` | Per-change feature-coverage contract (CH-## → case) |

## Known gaps (see `design.md` Risk 1)

Two baseline cases are `#[ignore]`d for pre-existing product bugs (both flagged
for dedicated fixes, not test-harness issues):

- **Memory** (`memory_write_then_recall`): needs `surreal-memory`'s
  `local-embeddings` Cargo feature, not enabled in this workspace.
- **RAG** (`rag_ingest_then_retrieve`): `VectorMatcher::embed_batch` returns
  zero-vector placeholder embeddings (`model.forward()` commented out), so
  search can never match; plus a SurrealQL `type::thing` call rejected by the
  pinned SurrealDB `=3.0.5`. Re-enable once the embedding path is real.

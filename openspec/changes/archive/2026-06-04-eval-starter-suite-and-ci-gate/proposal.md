# eval-starter-suite-and-ci-gate

## Why

The harness can run a suite and gate on regression, but **no suite ships and CI never runs it** (gap G1) — so the harness is dead weight. This change (EHH1, the phase finale) makes it load-bearing via the **two-tier gate** (decision D-A): a deterministic structural test on every PR (no key, no cost) and a real-model run on a schedule.

## What Changes

- **Starter suite** `evals/starter.yaml` — a few model-agnostic cases with `expected` substrings and **declared scorers**: `non_empty` + `contains` (hard, deterministic) and an advisory `llm_judge` (D-B).
- **`evals/README.md`** — documents the two tiers and how to establish/update the committed baseline.
- **Tier 1 (PR CI)** — a `#[cfg(test)]` structural test that loads the *shipped* `evals/starter.yaml`, builds its scorers, and runs it through a recorded provider, asserting it parses + scores. Runs under the existing `test` job — no API key, deterministic. Guards the suite file from rotting.
- **Tier 2 (scheduled)** — `.github/workflows/eval-nightly.yml`: on `schedule` + `workflow_dispatch`, build the binary and run `eval run evals/starter.yaml` against the real model using the `UAR_LLM__API_KEY` secret, exiting non-zero on regression (vs a committed baseline). Skips gracefully when the secret is absent (fork-safe).

Out of scope: auto-committing baselines from CI (baseline is updated by a deliberate local run + commit, git-friendly per D1); per-PR real-model runs (cost — Tier 2 is scheduled).

## Capabilities

### Modified Capabilities
- **`eval-harness`** — delta `specs/eval-harness/spec.md`. Adds a shipped starter suite and a two-tier CI regression gate.

## Impact

- **Affected code:** new `evals/starter.yaml`, `evals/README.md`, `.github/workflows/eval-nightly.yml`; `src/uar/eval/integration_tests.rs` (Tier-1 starter-suite test). No production logic change.
- **Security (Rule 33):** the real-model key is a repo secret, used only in the scheduled job, never echoed; the job is guarded so forks/keyless runs skip rather than fail.
- **No new dependency** (Rule 27).
- **KBD workflow state:** YES — EHH1, round 3 (finale) of `eval-harness-hardening`.

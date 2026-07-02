## Why

Unit and recorded-fixture tests prove the code parses and the harness wiring
holds together, but nothing today proves that a live model conversation
actually flows end-to-end through streaming, tool calls, memory, and RAG once
a change lands. The operator needs a build-time signal that features *work*,
not just that lines are covered — using the OpenAI-compatible proxy
(`ai.prometheus.openai-proxy`, `http://127.0.0.1:8181/v1`) already running
locally for the Karpathy LLM wiki. This change must land first in Round 1 so
every subsequent change (A2A gRPC, credential store, provider health,
prompt dialects) can add its live case to the same gate instead of each
change inventing its own integration harness.

## What Changes

- New `live` integration test tier (`tests/integration/live/`) that boots the
  real Axum server and exercises baseline feature flows end-to-end: streaming
  chat in all three SSE modes (`openai`, `agui`, `dual`), an MCP tool-loop
  round-trip, agent selection via the `model` param, memory write→recall,
  RAG ingest→retrieve, and credential-chain resolution.
- **Dual-backend execution** so the same case list runs in two contexts:
  (a) **live**, against `UAR_LLM__BASE_URL=http://127.0.0.1:8181/v1`
  (model `openai/gpt-5.4-mini`) for local/pre-push runs, and (b) **recorded**,
  through the existing recorded-fixture `CompletionProvider` for CI (GitHub-
  hosted runners cannot reach the local proxy).
- `scripts/live-integration.sh`: health-checks the proxy first and fails with
  the known remediation (`Codex re-login` +
  `launchctl kickstart -k gui/501/ai.prometheus.openai-proxy`) instead of a
  cryptic 401; falls through to the recorded backend when the proxy is
  unreachable and `--allow-recorded-fallback` is set (default in CI).
- `tests/integration/live/MATRIX.md`: a living map of `CH-## → live case(s)`.
  Every later change in this phase (a2a-grpc-enable, postgres-credential-store,
  provider-health-failover, prompt-dialect-engine, and all Round 2-4 changes)
  is required to append its case here in the same PR that lands the feature —
  this is the phase's "100% feature coverage" contract, distinct from and
  additive to the existing 80% line-coverage gate in `comprehensive-tests.yml`.
- **No change** to the existing eval gate (`evals/`, Tier-1/Tier-2, OP-1) — that
  gate checks model-quality regression; this gate checks feature correctness.
  Both may share the same proxy locally but serve different purposes and stay
  separate.

## Capabilities

- **New Capabilities:**
  - `live-integration-testing` — the dual-backend live/recorded integration
    tier, proxy health-check + remediation, and the per-change feature-matrix
    contract described above.
- **Modified Capabilities:** none. This introduces a new test tier; it does
  not change the behavior of any existing runtime capability.

## Impact

- **Affected code:** new `tests/integration/live/` module tree, new
  `scripts/live-integration.sh`, a small addition to `.github/workflows/`
  (or an extension of `comprehensive-tests.yml`) to run the recorded-backend
  variant in CI, and `tests/integration/live/MATRIX.md` as a living doc.
- **Affected config:** none required to run in CI (recorded backend is the
  default there); `UAR_LLM__BASE_URL` override only needed for local live runs.
- **Dependencies:** none new — reuses the existing recorded-fixture
  `CompletionProvider` (`src/uar/eval` / eval harness) and the OpenAI-compatible
  client already in `liter-llm`.
- **KBD workflow state:** yes — `.kbd-orchestrator/phases/uar-next-harness/`
  tracks this as the first Round-1 change (plan Amendment A2); every later
  change's completion criteria include "live case added to MATRIX.md," so
  `progress.json`/`current-waypoint.*` should reflect that dependency once this
  change ships.

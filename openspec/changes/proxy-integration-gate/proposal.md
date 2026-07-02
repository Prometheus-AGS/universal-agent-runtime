## Why

Unit and recorded-fixture tests prove the code parses and the harness wiring
holds together, but nothing today proves that a live model conversation
actually flows end-to-end once a change lands. The operator needs a
build-time signal that features *work*, not just that lines are covered —
using the OpenAI-compatible proxy (`ai.prometheus.openai-proxy`,
`http://127.0.0.1:8181/v1`) already running locally for the Karpathy LLM
wiki. This change must land first in Round 1 so every subsequent change
(A2A gRPC, credential store, provider health, prompt dialects) can build
feature cases against the same interchangeable backend instead of each
change inventing its own mock/proxy plumbing.

**Scope note (revised during implementation):** this change originally
proposed the full live integration tier — backend selection *and* the 8
baseline feature cases *and* CI wiring. Building the minimal server-boot
harness needed for the baseline cases revealed a real, separately-sized gap
(no existing test in this repo boots a full `AppState`; see design.md's
Risks). That work is split into a follow-on change,
`live-integration-baseline-coverage`, so this change stays reviewable and
ships the interchangeable-backend mechanism on its own merits. This proposal
now covers only that mechanism.

## What Changes

- New `tests/integration/live/` module: an in-process OpenAI-compatible stub
  LLM server (`stub_llm.rs`) serving canned chat-completion responses
  (non-streaming and SSE streaming, plain content and tool-call fixtures)
  keyed by a request fingerprint, plus a `/v1/models` health-check endpoint.
- **Backend selection** (`backend.rs`): `UAR_LIVE_INTEGRATION_BACKEND=live|
  recorded` (default `recorded`) resolves which `base_url` a test case
  targets — the real local proxy for `live`, the in-process stub for
  `recorded` — through one shared code path (`ResolvedBackend`), not two
  implementations. Defaults to `recorded` on any unset/unrecognized value so
  this tier never silently calls a real model without explicit opt-in.
- `scripts/live-integration.sh`: health-checks the proxy first and fails with
  the known remediation (`Codex re-login` +
  `launchctl kickstart -k gui/501/ai.prometheus.openai-proxy`) instead of a
  cryptic 401; falls through to the recorded backend when the proxy is
  unreachable and `--allow-recorded-fallback` is set (default in CI).
- **No change** to the existing eval gate (`evals/`, Tier-1/Tier-2, OP-1) — that
  gate checks model-quality regression; this mechanism is feature-correctness
  infrastructure, used by its follow-on change. Both may share the same proxy
  locally but serve different purposes and stay separate.
- **Deferred to `live-integration-baseline-coverage`:** the minimal
  `AppState`-boot harness, the 8 baseline feature cases (streaming×3 modes,
  tool-loop, agent selection, memory, RAG, credentials),
  `tests/integration/live/MATRIX.md`, the CI wiring, docs, and verification.

## Capabilities

- **New Capabilities:**
  - `live-integration-testing` — the interchangeable live/recorded backend
    mechanism (stub server + selection + health-check script) described
    above. The follow-on change extends this same capability with baseline
    coverage and the per-change matrix contract.
- **Modified Capabilities:** none.

## Impact

- **Affected code:** new `tests/integration/live/{stub_llm.rs,backend.rs,mod.rs}`,
  new `scripts/live-integration.sh`, `tests/integration.rs` wiring.
- **Affected config:** none required to run in CI (recorded backend is the
  default there); `UAR_LLM__BASE_URL` override only needed for local live runs.
- **Dependencies:** none new — `axum`, `reqwest`, `serial_test` are already
  workspace dependencies.
- **KBD workflow state:** yes — `.kbd-orchestrator/phases/uar-next-harness/`
  tracks this as the first Round-1 change (plan Amendment A2); the phase plan
  is amended (A3) to insert `live-integration-baseline-coverage` immediately
  after this change, still ahead of CH-01..CH-04's matrix-row requirement.

## Context

The runtime has two existing verification layers today: (1) unit +
recorded-fixture integration tests, gated by `comprehensive-tests.yml`'s
cargo-llvm-cov 80% line-coverage threshold, and (2) the eval harness
(`evals/`), a two-tier gate that checks *model output quality* against a
committed baseline (Tier-1 keyless structural check on every PR, Tier-2
scheduled real-model run, both documented in `evals/README.md`). Neither layer
proves that a *feature* — streaming, tool calls, memory, RAG, credential
resolution — actually works end-to-end against a real model. The operator
already runs an OpenAI-compatible proxy locally (`ai.prometheus.openai-proxy`
on `127.0.0.1:8181`, backed by a Codex OAuth token) for the Karpathy LLM wiki
via `pk`; this change reuses that same endpoint as the "real model" backend
for feature-level integration testing, without adding a new provider
dependency.

CI (GitHub-hosted runners) cannot reach `127.0.0.1:8181` — it's a local
launchd service bound to loopback on the operator's machine. The design must
work in both places without forking the test suite.

## Goals / Non-Goals

**Goals:**
- One case list, two execution backends: live (local proxy) and recorded
  (existing fixture provider), so CI enforces structural/wiring correctness on
  every PR while the operator can additionally run the same cases against a
  real model before pushing.
- A per-change contract (the feature matrix) that scales through all 23
  changes in this phase without requiring a redesign each round.
- Fail loud and specific when the proxy is down (known remediation), never a
  silent skip when running locally.

**Non-Goals:**
- Replacing or subsuming the eval harness (`evals/`) — that stays the
  model-quality gate; this is the feature-correctness gate.
- Achieving 100% *line* coverage — explicitly rejected in the phase plan
  (Amendment A2.1); the existing 80% llvm-cov threshold is unchanged.
- Testing against multiple live providers — the proxy is OpenAI-compatible
  and suf1ficient for this tier; multi-provider live testing is out of scope.
- A hosted/CI-reachable proxy — out of scope; CI always uses the recorded
  backend.

## Decisions

**D1 (revised during implementation — see below): Dual-backend via a shared
`base_url`, not a trait seam.**
Both "live" and "recorded" run the exact same test cases through the exact
same code path — the real server plus a real HTTP client — differing only in
what `UAR_LLM__BASE_URL` points at. "live" points at the local
OpenAI-compatible proxy (`127.0.0.1:8181`); "recorded" points at a small
in-process Axum stub server (`tests/integration/live/stub_llm.rs`) serving
canned chat-completion responses (non-streaming and SSE) keyed by a
fingerprint of the incoming request (model + last user message + tool-schema
presence). Since `src/llm/registry.rs` already resolves an arbitrary
`base_url` per provider, there is no seam left to abstract — pointing at a
fixture server *is* the recorded backend.
**Original decision (superseded):** wrap the eval-harness's `CompletionProvider`
(`src/uar/eval/runner.rs:41`) as the recorded backend. **Rejected once
implementation started:** that trait is `async fn complete(&self, input: &str)
-> Result<String>` — no streaming, no tool calls, no HTTP representation —
so it cannot exercise the SSE-mode, tool-loop, memory, or RAG cases this spec
requires. Discovered while starting task 1.2; this revision is a
simplification (one fewer abstraction), not scope growth.
Alternative still considered and still rejected: separate `#[test]` fns per
backend — doubles maintenance and risks the two variants drifting apart.

**D2: Backend selection by environment, default to recorded.**
`UAR_LIVE_INTEGRATION_BACKEND=live|recorded` (default `recorded`) selects the
backend; `scripts/live-integration.sh` sets `live` locally after a successful
proxy health check, and CI never sets it (recorded by default). Alternative
considered: auto-detect by probing the proxy — rejected, made CI behavior
implicit and harder to reason about; explicit default is safer.

**D3: Health-check-first, actionable failure.**
`scripts/live-integration.sh` does a fast `GET /v1/models` (or equivalent)
against the proxy before running any case. On failure it prints the two-step
remediation (Codex re-login; `launchctl kickstart -k
gui/501/ai.prometheus.openai-proxy`) and exits non-zero, rather than letting
the first test case fail with a generic connection/401 error.

**D4: Feature matrix as a living markdown doc, checked by CI presence-check
only (not content-parsed).**
`tests/integration/live/MATRIX.md` maps `CH-## → test case name(s)`. A
lightweight CI step greps the matrix for the current change's `CH-##` token
and fails if absent, enforcing "every change adds its case" without building
a bespoke parser. Alternative considered: a structured YAML/JSON matrix with
schema validation — rejected as over-engineering (Rule 2) for a 23-row table
maintained by the same people writing the code.

## Risks / Trade-offs

- **[Risk]** The proxy's Codex-backed token can expire mid-session, causing
  spurious live-tier failures unrelated to code changes.
  **Mitigation:** D3's health check catches this before any test case runs and
  gives the exact remediation; live-tier failures after a passing health check
  are treated as real regressions.
- **[Risk]** Recorded-backend cases can drift from real model behavior over
  time (the fixture never changes; the real model does), giving false
  confidence in CI.
  **Mitigation:** this is why the live backend exists and why local/pre-push
  runs against the real proxy remain the operator's actual confidence signal;
  recorded-backend CI runs prove wiring, not model behavior.
- **[Risk]** Feature matrix can rot (case added, MATRIX.md not updated, or
  vice versa).
  **Mitigation:** D4's CI presence-check makes an unmatched change fail loudly
  instead of silently drifting.
- **[Risk]** No existing test helper boots a full real `AppState`
  (`src/server.rs:567` wires ~20 live services — DB pools, MCP registry,
  memory service, governance engine). Every existing integration test uses
  either `axum_test::TestServer` against a narrow sub-router or a standalone
  mock server, never the whole stack. Building a general-purpose full-server
  boot harness is a materially bigger lift than this change's original
  complexity estimate assumed — discovered during implementation, not
  planning.
  **Mitigation:** split delivery — the stub LLM server (D1) is self-contained
  and ships first with no `AppState` dependency; the minimal server-boot
  harness + the 8 baseline cases (tasks.md Section 3) are their own clearly
  labeled follow-on within this same change. Some baseline cases (memory
  write→recall, RAG ingest→retrieve) may need in-memory test-double backends
  to be boot-able without a running Postgres/SurrealDB — confirm per-case as
  the harness is built, don't assume up front.

## Migration Plan

1. Build the in-process stub LLM server (`tests/integration/live/stub_llm.rs`):
   non-streaming + SSE chat-completion responses, tool-call responses,
   fixtures keyed by request fingerprint. No `AppState` dependency.
2. Build a minimal test harness that boots the real UAR server with
   `UAR_LLM__BASE_URL` pointed at either the stub (recorded, default) or the
   real proxy (live, via env var) — scoped only to what's needed for the
   baseline cases, not a general-purpose harness.
3. Write the baseline case list (streaming×3 modes, tool loop, agent
   selection, memory write→recall, RAG ingest→retrieve, credential-chain
   resolution) against both backends.
4. Add `scripts/live-integration.sh` with the health check + remediation.
5. Add `tests/integration/live/MATRIX.md` seeded with the baseline cases.
6. Wire the recorded-backend run + matrix presence-check into
   `comprehensive-tests.yml` (or a new lightweight workflow) — additive, no
   existing job removed.
7. No rollback complexity: this is a net-new, additive test tier; reverting is
   a plain revert of the new files with no data migration.

## Open Questions

- Should the live-tier recorded-backend run block PR merge, or start
  advisory-only (like `llm_judge` in the eval harness) until the baseline case
  list stabilizes across Round 1? Recommend: advisory for this change's own
  landing, promote to blocking once CH-01/02/03/04 have each added a case
  without matrix drift.
- Should `scripts/live-integration.sh` also support the Anthropic-compatible
  shape of the proxy, or is OpenAI-compatible sufficient given liter-llm
  normalizes both? Recommend: OpenAI-compatible only for now; liter-llm's
  normalization means this doesn't test provider-specific dialect handling
  (that's prompt-dialect-engine's job).

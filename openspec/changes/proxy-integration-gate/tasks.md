## 1. Recorded-backend stub LLM server (revised — see design.md D1)

- [x] 1.1 Build `tests/integration/live/stub_llm.rs`: in-process Axum server
      (pattern per `tests/test_a2a_client.rs`'s `start_mock_server`) exposing
      `/v1/chat/completions`, non-streaming JSON + SSE streaming responses
- [x] 1.2 Add tool-call response support (fixture can return a `tool_calls`
      completion, keyed by presence of a tool schema in the request)
- [x] 1.3 Fixture lookup keyed by request fingerprint (model + last user
      message + tool-schema presence); missing fixture errors clearly
- [x] 1.4 Unit-test the stub server directly (non-streaming, streaming,
      tool-call, and missing-fixture-error cases)

## 1b. Backend selection (both backends = pick a base_url)

- [x] 1b.1 Add `UAR_LIVE_INTEGRATION_BACKEND=live|recorded` selection
      (default `recorded`); `recorded` points `UAR_LLM__BASE_URL` at the
      stub server from Section 1, `live` points it at
      `http://127.0.0.1:8181/v1` with model `openai/gpt-5.4-mini`
- [x] 1b.2 Unit-test the default-to-recorded behavior

## 2. Proxy health check + remediation script

- [ ] 2.1 Add `scripts/live-integration.sh`: fast health check
      (`GET /v1/models` or equivalent) against the configured proxy
- [ ] 2.2 On health-check failure: print Codex re-login step +
      `launchctl kickstart -k gui/501/ai.prometheus.openai-proxy`
      remediation, exit non-zero, run no test cases
- [ ] 2.3 On health-check success: set `UAR_LIVE_INTEGRATION_BACKEND=live`
      and run the live case suite
- [ ] 2.4 Add `--allow-recorded-fallback` flag: when the proxy is
      unreachable, skip the health-check failure path and run the
      `recorded` backend instead (this is the default in CI)

## 3. Minimal server-boot harness + baseline feature cases (see design.md
      Risks — scoped narrowly, not a general-purpose `AppState` test harness;
      no existing test in this repo boots the full real server today)

- [ ] 3.0 Build the minimal test harness: real `provider_registry` +
      `model_router` (the actual point of this tier) wired to
      `UAR_LLM__BASE_URL`; in-memory test doubles for persistence/memory/MCP
      only where a live DB/service isn't needed to prove the feature works
      (confirm per-case below, don't assume up front)
- [ ] 3.1 Streaming chat case for `stream_mode: openai`
- [ ] 3.2 Streaming chat case for `stream_mode: agui`
- [ ] 3.3 Streaming chat case for `stream_mode: dual`
- [ ] 3.4 MCP tool-loop round-trip case (tool call issued, result
      incorporated into the final response)
- [ ] 3.5 Agent selection via the `model` request parameter
- [ ] 3.6 Memory write followed by a recall
- [ ] 3.7 RAG document ingest followed by a retrieval
- [ ] 3.8 Credential-chain resolution case
- [ ] 3.9 Run all 3.1-3.8 against both the `recorded` and (locally) the
      `live` backend; confirm parity of pass/fail shape between backends

## 4. Feature coverage matrix + CI wiring

- [ ] 4.1 Create `tests/integration/live/MATRIX.md` seeded with the 8
      baseline cases from Section 3 (no `CH-##` yet — pre-existing baseline)
- [ ] 4.2 Add a CI step that greps `MATRIX.md` for the current change's
      `CH-##` token (sourced from the PR branch name or a required PR
      label) and fails the build when a change referenced in
      `.kbd-orchestrator/phases/uar-next-harness/plan.md` lands without a
      matching row
- [ ] 4.3 Wire the `recorded`-backend run of Section 3's cases into
      `comprehensive-tests.yml` (or a new lightweight workflow) as an
      additive job — no existing job removed or modified
- [ ] 4.4 Mark the new CI job **advisory** (non-blocking) for this change's
      own landing per design.md's open question; document the promotion
      criteria (blocking once CH-01/02/03/04 have each added a case without
      matrix drift) in `tests/integration/live/MATRIX.md`

## 5. Documentation + cross-tool usability

- [ ] 5.1 Document `scripts/live-integration.sh` usage (flags, env vars,
      remediation) in `evals/README.md` or a new
      `tests/integration/live/README.md`, clearly distinguishing this gate
      from the eval harness's model-quality gate
- [ ] 5.2 Confirm the script and matrix-check step run identically from
      Codex, Claude Code, Cursor, and OpenCode (plain bash + existing
      `cargo test` invocation — no tool-specific hooks required); note this
      in the same README
- [ ] 5.3 Update `.kbd-orchestrator/phases/uar-next-harness/plan.md` /
      `current-waypoint.md` to record this change as landed and to make
      "adds a `tests/integration/live/MATRIX.md` row" an explicit
      completion criterion for every remaining Round 1-4 change

## 6. Verification

- [ ] 6.1 `cargo test` (recorded backend) green in CI
- [ ] 6.2 Local run of `scripts/live-integration.sh` against the real proxy
      green (manual/operator verification — the proxy is local-only)
- [ ] 6.3 Deliberately break the health check (stop the proxy) and confirm
      the remediation message appears and no test case runs
- [ ] 6.4 Deliberately land a change without a matrix row in a scratch
      branch and confirm the CI presence-check fails as designed

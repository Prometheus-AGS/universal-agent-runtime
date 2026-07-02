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

- [x] 2.1 Add `scripts/live-integration.sh`: fast health check
      (`GET /v1/models` or equivalent) against the configured proxy
- [x] 2.2 On health-check failure: print Codex re-login step +
      `launchctl kickstart -k gui/501/ai.prometheus.openai-proxy`
      remediation, exit non-zero, run no test cases
- [x] 2.3 On health-check success: set `UAR_LIVE_INTEGRATION_BACKEND=live`
      and run the live case suite
- [x] 2.4 Add `--allow-recorded-fallback` flag: when the proxy is
      unreachable, skip the health-check failure path and run the
      `recorded` backend instead (this is the default in CI)

## Scope note (revised during implementation)

This change originally also covered the minimal server-boot harness, the 8
baseline feature cases, `MATRIX.md`, CI wiring, docs, and verification
(former Sections 3-6). That work moved to the follow-on change
`live-integration-baseline-coverage` — see proposal.md's Scope note and
design.md's Risks. This change is complete once Sections 1, 1b, and 2 above
are done.

Verification for *this* change's shipped scope: `cargo test --test
integration live::` green (9/9 as of this writing); `scripts/
live-integration.sh` smoke-tested for all three paths (healthy proxy,
unreachable+fail, unreachable+`--allow-recorded-fallback`) plus
unknown-argument rejection.

## 1. Minimal server-boot harness (see design.md D1/D2 — confirm each
      `AppState` field's construction per case, don't assume up front)

- [x] 1.1 Confirm which `AppState` fields each baseline case actually needs;
      record the answer (this is design work, not just coding) before
      writing `boot_test_server`
- [x] 1.2 Build `boot_test_server(fixtures, needs) -> TestServerHandle`
      (or equivalent) wiring real `orchestrator`, `provider_registry`,
      `model_router`, `sessions`, `run_manager` and defaulting other
      `AppState` fields per 1.1's findings
- [x] 1.3 Confirm `axum_test::TestServer`'s SSE support is sufficient for
      `stream_mode: dual`; fall back to a raw `TcpListener` +
      `axum::serve` pattern (per `tests/test_a2a_client.rs`) if not
- [x] 1.4 Confirm whether `MemoryService`/RAG have an embedded/in-memory
      construction path suitable for tests; if not, note the gap per
      design.md Risk 1 rather than silently work around it

## 2. Baseline feature cases

- [x] 2.1 Streaming chat case for `stream_mode: openai`
- [x] 2.2 Streaming chat case for `stream_mode: agui`
- [x] 2.3 Streaming chat case for `stream_mode: dual`
- [x] 2.4 MCP tool-loop round-trip case (tool call issued, result
      incorporated into the final response)
- [x] 2.5 Agent selection via the `agent_id` request field (corrected from
      `model` after checking code — see specs' "Known gap" note: proves
      fallback-safe resolution only, not behavior differentiation)
- [x] 2.6 Memory write followed by a recall — `#[ignore]`d (needs
      `local-embeddings` Cargo feature, not enabled in this workspace; see
      design.md Risk 1); root-caused and documented, not silently skipped
- [x] 2.7 RAG document ingest followed by a retrieval — `#[ignore]`d for two
      independent, root-caused reasons (both flagged via `spawn_task`, not
      fixed here): `VectorMatcher::embed_batch` returns unconditional
      zero-vector placeholder embeddings (`model.forward()` commented out,
      `src/uar/runtime/matching/vector.rs:210-213` — search can never find
      real matches); and a SurrealQL `type::thing(...)` call rejected by the
      pinned SurrealDB `=3.0.5` (`src/uar/persistence/providers/surreal.rs:524`
      — document status silently stays "pending" even when ingestion
      actually succeeds). Test polls search directly (not the broken status
      field) so it will correctly validate the embedding fix once landed.
- [ ] 2.8 Credential-chain resolution case (reuse
      `InMemoryCredentialStore` pattern from
      `tests/credentials_api_integration_test.rs`)
- [ ] 2.9 Run all non-ignored cases from 2.1-2.8 against both the
      `recorded` and (locally) the `live` backend; confirm parity of
      pass/fail shape between backends

## 3. Feature coverage matrix + CI wiring

- [ ] 3.0 CI risk to check before wiring the job: `start_server` loads the
      repo's real `mcp.json` (cwd-relative, hardcoded path,
      `src/server.rs:271`), which spawns `npx -y @mcpcentral/mcp-time` as a
      subprocess ("MCP Time Server running on stdio" appears in every
      baseline-case test's output). This worked fine locally (6.10s for 4
      server boots, npx likely had the package cached) but a cold CI runner
      may need network access for the first `npx` fetch. Loading failure is
      already non-fatal (falls back to an empty MCP registry with a logged
      warning), so worst case is slower CI, not a hard failure — confirm
      this holds on the actual CI runner before relying on it.
- [ ] 3.1 Create `tests/integration/live/MATRIX.md` seeded with the
      baseline cases from Section 2 (no `CH-##` yet — pre-existing
      baseline); note any `#[ignore]`d cases explicitly
- [ ] 3.2 Add a CI step that greps `MATRIX.md` for the current change's
      `CH-##` token (sourced from the PR branch name or a required PR
      label) and fails the build when a change referenced in
      `.kbd-orchestrator/phases/uar-next-harness/plan.md` lands without a
      matching row
- [ ] 3.3 Wire the `recorded`-backend run of Section 2's cases into
      `comprehensive-tests.yml` (or a new lightweight workflow) as an
      additive job — no existing job removed or modified
- [ ] 3.4 Mark the new CI job **advisory** (non-blocking) for this change's
      own landing; document the promotion criteria (blocking once
      CH-01/02/03/04 have each added a case without matrix drift) in
      `tests/integration/live/MATRIX.md`

## 4. Documentation + cross-tool usability

- [ ] 4.1 Document the full live integration tier (backend selection +
      baseline harness + matrix contract) in `evals/README.md` or a new
      `tests/integration/live/README.md`, clearly distinguishing this gate
      from the eval harness's model-quality gate
- [ ] 4.2 Confirm the CI job and matrix-check step run identically from
      Codex, Claude Code, Cursor, and OpenCode (plain bash + existing
      `cargo test` invocation — no tool-specific hooks required); note this
      in the same README
- [ ] 4.3 Update `.kbd-orchestrator/phases/uar-next-harness/plan.md` /
      `current-waypoint.md` to record this change as landed and to make
      "adds a `tests/integration/live/MATRIX.md` row" an explicit
      completion criterion for every remaining Round 1-4 change

## 5. Verification

- [ ] 5.1 `cargo test` (recorded backend) green in CI, including the new
      baseline cases
- [ ] 5.2 Local run of `scripts/live-integration.sh` (from
      `proxy-integration-gate`) against the real proxy, exercising the
      baseline cases, green (manual/operator verification)
- [ ] 5.3 Deliberately break the health check (stop the proxy) and confirm
      the remediation message appears and no test case runs (re-verifies
      `proxy-integration-gate`'s behavior still holds with real cases
      wired in)
- [ ] 5.4 Deliberately land a change without a matrix row in a scratch
      branch and confirm the CI presence-check fails as designed

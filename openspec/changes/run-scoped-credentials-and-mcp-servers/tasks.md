## 1. Contract tests first (must fail before implementation, pass after)

Shared fixtures (in `tests/support/`): a `wiremock` OpenAI-compatible provider that records the `Authorization` header of every request per path; a provider-registry entry for it; an in-process rmcp `StreamableHttpService` MCP server on `127.0.0.1:0` that records request headers, tool calls and connection close, and exposes one tool whose output names the server; a scripted `MockLlmDriver` turn that calls a named tool; a `tracing` capture layer at TRACE writing to a buffer; a `tempfile` data directory for persistence. Keys and tokens are random per test (`sk-run-A-<uuid>`, `tok-B-<uuid>`).

D1 falsifier map: F3 (credential containment) → 1.1–1.8; F4 (tool isolation, no residue) → 1.9–1.16; round-3 open risk 1 (resume without credential) → 1.4, 1.5, 1.14. F5 (host MCP endpoint abuse) is a the-boss test and is not served here.

- [ ] 1.1 `tests/run_scoped_credentials.rs::run_credential_overrides_every_other_key_for_its_provider` — seed a user-scope stored credential, a registry key and an env key for provider P; create a run with a run credential for P. Assert every recorded request to P carries only the run key. Verify: fails today (field ignored, stored key used).
- [ ] 1.2 `run_scoped_credentials.rs::concurrent_runs_use_only_their_own_credential` — two sessions, keys A and B, started together. Assert each recorded request's key matches the session that sent it (correlate by a per-session marker in the input). Verify: fails today.
- [ ] 1.3 `run_scoped_credentials.rs::child_run_reuses_parent_credential_in_memory` — scripted parent spawns a local child agent. Assert the child's provider requests carry the parent's key, and the child run record, events and delegated message contain no key.
- [ ] 1.4 `run_scoped_credentials.rs::resume_without_credential_fails_closed` — credentialed source run; `POST /runs/{id}/resume` and `POST /runs/{id}/resume/{checkpoint_id}` without `run_credentials`. Assert 422 `run_credential_required`, no `run_id` in the body, zero new provider requests. Then resume with the credential and assert requests carry the re-attached key.
- [ ] 1.5 `run_scoped_credentials.rs::a2ui_continuation_requires_credentials_and_inline_artifact` — credentialed run with an inline (unsaved) artifact renders a surface; `POST /runs/{id}/a2ui/actions` without credentials → 422 `run_credential_required`; with credentials but no artifact → 422; with both → 202 and the continuation's requests carry the key. Verify: today the last case fails with 409 "agent not found".
- [ ] 1.6 `run_scoped_credentials.rs::routing_and_failover_stay_on_credentialed_providers` — failover enabled with a fallback on provider Q (its own registry key); primary P returns 500. Assert Q's mock receives nothing and the run ends with RUN_ERROR code `run_credential_provider_unavailable`.
- [ ] 1.7 `run_scoped_credentials.rs::auth_failures_under_run_credentials_do_not_cool_down_the_provider` — session 1 sends runs with a wrong key until the health threshold; session 2 then runs with a valid key for P. Assert session 2's request reaches P.
- [ ] 1.8 `run_scoped_credentials.rs::run_credential_never_leaves_memory` — keys for: a plain run, a child run, a resumed run, a graph agent that writes a checkpoint which is then listed and resumed, and a run whose provider returns 401 with a body echoing the `Authorization` header. Afterwards byte-search, for each key in raw and base64 form: every file under the temp data directory, the captured log/trace buffer, every SSE stream body, `GET /runs/{id}/checkpoints` responses, all create/resume response bodies, and run error messages. Assert zero hits. Crash output is not covered: UAR writes no crash dump (search of `src/` only).
- [ ] 1.9 `run_credential_input_is_validated` (in `run_scoped_credentials.rs`) — empty key, duplicate provider id, provider id outside the pattern → 422 `run_credential_invalid`; kind `bedrock` → 422 `run_credential_provider_kind_unsupported` whose message names `bedrock` and the supported kinds; each body contains no submitted key or URL. (A provider id unknown to the registry is now valid; 1.18 covers it.)
- [ ] 1.10 `tests/run_scoped_mcp_servers.rs::run_scoped_server_is_the_only_mcp_server` — global registry holds `global-fs` (second test server); run supplies `boss-tools` with `Authorization: Bearer <tok>`. Assert the tool list sent to the model contains `boss-tools` tools and allowed native tools only; every request `boss-tools` received carries the bearer header; `global-fs` received nothing.
- [ ] 1.11 `run_scoped_mcp_servers.rs::call_to_a_tool_outside_the_run_catalog_is_refused` — scripted model calls a `global-fs` tool. Assert a tool error result and zero requests at `global-fs`.
- [ ] 1.12 `run_scoped_mcp_servers.rs::policy_selecting_a_non_run_scoped_server_is_rejected` — artifact `uar.run_policy.mcp_servers` selects `global-fs` while the request supplies `mcp_servers`. Assert 422 `mcp_server_not_run_scoped` and no run id.
- [ ] 1.13 `run_scoped_mcp_servers.rs::concurrent_sessions_cannot_see_or_call_each_others_tools` — two sessions, each with a `boss-tools` server at a different URL and token and a different tool set, run concurrently. Assert each model request lists only its own tools and each server receives calls only from its own run.
- [ ] 1.14 `run_scoped_mcp_servers.rs::run_scoped_servers_leave_no_residue` — for a finished, an errored and a cancelled run: `GET /api/uar/mcp/servers` names none of the run's servers; the persisted MCP config file is byte-identical to its pre-run copy; each server observes its connection closed within 2 s; a child binding retained past the root's end gets a tool error. Continuation without `mcp_servers` → 422 `run_mcp_servers_required`.
- [ ] 1.15 `run_scoped_mcp_servers.rs::run_scoped_server_input_is_validated` — non-allowlisted host, `ftp` scheme, bad name, duplicate name, forbidden header (`Host`, `Mcp-Session-Id`) → 422 `run_mcp_server_invalid`; the test servers record no connection.
- [ ] 1.16 `run_scoped_mcp_servers.rs::server_headers_never_leave_memory` — the byte search of 1.8 applied to MCP bearer tokens, including MCP lifecycle events (`uar.mcp.server_state`).
- [ ] 1.17 Record the failing output of 1.1–1.16 and 1.18–1.21 against the unmodified branch as evidence before any implementation task starts.
- [ ] 1.18 `run_scoped_credentials.rs::custom_base_url_receives_the_run_requests` (**operator decision 2026-09-23**) — two `wiremock` servers: A is the credential's `base_url` (`http://127.0.0.1:<port>/v1`), B is the registry entry's `base_url` for the same provider id. (a) Kind `openai_compatible`, provider id equal to the registry id: every chat request reaches A with the run key and an unprefixed model id; B receives nothing. (b) Provider id `local-llm`, unknown to the registry and the catalog, no model in the policy, `default_model: "tiny"`: the run completes one scripted tool call and A receives model `tiny`. (c) Kind `anthropic`, provider id `claude-work`: A receives requests on the Anthropic Messages path with the plain model id, not on `/chat/completions`. Verify: fails today (the fields are ignored; the registry URL is used).
- [ ] 1.19 `run_scoped_credentials.rs::invalid_base_url_is_rejected_before_any_connection` — cases: `http://models.example.com/v1` (plain http, remote), `https://user:pass@127.0.0.1:<port>/v1` (userinfo), `ftp://127.0.0.1/`, `not a url`, a relative path; and, with local-only mode on, `https://api.example.com/v1`. Each → 422 `run_credential_invalid`, no run id, zero requests at a listening mock, and the body contains neither the URL nor the key. Positive control: `http://localhost:<port>/v1` and `http://[::1]:<port>/v1` are accepted. Verify: fails today.
- [ ] 1.20 `run_scoped_credentials.rs::base_url_never_leaves_memory` — the byte search of 1.8 applied to a per-test marker in the base URL path (`/marker-<uuid>/v1`) for: a successful run at TRACE logging, a run whose endpoint refuses the connection, a run whose provider returns 500, and a resumed run. Also formats the run's `LlmConfig` with `{:?}` in a unit test and asserts the marker is absent. Zero hits in files under the data directory, the log/trace buffer, SSE bodies, run and checkpoint responses, run error messages and the `host_resources` marker. Verify: fails today (the field does not exist; a naive implementation leaks through `LlmConfig`'s `Debug`, which prints `base_url`, `src/config.rs:1721`).
- [ ] 1.21 `run_scoped_credentials.rs::capabilities_list_run_scoped_features` — `GET /api/uar/capabilities` (from `sidecar-launch-security`) lists `run_scoped_credentials` and `run_scoped_mcp_servers`. Verify: fails before this change lands.

## 2. Request surface

- [ ] 2.1 Add `RunCredentialInput` and `RunMcpServerInput` (Deserialize-only, `SecretString` values, redacting `Debug`) and wire them into `CreateRunRequest`, `ResumeRequest` and `A2uiActionPayload`; verify 1.9 and 1.15 pass.
- [ ] 2.2 Add `pub(crate)` `run_credentials` and `run_mcp_servers` to `RunExecutionRequest` with no `Serialize` impl; verify `cargo check` rejects a `serde_json::to_value` of either (a compile-fail doc test).
- [ ] 2.3 Add config key `mcp.run_scoped.allowed_hosts` with loopback default; verify 1.15 non-allowlisted case.

## 3. Credentials

- [ ] 3.1 Constrain routing to credentialed providers and apply the run credential after `apply_credential_layer` by replacing the run's `LlmConfig` (design D2 step 2, D10); verify 1.1, 1.2.
- [ ] 3.2 Build fallback drivers only for credentialed providers; fail with `run_credential_provider_unavailable` otherwise; verify 1.6.
- [ ] 3.3 Skip `ProviderHealthMonitor::record_failure` for every failure under a run credential (design D2 step 4); verify 1.7.
- [ ] 3.4 Child guard for providers outside the parent's credentialed set; verify 1.3.
- [ ] 3.5 Error-text scrubber for run secrets, including run-credential base URLs (raw and base64); verify the provider-echo case of 1.8 and the refused-connection case of 1.20.
- [ ] 3.6 Base URL validation at the route (scheme, loopback rule via `local_only::host_is_local`, userinfo, local-only mode) and the provider-kind check; build the run credential's `LlmConfig` directly (design D10), replacing — not patching — the resolved config; add the `#[serde(skip)]` run-provider kind to `LlmConfig`, dispatch on it in `build_driver`, and redact `base_url` in `Debug` when it is set. Verify 1.9, 1.18, 1.19, 1.20.
- [ ] 3.7 Check whether liter-llm's HTTP client follows redirects; if it does, disable them for drivers built from a run credential. Record the finding in the evidence folder. Verify with a `wiremock` 307 to a second mock in 1.18's harness: the second mock receives nothing.
- [ ] 3.8 Add `run_scoped_credentials` and `run_scoped_mcp_servers` to the capabilities list (`sidecar-launch-security` design Decision 12) in the commit that completes sections 3–5. Verify 1.21.

## 4. Resume and continuation

- [ ] 4.1 Record `host_resources` in the run context; reject resume/continuation missing credentials or servers; verify 1.4, 1.14 continuation case.
- [ ] 4.2 Accept and check the inline `artifact` on A2UI actions; pass it to `continue_with_interaction`; verify 1.5.

## 5. Run-scoped MCP

- [ ] 5.1 Add the run-local server definition and connector with `custom_headers`/`auth_header`, no proxy, no redirects; verify 1.10.
- [ ] 5.2 Build the run-local registry in place of `capture_root_mcp_resources` for host-owned runs and force `Selected` policy; reject non-run-scoped selections; verify 1.10–1.12.
- [ ] 5.3 Keep run-local bindings out of the binding cache and pass them to children narrowed; verify 1.13.
- [ ] 5.4 Tear down on root terminal state and `RunDelegationLifetime` drop; verify 1.14.

## 6. Phase-boundary verification

- [ ] 6.1 Run `cargo test --test run_scoped_credentials --test run_scoped_mcp_servers` once after sections 2–5 are complete, then the broader integration gate (`cargo test --tests`) once; record both outputs.
- [ ] 6.2 Run `openspec validate run-scoped-credentials-and-mcp-servers --strict`; verify it passes.

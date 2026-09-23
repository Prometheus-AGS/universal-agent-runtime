## Context

See proposal.md for why. The code this design builds on:

- **Request path.** `create_run` builds a `RunExecutionRequest` from `CreateRunRequest` (`src/uar/api/routes.rs:38-47, 63-86`). `RunExecutionRequest` already separates host-only inputs from JSON: `mcp_resources` is "never accepted from JSON" (`src/uar/runtime/turn/request.rs:29-32`), and several `pub(crate)` host constraints exist (`request.rs:37-48`).
- **Credential resolution.** The run's `LlmConfig` is resolved from the provider registry (`manager.rs:3232-3262`), optionally re-routed (`manager.rs:3280-3330`, `apply_routed_connection` at `manager.rs:519-531` swaps the key when the provider changes), then passed through `apply_credential_layer` (`manager.rs:459-505`, called at `manager.rs:3340-3347`), which applies session → agent → user → system stored credentials (`src/uar/security/credentials/resolver.rs:55-98`).
- **Model bindings.** `RunModelBindings::capture` builds the primary and fallback drivers once per root run (`src/uar/runtime/turn/bindings.rs:146-200`); children reuse them through `InheritedRunBindings.models` and never call `capture` (`bindings.rs:19-34`, doc at `bindings.rs:143-145`). Fallback drivers are built by `Orchestrator::build_fallback_driver`, which replaces the key with the fallback entry's own key (`src/llm/orchestrator.rs:1082-1097`).
- **Health.** `ProviderHealthMonitor` is keyed by provider id only (`src/llm/health.rs:42-80`), so one bad key trips cooldown for every session.
- **Where the key would live.** `LlmConfig.api_key` is a plain `Option<String>` that derives `Serialize` (`src/config.rs:1631-1648`). Its `Debug` impl redacts it (`config.rs:1707-1719`). Drivers are built from `LlmConfig`, so the key must reach that field.
- **Resume and continuation.** Resume handlers build a fresh request from `ResumeRequest` (`routes.rs:302-351, 353-455`). A2UI continuation loads the artifact from persistence by `run.agent_id` (`manager.rs:1690-1734`); the runs API never saves inline artifacts (`routes.rs:63-86`; no `save_agent` call in `manager.rs`). `get_run` reads only in-memory `active_runs` (`manager.rs:5644-5647`), so after a sidecar restart every old run id is 404.
- **MCP.** `McpServerEntry` has `Stdio` and `RemoteHttp { url, env }` variants, derives `Serialize`, and is what the admin API persists (`src/mcp/config.rs:40-56`); `RemoteHttp` has no header support. Remote HTTP connects without auth headers (`src/mcp/registry.rs:633-641, 739-742`). rmcp 3.1.2 exposes `auth_header` and `custom_headers` on `StreamableHttpClientTransportConfig` (verified in the crate source, `streamable_http_client.rs:1677-1702`). Server sources are `Global` and `Skill` (`src/mcp/catalog.rs:27-43`). Policy filtering is by server name (`src/mcp/registry.rs:1971-2001`, `src/uar/runtime/turn/contributors.rs:223-235`); an unknown `Selected` id only warns (`src/uar/domain/policy.rs:807-826`). Verified owners take the root MCP capture path (`manager.rs:2223-2229`), which checks the run cwd against the single process-wide MCP environment (`manager.rs:2250-2256`).

## Goals / Non-Goals

**Goals:**
- A key supplied for one run is used by that run and its local children only, and exists only in process memory for that lifetime.
- Run-scoped MCP servers are enforced as the run's only MCP servers at catalog construction, not by name filtering over a shared registry.
- Every path that creates a run from a prior run either carries the host resources again or refuses to start.

**Non-Goals:**
- the-boss's host MCP endpoint hardening (D1 falsifier 5). That lives in the-boss.
- Wire protocols other than OpenAI-compatible chat and Anthropic Messages for run credentials (D10).
- Per-session principals; that is `sidecar-session-principal`.
- Stdio run-scoped servers. Only Streamable HTTP is accepted.

## Decisions

### D1 — Carry secrets in a type that cannot be serialized
`CreateRunRequest`, `ResumeRequest` and `A2uiActionPayload` gain `run_credentials: Option<Vec<RunCredentialInput>>` and `mcp_servers: Option<Vec<RunMcpServerInput>>`. The input types derive `Deserialize` only, hold `secrecy::SecretString` for the key, the credential's `base_url` and every header value, and implement `Debug` with redaction. They convert into `RunCredentials` and `RunMcpServers` fields on `RunExecutionRequest` that are `pub(crate)` and have no `Serialize` impl, so no route can echo them and no code can put them in `run.context`.
- **Why:** a type without `Serialize` makes the leak a compile error rather than a review item. `#[serde(skip_serializing)]` on a serializable struct still allows a hand-written `json!` to include it.
- **Alternative rejected:** storing the key as a session-scoped credential row through `ProviderService`. That persists an encrypted copy to the database and outlives the run.

### D2 — Apply the run key last, and constrain routing before it
Resolution order becomes: registry config → routing → skill model → `apply_credential_layer` → **run credential**. When the run carries credentials:
1. Routing (`route_with_preferred_model`) receives the set of credentialed provider ids as a hard constraint (a new field on `RouteRequirements`; none exists today); `apply_routed_connection` never runs for a provider outside it.
2. After `apply_credential_layer`, if the routed provider has a run credential, **replace** the run's `LlmConfig` with one built from that credential (D10) — model, `base_url`, key, `resolved_provider_id`, `api_key_env: None`, empty `provider_keys`. Patching only `api_key` is not safe: when the policy's provider is not in the registry, resolution falls back to the process-wide `self.llm_config` with its own `base_url` (`src/uar/runtime/manager.rs:3251-3256`, and the routed-model fallbacks at `:3318-3330` return that same config), and a patched key would be sent to UAR's configured endpoint. If the routed provider has no run credential, fail the run with `run_credential_provider_unavailable` before `RunModelBindings::capture`.
3. `RunModelBindings::capture` builds fallback drivers only for providers with a run credential, each from a config built from that credential (D10). Other fallback entries are skipped with a warning naming the model.
4. No failure from a driver built from a run credential — 401, 403, 5xx or transport — is recorded with `ProviderHealthMonitor::record_failure`; it is a run error. The monitor is keyed by provider id only (`src/llm/health.rs:42-80`), and a run credential's endpoint is host-supplied, so its failures say nothing about the registry provider that may share its id. The run's own failover still sees the error.
- **Why:** the key must be applied after routing (coordinator inventory item 2), and routing must be constrained, or a routed or fallback provider would pick up UAR's own key.
- **Alternative rejected:** keying health by (provider, key fingerprint). It keeps a derived value of the key in a long-lived map and needs a hash choice. Not recording run-credential failures in the shared monitor is enough to stop one session's bad key or endpoint from cooling down a provider for the others.

### D3 — Children reuse captured clients; nothing is re-resolved
No new mechanism. Children already inherit `RunModelBindings` in memory. The change adds a guard: a child whose policy selects a provider outside the parent's credentialed set fails with `run_credential_provider_unavailable`, instead of falling back. A2A remote delegation already resolves on the peer (spec `mcp-runtime-projection`, delegation scenario); a test asserts the outbound request has no key.

### D4 — No credential outlives its run; resume re-attaches
UAR does not keep a run credential for later resume. The run record's `context` gains `host_resources: { credential_providers: [ids], mcp_servers: [names], artifact_inline: bool }`. Resume and continuation handlers compare the request with this marker before calling `execute_request` and reject with `run_credential_required` or `run_mcp_servers_required` when anything is missing. A marker is only a set of names, so it is safe to serialize.
- **Why:** round-3 open risk 1. Keeping the key for later resume would mean holding a secret after the run ends, which breaks the no-residue test. Re-attachment makes the host the only holder between runs.
- **After a sidecar restart:** the old run ids are 404 (`manager.rs:5644-5647`), which already fails closed. The-boss continues with a new run plus host-supplied history (`run-request-host-context`).

### D5 — A2UI continuation carries the inline artifact
`continue_with_interaction` loads the artifact from persistence (`manager.rs:1703-1711`). For a run whose artifact came inline, that load fails today with "agent not found". The action payload gains an optional `artifact`; when `host_resources.artifact_inline` is true it is required, and its `id` must equal the source run's `agent_id`, else 422. The run's recorded `effective_run_policy` is still used, as today (`manager.rs:1718-1722`).

### D6 — Scrub the known secret from error text
The run holds its secrets, so it can remove exact occurrences. Every error string that leaves the run kernel (the `NormalizedEvent::Error` message, `tracing` error fields emitted by the run path) passes through a scrubber that replaces each run secret — keys, run-credential base URLs, MCP header values — and its base64 form, with `[REDACTED]`. This covers provider error bodies that echo the key and transport errors that print the URL.

### D7 — Run-scoped MCP servers are a run-local registry, not registry entries
- A new `RunMcpServer` definition type holds `name`, `url` and secret headers. It is not an `McpServerEntry`, so it can never reach the admin API, `config.rs` persistence or the settings writer.
- For a run with `mcp_servers`, the manager skips `capture_root_mcp_resources` (`manager.rs:2223-2229`) and builds a fresh `McpRegistry` for the run, connecting each server through `StreamableHttpClientTransportConfig::with_uri(url)` with `custom_headers` (and `auth_header` for `Authorization`), with the existing no-proxy, no-redirect reqwest client settings (`registry.rs:733-736`). This registry is the run's MCP universe; the global registry is not consulted.
- Effective policy for such a run sets `mcp_servers` to `Selected` with exactly the supplied names. A run policy that selects any other server fails the request with `mcp_server_not_run_scoped` (today it would only warn, `policy.rs:807-826`).
- The run-local registry is never placed in the binding cache (`src/mcp/binding_cache.rs`), so two runs with identical definitions never share a connection.
- Children receive the run-local registry through `InheritedRunBindings.mcp` (`bindings.rs:28`), narrowed by child policy.
- No run-scoped definition is ever passed to `McpRegistry::upsert_server` or the admin save path. In the sidecar the global list stays whatever the operator configured (empty for the-boss); host-owned runs never read it, so its contents cannot leak into them, and test 1.14 asserts it gains no entries.
- The process-wide MCP environment and its cwd check (`manager.rs:2250-2256`) apply to stdio launches only. Run-scoped HTTP servers spawn no process, so they do not use that environment.

### D8 — Tear down at root terminal state
The root run's terminal path (done, error, cancelled) and `RunDelegationLifetime::drop` (`bindings.rs:69-78`) shut down the run-local registry. A child still holding an `Arc` to it gets `McpRuntimeError` on any later call, because the transports are closed.

### D9 — Host allowlist for run-scoped URLs
New config key `mcp.run_scoped.allowed_hosts` (proposed name; it does not exist yet) defaults to `["127.0.0.1", "::1", "localhost"]`. The runs API is an authenticated boundary, but a caller-supplied URL is still a request UAR makes on the caller's behalf; the allowlist keeps it from reaching internal services.

### D10 — A run credential is a whole run-local provider: kind, endpoint, key
*Operator decision 2026-09-23:* a run credential carries provider kind, base URL and key, not only a key, so providers the-boss runs against custom endpoints (local model servers, gateways) work.

Input: `{ provider_id, provider_kind, base_url, api_key, default_model? }`. `provider_id` is the run-local name the agent's policy and routing use (`<provider_id>/<model>`); it must match `^[a-z0-9][a-z0-9._-]{0,62}$` and be unique in the list. It need not exist in UAR's registry. When it does, the run credential replaces that registry entry for the run: the registry's key, `base_url`, `default_model` and model list are not read.

**Base URL validation** (at the route, before any run starts; failure is 422 `run_credential_invalid`, body without the URL or key):
- parses with `url::Url` (already used by `src/llm/local_only.rs:124`);
- scheme `https`, or `http` only when the host is loopback by `local_only::host_is_local` (`src/llm/local_only.rs:94-105`: IPv4 loopback, IPv6 loopback including IPv4-mapped, `localhost` case-insensitive);
- no userinfo (empty username and no password);
- has a host.
No host allowlist applies. A provider endpoint is normally remote (`api.openai.com`), unlike run-scoped MCP servers (D9). The caller is the launch-token-authenticated host; that is the boundary this relies on. When local-only mode is on, `build_driver` still refuses a non-loopback URL (`src/llm/orchestrator.rs:57`, `local_only.rs:117-136`), and the route maps that refusal to `run_credential_invalid` before the run starts.

**Mapping onto driver construction.** Today `resolve_to_llm_config` turns a `ProviderConfig` into an `LlmConfig`: an explicit `base_url` means the model is sent unprefixed, an empty model takes `default_model`, and `resolved_provider_id` names the provider (`src/llm/registry.rs:433-472`). A run credential builds the same shape directly, without the registry:
- `model` = the routed model id for this provider, else `default_model`, else the run fails with `run_credential_provider_unavailable`; sent unprefixed, as for any explicit `base_url`;
- `base_url` = the credential's URL; `api_key` = the credential's key; `api_key_env` = `None`; `provider_keys` empty; `resolved_provider_id` = `provider_id`.

`build_driver` picks the wire protocol from `resolved_provider_id == "anthropic"` (`src/llm/orchestrator.rs:59-63`), and liter-llm, given a `base_url`, picks it from the model prefix: `anthropic/` and `azure/` get their own providers, anything else is OpenAI-compatible (`build_provider`, `vendor/git/liter-llm/crates/liter-llm/src/client/mod.rs:976-994`, read in the main checkout because the worktree's submodule is not checked out). A host-chosen `provider_id` such as `claude-work` would therefore be sent as OpenAI-compatible. So the kind, not the id, selects the driver: `LlmConfig` gains a `#[serde(skip)]` run-provider kind, and `build_driver` dispatches on it when present:
- `openai_compatible` → `LiterLlmDriver` with the `base_url` and an unprefixed model; liter-llm builds its OpenAI-compatible provider for any model without an `anthropic/` or `azure/` prefix. Covers OpenAI itself, local servers with OpenAI-compatible APIs, and OpenAI-compatible gateways.
- `anthropic` → the native `AnthropicDriver` with the `base_url` when `anthropic_native_driver_enabled()` (the branch at `orchestrator.rs:63-77`); otherwise `LiterLlmDriver` with model `anthropic/<model>` so liter-llm builds its Anthropic provider with that base URL. Whether liter-llm strips the prefix before sending was not checked; test 1.18 asserts the model id the mock receives.
- any other kind → 422 `run_credential_provider_kind_unsupported` at the route, naming the kind and the supported set, and no run starts. Azure is not accepted: liter-llm routes `azure/` with a base URL, but deployment and API-version handling was not checked.

**Confinement.** The base URL gets the key's treatment (D1, D6). It is a `SecretString` in the input and `RunCredentials`. It is never passed to `ProviderRegistry::register`, which logs `base_url` (`registry.rs:266`, `:339`). `LlmConfig`'s `Debug` prints `base_url` in clear (`src/config.rs:1721`), so the same `#[serde(skip)]` marker makes `Debug` redact `base_url` for run-credential configs. `EgressDenial` messages embed the raw URL (`local_only.rs:124-134`); the route maps them to a code and drops the text. The D6 scrubber removes the URL, like the key, from error text. The `host_resources` marker records provider ids only.
- *Alternative rejected:* key only, registry-known providers only (this change's first draft). the-boss's custom-endpoint providers would not work, and the fallback at `manager.rs:3251-3256` would pair the host key with UAR's own endpoint.

## Risks / Trade-offs

- [The key and base URL sit in plain `String`s inside `LlmConfig` for the run's lifetime, because drivers are built from it] → The containment guarantee depends on no code path serializing a run's `LlmConfig`. `LlmConfig` derives `Serialize` (`config.rs:1631`). The byte-search contract test is the enforcement; a new serialization site fails it.
- [A host-supplied `https` URL can point at an internal service] → accepted: the caller is the launch-token-authenticated host, which already controls the machine's the-boss configuration. No host allowlist, unlike D9. Local-only mode still confines it to loopback.
- [Redirects] → whether liter-llm's HTTP client follows redirects, which would carry the key to another host, was not checked. Task 3.7 checks it and, if it does, disables redirects for run-credential drivers.
- [Model capability data] → a model on a run-local provider may be unknown to the catalog; how routing and capability checks treat an unknown model id was not checked. Test 1.18 runs a tool call through a model id the catalog does not know.
- [The context-budget endpoint fingerprint] → `LiterLlmDriver` keys budgets by an unsalted SHA-256 of the base URL (`src/llm/liter_driver.rs:86`, `src/uar/runtime/context/budget.rs:133-141`). It is a derived value, not the URL; whether it is persisted was not checked.
- [`SecretString` zeroizes on drop; copies in `LlmConfig`, driver HTTP headers and reqwest buffers do not] → Accepted. The requirement is "not persisted, logged or traced", not "wiped from memory". A core dump would contain it; UAR writes no crash dump itself (not verified beyond a search of `src/`).
- [the-boss must resend secrets on every resume and continuation] → Intended. A missing field fails closed with a named error; it never silently uses UAR's own keys.
- [Scrubbing covers exact and base64 forms only] → A provider that echoes a transformed key (partial, URL-encoded) would still leak through error text. The test covers the forms UAR can know.
- [Name-based policy filtering elsewhere still exists] → For host-owned runs the global registry is never in the run's catalog, so a name collision with a global server cannot select it.

## Migration Plan

Additive for existing callers. Deploy with `sidecar-session-principal` and `sidecar-launch-security`; the-boss's driver refuses UAR agents until `GET /api/uar/capabilities` (`sidecar-launch-security` Decision 12) lists `run_scoped_credentials` and `run_scoped_mcp_servers` (D2 version gate). This change adds both names when its behaviour lands. Rollback: revert; runs without the new fields are unaffected.

## Open Questions

- Maximum `run_credentials` and `mcp_servers` entries per run (proposed 8 and 16). Tuning only.
- Whether rmcp 3.1.2 sends an HTTP `DELETE` for the MCP session when the client transport is dropped. The spec requires only that the server observes the connection closed; the test asserts that.
- Whether more provider kinds (Azure, Google, Bedrock) are needed. Each needs its own driver mapping; none is guessed here.

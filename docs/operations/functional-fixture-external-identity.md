# External identity for selected functional fixtures

The `truthful_tool_outcomes` (dot04) and `stream_terminal_contract` (dot05) targets use externally provisioned, exclusively owned loopback infrastructure. Functional code never creates, reads, copies, renews or sends authentication credentials. The external gateway supplies a stable verified owner and tenant; database inspection gateways own database authentication. This does not relax production identity admission.

The uncomfortable prerequisite is that these targets cannot run against an arbitrary existing UAR process or ambient developer cloud environment. Infrastructure must provision the exact isolated profiles below before a future authorized run. Source authoring alone does not establish that those profiles exist or that any scenario passes.

## Manifest contract

`UAR_EXTERNAL_IDENTITY_FIXTURE` selects a nonsecret JSON manifest. Unknown fields are refused. Its shape is `{ "dedicated": true, "suites": { "dot04": { "local": PROFILE, "shared": PROFILE }, "dot05": { "local": PROFILE } } }`.

Each PROFILE has these fields:

| Field | Meaning |
| --- | --- |
| `root` | Absolute private evidence directory. Dot04 creates it exclusively; its parent exists and the directory must not exist yet. |
| `nodes` | Map of stage names to NODE entries. |
| `databases` | Map of store names to `{inspectionUrl,namespace,name}`. Inspection URLs are credential-free literal loopback endpoints, or absolute `surrealkv://` paths under `root`. |
| `llmProfiles` | Map of controlled driver lanes to absolute opaque product configuration paths. |

A NODE has `backendBind`, `grpcBind`, `gatewayUrl`, `workspace`, `productConfigPath`, optional `productEnvPath`, `modelPeerBind`, and `database`. Binds are fixed nonzero loopback socket addresses. `gatewayUrl` is a literal loopback HTTP(S) origin without userinfo, query, fragment or path prefix. `database` references the corresponding database-map entry. Config and optional env paths are absolute; the fixture does not read their contents. Do not place secrets in the manifest.

External gateway routing must already target the declared backend addresses. No test remote-control endpoint exists. For process stages, the real product binary consumes `--config PATH`, `--port PORT`, and optional `--env-file PATH` through its existing CLI. Its externally owned configuration supplies matching HTTP/gRPC bindings, provider endpoint/model, database, service-instance identity and all required host settings. Fixture code clears inherited child environment; the product entrypoint loads an explicitly selected env file. It sets only existing nonsecret PATH/HOME/MCP_CONFIG_PATH/UAR_BUILTIN_SKILLS_DIR selectors and keeps original child working directories.

The Dot04 team and agent-instance writer gateways provision host admin authority for the exact `GET /api/uar/skills/deployment-catalog` route. The host supplies that key from the stage's private product configuration and forwards the request through the existing verified-owner gateway. The host drops caller-supplied authentication headers. Other methods and routes receive no admin key from this ingress. Rust and browser clients still use the declared credential-free origin.

## Owner capture

Both suites require an `identity-capture` node. This node has no `productEnvPath`; its opaque product config must be loadable by `ConfigManager::load_without_watcher`. The trusted-host fixture starts the actual server using `server::start_server_with_host_routes`, placing one nonce capture route before the existing authentication/governance/trace layers. A credential-free gateway request reaches that route. The route accepts only the actual middleware's verified `UserContext`, sends that typed value through an in-process channel, and returns 204 without identity JSON. No synthetic extension, deserialized identity or alternate auth scheme is used.

The external owner must be `dot04-outcome-owner` for dot04 and `dot05-stream-owner` for dot05. Keep tenant and workspace context stable across capture, direct component operations, child servers and reopened stores. Dot04 process nodes use workspace `dot04-process-workspace`. The capture host shuts down through the real production cancellation and cleanup path; component cases retain only its legitimately verified context in process.

## Dot04 stages and stores

Dot04 `llmProfiles.component` is a product config selecting `openai/gpt-4o-mini`; only the nonsecret actual ephemeral peer URL is substituted by the component fixture. The `component` database entry names the existing direct/graph/activation/scoped/reopen store. Local and shared profiles have distinct fresh roots and stores.

Provide nodes and matching primary database entries for:

- `scoped-success`, `scoped-application`, `scoped-predispatch`, `scoped-unknown`.
- `server_process-success`, `server_process-application`, `server_process-predispatch`, `server_process-unknown`.
- `team_process-success`, `team_process-application`, `team_process-unknown`, `team_process-no_tool`.
- `instance-process-success`, `instance-process-application`, `instance-process-unknown`.
- Shared only: `team-process-legacy-hold`.

Team stages additionally require `<label>-restarted-observer`; shared team stages also require `<label>-concurrent-observer`. Observer nodes reference the writer's database entry, preserve its actual service-instance identity, and have separate backend/gRPC ports and gateway URLs. Writer and observer profiles share that case's fixed `modelPeerBind`; one real model peer remains alive across concurrent and restarted nodes. The legacy-hold stage follows the same observer convention. Each primary case otherwise owns an isolated database and model-peer binding.

The fixture still starts/stops actual binaries, discovers actual MCP peers, kills the owned peer before dispatch, shuts down and reopens stores, and observes concurrent A/B and cold-restarted nodes. Original typed outcomes, canonical receipts, approval/task/attempt history, idempotency, execution ownership and persistence assertions remain. No lifecycle is replaced with a stub or opaque success callback. The shared component child forwards `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED` only from external provisioning; it no longer manufactures that attestation.

## Dot05 controlled transport lanes

`llmProfiles` keys are `OpenAi`, `Azure`, `LiterAnthropic`, `NativeAnthropic`, `GoogleAi`, `Vertex`, `Cohere`, and `Bedrock`; models remain those in `tests/support/stream_terminal_peer.rs`. The trusted Dot05 fixture marks the loaded connection as host-supplied before creating the drivers. This marker is request-scoped and stays outside serialized profile files. Product `load_llm_config` and `build_client_config`, or the existing public driver factory for native Anthropic, consume whole configs; fixture code does not inspect provider credential fields. HTTP peers use their actual ephemeral loopback URL. Native Anthropic selection must be provided externally through the existing product driver selector.

The four native HTTPS lanes retain the original verified TLS collaborator and custom DNS mapping: `generativelanguage.googleapis.com`, `us-central1-aiplatform.googleapis.com`, `api.cohere.com`, and `bedrock-runtime.us-east-1.amazonaws.com` resolve to the owned ephemeral HTTPS peer. Redirects and proxies are disabled. Keep Vertex's externally selected project `dot05-fixture` and region `us-central1`; Bedrock's selected region must be `us-east-1`. The fixture asserts Bedrock's nonsecret region precedence (`AWS_DEFAULT_REGION`, then `AWS_REGION`, then pinned default), while Liter internally resolves any required provider signing material. The existing actual provider path/body/count assertions remain, including Google, Vertex, Cohere and Bedrock.

Provision a clean isolated test-process environment and outbound deny policy allowing only the owned loopback peers/gateways. Do not inherit real developer/cloud credentials, provider endpoint overrides, workload metadata access or proxy settings. `host_supplied_connection` disables Liter ambient API-key fallback, but the product loader has its own documented config/environment precedence and Bedrock resolves signing material internally. Infrastructure, not test code, owns any required isolated provider protocol material. The test never sets placeholder API/SigV4 keys. An explicit `--config` suppresses implicit cwd/home config lookup. Product defaults precede that file; explicit CLI overrides and manual/environment resolution still apply (`AppConfig::load_with_cli`), so external isolation must also remove unrelated `UAR_*`, legacy LLM and provider overrides. Product `build_client_config` resolves material internally in its actual explicit-config, configured env-indirection, legacy environment, then provider-map order; tests pass the whole config without inspecting that material. The `shared` database entry supplies dot05's existing shared-store inspection lane; local embedded and shared graph assertions both remain.

## Future execution and coverage boundary

Run these selected targets serially, with `--test-threads=1`, and serialize their target commands. Fixed profile bindings cannot support default parallel test execution. Intentional A/B concurrency inside one case remains required. No command is authorized by this document; the full-phase operator verification lock remains closed until the lead receives explicit approval for the final command packet.

Dedicated credential-security regressions remain separate and unchanged: `j10_invalid_bearer_is_unauthorized_when_jwt_optional`, `j10_invalid_bearers_are_unauthorized_never_anonymous_when_jwt_optional`, `test_resolve_user_context_rejects_token_signed_with_wrong_secret`, and `j10_verification_paths_leak_no_token_or_key_material` in `src/uar/security/middleware.rs`. They retain the negative malformed/wrong-signature/no-anonymous-fallback assertions removed from the functional identity bootstrap. This is a source coverage mapping, not a current test verdict.

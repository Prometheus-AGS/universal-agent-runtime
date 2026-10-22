# Caller-tools local gate bindings (source inspection)

Phase: Execute, `uar-post-lifecycle-verification-readiness`; waypoint says
`implementationCompleted: 144`, `implementationTotal: 166`, and explicitly
keeps verification CLOSED. This is a source-derived gate recipe, not evidence
that any service or acceptance case ran. Exact build `cargo build --features
server-full,postgres-backend` was reported PASS by the lead; binary path is
`target/uar-post-lifecycle-main/debug/universal-agent-runtime`.

## Minimal supported local shape

The acceptance program is an HTTP client only: it neither starts the candidate
nor provisions persistence, agents, provider binding, credentials, or MCP. The
separate fixture is a real HTTP peer implementing OpenAI/Anthropic provider
routes and stateless MCP at `/mcp`; its `_control` and `_trace` endpoints reset
and expose synthetic trace/effect data. These roles are explicit in the module
docstrings of `scripts/release/client_tools_acceptance.py` and
`tests/fixtures/client_tools_provider.py`.

The least-complex durable store actually supported for this gate is embedded
SurrealKV: `src/config.rs::AppConfig::load_with_cli` defaults to
`persistence.provider: surreal` and `surrealkv://./data/uar.db`, while
`src/uar/persistence/providers/surreal.rs::SurrealDbProvider` sets
`durable_instances` for `SurrealEndpoint::EmbeddedFile`. That capability gates
`supports_durable_client_tools()`. Point it at a fresh, isolated absolute file
path via `UAR_PERSISTENCE__DATABASE_URL=surrealkv:///tmp/<unique>/uar.db` (or
equivalent config YAML); do not reuse the checkout's default database. Startup
constructs the configured provider in `src/server.rs` and requires the
`surreal-backend` feature. A separate Surreal service, Docker, remote durability
attestation, and `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED` are unnecessary for
this local-file shape.

Use a fresh YAML config with `server.host: 127.0.0.1`, `server.port: 8080`,
`security.jwt_required: true`, a one-off `security.jwt_secret`,
`security.jwt_algorithm: HS256`, `security.jwt_issuer`, and
`security.jwt_audience`; keep the secret out of command output and reports.
`src/config.rs::Cli` accepts `--config`/`CONFIG_FILE`, and `src/main.rs::run_main`
starts the server when no subcommand is selected. The config validation in
`SecurityConfig` requires a deliberate secret when JWT is required. A valid
JWT's source-defined claims are in `src/uar/security/claims.rs::UserClaims`;
the existing release acceptance helper `scripts/release/uar_1_1_acceptance.py`
shows the repository's HS256 signing convention and claim set. Create two test
JWTs with distinct `sub`, the same configured `tenant_id`, issuer/audience,
`nbf`, and `exp`, signed with the isolated test secret. Feed them through the
acceptance program's two environment-variable names. Do not disable auth.

Provision one persisted agent bound to the fixture provider/model and eligible
for the test MCP server. The actual endpoint is `POST /api/agents`, whose
handler accepts a full `AgentArtifact` (`src/uar/api/discovery.rs::create_agent`);
the artifact shape is concretely represented by
`scripts/release/uar_1_1_acceptance.py::agent_artifact` and the types in
`src/uar/domain/artifact.rs`. Provider config supports `id`, `base_url`,
`protocol`, `default_model`, `models`, and `enabled` in
`src/llm/registry.rs::ProviderConfig`; global config accepts `providers:` in
`src/config.rs`. The test fixture implements `/v1/chat/completions`,
`/v1/messages`, and `/v1/models`; use the configured fixture provider ID and
provider-qualified Anthropic model. No actual provider credential is needed for
the local fixture.

For optional mixed host/caller coverage, write an MCP config file and point
`MCP_CONFIG_PATH` at it. `src/mcp/config.rs::McpServerEntry::RemoteHttp` takes
`url`; the exact fixture destination is `http://127.0.0.1:8099/mcp`. The agent
must select that registered server and the acceptance driver's
`--host-tool-json` must contain its actual namespaced advertised UAR tool name
and valid arguments. The fixture advertises raw tool `effect`, records every
real `tools/call` in `_trace.host_effects`, and has no arbitrary command
execution. Source confirms `MCP_CONFIG_PATH` resolution in
`src/mcp/registry.rs::resolve_mcp_config_path`.

## Proposed commands (not run)

Run only after the operator opens the gate and supplies the exact command list.
Use a per-run temporary directory and unique `surrealkv` database path. Fixture
command, with no test or candidate startup side effect in the acceptance script:

```bash
python3 tests/fixtures/client_tools_provider.py --host 127.0.0.1 --port 8099 --port-file /tmp/uar-client-tools-provider.port --trace /tmp/uar-client-tools-provider.jsonl
```

Candidate process shape, source-derived from `Cli`, `src/main.rs`, and config
loading (the selected YAML contains the isolated auth and provider values):

```bash
MCP_CONFIG_PATH=/tmp/uar-client-tools-mcp.json target/uar-post-lifecycle-main/debug/universal-agent-runtime --config /tmp/uar-client-tools-config.yaml
```

The minimum caller-tool acceptance command is the script's documented
OpenAI-only matrix recipe; substitute the actual provider-qualified model that
was provisioned, and the actual locally provisioned agent ID:

```bash
python3 scripts/release/client_tools_acceptance.py --base-url http://127.0.0.1:8080 --provider-base-url http://127.0.0.1:8099 --bearer-token-env UAR_ACCEPTANCE_TOKEN --other-bearer-token-env UAR_OTHER_ACCEPTANCE_TOKEN --agent-id <provisioned-agent-id> --openai-model <provisioned-agent-id> --anthropic-model <provider-id>/<model-id> --protocols openai --mode matrix --report /tmp/uar-client-tools-openai.json
```

That matrix runs OpenAI JSON and SSE caller loops, repeated handoffs, choice,
parallel control, concurrent identical result submissions, disconnect-held
continuation, invalid/truncated upstream streams, and rejection mutations. It
will explicitly report missing mixed host/caller setup and separate
restart/budget/graph gates as coverage gaps unless corresponding options/modes
are provisioned. The release doc and acceptance handoff document these limits.

## Concrete blockers to calling this a complete five-interface gate

- No current local candidate config, isolated database path, provider config,
  agent artifact, MCP config, or two signed test tokens was supplied or created
  in this research task. Their existence and values remain unverified.
- `POST /api/agents` schema is a whole `AgentArtifact`; creating a valid
  execution-ready agent still needs selected provider/model and applicable
  policy. Provider settings endpoint is mounted at `/api/providers`, but no
  payload has been issued or verified here.
- The fixture's `tools/list` exposes `effect`, but the executable name UAR
  exposes to a configured agent is namespaced and depends on registry
  projection. The acceptance script requires the exact actual name in
  `--host-tool-json`; do not guess it. Mixed-host coverage remains pending
  until operator provisioning captures it.
- The acceptance driver accepts the two bearer-token environment variable
  *names* and checks the values differ. This task has not verified how tokens
  will be issued for a fresh local instance. The repository's API-key exchange
  is `POST /api/uar/auth/exchange` (`src/uar/api/auth.rs::exchange_key`) and
  requires already provisioned keys; token issuance therefore remains an
  operator/configuration step unless a separately authorized local fixture
  provisions claims using the source-supported HS256 verifier.
- One `--mode matrix` invocation does not cover cold restart, cumulative
  budget, graph continuation, or mixed host/caller unless separately configured.
  Cold restart must preserve the same embedded DB path and peer process across
  externally supervised stop/join/restart; this script does not supervise it.
- The task's goal mentions all five interfaces, but the smallest recipe above
  deliberately proposes only OpenAI as a first real-path gate. For all-five,
  add `--protocols openai anthropic agui a2a acp` only after each endpoint is
  configured and route semantics are checked against its actual source contract
  in `docs/releases/v1.2.1-kubo.md`. In particular ACP's path comes from config
  (and defaults via its endpoint config), A2A can need configured route and
  metadata, and Anthropic needs its separate provider-qualified selector.

No service, acceptance program, import, test, script, build, readiness check, or
other verification was run in this task. No production files were changed.

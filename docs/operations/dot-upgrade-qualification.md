# Actual prior-to-current provider upgrade qualification

`scripts/dot/upgrade_journey.py` authors the `run_upgrade(profile, evidence)`
entry point already selected by the Dot17 `upgrade` stage. It has not been
executed. The full expanded phase implementation boundary must be open before
launching it. It starts the actual prior installed binary, changes its durable
settings using production APIs, joins its orderly shutdown, and starts the actual
current binary over the same isolated stores. A current-binary restart alone
cannot satisfy the upgrade claim.

The uncomfortable limit: the prior release must expose compatible provider,
settings and default-inference routes and accept the retained private config.
Missing routes or incompatible contracts leave the upgrade blocked with actual
HTTP evidence and process logs. The fixture never repairs incompatibility by
seeding state with the current binary or writing protected database rows. Prior
release publication metadata and an independently installed prior binary are
also necessary; a caller-supplied version label or digest alone does not prove
that the executable is an earlier release.

## Configuration

Each storage profile contains `upgrade` with:

- `prior_host` and `current_host`: ordinary `acceptance.Host` configurations.
  Both use `lifecycle: "child_process"`. Their actual binary paths and SHA-256
  digests must differ. Config path, working directory, base URL, workspace,
  explicit environment, inherited environment names, credentials and settings
  admin-key reference must be exactly equal. The same private config file is
  hashed again before each transition; it cannot seed or repair the selection.
- `prior_publication`: the existing `publication.installed_identity` contract.
  Supply the actual HTTPS `release_manifest_url`, `tag`, `source_revision`,
  `installed_binary`, `archive_name`, `downloaded_archive`, `image_index_url`,
  `image_manifest_urls` for both architectures, and optional registry token
  environment reference. The existing reader independently fetches published
  manifest/index bytes and compares archive and installed executable digests.
  Its source revision must differ from the current phase source. The prior
  executable must be installed outside the source checkout.
- `fresh_data_root`: a new private absolute directory; preexistence blocks the
  run rather than deleting data. The fixture creates it with mode 0700. For
  local SurrealKV, the explicit absolute database path must be inside this root.
- Optional `provider_ports`: two ports for the real controlled upstream peers;
  the default `[0, 0]` allocates two actual loopback listeners. The exact allocated
  endpoints are persisted through the provider API and stay alive across both
  binaries and the final current-binary restart.
- Optional `fixture_model_ids`: two distinct model IDs understood by the chosen
  installed drivers; default `gpt-4o-mini` and `gpt-4o`. The controlled peer records
  the exact requested model. Its text result is deterministic; it does not
  substitute for host admission, persistence, routing or run execution.

The private runtime config must be JSON with no preconfigured `providers` and an
empty `llm.model`. Set `persistence.provider: "surreal"`. The local profile uses
`surrealkv://` with an absolute path inside `fresh_data_root`; the shared profile
uses a real network SurrealDB endpoint and a dedicated `surreal_ns` beginning
`dot17-upgrade-` with explicit `surreal_db`. Allocate that namespace uniquely and
keep every other service off these stores for the case. Both launches share its
same credentials, namespace and database. This stage intentionally runs one
writer at a time even in the shared-store profile; simultaneous multi-runtime
semantics are covered by separate profile cases.

Keep optional memory capture disabled or point any enabled memory store into
the same isolated installation. Provision all other configured paths and secrets
for this dedicated case, including persistent credential encryption authority.
Do not use operator/customer stores. Provider/model/persistence `UAR_*`
environment overrides are refused so the input config describes the tested
selection and storage. Other environment prerequisites remain those required by
both installed binaries. No runtime crypto or auth settings are weakened.

The current executable must be the profile's actual phase-candidate host binary.
When the runner carries independently installed current publication evidence,
the current upgrade executable path and digest must match that publication.
The fixture verifies both actual binary digests before their launches through
the normal `Host` lifecycle. It does not build or download an executable.

## Observed sequence

1. Prove the prior publication and distinct prior/current executable identities.
   Start two existing controlled `Provider` peers on real loopback sockets. Their
   full received wire messages are fsynced into private evidence traces.
2. Launch the actual prior binary and require its provider registry to be empty.
   Create providers `dot17-upgrade-a` and `dot17-upgrade-b` using
   `POST /api/uar/providers/`. Each has explicit `protocol: "chat"`, a different
   model, its actual peer endpoint, an enabled model and a fixed fixture-only API
   credential. This string has no authority outside the controlled peers.
3. Verify both real configured routes through `POST /api/uar/providers/{id}/test`.
   Select A with `POST /api/uar/providers/{id}/default`, inspect saved provider
   settings, live registry default and `/api/uar/resolve-model`, then perform
   actual default inference. Repeat for B and deliberately save the qualified
   B selector in `llm.model` through the settings API.
4. For every inference request, omit explicit model/provider selection, run
   credentials, artifact policies and agent overrides. Submit a unique text
   marker through `/v1/chat/completions`. The returned completion must contain
   it, the selected peer must record the exact intended model, and the other
   peer must record no request carrying that marker. A settings readback alone
   cannot satisfy routing.
5. Stop and join the prior process with exit code zero. No forced termination or
   crash-recovery fallback is permitted. Start the distinct current binary on
   the identical configuration/stores. Before any corrective provider mutation,
   require the saved B provider/default/model selection, live registry default,
   resolver and current `/api/uar/settings/effective` aggregate to agree.
6. Reverify both migrated providers using real inference. This may legitimately
   replace model-verification evidence invalidated by the upgrade; it does not
   rewrite the migrated provider records or repair the default/model selection.
   Observe actual default routing to B.
7. Switch A and B live on the current process through the provider default API.
   Each switch must change the aggregate revision and produce the intended
   provider/model in the aggregate and in actual peer traffic. The saved global
   `llm.model` remains deliberately B while A is temporarily live, exercising the
   provider registry's current-default precedence rather than a config fallback.
8. Join an orderly current-binary stop, restart it on the same unchanged stores,
   then require B's saved choice, effective aggregate and actual routing again.
   This final persistence check supplements the preceding genuine upgrade;
   it is never reported as the upgrade itself.

## Contracts and limits

Current source mounts provider routes at `/api/uar/providers` in `src/server.rs`.
`ProviderConfig` in `src/llm/registry.rs` names `default_model` and `models`.
The default-provider mutation durably writes `llm.default_provider` through
`SettingsManager` before changing the registry. Provider records live under
`provider.{id}`; startup hydration reads them and the saved default. The aggregate
reads the live registry and reports `llm.provider_id`, `llm.model`,
`llm.provider_host` and a content-derived `revision`.

For historical settings metadata, the helper accepts either `saved` or the
retained `data` field and sends `expected_revision` when that prior API exposes
it. The current API must expose its revision. Other incompatible prior APIs
block; no alternative arbitrary path/payload adapter is accepted. The new
aggregate is asserted on the current binary, not required to exist in an older
release. The fixture separately proves the prior selection through its saved
settings, registry/resolver and actual wire routing.

Only source reading and authoring occurred. No Python invocation, syntax check,
formatter, test, build, process launch, HTTP request, package retrieval or product
review was executed. Publication availability, prior API compatibility, migrations,
credential persistence, aggregate revisions and actual routing remain unverified.
No production source, dependency, config key, API authority, migration or task
checkbox was added or changed. Identity, isolated-store, joined-stop and routing
guards address the named false-upgrade and wrong-default failure scenarios.

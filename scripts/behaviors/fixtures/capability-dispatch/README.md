# Capability fixture preparation — authored only

All feature/step/peer changes are NOT_EXERCISED. Verification is CLOSED. This
folder owns synthetic data only; no real visual fact, user image or download is
included. `generate-png.py` writes a 2x2 RGB PNG from explicit pixel constants and
a structurally complete 6200x6200 zero-pixel PNG (115,320,000 decoded RGB bytes)
for the 100 MiB allocation refusal case. It uses Python standard-library PNG
chunk/CRC/zlib writing, without importing application or scenario code.

Preparation commands actually used (asset writing, no acceptance credit):

```
python3 scripts/behaviors/fixtures/capability-dispatch/generate-png.py
/usr/bin/sips --setProperty format jpeg scripts/behaviors/fixtures/capability-dispatch/synthetic.png --out scripts/behaviors/fixtures/capability-dispatch/synthetic.jpeg
```

`sips --help` and `sips --helpProperties` supplied the actual conversion flags.
Only our synthetic PNG was converted. No codec/dependency was installed or
fetched. Positive decoder acceptance and negative allocation refusal have not
been observed. The retained JPEG is the fixed oracle; do not regenerate it during
a scenario. The runtime helper independently hashes retained asset bytes.

## External provisioning manifest

The lead/external host owns provisioning and final identity freeze. Scenarios do
not launch/restart containers, provision providers, probe models, or create/copy/
send credentials. `UAR_BASE_URL` must explicitly name the owned external identity
endpoint; `UAR_WORKSPACE_ID` selects its nonsecret workspace. The external host
supplies `UAR_CAPABILITY_FIXTURE`, a local path to credential-free JSON:

```
{
  "namespace": "capability-<generated UUID>",
  "identity": {
    "platform": "linux/amd64",
    "sourceCommit": "<full 40 lowercase hex committed Git SHA>",
    "imageDigest": "sha256:<64 lowercase hex local Docker image ID>",
    "configDigest": "<64 lowercase hex nonsecret configuration digest>",
    "contractDigest": "<64 lowercase hex recomputed adopted contract digest>"
  },
  "failover": { "enabled": true, "strategy": "priority", "error_threshold": 3, "cooldown_secs": 60,
    "fallback_models": [{ "model": "capability-<namespace>-outside/outside", "base_url": "http://127.0.0.1:4612/v1" }] },
  "peers": [
    { "id": "capable", "url": "http://127.0.0.1:<owned-host-control-port>/",
      "wireBaseUrl": "http://127.0.0.1:<same-owned-peer-shared-namespace-port>/" },
    { "id": "fallback", "url": "http://127.0.0.1:<second-owned-host-control-port>/",
      "wireBaseUrl": "http://127.0.0.1:<second-owned-peer-shared-namespace-port>/" }
  ],
  "models": {
    "capable": {
      "destination": "capability-<namespace>-capable/<wire-model-id>",
      "peer": "capable", "wireModel": "<actual configured wire alias>",
      "provider_enabled": true, "enabled": true,
      "supports_vision": true, "supports_tools": true,
      "supports_structured_output": true,
      "context_window": 32768, "max_output_tokens": 256
    }
  }
}
```

This is a schema example, not a runnable registration or asserted candidate
identity. The helper requires a full40hex Git source commit, `sha256:` plus64hex
for `imageDigest`, and64hex config/contract digests; placeholder text is rejected.
For this local `buildx --load` lane, `imageDigest` means the **local Docker image
configuration ID** used to identify the exact loaded image/container. A registry
manifest digest identifies a different object and must never be substituted or
reported as that image ID. No registry publication is implied. The lead retains
the independent local image/container/source/config identity receipts; formatting
validation alone cannot prove which process serves an endpoint.

The helper recomputes the actual adopted file digest as SHA256 of UTF8 JSON with
recursively sorted object keys, compact separators and preserved array order,
excluding only `identity.contractDigest` and root `digestNormalization`. It
requires the declared normalization algorithm/representation, compares the
recomputed value to both the adopted file stamp and manifest contractDigest, and
records the recomputed value. A stale or fabricated matching stamp is insufficient.
No test/digest check has been executed during authoring.

Expand peers/models to **every** role below. Each failure/cooldown role
requires its own provider and peer so health state cannot contaminate subsequent
cases. Other roles may share a peer only if the expected actual model identities
remain distinct; failure primary and fallback must have different peers.

| Model role | Required independently provisioned facts |
| --- | --- |
| capable | Enabled; vision/tools/structured; sufficient window/output; generic-compatible actual adapter |
| text_only | Enabled; supports_vision false; other declarations frozen |
| no_tools | Enabled; supports_tools false |
| no_structured | Enabled; supports_structured_output false |
| tiny | Enabled; context/output capacity small enough to refuse protected request; no successful compaction alternative |
| disabled | Registered provider or model disabled; namespace remains owned |
| cooldown | Dedicated provider; initial healthy; real 503s establish cooldown under the actual frozen failover settings |
| failure_primary | Dedicated primary that returns controlled 503 before output; vision capable |
| fallback | Distinct capable provider/model; smaller own output cap; `wireSettings` contains independently configured expected safe scalar fields (`temperature`, `max_tokens`, `max_completion_tokens`) |
| failure_exact | Dedicated primary for exact-target outage; a policy fallback exists but must never send |
| failure_scope | Dedicated primary with incapable captured fallback; global capable outside candidate must never rescue |
| outside | Enabled capable actual GLOBAL fallback at port4612, absent from each restricted agent's nonempty captured policy; prove it serves an unrestricted control first |

The initial registry inspection requires exactly the frozen manifest provider set, every registration keyless and every base URL an owned controlled peer. This prevents a routing defect from escaping into a real model. All twelve model roles are required, including outside, and failure/cooldown roles require unique providers and peers.

The registration inspector reads existing `GET /api/uar/providers/{id}` and
requires `credential_configured:false`, exact safe capability fields, and the
owned peer base URL. If the external endpoint's administrative read policy
refuses, report a provisioning/read-authority prerequisite; tests never add an
admin key or weaken production boundaries. The external host's configuration
receipt establishes failover settings; the cooldown case also reads actual
`GET /api/uar/providers/health` and requires a positive remaining cooldown.

The selected topology is strictly local. `UAR_BASE_URL`, every peer control URL,
and every registered wire base URL must use literal `127.0.0.1` or `[::1]` with
HTTP/HTTPS and no credentials, query or fragment. Public domains, arbitrary remote
HTTPS endpoints, private-network addresses, and DNS names are refused by the
helper; declaring a keyless peer URL does not establish ownership. `redirect:error`
prevents a local endpoint from redirecting a request elsewhere.

The peer still defaults to127.0.0.1. A **new explicit fixture-only**
`--bind-host` choice accepts127.0.0.1 or0.0.0.0; the selected owned-container lane
uses0.0.0.0 **inside that container network namespace only**. Its UAR wire URLs
remain literal container-loopback127.0.0.1. The runtime overlay now sets
server.host0.0.0.0 inside the same isolated container for externally provisioned
host-proxy access. No JWT/admin/remote-identity setting is changed.

The external owner publishes only literal **host127.0.0.1** mappings for API1906
and each peer4601..4612. Thus a host-side client/control URL127.0.0.1 reaches the
peer's container-interface listener, while the server uses its own loopback to
reach that same peer. Host/control and internal wire port numbers must match the
actual frozen mappings; if different host ports are chosen, only manifest peer
control URLs change. Do not start a host process bound0.0.0.0 for this fixture,
expose a public host bind, reuse another chat's resources, or infer reachability
from EXPOSE or a port number. Default127-only peers are **not** exposed by ordinary
container-interface NAT. This explicit listener mode removes any need to invent
a loopback forwarding service. An alternatively externally provisioned client
inside the same owned network namespace may use the unchanged127 listener.

Exact container/image/namespace/labels/ports/volume/proxy/peer-process identities
remain external prerequisites. No Docker/config/startup/readiness command has
run here, no third-party resource is changed, and neither the bind option nor
manifest implies public deployment, credentials or a paid model call. Never
inspect/log Docker Config.Env or restart another chat's service.

## Evidence and cleanup

The dedicated Cucumber config uses the installed runner, serial execution and
retry0. JSON report path is selected by `UAR_CUCUMBER_REPORT`; per-case safe
receipts are written with exclusive creation under
`.prometheus/verification/capability-dispatch/`. They contain hashes/counts,
identity, safe observations and statuses only. Raw requests/responses, headers,
data URLs/base64 and private sentinels are not attached. All peer HTTP chat
arrivals count before body parsing, so a peer-side 1 MiB body rejection cannot
masquerade as zero provider sends. The bounded observation ring must retain the
entire case or the case fails.

Scenarios create fresh agent IDs and sessions only. They delete those exact
agents and owner-scoped sessions, reset their own peer controls, and fail on
cleanup uncertainty. Peers/providers/container/volume are externally provisioned
fixtures; their final teardown belongs to the lead's owned lifecycle record,
not the scenario's authority. An interrupted run must reconcile those owned
records before retrying; retry0 is intentional.

The finite-cost CD-09 case uses existing agent `extensions.budgets.max_cost_per_session_usd:1`, requires a real successful text control under that same agent/policy, then requires image refusal before send. Provision usable existing price/accounting facts for the capable model; an unrelated blanket text refusal cannot count as image-policy evidence. Scenarios do not mutate provider pricing.

CD-08 and strict real-image CD-09 have unresolved source-supported interfaces,
detailed in the phase scenario-to-assertion map. A blocked prerequisite cannot
be counted as wire, checkpoint, budget, or real-model acceptance.

## Source-backed twelve-role operational templates

New files `runtime-overlay.template.yaml` and `manifest.template.json` spell out
all twelve actual provider/model roles, with one loopback peer per role at
ports4601..4612. They are **authored, not exercised**, and deliberately contain
unresolved `__CAPABILITY_NAMESPACE__` and exact-identity placeholders. They are
not a standalone bootable deployment and do not create a container or identity
endpoint. The external owner chooses a fresh `capability-<UUID>` namespace and
renders both files consistently; the application config loader itself does not
expand these placeholder strings. No templating command, new runner or service
launcher is provided.

`runtime-overlay.template.yaml` contains JSON, which is accepted by the existing
YAML config path (`scripts/release/client_tools_local_gate.py::configuration`
already uses this representation). The application accepts the explicit existing
`--config` / `CONFIG_FILE` path (`config.rs::Cli` / `AppConfig::load_with_cli`).
The external owner must merge these nonsecret operational fields into the
already-provisioned trusted host's secure configuration and retain that merged
configuration's nonsecret identity receipt. This is an overlay **by ownership
contract**, not a new application merge flag or an executable merge mechanism.

| Template fields | Actual source and intended behavior |
| --- | --- |
| server.host/port/grpc_port/ingest_directory/cors_allowed_origins | `config.rs::ServerConfig`; container-interface API1906 behind host-loopback-only publication, existing A2A50051 not required to be published, owned data directory |
| persistence.provider/database_url/surreal_ns/surreal_db/vector_dimension/external_cache_enabled | `config.rs::PersistenceConfig`; root Dockerfile's embedded owned SurrealKV store path,384dim local assets, no DB credentials |
| models.models_dir and llm.embedding backend/model/models_dir/vector_dimension | `config.rs::ModelsConfig/EmbeddingBackendConfig`; existing fastembed local model assets copied by root Dockerfile into /opt/uar/models; `embeddings/fastembed.rs` resolves local files and fails when absent rather than requiring a hosted key |
| llm.model/base_url/protocol/max_retries/timeout_secs | `config.rs::LlmConfig`; owned primary, generic-compatible chat path, no provider retry inflation |
| providers[].id/display_name/base_url/protocol/default_model/models/enabled | `llm/registry.rs::ProviderConfig`; actual config seeding overwrites the matching primary seed with complete owned model declarations |
| models[].pricing_identity.provider_id/model_id | `llm/registry.rs::ModelPricingIdentity`; known existing openai/gpt-4o-mini catalog identity, also used by the source-existing caller-tool fixture; metadata for finite-cost liability only, never an external inference destination |
| models[].context_window/supports_vision/supports_tools/supports_reasoning/supports_structured_output/supports_streaming/max_output_tokens/enabled | `llm/registry.rs::ModelConfig`; explicit role differences, tiny128token context, fallback8192context/32output; no invented custom policy/profile fields |
| failover.enabled/strategy/error_threshold/cooldown_secs/fallback_models | `config.rs::FailoverConfig/FailoverStrategy`; enabled priority, threshold3, cooldown60; global fallback list contains the real outside role at127.0.0.1:4612/v1; nonempty artifact lists must override/restrict it |
| memory.enabled/db_path and sycophancy.enabled/auto_correct | actual `MemoryConfig/SycophancyConfig`; no memory capture or corrective extra model pass in this controlled wire fixture |

The committed `catalog/provider_catalog.json` is copied into builds by
`build.rs::copy_provider_catalog_snapshot`; `ModelPricingIdentity::qualified_model`
requires a catalog price. Templates never copy a price number or invoke OpenAI:
all twelve configured model endpoints remain literal loopback controlled peers.
Catalog pricing is accounting provenance, **not image counting or fit evidence**.
No `api_key`, `api_key_env`, token, JWT, admin key or signing field is in either
new template. Do not fill such fields into these files.

The root Dockerfile already carries Python/Node and copied local model assets.
The selected source-authored topology keeps server and peers in one owned
container network namespace, uses explicit fixture --bind-host0.0.0.0 for peer
interface listeners, and publishes control/API ports only on host127.0.0.1.
Internal provider and real global fallback URLs remain127.0.0.1. The host-side
external identity proxy remains responsible for its existing verified admission;
listener/interface reachability does not replace identity. Client-inside-namespace
with default127 peers remains an alternative if externally provided. None of
these process/listener/mapping arrangements has been launched or observed here.

### Exact remaining external prerequisites

1. A deliberate, already-provisioned trusted identity/configuration boundary.
   `SecurityConfig::validate` rejects the built-in fallback secret when JWT is
   required and requires an admin authority when settings administration is
   protected. These templates deliberately omit **all security fields**; they
   cannot boot against defaults alone, and do not disable JWT/admin admission.
   The external owner keeps its secret-bearing secure baseline outside scenario
   artifacts and supplies the existing loopback external identity endpoint.
   Tests never mint, renew, copy or send credentials. No auth implementation or
   secret migration is requested by this template.
2. That endpoint must preserve one stable verified owner/tenant/workspace and
   allow the scenario's existing nonsecret API operations under external host
   authority: registry/health and effective /api/uar/settings/llm-failover reads, default-agent read, disposable-agent create/
   delete, completion, owner-scoped run/event inspection and session deletion.
   Provider reads are protected by `providers.rs::provider_admin_boundary` when
   enabled. Missing read authority is an external provisioning obstacle; tests
   must not solve it by adding X-UAR-Admin-Key, Authorization or a login step.
3. One wholly owned, fresh Surreal volume/settings namespace. Startup seeds
   registry from llm then providers, but later persisted settings can hydrate
   different providers (`server.rs` initialization). A prior operator volume is
   therefore not a disposable synthetic registry. The scenario requires exactly
   the declared keyless provider set; it will refuse an extra live registration.
   External launch environment must not import provider-key shortcuts, different
   LLM selections or unknown settings into this isolated runtime. No private
   environment values are read or attached by scenarios. The existing optional
   `UAR_LOCAL_ONLY=true` guard is source-supported in `llm/local_only.rs`; the
   external owner may include it as its scoped egress setting, without using it
   to replace the manifest's actual local endpoint assertions.
4. Frozen source40hex, local Docker image ID, Linux/amd64 artifact, owned
   container/labels/ports/volume/peer-process identities, actual merged nonsecret
   configuration digest and recomputed current contract digest. Template
   placeholders are not evidence and the scenario helper rejects them.
5. All twelve controlled peer processes with the exact amended Python fixture
   and owned output paths. Existing parser arguments are `--port`, `--port-file`,
   `--receipt-file`, `--dimensions`; the new explicit `--bind-host` selector
   defaults127.0.0.1 and is0.0.0.0 only for the selected owned-container lane.
   Use role ports4601..4612 and dimensions384.
   Each process must be independently owned and externally managed. This is a
   parser contract description, not a service launch or readiness command.
6. The actual installed HTTP Cucumber dependency/source/asset mount for the client
   must be provided by the external owner. Root runtime image shipping Node does
   not imply that repository node_modules or new feature assets are present.
   No install, dependency import, runner discovery or readiness command ran.

### Implemented registered fallback-output interface

The authored CD06 assertion requires the capable fallback's own32token output
ceiling and `wireSettings.max_completion_tokens:32`. The operational template's
actual `ModelConfig.max_output_tokens:32` now has a source-present final send
restriction; no special team execution_profile or destination budget API is
needed to express that scalar ceiling.

Source inspection after the provider/runtime handoffs found:

- `router.rs::ModelRouter::evaluate_destination` returns the currently registered
  optional output limit after live enabled/health/capability evaluation.
- `DestinationAdmission::admit` retains it in `AdmittedDestination`.
- `AdmittedDestination::restrict_output` takes the minimum with the prepared
  typed request's output_token_cap and, if already present, the existing budget
  contract's limits.output_tokens. It cannot raise a bound or create a contract.
- `orchestrator.rs::admit_prepared_user_attempt` applies it to actual primary/
  retry and fallback prepared requests before opening the driver stream.
- `manager.rs::DestinationBudgetDriver::stream` applies the same admitted result
  to graph requests before forwarding to inner driver/accounting collaborators.
- `liter_driver.rs::build_chat_request` writes max_completion_tokens for this
  generic non-Anthropic path, so the fixture's expected scalar wire field is
  `max_completion_tokens:32`, without any temperature/custom-profile setting.

This resolves the earlier source-discovered public configuration obstacle. It is
an implemented scalar registered output restriction, independent of existing#388
synthetic exact `DestinationRequestPreparations`. It does not install an exact
image-counting profile, supply a full-image token count or certify fit. Existing
exact preparations remain authoritative and can only be tightened by the scalar
restriction. The actual32token peer observation is still NOT_EXERCISED; no source
inspection is substituted for the eventual real-path assertion.

Item7 authoring is complete with both operational templates. External secure
identity/read authority, exact candidate/container/store/process/config identities,
shared loopback namespace, local assets and installed scenario-client dependencies
remain provisioning prerequisites. They are not pending fixture implementation,
and neither template creates or qualifies them. No checks/imports/hash execution,
Docker/service/config activation or model call ran while authoring these files.

## Review repair: bounded client lifetime and uncertain effects

The focused Before now owns one100second monotonic case deadline, shared by
**all** UAR requests, peer status/control requests and complete response-body
reads. Twelve serial slow peer responses consume that same budget rather than
receiving twelve fresh per-request budgets. The After aborts/closes and joins
owned case HTTP/body promises before using a new100second cleanup controller.
Cucumber's installed Before/After timeout option and180second wrapper remain the
outer limit; raising that number alone is not the isolation mechanism.

Local fixture reads and receipt mkdir/write calls are synchronous and checked
against the owning deadline before/after, so there is no asynchronous filesystem
promise left behind a timed-out callback. A blocked local filesystem call can
still delay this process; synchronous I/O does not certify an OS deadline, but
Cucumber cannot advance concurrently while it blocks. Installed asynchronous
fsPromises declarations describe cancellation as best effort, so the fixture
deliberately does not rely on that as proof of joined physical filesystem I/O.
No filesystem/import/timeout probe has been executed during this repair.

Aborting/joining a client request **cannot** prove that the server stopped a
control POST or agent creation after request bytes left. Any deadline, uncertain
HTTP outcome or incomplete cleanup poisons this focused fixture module. Every
subsequent Before refuses before initialization/configure/record changes. After
may attempt bounded deletion of exact known owned IDs, but skips peer reset when
poisoned and retains original agent/session IDs, confirmed delete responses and
unresolved identities for external reconciliation. A404 after an uncertain create
is not promoted to proof that a later create cannot finish. The latch is never
cleared within the run; reconcile owned effects before starting a new process.
This mechanism is local to these scenarios, not a global framework or production
policy. No automatic retry is introduced.

## Review repair: actual estimated-accounting oracle

CD09 must now observe an actual uar.request-budget.v1 **final_provider_request**
receipt for its independently observed image-bearing wire model. A false fit flag
on uar.attempt-manifest.v1 alone cannot satisfy it. It requires actual approximate
count quality/revision, serialized_request, output_ceiling_enforced, positive
counted/serialized facts,64hex serialized digest, tools/instructions included,
reserved_output_tokens null and fit_guarantee false, all from the literal existing
Liter receipt schema. No missing image/contract flags are manufactured. Its same
comparator rejects removing only final-wire receipts while preserving actual
attempt manifests and image observations. This is JSON-wire accounting evidence,
not complete image token accounting, visual perception or fit certification.

## Review repair: observational compatibility

The Python peer's new observer normalizes explicit tools:null to an empty
observational list. Malformed non-list tools/tool entries/content/image metadata
produce fixed invalid_shape summaries instead of instrumentation exceptions. It
does not rewrite the request body or replace the configured success/503/tool mode.
The lead owns later actual null-tools success/503 and slow-peer lifetime probes;
none ran here. Existing source reviews used fresh native contexts in the same
model family, which is weaker than independent-family review. All changes remain
NOT_EXERCISED until the renewed whole-phase boundary and authorized gate.

## Corrective review P2: persist ownership before cleanup

After joining case client I/O, the hook now synchronously creates an exclusive
`<case-uuid>.ownership.json` with0600mode and the installed writeFileSync flush
option, **before** starting cleanup's100second lifetime or any cleanup request.
It contains only safe case/scenario/identity/manifest/namespace/workspace anchors,
exact original owned agent/session IDs, initial isolation state, the planned
terminal receipt path and an explicit cleanup-pending/reconciliation disposition.
The normal terminal receipt references this record.

This ownership record is immutable. Cleanup deadline exhaustion, an uncertain
creation, a DELETE404, a failed peer reset, or inability to publish the terminal
receipt cannot erase its original IDs or turn pending reconciliation into proof
of absence. A missing/failed terminal receipt means reconcile the retained record;
only the actual terminal/Cucumber evidence can establish completed cleanup.
If even the pre-cleanup synchronous write fails, the hook poisons the fixture,
attaches the safe ownership data where runner output remains available, starts
no cleanup requests, and reports persistence failure. It cannot promise durable
storage when the filesystem itself fails. Synchronous I/O may still block the
process; no hard OS deadline, crash/power-loss recovery or actual lifecycle result
is certified. The existing bounded cleanup and poison latch remain unchanged.

The corrective finding report is retained. This source repair was not followed
by another review cycle or any execution/check/import/probe. Ownership persistence
and cleanup-timeout behavior remain NOT_EXERCISED until the renewed phase gate.

Before starting cleanup after this synchronous write, the hook also requires its
complete100second cleanup lifetime to fit inside the remaining180second outer
hook time. If prior join/ownership I/O consumed that margin, it poisons and starts
no cleanup request; the already-written ownership record remains pending. Pending
case client work at hook entry is marked uncertain before join. This prevents a
slow ownership write from starting a fresh remote-cleanup budget after the outer
wrapper has effectively expired; it does not promise to interrupt blocked OS I/O.

## Real global fallback scope control — authored source cohort

Both templates now configure the existing actual FailoverConfig.fallback_models
entry `{model:"<namespace>-outside/outside",base_url:"http://127.0.0.1:4612/v1"}`,
with no key, rather than leaving outside only as a registry candidate. Manifest
failover fields use existing enabled/strategy/error_threshold/cooldown_secs/
fallback_models names. Initialization reads existing GET
`/api/uar/settings/llm-failover` and compares each authoritative row's effective
value/application_status to the frozen manifest; safe effective values and row
revisions enter evidence. Existing external settings-read authority is required;
scenarios never add an admin credential or mutate settings. Saved/pending rows
cannot stand in for the selected effective global chain.

Before CD06 positive and scope/exact negative runs, an explicit unrestricted
control uses a fresh disposable agent with an **empty** artifact fallback list
and its own freshly tracked session. One controlled primary failure must actually
reach the configured global outside peer and return the useful canned completion
with exact image parts. The original restricted agent/session/nonempty fallback
list are restored in finally before the real assertion. Control history and
conversation policy cannot contaminate the restricted run. The control's IDs
are tracked before their calls and included in immutable ownership/normal and
uncertainty receipts; cleanup visits **all** owned sessions and agents.

Role reuse is bounded: failure_primary occurs once in the positive case,
failure_exact once in the exact-negative row and failure_scope once in the
incapable-fallback row. Each dedicated provider must have no prior failures; after
its one global control health must show exactly one failure, still healthy and
below the actual threshold3 before the restricted attempt. The other cooldown
role remains separate. Global controls are not retried, and llm.max_retries0
bounds provider establishment retries (`orchestrator` uses its minimum with the
resilience attempt bound). If the old path ignores the nonempty artifact list
and uses global outside, positive CD06 cannot observe its required capable
captured fallback and the scope negative cannot remain failed/zero-outside; both
assertions fail independently of merely having an outside registration.

All twelve peers and existing image/scope/output/cleanup/privacy assertions
remain. Real global-chain controls, interface binding/publication, settings
provenance and all runtime outcomes are NOT_EXERCISED. No checks/import/hash/
compile/build/Docker/service/network probe was run for this source cohort.

Global-control wire identity is source-traced, not guessed from route spelling:
legacy FallbackModel.model remains namespace-qualified, and Liter build_chat_request
initially copies that value. The pinned vendored client constructs its generic
OpenAiCompatibleProvider with that model_hint; hint_provider_name captures the
first segment, strip_model_prefix removes exactly that matching segment, and
prepare_request writes the resulting bare model into the wire body. Therefore
this selected global route namespace-outside/outside serializes as outside. The
control asserts the actual peer model hash against this derived alias and frozen
outside.wireModel, retaining both qualified routing identity and wire hash in
evidence. The captured registered capable fallback's existing alias assertion is
unchanged. No public identity/capability key or comparison was weakened. This
serializer trace is source inspection of vendor/git/liter-llm, not execution.

## D2 source audit: both establishment retry layers

The one-failure control is source-reachable through **both** existing retry
layers, rather than relying on an assumption about Liter alone:

1. `config.rs::build_client_config` passes llm.max_retries into the Liter client;
   ProviderRegistry's inherited LLM settings retain the global0 when resolving
   the actual primary. The existing overlay keeps llm.max_retries0.
2. `orchestrator.rs` computes UAR establishment attempts as
   `retries_enabled ? max(1, min(policy.retry_max_attempts, llm.max_retries+1)) : 1`.
   Therefore global llm0 also limits ordinary UAR attempts to1. The overlay now
   explicitly sets real ResilienceConfig fields retries_enabled:false and
   retry_max_attempts:1 as its independent host setting, without inventing an API.
3. `manager.rs` resolves persisted next-turn resilience through
   settings/persisted_resilience.rs and passes it into with_resilience_policy.
   That resolver loads existing global resilience.* settings and can apply a
   persisted agent_config.<id>.resilience override. Even an enabled override
   cannot raise ordinary UAR attempts above the llm0+1 minimum. Fresh owned IDs/
   store remain prerequisites; no per-agent settings mutation was added.
4. A controlled503 is a pre-semantic provider opening failure and is normally
   retryable when policy permits; this is **not** a post-output rule that happens
   to bypass retries. Existing is_retryable classification plus retry predicate/
   backoff would repeat it if enabled. The actual source attempt caps prevent
   that repetition, then the configured failover list is considered.

The manifest now includes actual llm.max_retries0 and
resilience.retries_enabled:false/retry_max_attempts:1 fields. Initialization reads
only the three nonsecret existing settings keys through GET
`/api/uar/settings/{key}`, requires exact effective values/application_status,
and retains effective values/revisions in normal evidence and immutable
pre-cleanup ownership. The external identity endpoint must provide its existing
settings-read authority; no credentials, namespace write or schema field is
introduced. Runtime control requires exactly one observed primary HTTP request,
then exactly one health failure while still healthy/below threshold; it cannot
accept hidden primary retries as the intended control. Existing global fallback,
actual aliases, all12 peers and other assertions remain unchanged.

This audit inspected current source/configuration and authored explicit supported
settings only. No retry path, settings read,503 request, check/import/build or
probe was executed; both retry-layer behavior and operational settings remain
NOT_EXERCISED until the renewed whole-phase gate.

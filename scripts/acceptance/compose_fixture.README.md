# Fixture composition contract

Source authored only. Verification is CLOSED. The composer has not been imported,
parsed by Python, invoked, or used to hash installed/source bytes. No example is
execution authority. Do not run it until the operator names the composition step.

`compose_fixture.py --propose` accepts absolute `--bindings` and a new `--output`
to author inert pending inputs/recipes without a final inventory, artifact,
signature or installed-file reads/hashes. `--compose` additionally requires the
actual final `--inventory` and completed build/authority inputs. Both modes are
unrun. They use standard-library
file reads/writes only; there are no subprocesses, imports of runner/case/observer
modules, network requests, package installation, credential resolution, compiler
probes or services. It neither runs the acceptance packet nor opens the gate.

The uncomfortable prerequisite is that real source/model capability, installed
dependency closure and price exposure cannot be manufactured from a JSON schema.
The lead must supply actual approved bindings, the complete final inventory and
the successful approved build receipt. This composer verifies those bytes and
normalizes the supplied contracts; it cannot turn a partial inventory or static
catalog capability into proof of working inference.

## Inputs

`fixtures/bindings.example.json` lists the required sections with null unresolved
values. It is refused immediately because `example=true`, `executable=false` and
`full_closure_approved=false`. Filling arbitrary paths/aliases or flipping one
flag is insufficient: complete final byte inventory, approvals, configured scopes,
models, embeddings, actual build receipt and dependency manifests are required.

`fixtures/bindings.local-draft.json` records only the lead's source-inspected
existing LaunchAgent/service configuration and the actual aliases `gpt-5.4`, `k3`
and `MiniMax-M3`. Static capabilities were read from the pinned Liter catalog and
remain labeled unverified. The lead subsequently confirmed the existing Cohere
profile model/credential reference and applied the user's explicit URL correction
to its OpenAI compatibility base. The draft records documented1024/1024/384
A/B/C model proposals with availability unverified and approval digests null. This
worker copied no secret value, whole profile or current hash and changed no user
configuration. The draft remains incomplete and nonexecutable.

The operator selected existing Liter models on port4000 and existing Cohere
embeddings at `https://api.cohere.ai/compatibility/v1/embeddings`. Model bindings
use the approved existing-service chat route; the embedding listener forwards
unchanged OpenAI-compatible wire to that selected Cohere endpoint. Configured
model vendor URLs are provenance, not gateway bypass routes.
The service itself currently has configured retries3 and cost_tracking=false.
An approved per-attempt charge ceiling therefore must cover the complete actual
gateway retry/fallback exposure; one forwarding attempt is not a claim of one
physical upstream-provider attempt or verified actual spend.

The operator's exact model billing statement is “Subscriptions; no incremental
charge”, scoped to those existing Liter models. The draft therefore proposes
zero incremental per-attempt model reservation, including their internal retry
chain. Cohere's separate exact statement is “Production/pay-as-you-go”; it is not
zero charge. The operator supplied Cohere V3 text pricing of USD0.10 per million
input tokens and authorized two separate ceilings: USD10 Cohere and USD10 OpenAI
embeddings, with no transfers. This is operator-provided pricing evidence; a
conservative maximum full-request input exposure and its charge approval still
remain pending. `spend_pools` has exactly `cohere=10000000`,
`openai_embeddings=10000000`, `liter=0` in microunits. Aggregate cost is at most
20000000; it does not permit one pool to borrow from another. The reconciled seed
must contain all three `spend_pool_consumed_microunits` counters and their sum must
equal its global cost consumption. Binding and charge approval must name the same
pool and limit. Failed or unknown requests remain reserved by the budget daemon.

Current `selected_embedding_services=["cohere"]` keeps the existing A/B/C
denominator. The OpenAI allocation is unused: no OpenAI model is selected and no
third key is copied. An explicit alternative selection additionally names
`openai_embeddings`, requires preparation of `OPENAI_API_KEY`, and binds only
`https://api.openai.com/v1/embeddings` to that separate pool. Cohere stays bound to
its compatibility endpoint and `COHERE_API_KEY`. Spending authorization does not
open the gate, sign a packet, adopt a fixture or verify pricing or availability.
All proposal approval flags remain false until the actual named packet.

The local draft fixes the lead-selected packet root, ports, model roles and exact
source-pinned Postgres/Surreal image names. `packet_recipe` and
`model_role_recipe` are source proposal inputs: `--propose` translates them into
ordinary pending `providers`, budget bindings and backend port fields. Missing
byte ceilings, approvals and artifact identities remain explicit null slots.
`private-inputs.local-draft.json` fixes the same root/ports and Cohere-only key
selection. It remains inert with pending epoch/validity/source digests and exact
preparation approval. Local image availability is unverified and runtime uses
`--pull=never`; an image pull requires its own named preparation approval.

The local proposal's `native_policy_source` names the owned
`fixtures/native-policy.local-draft.json`. Proposal mode reads its inert typed
`native_configuration` before applying explicit local configuration overrides.
It also retains its `source_recipes` and merges declared team settings into
pending `teams`; recipes are never invented native configuration keys. The
actual private preparation security patch is merged only during final composition,
so generated principals and workspace authorities replace no unrelated policy.
This policy and both preparation programs belong to the final source denominator.
Artifact preparation returns `artifacts/snapshot/preparation.json`; the lead
merges its actual installed paths, successful build receipt, complete closures
and source identities with private inputs and resource authority. A producer
fragment alone is never a finalized fixture inventory.

Proposal mode materializes the protected-overflow candidate into actual
`budget_vectors`: selected primary `fixture-primary/gpt-5.4`, declared catalog
context1050000, user text `xq7 ` repeated2100000 times (8400000 ASCII bytes),
streamfalse and output cap2048. It bounds source allocation and complete serialized
JSON below the mounted10MiB limit, leaving space for the later session UUID.
Configured provider model declarations carry the source catalog context/output
limits and tool/vision/structured capability fields instead of losing them to
custom-provider defaults. These declarations remain unverified against the
genuine service. No exact tokenizer, guaranteed overflow or exact fit is claimed:
the actual correlated durable preparation refusal before dispatch and zero host
effects remain mandatory. HTTP413, proxy byte rejection or a missing run receipt
cannot pass U13. Final composition preserves the frozen candidate body; it does
not generate one from live provider behavior.

Deterministic producer order is source freeze, separately approved artifact
preparation, separately approved private input preparation, exact resource/seed
adoption, then final composition and external derived-input adoption. The future
argv forms are `[frozen_python, prepare_private_inputs.py, private_config]` and
`[frozen_python, compose_fixture.py, --bindings, actual_bindings, --inventory,
actual_inventory, --output, exclusive_composition_directory, --compose]`, with
every path absolute. Artifact producer bytes and closures come from the lead's
`prepare_artifacts.py` handoff; the composer accepts its actual installed artifact,
build receipt and closure manifests, not a claim that the producer will succeed.
Epoch and prior consumption are inherited unchanged; no new zero seed is created.
The final derived receipt stays outside its own source/signature preimage.

The lead-selected finite local proposal uses runner wall86400/request120/
command180/cleanup180 seconds and observer/ready120 seconds. The original budget
epoch remains `uar-user-outcomes-original-20261024`, starting
2026-10-24T13:38:17.685447Z and ending 2026-10-27T13:38:17.685447Z. Its allowance
and actual consumed counters must not be reset by a regenerated fixture. The
budget counts at most6000 model and20000 embedding forwarding attempts; these are
independent stopping limits, not a promise that the cost pool buys that many.
The proposed Cohere ceiling is5000 microunits per full request:96 textual inputs
times512 billable tokens at the operator's rate costs4915.2 microunits, rounded
up. This requires the proxy's real text/batch<=96 guard and explicit adoption of
the assumption that all billable tokens fit the V3 per-input context and there
are no extra charges. It is not an exact tokenizer or invoicing claim. With no
prior reservations, the USD10 pool admits at most2000 such worst-case reservations;
prior/unknown reservations reduce it. No refunds, alternative-provider retry or
transfer from unused OpenAI funds increases the allowance.

Input details beyond the example skeleton:

- `providers`: at least three distinct configured UAR provider IDs: the primary,
  an eligible alternate and a separate structured metadata-forwarding binding.
  Each row has `kind=genuine`, actual `qualified_model`, explicit actual
  `wire_model`, `credential_env`, `budget_binding_id`, and real mounted provider
  `config` (id/display_name/protocol=chat/default_model/enabled/models). Credentials
  remain environment references, never `config.api_key`. The default model must
  occur as an enabled model in that config. `wire_model` must be on the approved
  proxy allowlist; a registry/display name is not assumed to be the actual wire
  alias. The composer derives the owned proxy base URL.
- `budget_proxy`: the complete real config from `budget_proxy.schema.json` and
  `budget_proxy.README.md`, including no-default resource limits, epoch, adopted
  packet/signature/seed references, per-binding charge documents, and exact
  existing-service provenance/authority references. Every referenced document is
  frozen in the final inventory. Prior consumption is retained and any unknown
  reservation refuses composition. Zero charge requires explicit approval.
  `header_prefixes={"Authorization":"Bearer "}` explicitly binds the existing
  raw `LITER_LLM_MASTER_KEY` reference to the gateway's actual Bearer contract;
  the charge and service-authority approvals must contain that prefix map.
  Per-model ceilings must admit the existing summary4096, classifier2048 and
  primary default1000 role caps; the composer never injects caps at the proxy.
- `existing_service_observer`: the concrete `observers_service.py` input with
  exact frozen launchctl/ps/lsof and actual gateway binary/LaunchAgent/launcher/
  TOML source references, real uid, expected argv/configured aliases, allowed
  port4000 socket names, configured retries3/costtrackingfalse, named credential
  references and finite process/output/wall bounds. `existing_service_environment_env`
  is a reference-only process map. The composer derives one exclusive receipt
  leaf under the private packet parent. Root's packet owner executes this finite
  source-backed observation once before the budget daemon and positive calls;
  no existing service is launched, restarted or reconfigured by this hook.
- `knowledge.embedding_spaces`: exactly three actual declared spaces A/B/C,
  each `kind=genuine`, `synthetic=false`, exact `fingerprint` (backend/model/
  dimension and any other actual fingerprint fields), `source_kind` equal to
  `approved_hosted_endpoint`, nonempty source approval digest/reference and
  `budget_binding_id`. A/B differ at equal dimension; C changes dimension.
  All three must share one explicitly approved multi-model embedding listener
  because actual model selection persists model/dimension on the existing global
  backend rather than inventing a new endpoint per case. These are OpenAI backend
  spaces with `send_dimensions=false`; Cohere compatibility does not support
  that request field. Returned vector widths remain validated. The draft names
  multilingual-v3.0 and english-v3.0 at1024 and english-light-v3.0 at384; actual
  availability remains unverified. `memory_budget_binding_id` names its compatible
  hosted listener/model too. Memory's OpenAI SDK already omits dimensions; set its
  explicit model/store dimension1024 and protected raw ingress key. Both SDKs
  prepend Bearer, so every proxy binding explicitly sets
  `ingress_authorization_prefix="Bearer "` against a raw named ingress key.
- `installed`: actual regular resolved native/Python/Node/psql/Docker paths,
  installed Python/compiled-JavaScript SDK examples, built Rust SDK and gRPC
  test artifacts, Chromium, Playwright entry, video encoder and WASM guest.
  No client argv invokes a build or package manager. Browser static manifests
  bind actual emitted asset URLs to byte identities; index.html is mandatory.
- `native_configuration`: actual source-backed security, resilience, runs,
  sessions, file processing, vision, intent classifier, harness, llm (including
  embedding config), native-tools, memory and A2A objects. Whole-leaf secret
  fields use `@env:NAME@`. JWT and settings-admin authentication, positive
  persisted-session TTL, owned embedded memory and auto_capture must be enabled.
  The source templates choose exact loopback listeners, log_format=json, MCP,
  ACP, both actual durable adapters, no unstructured service, no skill evolution,
  sycophancy.auto_correct=false, and fixture-only failover health recording with
  error_threshold1/cooldown2/empty fallback list as directed by the lead.
  Native credential authority, team concurrency and classifier/local model
  resources must match the actual intended production profiles. No unknown
  native configuration keys or dependency versions are invented by composition.
- `orchestrator`: the actual authenticated expected principal storage key,
  history-owner key and subject. `teams`: actual protected model credential and
  storage refs, profile/settings revision, sufficient finite team budget and
  per-child reservations, case deadline and distinct-principal authorization ref.
  `capabilities`: the declared structured model, actual MCP server name, foreign
  owner reference, finite case deadline and real routing requirements. Tools
  roots/MCP argv derive from the owned runtime scope, never operator files.
- `budget_vectors`: exactly one real over_limit mounted primary request and at
  most one real exact_boundary vector. Each supplies actual primary model,
  nonstreaming messages and an explicit positive output ceiling. The case later
  creates a new session ID for correlation. A vector's name does not prove an
  overflow; the actual durable admission/effect evidence must do so. No genuine
  exact framing contract is inferred from the selected chat aliases.
- `backends`: both named adapter profiles, distinct explicit loopback HTTP/gRPC
  ports, distinct existing private mode0700 parents, environment variable names
  for those parents/generated configs and actual owner/admin header refs. Surreal
  adds its explicit private loopback service port; all listener ports must be
  distinct from budget/metadata listeners. The packet's budget ledger parent is
  independent and persists across both backend cleanup boundaries.
- `postgres`/`surreal`: named connection/auth references, actual namespace,
  digest-pinned Docker image and exact daemon Args. The runtime observer owns
  exact fresh schema/DB creation and removal. The composer constructs only the
  name/label/volume/loopback-publication Docker argv from these supplied values;
  it does not run Docker or create/adopt a datastore.
- `postgres_store`: the explicit digest-pinned already-local pgvector image,
  container port/mount/PGDATA, approved Docker environment reference map and
  source-backed lifecycle limits. The composer emits `postgres-store.json` and
  binds `[python,stores_fixture.py,postgres-prepare|postgres-cleanup,config]`.
  The helper creates only its exact owned container/database and pgvector, proves
  CID/image/nonce/labels/mount/ports/actualdata_directory, then the runtime creates
  its schema. It never assumes generated credentials imply a provisioned DB.
  Its total owner limit is at most110sec, observation is at least total+5 and at
  most120sec, and runner command/cleanup allowances cover it. No image pull,
  automatic error retry or unrelated container adoption is enabled.
- `private_inputs_config` and `private_inputs.inputs/receipt`: exact digest refs
  for the separately approved actual preparer outputs. The composer matches
  epoch/config/input identities and future output path, merges only the real
  security patch into complete approved policy, uses actual principal/history
  keys and team workspace, and derives the prepared backend/packet state parents
  and canonical environment-reference aliases. It does not parse env.json.

`fixtures/inventory.example.json` is also deliberately nonfinal. A final inventory
has `finalized=true`, `full_closure=true`, absolute regular `files[{path,sha256}]`,
the complete selected `source_files`, installed native artifact and build receipt
references, and all named closure groups. Each group has `complete=true`, all of
its actual dependency/resource paths and a frozen `manifest` reference. Each
manifest is `{kind:<group>,files:[{path,sha256},...]}` and must exactly agree with
the final inventory. Groups cover native runtime, Python, Node, each SDK, gRPC,
Playwright, browser/resources, video encoder, WASM classifier and local embedding
assets. Whole closure is the lead's explicit inspected installation contract;
listing an entry executable alone does not discover or prove all dependencies.

The actual successful build receipt is frozen JSON containing `source_identity`,
`artifact_identity`, `artifact_path`, `command` argv, matching
`operator_approved_command` argv and `exit_code=0`. The source identity is the
canonical sorted `{path,sha256}` selected-source inventory; artifact identity is
the native file's byte digest. This receipt must be produced after the future
approved customer build. None exists by virtue of authoring this composer, and
the composer cannot certify an earlier artifact against edited source.

`source_files` is the code/authored-template source denominator. It excludes the
later built artifact, build receipt, actual operator bindings/inventory, private
preparation outputs and resource approval/signature/seed. Those remain frozen
input files and feed the appropriate artifact/configuration identity, rather
than retroactively changing the code preimage after the approved build.

## Outputs and ownership

The new mode0700 output directory contains mode0600 private JSON files:

- `fixture.json`: the literal frozen manifest (currently97 cases/582 backend
  repetition rows), all actual modules/hooks/clients, including bounded skill
  retention and its concrete observer, three repetitions and both adapters,
  approved bindings, identities, installed dependency closure and frozen files.
- `postgres_pgvector-{runtime,startup,server,mcp}.json` and equivalent
  `surreal_disk-*`: source-backed observer/native configuration, owned loopback
  listeners, rendered private memory/tool roots and exact MCP peer program.
- `budget-proxy.json`: one persistent packet forwarding epoch across both
  adapters/restarts, never a fresh per-backend allowance. `packet_processes`
  binds exact argv/environment/epoch/listeners/readiness to the lead's one owner.
- `existing-service-observer.json`: one finite actual service identity/config/
  listener observation, with the same frozen service source references as every
  budget authority. `packet_processes.existing_service` supplies the concrete argv.
- `postgres-store.json`: exact owned lifecycle config and frozen tools/limits;
  the runtime's lifecycle hooks execute its pair only in the later approved gate.
- `team-agent.json`, `team-template.json`, `team-binding.json`: actual draft.2
  template documents with computed source/content digests; the case constructs
  its fresh real IDs/member graph/instructions/budgets before installation.
- `adoption.json`: non-secret path/environment bindings and the explicit status
  `COMPOSED_NOT_EXECUTED_NOT_GATE_AUTHORITY`. No env file is sourced, no secret
  parsed, and no PASS receipt is manufactured. It names the protected environment
  file for an explicitly approved parent-memory load, never a shell source/log.
- `derived-input-receipt.json`: actual generated input hashes, excluding its own
  hash and any future external adoption signature. The operator gate adopts it
  externally; it is never inserted back into the fixture it names.

Source identity, global configuration revision and schema revision use the exact
canonical digest contract implemented by the runtime observer. Configuration
inputs include both adapters' configs/templates, the MCP templates, team inputs,
the operator bindings/final inventory and packet proxy config. They contain
paths rather than their own digest, so there is no circular self-hash. The final
fixture freezes their actual bytes and is itself bound by the runner's loaded
fixture digest. A partial failed composition is not adopted; use a new exclusive
output directory after repairing its named input rather than silently overwriting.

The PG schema contract derives all top-level SQLx migration versions, created
columns and source ADD/RENAME/DROP COLUMN changes. Its installed checksum gate
remains the actual complete SQLx ledger. Surreal follows the migration sources
actually included by `surreal_host`, ingestion/accounting initializers and the
inline checkpoint initializer. It does not import the unused legacy base
schema.surql or require its unrelated tables. Field arrays are intentionally empty
table-presence obligations; source includes and post-prompt canonical observers
retain their separate record/field semantics. Seeded agents are required at boot;
sessions are checked after actual prompt creation, not falsely called a migration.

All runtime compatibility, SQL parsing, import/syntax, client resolution,
forwarder provenance, port availability, models/embeddings, native build and the
all manifest outcomes remain unverified under the CLOSED gate. Local FastEmbed classifier
work is frozen local compute; forwarded attempt counters bound external gateway
model/embedding attempts and must not be mislabeled as every in-process local
embedding invocation. Packet resource ceilings may exhaust before completion;
finite caps are not evidence that the complete suite fits them.

## Source, build and authority order

Source authoring and `--propose` output are separate from executable inputs.
Proposal mode writes `bindings.pending.json`, `inventory.pending.json` and
`proposal.json`, all nonexecutable, with explicit pending slots, full manifest
and a nonrunnable composition argv recipe. It does not inspect binaries, hash an
installed artifact, read secrets, invoke the private-input preparer or claim a
build/signature exists. An example can inform a proposal but cannot become
executable through this mode.

The lead first freezes complete source and presents the whole-phase inventory.
Only the exact operator-approved build command can produce the customer artifact
and actual successful source-bound build receipt. If the current gate cannot
permit that build before derived inputs exist, the lead must request the exact
build exception and name those remaining derived inputs. Neither a circular
completion claim nor implicit build permission is valid.

The approved private-input preparer separately creates scoped credentials and
nonsecret identity patches without emitting values. Resource packet/signature,
tariff ceilings and reconciled usage seed follow their own operator authority.
The signed preimage cannot contain its own signature hash or the derived fixture
hash, because the fixture references that signature. After actual build and
resource signing, deterministic `--compose` binds the real inputs and emits the
fixture plus a new derived-input receipt. Final exact gate approval/signing adopts
that receipt externally. Neither the receipt nor its future signature is fed back
into its own fixture/configuration preimage.

The local strategy resource is derived from the single frozen local_embedding_assets closure root containing the packaged ONNX and four tokenizer/config files. Final native models.models_dir explicitly names that copied immutable directory. A hosted RAG binding cannot stand in for this local resource. The dedicated intent-classifier-guest template must be componentized under the separately approved resource stages; arbitrary skill.wasm/WIT source does not satisfy installed.classifier_wasm. Runtime strategy acceptance remains pending.

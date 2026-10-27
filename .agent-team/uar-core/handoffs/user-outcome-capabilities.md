# Task2.8 capability case/fixture handoff

Phase `uar-post-lifecycle-verification-readiness`; **verification CLOSED**. Only source inspection and authorized source/document writing ran. This writer owns NEW `scripts/acceptance/cases_capabilities.py`, `observers_capabilities.py`, `tools_fixture.py`, and this handoff. Existing core, Rust, UI, strategy source and other writers' edits were preserved. No subagent, test, import, py_compile, formatter, linter, compiler/Cargo/build, validator, model call, database query, service or acceptance command ran. Every authored case and SQL/API compatibility claim remains unverified.

## Source-backed public support dispositions

Read `docs/product-support-matrix.{json,md}`, mounted chat/native run APIs, tool discovery/definitions/approval paths, file_read/file_write implementations, MCP stdio fixtures and namespace mapping, conversation policies, provider/routing paths, memory owned controls/retention/store/injection and strategy source. The immutable existing matrix remains authoritative about advertised scope; this handoff is not new acceptance or a downgrade.

- Tier1 direct OpenAI-compatible structured/vision paths are required positive capabilities. Graph-plus-structured is a separate explicit unsupported combination; normal text graph behavior is unaffected. The structured-path writer fixed typed response_format ingress/capture/Liter forwarding and post-projection validation; read `.agent-team/uar-core/handoffs/user-outcome-structured-output.md` for its precise supported combinations and supplementary edge obligations. No prompted JSON-only answer is counted as structured-format evidence here.
- Native and MCP-discovered tools use actual discovery, policy and host approval; stored HTTP tool definitions are declaration-only/forbidden egress, not a newly invented executor. WASM is preview only in the explicitly enabled profile; browser WASM is unsupported. This source does not fabricate a WASM positive or promote catalog-only providers into execution promises.
- Stable routing promises are actual declared capability, availability and preference filters. Learned adaptive routing is experimental and remains a non-promise. The authored positive uses genuine declared provider bindings, not a fabricated catalog model.
- Memory is actual enabled scoped MemoryService, with credential-principal ownership and its own embedded disk persistence even when primary runtime storage is PostgreSQL. It is not silently inferred from primary DB type or the no-op legacy fallback. Manual scoped memory CRUD/recall and explicit capture consent/retention are separate journeys.
- Activation count denial uses the actual `max_active=1` and typed `ActivationFailure::LimitReached` (`limit_reached`). Token pressure uses the existing bounded/truncating reattachment contract; no token-denial policy was added. Exact per-skill/total retained-token pressure receipts remain a complementary original obligation, not falsely certified by the count case.

## Required exported cases

`cases_capabilities.cases()` exports the following 13 Case IDs with their actual U-family and three fresh repetitions. Durable ones run on both required adapters through the existing runner.

1. `U22.native-tool-approval` — actual discovered file_write/file_read. Each repetition has three fresh runs: Ask approve, Ask deny, and hard Deny. The host resolves only exact observed approval IDs; model prose never approves. Real append once + actual read must produce the private file fact/arithmetic in a subsequent genuine round. Independent bytes/digest show exactly one append. Denied runs leave bytes unchanged; hard Deny has no approval bypass and actual effective deny policy.
2. `U22.mcp-tool-approval` — identical predicates through an actual registered stdio owned_record tool, using catalog-resolved namespaced identity. The peer performs real confined append/read; fsynced separately retained per-operation receipts distinguish execution from a model's claim. Positive requires one actual append and one actual read, complete paired canonical tool receipts and genuine result consumption. Ask deny/hard Deny have zero peer dispatch/effect receipts.
3. `U23.owner-isolation` — actual distinct credential principal cannot read run/events or cancel another owner's run. The same external conversation UUID produces two separate durable credential-principal session records; foreign continuation must return UNKNOWN while the owner retains its private fact after a true process restart.
4. `U23.failure-honesty` — separate real loopback negative provider listener returns401 for actual model requests (no assistant success generator). Caller may not receive a success terminal, and a separately recorded actual401 proves it reached the peer. A name-scoped real DB trigger/event then refuses exactly one new run-owned principal KB insert; API must visibly fail and physical DB reads show no committed row. Trigger/event removed in finally; genuine primary inference remains useful afterward.
5. `U23.unknown-tool-effect` — legitimate MCP file operation fsyncs one append, records it, then really exits23 before replying. Actual task must fail, retain one canonical outcome_unknown plus admission uncertainty, and never silently retry. After true restart the file digest/count and unknown receipt remain unchanged. This is an actual unknown effect witness, not a fixture boolean.
6. `U25.memory-lifecycle` — owner-saved private memory drives genuine pre-call memory-recall event and useful answer; foreign owner sees404. Actual embedded disk/store identity and record revision remain unchanged after PID change. Update changes useful recalled fact and has history. Delete yields unqueryable tombstone; fresh question is UNKNOWN. Another restart must retain deletion and show actual physical purge control complete.
7. `U25.memory-consent` — actual host auto_capture capability required. Explicit owner seven-day consent captures a real private conversation fact. Disabling increments consent epoch; later opt-out fact remains absent after actual stop/join/restart, while prior retained fact survives. Original owned capture policy is restored.
8. `U25.formats-vision` — a random three-panel PNG fixture supplies genuine visual evidence; answer must identify exact colors in order. Strict schema nonce exists only in response_format, absent from prompt. Genuine answer must satisfy exact schema-only nonce/calculation. Actual forwarded provider wire must retain schema/name/strict/model and an output ceiling; successful answer alone cannot pass.
9. `U25.routing-capabilities` — actual capabilities/tools/vision/structured requirements and preferred genuine provider; disable it, route to a declared genuine eligible alternative, execute genuine useful inference there, reject impossible context size. Restore the primary provider. Additional configured genuine bindings are allowed alternatives; an undeclared catalog destination cannot pass.
10. `U25.lexical-semantic-disagreement` — finance bank keyword versus natural river/mooring semantics. Actual keyword/embedding methods must rank different intended owned skills, expose accepted scored candidates and consume different private bodies. No selected-policy bypass. Restore matching config.
11. `U25.context-threshold-restart` — all six current structural strategies, each below threshold. No source/prepared history rewrite or summary-tier call may occur; private recall must remain correct after true restart with same configured strategy.
12. `U22.skill-activation-count-limit` — catalog mode, two actual ordered attachments, real registered max_active1 agent. Second attachment must return typed limit_reached with limit1; first remains the only active skill and its private body determines genuine output, second body never affects output. Restore activation mode. This closes the authored count-limit vector, not all token-pressure obligations.
13. `U25.protected-multimodal-context` — actual random image, actual approved native file read with paired canonical result and genuine useful next round, then pressure through a small real sliding window. Final answer must retain exact image/tool facts; reduction must change history with protected_groups_preserved and canonical image retained. Source file remains unchanged. No planted assistant/tool result substitutes for the real initial operation.

## Concrete observer hooks and actions

Every hook below binds the same exact frozen Python interpreter plus `observers_capabilities.py` and the named single action. They use `UAR_ACCEPTANCE_CONTEXT`, `UAR_ACCEPTANCE_FIXTURE_PATH`, `UAR_ACCEPTANCE_RUNTIME_CONFIG`; imported `observers_runtime.py`, `observers_strategies.py`, `observers_startup.py`, and `tools_fixture.py` must be frozen too.

| Hook | Action | Actual source/effect |
|---|---|---|
| cap_prepare_effect | prepare_effect | Exclusively create one private random-fact/number record and host-only expected digest under exact owned tools-effects root |
| cap_effect | effect | Independently read actual file bytes/digest, exact append count, unchanged baseline and fsynced peer receipts |
| cap_run_evidence | run_evidence | Actual durable_run_records, credential-owned canonical session, canonical_tool_receipts, tool_admission_evidence and mounted public event history; project tool call/result IDs/positions/arguments, outcome/status/completeness, image count and context/activation receipts |
| cap_provider_ready | provider_ready | Actual invocation-owned listener bind receipt (exact PID), separately recorded provider401 request count |
| cap_store_fault | store_fault | Exact owned-name PostgreSQL trigger/function or Surreal event installation/removal; actual physical row read; no general failure callback |
| cap_memory | memory | Mounted owner-scoped real record/control/policy plus actual launched memory config, exact private embedded disk path and persisted file bytes; no primary-DB memory fallback assertion |
| cap_image | image | Create actual bounded PNG test data using stdlib compression, return expected random pixel colors and bytes/digest |
| cap_isolation | isolation | Actual two session records by exact external UUID; separate credential-principal owners and privately computed marker presence/history digests |
| cap_wire | wire | Read actual protected forwarding transport metadata, correlate unique schema name, exact actual format/model/approved upstream/output ceiling and successful upstream response digest |

Raw system prompts, raw model reasoning, credential headers and arbitrary upstream diagnostics are not emitted. Tool arguments are the explicitly case-owned file/marker/skill arguments; hidden file bytes and memory content are represented by hashes/provenance except the case's deliberately planted facts needed for answer grading. Private full histories are parsed inside observer and replaced with call/result/image metadata.

## Actual tool/transport fixture composition

`tools_fixture.py` has three distinct executable modes; nothing starts on import:

- `mcp <absolute-owned-root>/tools-effects`: real JSON-lines stdio MCP initialize/tools-list/tools-call/ping. Tool owned_record has read/append/commit_disconnect operations on safe leaf names only, never arbitrary paths. Marker is exactly24 lowercase hex chars. Append and receipt are fsynced. commit_disconnect exits only after the real file/receipt commit, with no invented result.
- `provider-fault <absolute-owned-root>/tools-effects`: separate invocation-owned loopback listener binds an actual ephemeral port and writes provider-fault-PID.json. Only returns real401 with typed run-owned error. It never returns a positive assistant result. Case launches/joins it through Context.owned_process.
- `genuine-proxy <absolute-owned-root>/tools-effects <frozen-port> <frozen-HTTPS-upstream-base>`: a real forwarding transport for only /v1/chat/completions. Forwards exact model request bytes to the exact HTTPS base + /chat/completions with no redirect; Authorization is forwarded but never recorded. It does not generate, modify or replay assistant responses. It records actual response_format/model/stream/output cap and request/response hashes/status/byte count. No raw request messages, response content or credentials are recorded. Listener publishes an actual PID/port/upstream bind receipt. First formats case starts it through owned_process; subsequent repetitions reuse only an actual live process already in this Context. It remains owned until whole-context cleanup. This source currently enforces only per-request/response byte and transport bounds, NOT packet money/request/token allowances.

The shared runtime fixture must register the real global stdio MCP server with exact frozen interpreter/program argv and rendered exact private tools-effects root. `RunMcpServerInput` itself accepts the registered server name rather than arbitrary inline stdio command. Native file-tools config must enable file_read/file_write with allowed_paths equal to that exact root, never wildcard/operator files. The trusted fixture peer creates tools-effects only beneath the already-created private run-owned parent, so startup connection does not depend on a later case prepare callback.

## Required fixture fields and resource bounds

Fixture `capabilities` object:

- `case_seconds`: finite positive bounded case-control allowance. Shared runner wall/HTTP/output/canonical limits still apply and exhaustion blocks; this is not automatic verification retry.
- `foreign_authorization_env`: actual distinct principal Authorization reference; include in secret_env/explicit environment propagation so credentials are available and redacted.
- `mcp_server_name`: exact actual registered owned-file stdio server name. MCP tool provider name is obtained from actual /api/tools inventory, not guessed.
- `python_binary`, `tools_fixture_path`: absolute frozen regular executable/source paths. Same exact program is registered in server MCP config.
- `routing_requirements`: actual needs_tools=true, needs_vision=true, needs_structured_output=true and any frozen context/cost requirements. Two declared genuine eligible provider bindings are required; no unsupported catalog entry can become positive by declaration.
- `structured_binding_model`: a declared genuine supported preserving OpenAI-compatible provider/model binding. Its frozen base_url must equal http://127.0.0.1:<genuine_proxy_port>/v1. This can be a separate genuine binding from direct primary. Shared provenance observer must explicitly approve the exact loopback forwarding origin, with this frozen forwarding source and approved actual upstream receipts; trusting an arbitrary local origin as genuine is insufficient.
- `genuine_proxy_port`: exact owned nonprivileged free port reserved by the approved composition. No implicit port-discovery shell/preflight is added.
- `genuine_upstream`: exact operator-approved credential-free HTTPS provider base. It is a frozen forwarding destination, not a secret-containing URL. Real provider credential remains a named protected environment reference, never argv or receipt.

Host memory must actually enable scoped MemoryService and auto_capture for consent case. The shared runtime observer already requires an enabled memory namespace/database equal the exact run scope and embedded DB path beneath the owned private root, not remote ambient memory. This persists independently of the primary Postgres/Surreal adapter. No case may return green for disabled memory; unsupported actual resources block.

Both actual primary storage adapters must support the named fault injection in their isolated fixture only. PostgreSQL effect: one trigger/function in the run schema, before insert on principal_knowledge_bases, refusing only the fresh acceptance-run name. Surreal effect: one event on the run DB principal_knowledge_bases, refusing only that exact new name. Removal occurs in finally. Parent cleanup owns the entire isolated schema/DB if an allowance or abnormal stop prevents removal; no operator datastore is touched.

The negative provider is a distinct fixture provider configured through the real provider administration path, copied from the genuine supported protocol/model configuration, with its own controlled401 endpoint and invalid run-owned credential. Its actual receipt must show at least one request; early unsupported-catalog refusal cannot substitute for provider denial. It is deleted and the listener joined afterward.

## Request/resource accounting is an OPEN implementation obligation

Current source exports13 cases ×3 fresh reps ×2 required adapters =78 case instances. Source loop inventory implies294 explicit native/chat agent task requests plus90 configure_provider /test calls =384 potential model-request floor, BEFORE extra tool rounds, classifier/title/summary/private inference, production retries/fallbacks and embeddings. This is not an upper bound or executed consumption. The whole-suite draft600 model/2000 embedding/USD10 envelope is not enforced by this source and cannot be certified from HTTP/output allowances or output caps. No limit, prior high-water consumption or unknown reservation was reset/adopted.

Lead requested a separate source-only budget forwarder next: every primary/helper/test/embedding endpoint must traverse frozen per-binding forwarders with one atomic persistent epoch ledger across backend contexts/restarts, operator-approved per-attempt worst-case charge ceilings, request/generated-token/USD/byte/wall ceilings, and unknown reservations retained. That new work is not completed by the present genuine metadata proxy. Signatures/current spend/pricing/remaining allowances must be reconciled at task2.9; a claimed USD bound from estimated token counts would be false.

## Remaining complementary obligations and limits

The authored witness cases cover the requested user capability predicates. They do not replace: the full structured writer's malformed/invalid-output/caller-tool/schema-secret/helper-isolation/SDK/native-streaming supplementary matrix; token-pressure bounded/truncated reattachment evidence; actual cooldown-state routing recovery proof (disabled-provider exclusion is observed here); complete tool/SDK/UI support ledger and every original Issue426/liability/CAS/error/history/ingestion/six-profile/spec obligation. Lead owns consolidation and must not mark those original rows satisfied merely because these cases exist. Additional observer/runtime schemas, actual model compliance, SQL trigger/event grammar, process timing and cold persistence remain unverified under the CLOSED gate.

No new token-denial policy, runtime API, general-purpose external executor or provider-supported-mode fiction was introduced. Guards in this source trace to exact owned storage/credential boundaries or named outcome predicates. Future permission must name actual fixture services, native/MCP file effects, scoped DB DDL fault effects, genuine provider calls and all proxy/cleanup effects explicitly. This handoff supplies source and exact composition inputs, not permission or product acceptance.

## Shared packet budget forwarding implementation — supersedes OPEN source obligation above

Source-authored `scripts/acceptance/budget_proxy.py`, `budget_proxy.README.md`,
`budget_proxy.schema.json`, and deliberately nonexecutable
`budget_proxy.example.json`. Exact argv is frozen Python + absolute program +
absolute private executable config. One lifetime exclusive epoch ledger spans
both backends and runtime cold restarts; parent starts it once before backend
contexts and keeps its private shared parent outside backend cleanup. Inputs bind
exact packet/signature/seed/config bytes; no reset command exists. Seed explicitly
reconciles prior counters/evidence/unknowns. Unknowns block. Exact HTTPS binding,
path/model allowlists, named header secret references, per-attempt approved charge
ceilings and atomic attempt/token/cost/request/response byte reservations precede
outbound. All failures retain their ceilings; missing usage/partial transport
blocks accounting. Absolute wall deadline terminates75; normal SIGTERM/SIGINT
closes admission and joins workers before closing synchronous FULL SQLite.

Root owns startup/readiness/cleanup/inference integration. Ready first stdout JSON
matches owned mode600 ready-NONCE.json and identifies child PID, nonce, exact
config_sha256, listeners, epoch/input_identity and totals/blocked state. Root must
reject missing terminal receipt, nonzero/forced exit and unknown reservations.
Source does not verify cryptographic signatures or pricing. Operator approval of
full-context/output/embedding worst-case charge ceilings remains mandatory;
zero-charge needs explicit approval. Aggregate reservations bound admitted
exposure, not verified actual charges, and may exhaust before suite completion.

SQLite reservations rows carry request_id, binding_id,kind,state plus JSON
reserved and receipt. Raw request_wire_sha256 hashes actual serialization bytes;
request_semantic_sha256 is separately canonical. Raw unchanged bytes are forwarded
once. Receipt includes actual HTTPS URL, model/path, owned PID/nonce/inputidentity,
attempt interval, status/response SHA/byte count/actual supported usage. Primary
receipt can match durable final-wire SHA. Auxiliary calls lack unique call IDs;
only exclusive binding/model/timewindow/count traffic proof is available. Never
claim unique auxiliary correlation from matching model alone. These are no raw
body/key/reasoning logs and no cryptographic proof of remote service authorship.

Updated tools_fixture.py/cases_capabilities.py/observers_capabilities.py to compose
metadata listener → owned exact budget model listener → frozen HTTPS provider.
The genuine-proxy argv now appends exact budget_config after upstream base.
Fixture capabilities requires structured_proxy_budget_config (absolute path) and
structured_proxy_budget_hop_approved=true. owned_hop checks config/ledger/ready
identity. Wire observer requires exactly one completed matching raw request hash,
model/status and actual response SHA/bytes in that binding's owned ledger.
Metadata forwarding disables ambient HTTP proxies and redirects. The original
five-argument direct HTTPS utility remains source-compatible but the acceptance
case always uses the budget hop; it cannot substitute a direct receipt.

All these edits remain unexecuted. No tests, imports, probes, services, modelcalls,
formatters, compilers, validators or review verification were run. The phase gate
remains CLOSED. Root/composer must finish global routing, approvals and every
other outstanding implementation item before renewed operator verification.

## Bounded native/probe output integration and selected Liter service authority

Additional lead-approved source spans: manager.rs captures positive output caps
from existing known context response reserve1000 when omitted, preserves explicit
caller caps within exact contract and known model maximum, and rejects unknown
capacity instead of using legacy8192 fallback for output admission. ResolvedTurn
and graph captured driver both enforce it. Initial/per-step estimated reducers
withhold larger explicit output reserves; fit remains estimated. Exact contract
continues governing final-wire fit. liter_driver.rs refuses extra_params attempts
to override typed output ceilings. team_profile.rs permits max_completion_tokens
but retains output_ceiling=None/settings-only/no guaranteed fit. providers.rs
TestProviderRequest now accepts optional output_token_cap; real /test supplies
explicit positive cap or known-capacity existing1000reserve, bounded by model
maximum. No unknown-capacity generic driver output fallback was added.

Existing classifier/helper allowances already cap2048; summary allowance4096 is
preserved. Per-binding proxy must admit4096 if selected summary calls use it; the
fixture's2048 primary cap is not a new global helper policy. Optional sycophancy
server correction chat_non_streaming still has no independent output cap: lead
chooses explicit auto_correct=false/log_only=true in frozen packet. That optional
path is not claimed as newly governed here.

Operator selected configured ai.prometheus.liter-llm-api launch service. Narrow
budget binding upstream_authority support now allows only exact HTTP loopback
hostname127.0.0.1 orlocalhost, port4000, /v1/chat/completions. All other HTTP is
refused. Authority includes exact digest refs launchagent/launch_script/service_config
and separate private approval binding epoch/binding/endpoint/service identity;
charge approval names upstream_authority_kind=existing-service-loopback. Receipt
records authority kind and frozen refs, not secret contents. Parent must prove
actual selected service identity; these hashes do not prove live loaded config
or downstream service execution. No probe/start/readiness or model call occurred.
Full contract and existing-service exception are in budget_proxy.README.md.

All changes source-only/unverified under CLOSED gate. No build, check, tests,
imports, formatter, provider calls, service commands or review verification run.

The selected gateway's existing raw LITER_LLM_MASTER_KEY now supports explicit
binding.header_prefixes={"Authorization":"Bearer "}. Prefix map is frozen in
config and must exactly match charge approval and loopback authority approval
(empty map in authority approval when none). Only that header/prefix pair is
supported; no automatic provider-wide prefix or duplicated/printed secret was
introduced. Upstream constructed value stays in memory. Ingress reference still
contains the full expected Authorization header. Composer notified of exact map.

## Exact-owned Postgres partial-creation cleanup

Authored stores_fixture.py and its README as the dedicated owned Docker Postgres
prepare/cleanup lifecycle. Root/composer own runtime hook integration. Preparation
uses actual runtime root ownership marker, explicit digest-pinned locally available
image, frozen tools, exact loopback port, private env-file credentials, actual
SQL database/user/data_directory/port/vector availability and installed pgvector
version. Actual inspected container labels/digest/mount/ID are required. No Docker
command or query was executed during authoring.

Follow-up cleanup now proves actual exact-name absence with a successful bounded
Docker listing instead of requiring inspect when no container was created.
Creation claim, if present, must match config SHA/run/scope/image/mount/nonce/labels;
runtime ownership marker is always required. Empty successful exact-name listing
returns cleaned/container_absenttrue/container_removedfalse with actual timestamp
and listing SHA, no invented container ID. Missing claim plus confirmed absence
also permits zero-action cleanup; a present container without its claim is refused.
Command diagnostics, nonzero exit, malformed IDs, timeout and transport errors
remain blocking, never converted to absence. Existing owned-container path still
validates immutable actual ID/labels/digest/mount/port before stop/remove; normal
receipt now also includes configsha/nonce/container_absenttrue/absence_evidence.
Data is retained for root's evidence and separately approved owned-root cleanup.

Handshake paths: root/postgres-fixture-claim.json,
root/postgres-fixture-prepared.json, root/postgres-fixture-cleaned.json.
Manager writer owns observers_runtime partial-startup cleanup and will retain
forensics when store queries fail. PG helper resource cleanup is not a successful
application preparation or product acceptance. Source-only, gate CLOSED; no
imports/tests/checks/builds/service/Docker/psql/provider commands executed.

## Actual U10 classifier guest source template and provenance

Added only the bounded standalone source template:
scripts/acceptance/fixtures/intent-classifier-guest/Cargo.toml.in,
src/lib.rs, wit/intent-classifier.wit and README.md. Removed only this writer's
three newly authored superseded flat intent-classifier.rs/.wit/.README.md files;
no unrelated or historical source was removed. Product manifests/lock/host source
unchanged. Template is inert until copied to the approved generated root:
target/uar-user-outcomes-001/classifier-guest. Expected output is exactly
cargo-target/wasm32-unknown-unknown/release/uar_intent_classifier_fixture.wasm
beneath that root. installed.classifier_wasm must point to actual frozen component
bytes, never a generic skill.wasm or source WIT.

Guest parses the actual host's last eligible-metadata context string, scores real
registered keywords against current query, sorts matching current labels, and
returns out-of-scope when none match. It contains no fixed case label/private
parcel answer/canned assistant result, no WASI/host imports, no filesystem/network/
clock/environment/tool/random API. Bounds complement actual host memory/fuel;
runtime fit and useful outcome remain unverified. Modern WIT f32 matches the
host's Val::Float32 ABI. Host's illustrative float32 WIT string is not runtime-
parsed, so that unrelated production documentation string was left untouched.

Selected installed metadata read narrowly from ~/.cargo/.crates2.json reports
cargo-component0.21.1; no CLI version/status/compiler probe was run. Source verified
from official tag URLs:
- https://github.com/bytecodealliance/cargo-component/blob/v0.21.1/Cargo.toml
- https://github.com/bytecodealliance/cargo-component/blob/v0.21.1/src/bindings.rs
- https://github.com/bytecodealliance/cargo-component/blob/v0.21.1/src/metadata.rs
- https://github.com/bytecodealliance/cargo-component/blob/v0.21.1/src/lib.rs
- https://github.com/bytecodealliance/cargo-component/blob/v0.21.1/src/target.rs
- https://github.com/bytecodealliance/cargo-component/blob/v0.21.1/tests/build.rs
- https://github.com/bytecodealliance/wit-bindgen/blob/v0.41.0/Cargo.toml
- https://github.com/bytecodealliance/wit-bindgen/blob/v0.41.0/crates/guest-rust/rt/Cargo.toml

Exact direct guest pins are wit-bindgen-rt0.41.0 (pinned tool's generated runtime
path/version family) and serde_json1.0.154 (actual main Cargo.lock and installed
published crate source). Cargo component generates src/bindings.rs in the copied
package; bindings.formatfalse prevents its implicit rustfmt. Empty standalone
workspace isolates the template from product workspace. No new componentizer
crate or wasm-tools install is needed. Derived Cargo.toml/lock/bindings/actual
component and producer tool/compiler closure must join the final inventory.

README supplies future serial dependency-resolution and component-build argv;
no Cargo executed. Root will include explicit target-standard-library resource
preparation in the named P0 approval. Upstream0.21.1 internally invokes rustc
--printsysroot and checks wasip1 even with explicit unknown target; source may
install it when RUSTUP_TOOLCHAIN is present. Those internals were disclosed, not
silently run as preflight. Build targets wasm32-unknown-unknown so the guest itself
needs no WASI linker imports. Missing actual output or unsupported ABI must block
U10; source004 historical acceptance cannot certify this new source/artifact.

Gate CLOSED. Source writes, local package metadata/source reads and official
upstream documentation/source reads only. No build, dependency resolution,
compiler probe, script import, tests, formatting, service/model calls or review
verification was run. Ownership of the four guest template files is released.

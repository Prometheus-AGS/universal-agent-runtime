# Packet budget forwarder — source authored, execution not approved

This daemon forwards actual provider bytes to frozen HTTPS endpoints or the
explicitly approved existing Liter loopback service. It never
constructs a positive assistant or embedding response. The verification gate is
CLOSED. These files are implementation artifacts; no import, listener, model
request, test, or validation has been run for them.

The uncomfortable limitation is that an approved reservation ceiling is not a
verified tariff or actual bill. The operator must approve a conservative charge
ceiling for each binding using that provider's full permitted input context,
maximum output, and embedding pricing. Its scope includes the declared request
byte limit. A declaration of zero charge needs separate explicit approval. This
program verifies the adopted file bytes and their relationships; it does not
cryptographically verify a detached signature or establish that a remote operator
has not supplied a fake service behind an approved endpoint.

## Startup and ownership

The future approved parent packet starts exactly one process, before either
backend, with argv `[frozen_python, absolute_budget_proxy_py, absolute_config]`.
No shell and no extra flags are supported. All model, classifier, title, summary,
model-test, fallback, and embedding endpoints must use its declared listeners.
The named ingress environment value is either a full Authorization header or a
raw key, selected explicitly by `ingress_authorization_prefix` (empty or
`Bearer `). This prevents a client that adds Bearer from sending it twice.
Actual upstream headers come solely from
named environment references. Ambient HTTP proxies and HTTP redirects are disabled.
The raw JSON request body is forwarded unchanged. There is no outbound retry.
The distinct tools-fixture fault listener returns a real 401 and never forwards.

Each binding has one exact loopback port, request path, HTTPS URL and explicit
model allowlist. Every port must bind successfully before requests are served.
`ledger_root` is an absolute private directory with basename
`uar_budget_` plus the first 20 hex characters of SHA256(epoch_id UTF-8). Its
existing private parent is shared packet state, outside both backend scratch
roots. Backend cleanup must never delete it. No reset command exists.

The existing-service authority allows only localhost/127.0.0.1 port4000,
`/v1/chat/completions` for a model binding or `/v1/embeddings` for an embedding
binding, with exact launchagent/launcher/config references and separate signed
scope approval. It grants no arbitrary cleartext upstream. Source configuration
is not proof of live service identity; the parent packet's named read-only
service observation precedes positive traffic. Counters measure gateway
requests; the gateway's configured internal retries are not separate rows here.
Worst-case charge approvals must cover that entire retry/fallback chain.

Inputs and approval files are regular, nonsymlink, mode-600 files up to 1 MiB.
Config, seed, packet and signature bytes determine `input_identity` (sha256:hex).
The per-binding approval reference in config also freezes each charge document.
The parent must freeze all source and input files in its packet inventory.
Secret values are never emitted. Declared header names are emitted only in config.

The first stdout line is a JSON readiness receipt, also fsynced to
`ledger_root/ready-<nonce>.json`, mode 600:

```
{"pid":123,"nonce":"32hex","config_sha256":"sha256:hex","ready_path":"/private/uar_budget_epoch/ready-32hex.json","listeners":[18081],"epoch":{"epoch_id":"approved-name","input_identity":"sha256:hex","totals_reserved":{},"blocked":null,"meaning":"operator-approved conservative reservations, not verified actual spend"}}
```

The actual basename follows the hash rule above. The parent uses its owned child
PID, frozen config digest, exact ports, epoch identity and this owned file receipt
for readiness. It must reject unexpected stdout, a blocked epoch or early exit.
Readiness needs no network probe. Total stdout is two small JSON lines on normal
shutdown, or a closed error JSON on startup failure. Logs contain no bodies,
keys, reasoning, prompts or upstream error text. Parent stdout/stderr byte and
readiness-time limits still apply.

SIGTERM/SIGINT close admission; the daemon shuts listeners, joins request workers,
then emits `{"terminal": <epoch_snapshot>}`, closes SQLite and releases the lock.
SQLite uses synchronous FULL transactions; each reservation is committed before
outbound. Parent cleanup must exceed ingress plus upstream timeout, also bounded
by the epoch deadline. On the absolute wall deadline a hard timer exits 75 even
if a peer trickles bytes. An interrupted reservation remains reserved and blocks
restart as unknown; it never earns successful acceptance. Missing terminal output,
forced stop, nonzero exit, or an unresolved reservation must fail the packet.

## Configuration and approval inputs

`budget_proxy.example.json` is explicitly nonexecutable. Replace every input using
the named operator packet; do not adopt its illustrative values as authorization.
`budget_proxy.schema.json` describes the executable config. Runtime validates
additional exact relationships (digests, private paths, seed reconciliation,
charge document equality, deadlines and live exclusive ownership).

Config fields are `schema_version=1`, `executable=true`, `epoch_id`, `ledger_root`,
`approval_packet`, `approval_signature`, `seed`, `limits`,
`upstream_timeout_seconds`, `ingress_timeout_seconds`, and `bindings`.
Each reference is `{path: absolute_private_file, sha256: "sha256:<64hex>"}`.
All limits are finite integers. The six total counters are `model_attempts`,
`embedding_attempts`, `generated_tokens`, `cost_microunits` (USD millionths),
`request_bytes`, and `response_bytes`. `max_in_flight` and `wall_seconds` are
positive. Zero total allowance deliberately disables the corresponding activity.

The mandatory seed names `epoch_id`, `approval_packet_sha256`,
`approval_signature_sha256`, `prior_epoch_id`, nonempty
`prior_consumption_evidence` reference list, explicit `unresolved_reservations`
list, `consumed` with all six counters, integer `epoch_started_unix`, and
integer `deadline_unix`. A first epoch still needs an approved genesis accounting
record and an explicit prior identity. Prior consumption must include earlier
attempts, failures and uncertainty rather than silently starting at zero. Any
unresolved reservation blocks startup. The deadline is fixed across process
restarts; deadline minus start cannot exceed approved wall_seconds. Seed counters
cannot exceed caps. Changing any adopted input rejects an existing epoch ledger.

A binding requires `id`, `kind` (`model` or `embedding`), `listen_port`,
`request_path`, `upstream_url`, `model_path=/model`, `allowed_models`,
`max_request_bytes`, `max_response_bytes`,
`approved_charge_ceiling_microunits`, `charge_approval`, `headers_env`, and
`ingress_authorization_env`. Model bindings additionally require
`output_token_paths` and `max_output_tokens_per_attempt`. Supported paths are
JSON object pointers without escapes; freeze only the actual adapter's output
cap fields. Every model request must carry an explicit positive integer cap;
multiple generations and best_of are refused. No cap is silently injected.

A charge document must contain matching `epoch_id`, `binding_id`, `upstream_url`,
`allowed_models`, `charge_ceiling_microunits`, `max_request_bytes`,
`max_response_bytes`, `operator_approved=true`, nonempty `pricing_evidence`, and
`basis=full-context-plus-output-ceiling` for models or
`basis=embedding-request-ceiling` for embeddings. Models also match
`max_output_tokens_per_attempt` and declare positive `full_context_tokens`.
Zero requires `zero_charge_explicitly_approved=true`. Pricing evidence is an
operator statement/reference, not a claim this program verified prices.

## Durable accounting and provenance

`owner.lock` is exclusively flocked for the daemon lifetime. `ledger.sqlite3`
contains `meta(key,value)` and
`reservations(request_id,binding_id,kind,state,reserved,receipt)`. JSON fields
`reserved` and `receipt` are explicit inspectable state. `meta` keys include
`identity`, `epoch_id`, `seed`, `totals`, `blocked`, and `owner` (pid/nonce).
Use a read-only SQLite connection to observe it; no live network request is
needed. Do not rewrite it or use a second accounting process as a writer.

Each attempt atomically reserves one kind-specific attempt, its declared maximum
generated output tokens, the full approved USD ceiling, exact request bytes,
and maximum response bytes. Failed responses retain every reservation. On known
success only unused response bytes are released. No cost/output-token refund is
inferred. Exhaustion refuses before forwarding. Missing success usage, transport
uncertainty, incomplete response, over-cap upstream output or stale in-flight
state makes the entire epoch unknown/blocked. Actual supported usage fields are
recorded separately and never presented as verified cost.

`request_id` is the reservation ID. Its receipt contains `binding_id`, `kind`,
`request_path`, exact credential-free HTTPS `upstream_url`, `model`,
`owner_pid`, `owner_nonce`, `input_identity`, `epoch_id`,
`reserved_at_unix`, `outbound_started_unix`, `outbound_finished_unix`,
`settled_at_unix`, `http_status`, `response_bytes`, `response_sha256`, and
`actual_usage` on completion. Unknown receipts have bounded failure codes and
partial byte/hash fields instead. `request_wire_sha256` hashes the actual raw
request serialization bytes; `request_semantic_sha256` hashes canonical JSON.
They are intentionally distinct. `state=completed` plus matching raw wire SHA,
model, owner, interval and response metadata is required for a positive receipt.
A model's content may reach its caller before usage settlement; completion of
that stream alone does not constitute acceptance when accounting is unknown.

The root observer correlates the durable primary request_budgets serialized SHA
with an actual completed HTTPS forwarding receipt. Auxiliary calls currently lack
unique run/call headers, so auxiliary evidence is restricted to fully owned
binding traffic, model, interval and counts. Matching model alone never proves
unique per-auxiliary-call correlation. Receipt identity proves forwarding through
the approved HTTPS binding, not cryptographic proof about remote service authorship.

The structured metadata listener composes with:
`[python, tools_fixture.py, "genuine-proxy", tools_effects_root, metadata_port,
"http://127.0.0.1:<budget_model_port>/v1", budget_config_path]`.
`owned_hop` permits only an exact `/v1/chat/completions` model binding and matching
owned ready/config/ledger identity. Fixture `capabilities` must set
`structured_proxy_budget_hop_approved=true` and
`structured_proxy_budget_config=<absolute config>`. Its wire observer additionally
matches exactly one completed budget reservation by raw request hash and actual
response hash/status/bytes. This positive chain never bypasses the packet ledger.

The source-derived 13 capability-case floor is 384 potential model requests
(294 explicit agent/run requests and 90 provider-test requests), before helper
inference, tool rounds, retries, fallbacks and embeddings. It is not an upper
bound for this family or the complete 96-case/576-row suite. Only the approved
aggregate ledger caps bound admitted attempts; they may exhaust before the suite
finishes. Exhaustion is an incomplete result, never permission to add allowance.

Response reads stop at the reserved body-byte boundary. If the boundary is fully
consumed before EOF is confirmed, the outcome becomes unknown and blocks the
epoch, even when the peer might have ended at exactly that boundary. No extra
body byte is read to discover whether the approved allowance was exceeded.

## Explicit existing Liter launch service exception

The operator selected the already configured Liter launch service. An exact
`http://127.0.0.1:4000/v1/chat/completions` or
`http://localhost:4000/v1/chat/completions` binding may carry `upstream_authority`:

```
{"kind":"existing-service-loopback","service_id":"ai.prometheus.liter-llm-api","operator_approved":true,"endpoint_url":"http://127.0.0.1:4000/v1/chat/completions","launchagent":{"path":"/absolute/ai.prometheus.liter-llm-api.plist","sha256":"sha256:hex"},"launch_script":{"path":"/absolute/liter-llm-api-launch.sh","sha256":"sha256:hex"},"service_config":{"path":"/absolute/liter-llm-proxy.toml","sha256":"sha256:hex"},"approval":{"path":"/private/service-approval.json","sha256":"sha256:hex"}}
```

This is a separately approved authority, never an inferred HTTPS identity.
The private approval JSON must equal authority's kind/service_id/operator_approved,
endpoint_url and the three source references, plus exact epoch_id and binding_id;
exclude the self-referential approval field. Charge approval additionally names
`upstream_authority_kind=existing-service-loopback`. Config/approval/charge bytes
remain packet/signature bound. Source references are absolute regular nonsymlink
files, at most1MiB, with exact SHA; unlike secret input documents, an existing
plist/script need not be mode600. No source contents or environment secrets are
copied into receipts. Receipts include authority kind and its frozen references.
Every other HTTP host/port/path remains refused. Embedding use through a different
local URL is not implicitly authorized by this exception.

The parent must establish the selected running LaunchAgent/configured service's
identity in the approved packet. Frozen files alone do not prove which service
process is listening or prove its downstream model execution. This daemon proves
actual forwarding to that exact selected listener with response hashes, rather
than claiming a direct HTTPS provider receipt for a loopback hop. Service start,
probe and adoption remain pending exact operator permission; none occurred here.

For a service requiring a raw named master key with a Bearer header, binding may
explicitly declare `header_prefixes={"Authorization":"Bearer "}`. That exact
prefix map must also occur in its charge approval and its loopback authority
approval (empty map there when none). Only this prefix/header pair is supported;
there is no default prefix and no new copied secret. `headers_env.Authorization`
may then name the existing `LITER_LLM_MASTER_KEY` directly. Runtime constructs
only the upstream header value in memory and never records it. The ingress
Authorization reference remains a full header value and is not altered.

## Independent cumulative provider allocations

Operator allocations are CohereUSD10, OpenAI embeddingsUSD10, global maximumUSD20
and selected Liter incrementalUSD0. They are enforced as independent cumulative
reservation pools, never transferable allowances or per-binding spending caps.
Explicit executable config now requires:

```
"spend_pools": {
  "cohere": {"limit_microunits": 10000000},
  "openai_embeddings": {"limit_microunits": 10000000},
  "liter": {"limit_microunits": 0}
}
```

Limits may be smaller but never exceed these operator maxima. Global
limits.cost_microunits must be explicit, <=20000000 and <=sum of pool limits.
No default value adopts or resets an allowance. An unused OpenAI pool is valid;
it does not authorize selecting any OpenAI endpoint/model or moving its unused
balance to Cohere. Current CohereA/B/C fixtures need not use that alternative.

Every binding requires spend_pool. Pool identity is checked against the actual
frozen upstream URL/kind: cohere requires direct HTTPS api.cohere.ai:443
/compatibility/v1/embeddings; openai_embeddings requires direct HTTPS
api.openai.com:443/v1/embeddings; liter requires the existing approved loopback
MODEL service. Charged pools require a positive per-attempt approved ceiling;
Liter requires zero plus existing explicit zero-charge approval. Arbitrary HTTPS
or embedding traffic cannot select the free Liter model pool. Config/charge
approvals still bind each exact model allowlist, endpoint, bytes and limits.
Charge approval additionally requires spend_pool and
spend_pool_limit_microunits matching that binding's named pool. These are adopted
inputs, not extra approvals silently inferred from a model or hostname.

Mandatory seed.spend_pool_consumed_microunits has exactly cohere,
openai_embeddings,liter keys. Each nonnegative amount must fit its own pool, and
the sum must equal seed.consumed.cost_microunits. All earlier charged attempts,
failed/unknown reservations and high-water exposure must be reconciled into the
correct named pool with existing prior-consumption evidence and signature binding.
Missing provider allocation blocks; it never becomes zero or moves to another
provider. The seed/config bytes remain part of input_identity. Existing ledgers
without pool metadata are refused; no automatic upgrade guesses allocation or
reinitializes prior totals. A new epoch needs explicit signed reconciliation,
including unresolved reservations, rather than restarting from an invented zero.

SQLite meta.spend_pool_totals persists the three totals. One BEGIN IMMEDIATE
transaction checks and reserves BOTH global counters and the matching pool,
updates the global and pool totals, and inserts the attempt before any outbound
request. All Cohere listeners/models and both backend contexts share this one
counter/owner. Concurrent requests cannot each consume an independently reset
binding balance. A pool can exhaust before global allowance does; the denied
request is never forwarded. No pooling transfer or refill/reset operation exists.

Known success releases only unused response bytes. It never refunds charge
reservations, including failed HTTP responses, incomplete requests, unknown
outcomes or unreported final charges. On restart the existing durable pool totals
must still sum to global cost and match adopted limits. Stale reserved attempts
become unknown and block as before, retaining both global and provider exposure.
Receipts add spend_pool, reserved_charge_microunits and
spend_pool_total_after_microunits. Snapshot/ready/terminal add
spend_pool_reserved_microunits and spend_pool_limits_microunits so remaining
allocations can be reported separately without implying actual invoice precision.

The operator supplied Coherev3 text rate USD0.10 per1million tokens (100000 USD
microunits per1million tokens). This is operator-provided pricing, not independently
verified tariff data. Existing explicit per-request charge ceilings remain required;
this change does not invent a tokenizer or infer a zero charge from a subscription.
A UTF8-input-byte exposure calculation is acceptable only when its bound assumption,
framing/array allowance, rate, integer rounding and scope are explicitly part of
the approved charge input. Byte count is not an exact token count and is not
presented here as a proved upper bound. A selected model's supported request/input
limits can instead underlie an operator-approved worst-case request ceiling.
Failed/unknown exposure still uses that full approved ceiling. Global/provider
reservation caps are concrete; a claim about actual spend additionally depends on
those ceilings covering real provider billing, including any internal gateway
retries/fallbacks. Liter's zero is the operator's incremental-cost disposition,
not an invented physical-provider-attempt count or a verified underlying tariff.

Source implementation only; no schema validation, import, test, formatter, build,
ledger execution, service operation or provider call was performed. Gate CLOSED.


Cohere text admission for the selected fixture accepts only model/input/encoding_format fields and a nonempty string or at most96 nonempty text strings. This guard runs before reservation/forwarding and excludes image or alternate request envelopes from the text-only charge approval. It does not tokenize or prove a billable-token count. The proposed5000microUSD request ceiling uses the operator-provided V3 rate and the explicitly adopted96-input/512-token-per-input model-context assumption (4915.2microUSD rounded conservatively to5000). This permits at most2000 full-ceiling reservations from a previously unused USD10 pool, even though the independent embedding-attempt ceiling is20000. Caps are ceilings, not a guarantee the582-case packet fits. Unknown/failed charge reservations stay held.

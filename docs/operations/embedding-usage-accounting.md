# Embedding usage accounting

`POST /v1/embeddings` uses one pinned configured backend for model identity,
dimension, vectors and usage throughout a request. A settings swap affects the
next request. It does not change an admitted batch's embedding space. Model
selection remains the served `<backend>/<model>` or bare `<model>`; callers
cannot choose an arbitrary provider or truncate dimensions.

The response retains OpenAI-compatible numeric `usage.prompt_tokens` and
`usage.total_tokens`. Read the corresponding `uar_usage` fields before treating
those numbers as measurements:

```json
{
  "usage": { "prompt_tokens": 7, "total_tokens": 0 },
  "uar_usage": {
    "prompt_tokens": { "available": true, "source": "provider_reported" },
    "total_tokens": { "available": false, "source": "unavailable" }
  }
}
```

Here the prompt count is reported; the total zero is a compatibility placeholder.
A measured zero has `available:true` and `source:"provider_reported"`. Missing
counters have `available:false` and `source:"unavailable"`, even if the other
counter is known. UAR neither estimates tokens nor infers total from prompt.
These counters describe reported work; unavailable accounting does not establish
free work, a billing waiver or a settled cost liability.

The backend API preserves existing `embed(&[&str]) -> Vec<Vec<f32>>` consumers.
`embed_with_usage` returns `EmbeddingBatch { vectors, usage }`, where
`EmbeddingTokenUsage` holds independent optional `u64` prompt and total counts.
The default adapter implementation reports both unavailable. BackendSlot forwards
the usage-bearing operation to the same pinned backend instance.

The OpenAI-compatible remote adapter parses the documented `usage.prompt_tokens`
and `usage.total_tokens` fields. For a request split into chunks, a counter is
available only when every successful chunk reported it and checked addition did
not overflow. One absent counter or an overflow makes that counter unavailable
for the whole request; later reports cannot turn a partial sum into a complete
one. No provider call for an empty internal batch means no measured usage.
The [OpenAI embeddings reference](https://developers.openai.com/api/reference/resources/embeddings)
defines the upstream index and token fields; consulted November 2, 2026.

Remote chunks must return exactly one vector for every input with the complete
unique index set `0..input_count`. Missing, duplicate and out-of-range indices,
wrong dimensions, non-finite vectors, malformed JSON or malformed reported counts
refuse the whole batch. The public endpoint also checks dimensions, batch length
and finite values for every backend before returning float or base64 vectors.
A later failed chunk never produces a successful partial response.

Provider transport URLs, raw error bodies and parse errors do not cross the
OpenAI adapter's error boundary. Non-success status retains only its numeric HTTP
status. The mounted endpoint logs fixed error categories and returns generic
backend failure/input rejection text, preserving backend availability and request
error status contracts without echoing provider details.

## Authored qualification fixtures

No current-tree qualification is claimed here. All checks remain closed pending
whole-phase implementation and the operator's named combined batch approval.
Existing `tests/openai_embedding_batching.rs` contains authored remote-boundary
cases for independent chunk usage, nonzero/zero/unavailable counts, overflow,
index/dimension/usage corruption, private provider errors and pinned swaps.
The endpoint's existing helper tests cover the public extension and its vector-only
backend default, plus non-finite and private-input-error refusals. Those helper
cases do not establish mounted routing, identity, policy or rate-limit acceptance.

The existing `startEmbeddingStub(dimension, options)` fixture keeps its default
measured-zero behavior. Optional `mode` is one declared mode or a finite ordered
mode list, consumed once per request. Modes are `zero`, `nonzero`, `absent`,
`prompt-only`, `total-only`, `malformed-usage`, `duplicate-index`, `missing-index`,
`out-of-range-index`, `incomplete`, `bad-dimension` and `provider-error`.
Unknown modes/options and exhausted lists fail explicitly. `duplicate-index`
requires a batch of at least two inputs and refuses the fixture setup otherwise.
`holdResponses:true` retains owned responses; `embedding-request` events announce
arrival and `releaseResponses()` releases them for an in-flight settings-swap
scenario. The fixture never reads or records an Authorization header.

Mounted qualification must retain actual external identity/policy/limits/backend
arrival outcomes, safe usage responses and owned cleanup. Missing/invalid identity
lanes belong to externally provisioned ingress; functional scenarios never mint,
copy or send credentials. Existing C48/C49 evidence remains source-bound and is
not relabelled as evidence for this public endpoint.

## Native provider counts

Voyage v1 exposes `usage.total_tokens`, retained as total only; prompt stays
unavailable. Its [v1 response contract](https://docs.voyageai.com/reference/embeddings-api)
also defines the per-vector index. The adapter enforces its complete index set,
served dimensions and finite vectors before returning any batch.

Cohere's configured `/v2/embed` contract reports
`meta.billed_units.input_tokens`. These billing units are preserved separately as
`EmbeddingTokenUsage.billed_input_tokens`. They never become inferred prompt or
total processed tokens. The endpoint exposes them under
`uar_usage.billed_input_tokens` with `value` (a number or null), `available`, and
`source` (`provider_billed_units` or `unavailable`). A zero reported billing count
has value zero and is available. The [Cohere v2 example](https://docs.cohere.com/v2/reference/embed)
is the source of this field; consulted November 2, 2026. This contract makes no
claim that Cohere reported the other two counters. Both native adapters use the
same generic provider-error boundary and reject malformed/incomplete vectors.

## Mounted fixture provisioning contract

`scripts/behaviors/embedding-usage.cucumber.mjs` uses the existing Cucumber
facility. It has no retry, service startup or credential setup. Its feature and
steps are authored only. The external host supplies `UAR_BASE_URL`,
`UAR_WORKSPACE_ID` and a local `UAR_EMBEDDING_FIXTURE` JSON manifest. The fixture
must be a dedicated synthetic Linux amd64 candidate; every endpoint must be
literal loopback with no URL credentials, query or fragment. Source/image/config
fields bind the proposed receipt; the lead must independently establish the
serving process's identity, since matching a manifest's syntax proves no such
identity.

Required manifest fields:

| Field | Meaning |
| --- | --- |
| `dedicated` | Must be true; this candidate/store/peer/settings are exclusively owned by this qualification. |
| `namespace` | Alphanumeric/underscore/hyphen fixture namespace, at most 96 characters. |
| `identity` | `platform:"linux/amd64"`, full `sourceCommit`, local `imageDigest` (`sha256:` plus 64 hex), nonsecret `configDigest`. These are independent external receipts, never discovered by reading credentials. |
| `controlUrl` | Owned namespace-enabled embedding peer's loopback HTTP base URL. |
| `model`, `nextModel` | Distinct model identifiers beginning with `namespace-`; current and next configured embedding spaces. |
| `dimension`, `batchSize` | Served width, and actual backend chunk size at least two for the single-chunk mounted counter oracle. |
| `expectedVectors` | Frozen independent numeric vectors by model for `owned-first`, `owned-second`, in that order. Each row has the declared width; two model spaces must differ. |
| `refusalLanes` | External `missing-identity`, `invalid-identity`, `denied-policy`, `rate-limited`, `unavailable` lanes, each with this namespace and a loopback `baseUrl`. The external host provisions their admission behavior; scripts carry no credentials. |

Externally start the existing embedding peer with `namespace` set. Namespaced
peers salt their deterministic vector input with the request's actual model,
so the frozen two-model vector oracles differ. The peer's default behavior
without a namespace remains unchanged for existing scenarios. The host configures
the UAR's OpenAI-compatible embedding adapter to send to this keyless owned peer;
no script changes endpoint URLs, API keys or credential sources. The unavailable
lane uses the unavailable backend; the rate-limited lane rejects its first
request under externally fixed limits. Admission lanes share no competing
traffic and must not forward to a paid provider.

Namespace-enabled peer controls are `GET /__embedding/state`,
`POST /__embedding/control` with `{namespace, mode, holdResponses}`, and
`POST /__embedding/release` with `{namespace}`. Controls are disabled when no
namespace was supplied. The state retains monotonic arrival counts and at most
256 request summaries; overflow explicitly fails the scenario rather than hiding
older arrivals. No input text, headers or provider body is recorded. Release
settles only this peer's held responses. Namespace comparison is fixture resource
selection; it is not a production authentication mechanism.

The mounted suite covers nonzero, zero, unavailable, independent counters,
float/base64, both model-name forms, exact dimensions, malformed provider
usage/index/vector/status, body/batch/input/model/encoding refusal, external
admission lanes and concurrent settings change. It changes only the dedicated
`llm.embedding.model` field through the existing revision-checked settings API.
The first request must have arrived at a held old backend before the new model
is published. Old and new requests must match their separate frozen vector/model
oracles and usage modes. Cleanup releases held responses, awaits the admitted
request, and restores the exact owned model by revision. Any unexpected model,
uncertain response or failed restoration poisons further scenarios until external
reconciliation. This is not authority to modify another chat's settings.

Safe receipts contain exact supplied identity, HTTP path/status observations,
counter provenance, backend-arrival counts and cleanup status; raw public/provider
bodies and inputs are not attached. Logs and raw serving identity remain part of
the lead's final combined batch evidence, not an assertion that this authored
suite has already run.

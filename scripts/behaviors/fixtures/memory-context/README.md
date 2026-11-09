# Independent memory provider-boundary fixture

Authored only. ALL validation is CLOSED until the whole-set implementation and
operator's explicit named final batch. Nothing here starts a service or grants
permission to execute a peer, Cucumber, API call, build, import or probe.

The uncomfortable thing: a synthetic peer cannot demonstrate that a real model
understands a fact. It can independently capture what the production native
memory path sent to the model endpoint. No recall answer or self-reported peer
success substitutes for captured input and original native run evidence.

## External prerequisites

The final-batch operator/fixture owner supplies an absolute
`UAR_MEMORY_CONTEXT_FIXTURE` pointing to a completed copy of
`manifest.template.json`. Freeze exact full-set source/customer artifact and
per-profile configuration hashes; record the peer source hash. Placeholders are
not accepted by the authored steps. All HTTP URLs are explicit literal-loopback
and credential-free, using the same external-identity gateway convention as the
existing memory and owner-isolation features. Two gateways per profile supply
verified distinct principals against the same runtime. Keep each profile's
memory partitions empty and its independent memory store isolated from other
work. No test creates, reads, renews, mints, copies or sends a credential. External
infrastructure owns identity and provider-adapter setup outside scenarios.

Externally provision three runtime profiles against the exact same source:

- `positive`: memory enabled and inject_context true, auto_capture false,
  injection_min_similarity0.5, max_context_tokens at least4096; selected chat
  model context window32768, max output at least32.
- `absent`: same except injection_min_similarity null/absent in the actual memory
  service startup configuration. A null persisted override must remain null.
- `budget`: positive memory settings, but the selected model's genuine context
  window128 and max output32. The ordinary run must fail at context_overflow and
  dispatch zero model requests; no fake limiter or changed assertion is allowed.

Use each actual independently constructed MemoryService with
`memory.embedding_api: openai`, embedding_model `memory-context-fixture-2`,
embedding_dim2, embedding_base_url pointing at the owned peer's `/v1` base.
The existing memory adapter's runtime configuration prerequisites are supplied
by the external host, never by scenario steps or manifest credentials. Disable
automatic capture, model/provider retries, fallback routes and unrelated shared
memory data. Preserve actual selected native runtime and finite-budget paths;
configure available skill matching normally even though all skill/tool/KB/MCP/
presentation selection is explicitly none. Register and verify the dedicated
model(s) outside the scenarios. The peer supports provisioning READY responses.
Memory source/engine/provider construction remains production code; the peer
supplies deterministic vectors only, not a replacement retrieval implementation.

The externally owned peer's implemented launch inputs are
`UAR_MEMORY_CONTEXT_NAMESPACE`, `UAR_MEMORY_CONTEXT_PEER_PORT`, and
`UAR_MEMORY_CONTEXT_PEER_JOURNAL`. It binds127.0.0.1 only and creates a fresh
absolute journal exclusively with mode0600. The external owner launches/stops it;
steps do not use process/container helpers or restart shared services. Its routes:

- POST `/v1/embeddings` (also `/embeddings`): string inputs map to2D vectors;
  candidate content with MC_VECTOR_NEGATIVE maps to[-1,0], everything else[1,0].
  The journal records input hashes and vectors, never headers.
- POST `/v1/chat/completions` (also `/chat/completions`): captures the exact
  synthetic case request before responding. Modes success, error-first503,
  partial-error (positive text then missing-terminal EOF), opt-out and
  budget-refusal. A correct budget-refusal case never reaches this route.
- GET `/__memory_context/status`: namespace/protocol/dimension/ordinal only.

`memory-context.cucumber.mjs` is the dedicated existing-framework configuration;
it imports only its bound feature/steps. Do not run it before approval. The
fixture writes only unique disposable agent/session/memory rows through existing
APIs and removes those exact owned rows afterward. It neither changes runtime
settings nor starts services. Cleanup reconciles unknown memory-save outcomes by
listing only the owned principal and matching the fresh synthetic case identity;
it never repeats an uncertain mutation automatically.

## Assertion provenance

The fresh private synthetic fact is written by POST `/api/memory`, then read from
GET `/api/memory/records/{id}`. Fresh native POST `/v1/chat/completions` receives a
new session and stable turn/user-message IDs, exactly one question without the
fact. Tools, memory recall/list tools, KBs, skills and replayed assistant history
are absent both in policy and in the independently captured final wire request.

Positive evidence requires the full marked original memory fragment inside an
actual System message, exact original memory IDs in host submitted provenance,
and actual candidate/query re-embedding input hashes in the peer journal. The
below-cutoff case additionally requires the exact host
memory_below_similarity_or_removed reason and opposite vectors; merely no-match
without a demonstrated candidate comparison does not pass. Foreign uses the SAME
agent ID against the primary identity, so agent selection cannot explain refusal.
Deletion requires an actual not-found record read before the fresh query.

Submission evidence_basis is final_driver_request_and_provider_response. The
host request hash covers logical final-driver messages and anthropic_system;
the peer request hash covers literal protocol bytes. The fixture records both
without falsely asserting cross-dialect byte equality. It independently asserts
full System content, model and original IDs. Host provenance may contain no fact.

A first provider503 gives no positive response and must publish no submitted
claim. Positive partial text followed by missing-terminal EOF proves submission
but must end with native status error and preserved partial text. Submitted means
input evidence, not answer completion or recall success. Final context refusal
must be context_overflow, with zero model-peer arrivals and no submission; an
unrelated matching/admission failure cannot satisfy the case.

Peer raw bodies are synthetic-only fixture evidence, never production logging.
The create-only journal and per-case mode0600 receipts retain exact provider
observations. Cucumber's actual scenario result is recorded; the peer provides no
pass flag. No receipt or runtime result currently exists: all scenarios and
provisioning remain UNEXECUTED. Real-model recall, expired facts, remote service
faults and alternate dialects remain separately required final evidence rather
than claims inferred from this controlled fixture.

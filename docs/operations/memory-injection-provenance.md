# Automatic memory context selection

Status: P06 source authored for #474; runtime/provider qualification pending the
whole-set gate. Native memory tools retain their existing separate policy and
owner-scoped lifecycle. Memory remains an independent service, including with a
PostgreSQL primary. This does not add PostgreSQL-native memory or team memory.

The uncomfortable thing: a retrieved memory and a prepared prompt are not proof
that a model received it. Even an injected marker cannot establish understanding;
that requires a separate fresh-fact real-model result with history and memory tools
excluded as alternate sources.

## Operator relevance boundary

`memory.injection_min_similarity` is an optional inclusive cosine cutoff in
`[-1, 1]`. It must be finite. There is deliberately no numeric default. Configure
it through runtime configuration (`UAR_MEMORY__INJECTION_MIN_SIMILARITY`) or the
existing memory setting; null/absent means not configured and refuses automatic
injection with `memory_relevance_not_configured`. Restart applies the memory
service's effective settings. `memory.inject_context=false` still disables
injection; per-turn memory opt-out remains authoritative. Neither setting grants
permission to use memory tools or overrides their policy.

Automatic selection uses three existing owner-scoped hybrid lanes, each capped at
twenty results: current conversation, current agent without a conversation filter,
and owner-level search without agent/conversation filters. The first retains only
Session records for the current session, the second only matching Agent records,
and the third only User/Global records. Explicit agent/session narrowing on any
record still applies; Task records are excluded. Global is a classification
inside the same verified owner partition, never access to other owners.

The host merges retained lanes by stable rank round-robin (Session, Agent, owner),
preserves each lane's hybrid order, deduplicates original record IDs, then keeps
at most twenty candidates before relevance inference. The result is deliberately
nonexhaustive: inapplicable rows can occupy a backend lane's top twenty and crowd
out a lower-ranked applicable row. `no_match` means no retained applicable/
relevance candidate, not proof that the owner's entire store has no matching
fact. This bounded contract avoids broadening tool retrieval or adding a second
storage/query architecture. Fresh owner-level facts saved without a session are
no longer excluded by the new conversation's exact-session filter.

The service rechecks each original memory ID against complete owner,
current version/content, deletion and expiry controls. It embeds the query and
visible candidate contents through the same retained configured memory embedding
engine, compares cosine similarity against the operator cutoff, and retains the
merged ordering of eligible IDs. It rechecks current records after inference.
Historical row vectors are not compared with a potentially different current
model. No unowned entity semantic search or global embeddings query supplies
eligibility. Invalid, nonfinite, zero or mismatched vectors cannot authorize
injection. Retrieval/inference failure is unavailable, not lexical fallback.

The three retrieval lanes perform up to three existing hybrid searches rather
than one. Recomputing eligibility adds one query embedding plus a batch of up to
twenty memory embeddings beyond that retrieval work. Remote embedding services
may charge for that work. The existing service lifecycle owns these operations;
there is no second per-request engine. Configured embedding/model/store and final
provider input must be recorded by the acceptance receipt without exposing
credentials or fact bodies in diagnostics. Selection-time owner/version/TTL
rechecks do not claim an atomic transaction with a later remote model call.

## Observable provenance

`MemoryContextProvenance` contains `status`, `reason_code`, `retrieval_method` and
original `memory_ids`. It never contains fact text, endpoint or credentials.
Statuses are `disabled`, `opt_out`, `no_match`, `bounded_out`, `unavailable`,
`refused` and `selected_prepared`. A missing cutoff is `refused`; below-cutoff or
removed candidates are `no_match`; candidates excluded by the memory context
budget are `bounded_out`. `selected_prepared` means only that selection and the
bounded memory block were prepared. It is not a submission success event.

The lead-owned public request/runtime integration must carry this provenance
through final prompt assembly and report affirmative submission only when the
selected records are present in the actual dispatched request. Later policy,
representation/team restrictions, context fitting, refusal or provider admission
can exclude prepared material. Memory is always untrusted retrieved data and
cannot supply instructions, permissions, credentials or user approval.

## Final evidence cases

Extend the existing memory fixtures and controlled-provider observation, using the
externally provisioned local identity endpoint for functional scenarios; no test
step handles credentials. Use a fresh private nonce, a new empty-history
conversation and an actual selected tool list without memory recall/list tools.
Observe the final provider input independently of the answer and any memory event.
Repeat with injection disabled, turn opt-out, same subject/foreign issuer and
tenant, expired/deleted fact, no match, below configured cutoff, invalid embedding,
retrieval failure and final fit/refusal. No excluded fact may appear in input.

Record exact config/source/store/model and original memory IDs. Deterministic
provider wiring and real-model recall are separate results. Existing C39 tool
receipts remain historical tool evidence and do not certify automatic injection.

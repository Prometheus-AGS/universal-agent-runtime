# UAR user-outcome verification and configuration

This document describes the authored implementation for the active UAR1.3.0
phase. Verification is CLOSED. No build, integration result, installed-browser
result, provider availability or backend-parity PASS is established by this
document. The phase ledger and exact operator-approved packet govern execution.

## Configured completion and interfaces

Positive acceptance uses the configured genuine provider/model and fresh facts
with independently computed expected results. A successful protocol terminal and
a useful answer are separate predicates. The literal ordered manifest is
`scripts/acceptance/fixtures/scenarios.json`:97 cases, three repetitions on each
of Postgres+pgvector and disk-backed SurrealDB, producing582 planned rows.
These are planned counts, not observed results or a statistical reliability SLO.

Coverage includes mounted native/chat, OpenAI, Anthropic, ACP, legacy and current
A2A HTTP/gRPC, AG-UI modes, installed SDK programs and the shipped UI. A2A legacy
gRPC task polling is reported as polling, not live token streaming. Missing
models, assets, credentials or exact client artifacts block the relevant case;
there is no positive synthetic replacement or successful skip.

## Agent definitions and orchestrator

The protected domain deletion boundary refuses deleting `default-agent` and
`orchestrator-agent`. Verification requires refusal to preserve their actual
stored definitions and continued usefulness after restart. Delete-and-reseed
does not satisfy protection. Custom agent CRUD remains supported. Built-in
seeding inserts absent definitions and preserves existing operator fields.

The compiled orchestrator graph resolves the currently eligible registered
roster and classifies the request through its governed model binding. Selection
is separate from skill matching. An invalid classification fails explicitly;
it does not silently choose a fixed specialist. The host revalidates the roster
before ordinary child admission and forks the permitted dialogue. This is not
an atomic pin against every possible concurrent definition write.

Official AG-UI exposes `CUSTOM` with `name: uar.agent.selected`. Its value has
`sourceAgentId`, `agent: {id,title}`, `selectedThreadId`, `selectionMethod` and
`rosterRevision`. The legacy stream exposes `agui.agent.selected`. The selected
child identity is persisted before this event; it precedes the parent answer,
but is not a guarantee that the child's internal work has not already begun.
No hidden reasoning is exposed. The shipped UI displays a routing notice,
preserves it in history and suppresses duplicate replay announcements. Selection
is not a completion claim.

## Standard A2UI actions

A standard rendered surface snapshots its actual edited data model at action
time. Its typed platform action posts to the source native run's
`/api/uar/runs/{run_id}/a2ui/actions` endpoint. The accepted continuation ID is
recorded before attaching its actual stream; HTTP202 means admitted, not done.
The UI requires a successful matching terminal and confirmed message persistence
before reporting completion. It does not submit a second chat request or create
a fake user message to simulate continuation.

The UI supports one submitted action per source-run/surface pair. Same-ID
updates and remounts do not reset consumption. An uncertain POST is consumed and
is not blindly retried. A restored pending receipt with no live document-owned
task becomes unconfirmed. Accepted interruption retains the known continuation
ID. This does not close the backend's non-idempotent POST crash window.

## Skill selection, context and request budgets

`skills.matching.engine` distinguishes the default service engine from the
intent-classifier engine. A selected model-dependent method uses its admitted
resources or reports unavailability; its configuration label cannot stand in
for actual execution. Legacy classifier configuration changes require restart.
Context strategy evidence records actual structural changes and genuine summary
tiers, with protected tool groups and multimodal records preserved or explicitly
refused on overflow. Canonical persisted history is not replaced by a prepared
request copy.

`context.skill_reattachment` records content-free hashes and explicitly
approximate counts for actual initial bounded skill fragments. The acceptance
case checks fresh bodies and useful head facts on initial, warm and cold runs in
typed and legacy harnesses. It does not infer later-tool-round behavior from
that initial receipt. Token pressure follows existing truncation/reattachment
semantics; it is not an invented token-based activation denial. The separate
activation-count limit remains a distinct control.

Ordinary final-request budget receipts hash/count Liter's actual prepared wire
bytes. Approximate counting is not an exact vendor-fit guarantee. Strict exact
contracts remain separate supporting evidence. Model/test/team/native output
ceilings are finite under known admitted capacity; unknown capacity is not
replaced by an arbitrary context window.

Direct supported native/OpenAI-compatible structured requests forward the
requested format and validate final JSON before public structured success.
Structured streams withhold partial JSON until validation. Graph-plus-structured
requests explicitly refuse this unsupported combination; normal text graph
execution remains supported. Private helpers keep their own contracts.

## Cohere through the OpenAI embedding interface

The operator's corrected Bash profile declares
`COHERE_EMBEDDING_URL=https://api.cohere.ai/compatibility/v1`, with the existing
`COHERE_API_KEY` reference and `embed-multilingual-v3.0` model. UAR selects
`llm.embedding.backend: openai`, `vector_dimension: 1024` and
`send_dimensions: false`. The default for `send_dimensions` remains true for
existing configurations. Omission never changes the expected vector width;
returned vectors must still match it. See
[the Cohere configuration guide](config/cohere-openai-embeddings.md) and
[Cohere's compatibility parameter contract](https://docs.cohere.com/v2/docs/compatibility-api).

Regeneration proposes the configured multilingual-v3 model, English-v3 at the
same1024 dimensions and English-light-v3 at384 dimensions, using the existing
credential. The documented model sizes are not proof of this account's access.
See [Cohere's model reference](https://docs.cohere.com/v2/docs/cohere-embed).

After selecting an actual global embedding space, POST
`/api/uar/knowledge-bases/{id}/regeneration` admits the retained-document cohort;
GET at the same path reads durable progress. Old workers cannot publish into
the replacement generation. Incomplete or incompatible generations are not
searchable. The positive journey requires genuine grounded retrieval after
same-width and changed-width rebuilds, including interruption and real restart,
on both durable adapters. A metadata-only model edit is not this workflow.

## Source, fixtures and execution authority

The owned fixture uses the existing Liter launch-agent service, its actual
configured aliases and the separate configured Cohere production API. The
operator confirmed the Liter subscriptions have no incremental per-request
charge. The operator authorized separate USD10 ceilings for Cohere and OpenAI
embeddings, with no transfers. The current fixture selects Cohere v3 text only;
OpenAI remains an unused alternative. Cohere pricing is operator-supplied
USD0.10 per million input tokens. Actual charge assumptions, prior consumption
and exact execution still need signed packet adoption.

The shared forwarding ledger spans both stores and cold restarts. It reserves
attempts, output tokens and approved conservative charges before forwarding real
bytes, with independent durable provider pools shared by every binding. Unknown
outcomes remain held. Counters measure gateway requests, not the
gateway's physical internal retries; charge authority must cover that chain.
The ledger does not count local in-process FastEmbed work. A named read-only
service-identity observation reports process/config/listener evidence and its
limits without restarting the operator's service.

Private preparation creates only owned fixture credentials/state. Owned PG and
Surreal provisioning uses explicit immutable images, identities, mounts, ports
and finite readiness. Partial setup still attempts exact-owned teardown;
unavailable forensics remain unavailable and private retained state prevents
false cleanup acceptance. No pre-existing operator database is adopted.

`compose_fixture.py --propose` authors an inert proposal. Actual composition needs
a source-bound successful approved build, complete installed dependency/assets
inventory, private inputs and adopted resource authority. Derived artifact and
signature receipts cannot be included in their own preimages. Neither mode is
permission to run a gate. The source-authoring handoffs, predecessor14-obligation
mapping and unsigned exit contract are under the active KBD phase. All prior
failures, overruns and unknown history remain preserved. Archive and reflection
wait for actual accepted exit criteria.


## Selected platform verification round

The operator selected macOS, Linux, Windows and iOS, then explicitly chose to report Windows BLOCKED until an actual Windows host is available and to add a minimal iOS simulator embedded-SDK harness. Android is DEFERRED pending those results. Neither disposition is a PASS or deletion of platform implementation.

The authored platform controller runs97cases/582backend-repetition rows separately for macOS and Linux, then the iOS simulator seed/restore journeys. The same live budget owner and prior accounting span all three. iOS covers the embedded SDK with host-owned SurrealKV; it does not certify the server HTTP interfaces, both datastore adapters on iOS, or physical-device behavior. Both server adapters still require equivalent real outcomes on macOS and Linux.

Collection mode preserves the original first-failure mode as a default, isolates case state, records prerequisite blockers, and admits at most one host-authorized correction and same-item rerun. Unconfirmed cleanup or exhausted/unknown allowance stops unsafe dispatch. New source never inherits a different artifact's passing evidence. The corrected supporting-input preparer retains nonregular inventory entries separately, so nullable metadata cannot be mistaken for executable hashes.

Prometheus Parking Lot0.4.0 is already selected in source and both lockfiles. Its production admission/lifecycle integration remains the first planned certification gate, not an observed PASS. Fixture configuration and current-source artifacts, actual source review and execution approvals remain pending.

# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. Legacy HTTP, tools and background memory paths need one complete ownership contract.

## Goals / Non-Goals

Goal: Assistant memory is scoped by verified complete owner. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

Apply the verified principal namespace to write/search/direct-ID/update/delete/history and context retrieval through HTTP, MCP/native and background paths. Reuse MemoryService and existing index. Migrate only proven owner mappings, quarantine ambiguous legacy memory and ensure retrieved text never carries authority.

Keep all mutations in trusted host services, retain complete verified owner scope, and expose durable business state through explicit inspectable records. Use fallible typed persistence outcomes; do not mask an unsupported backend or outage with success. No kernel mutation authority, no automatic replay of unknown effects, and no new lease expiry replacing exclusion.

Alternatives: existing UAR services are preferred over an independent queue/workflow authority because admission, receipt and owner transactions must agree. Apalis/Temporal remain analysis references, not dependencies. Rebuilding existing approvals/observer/credential machinery is excluded.



## Candidate evidence

### cand-005: Existing UAR MemoryService and vendored surreal-memory 0.1.0 — adapt

Preserve the existing memory engine and extend host scoping, consent, deletion and source provenance. A new memory database would not repair the identity boundary.

- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/Cargo.toml) — Memory is a path dependency under vendor/git/surreal-memory-server; Cargo.lock reports 0.1.0 without registry source.
- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/uar/memory/service.rs) — Service exposes retrieval/capture/mutation helpers but user scoping passes subject IDs without the complete tenant/issuer identity.

Risks: Vendor tree is committed source, not the historical revision named in versions.toml. Every memory surface, including native/MCP, legacy HTTP and embedded SDK, needs current owner enforcement. Workflow mirror persistence is not assistant memory privacy certification.

## Risks / Trade-offs

The uncomfortable thing: a persisted row or green isolated test does not prove cross-process authority and restart correctness. Validate the actual production boundary and retain negative controls. New retries/guards must trace to the reported failure or real owner/effect/consent boundary; do not harden unrelated code.

No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

## Migration Plan

Reuse the credential phase's complete principal encoding and auditable mapping procedure. Rewrite records and vector/index owner filters coherently; quarantine unproven legacy ownership. Rollback must not expose subject-only memory through any surface.

Dependencies: dot-02-principal-credentials. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

Equal subjects across issuer/tenant plus malicious direct IDs cannot access or inject another owner's memory through any supported surface. Verify migration conflicts, history isolation, background capture scoping and prompt retrieval provenance using real store and service.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.


## Task3 source correction: sessions and adjacent controls

Source inspection exposed raw-subject session, ACP, native session search and KB
API/ingestion keys, authenticated conversation-policy fallback, and unbound
session-deletion accounting cascades. Private history now uses verified full
principal keys and a disjoint session namespace; authenticated policies have no
subject-only fallback. Knowledge adapters use independent principal tables and
protected ingestion namespace provenance (see `docs/operations/knowledge-principal-isolation.md`).
The generic anonymous administration save rejects a caller-selected authenticated
owner. Existing catalog/Presentation partition semantics remain unchanged.

Real qualification sources and upgrade limitations are documented in
`docs/operations/session-principal-isolation.md`. Source authoring is not a passed
integration gate; every test, build and deployment remains deferred to the full
expanded Dot boundary. Legacy signed-in records remain preserved and inaccessible
until independent ownership evidence supports an explicitly authorized migration.

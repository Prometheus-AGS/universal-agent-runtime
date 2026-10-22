# Changelog

All notable changes to Universal Agent Runtime are documented here. UAR follows
Semantic Versioning for Stable HTTP, event, configuration, and artifact
contracts identified in the product support matrix.

## 1.2.2 — 2026-10-22 (native contract passed; image proof pending)

- Wired the actual mounted OpenAI endpoint to the durable caller-tool loop with
  complete history, controls, output caps and existing protected admission.
- Corrected AG-UI cleanup observation and completed concurrent replay; A2A mailbox,
  binding revision, replay and observer cancellation; and ACP selected-agent and
  validated cold-session restoration.
- Re-resolve current agent policy for caller-tool actor roots while preserving
  remote restrictions. Corrected Anthropic optional history fields and explicit
  endpoint adapter capability identity.
- Added configurable native ingestion directory, preserving `/data/ingest`, and
  retained substantive accounting diagnostics. Added real all-five safety and
  warm/cold continuation gates, plus governed graph-child caller handoff and
  root/child producer correlation. Preserve Anthropic/ACP budget-error reasons and
  keep local budget refusals out of provider outage cooldown. Functional acceptance
  passed locally across all five interfaces, including warm/cold graph, budgets and
  settled mixed-host-effect crash recovery. Exact published-image proof is pending.

## 1.2.1 — 2026-10-22 (functional acceptance pending)

- Authored caller-owned function declarations, selection/parallel controls,
  durable handoffs, exact result-batch continuation, and replay projections
  across OpenAI Chat Completions, Anthropic Messages, AG-UI, A2A 1.0, and ACP.
  Host tools retain their existing trusted execution and governance paths.
- Added the [candidate integration contract](docs/releases/v1.2.1-kubo.md),
  including native wire examples, provider control restrictions, and authored
  acceptance entry points. Bound Anthropic execution uses the shared runtime;
  unbound direct forwarding does not acquire durable host continuation guarantees.
- The Intel macOS development binary compiled and linked with
  `server-full,postgres-backend`; functional acceptance remains unexecuted. The
  active `uar-post-lifecycle-verification-readiness` phase is authoring isolated
  acceptance lifecycle bindings before renewed inventory and execution approval.
  The operator requested immediate version-tag and Docker publication before
  acceptance. Publication receipts establish the resulting image digest; runtime
  acceptance remains pending.
  Frozen v1.2.0 artifacts and historical release documents are unchanged.

## 1.2.0 — 2026-10-22

- Recorded the existing multi-platform image from source
  `035bbdc702ba20daaa1add7294b35f5783ed6e0e` with canonical index digest
  `sha256:dec5284e97b78ff65a66ef19eb43632154e23ea946f05a1d241cf979dd09566d`.
  The release publishes `1.2.0` and its `v1.2.0` alias to this same index.
- Added streaming OpenAI budget-error terminals, MCP catalog bootstrap and
  retained transport cleanup changes, credential tenancy/migration APIs and
  authenticated credential capture, and retained AG-UI terminal projection.
- Documented Kubo integration limits: OpenAI client-supplied tools are not
  forwarded; AG-UI rejects frontend tools. See the
  [v1.2.0 integration contract](docs/releases/v1.2.0-kubo.md).
- Image build/publication succeeded; functional acceptance remains pending.
  The source tag is retrospective and does not establish a tag-triggered build.
  v1.1.0 acceptance does not certify this image.

## 1.0.0 — 2026-07-12

- Established the Stable minimal server bundle with embedded SurrealDB.
- Published versioned OpenAI-compatible, AG-UI, A2UI, MCP, configuration, and
  release-evidence contracts.
- Added dependency-owned capability bundles and reproducible offline builds.
- Added provider evidence tiers, governed tool execution, React Runtime
  Console certification, and end-to-end knowledge/RAG coverage.
- Added cross-platform archive validation, operational resilience testing,
  SBOMs, checksums, signatures, provenance, and non-root container evidence.

Preview and Experimental capabilities remain governed by
[`docs/product-support-matrix.md`](docs/product-support-matrix.md) and do not
receive Stable compatibility guarantees.

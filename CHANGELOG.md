# Changelog

All notable changes to Universal Agent Runtime are documented here. UAR follows
Semantic Versioning for Stable HTTP, event, configuration, and artifact
contracts identified in the product support matrix.

## 1.4.0-rc.4 — qualified Linux amd64 API prerelease

- Qualified the UAR APIs behind a trusted host or gateway, including owned restart and recovery, independent MCP and A2A, document extraction and terminal behavior.
- Published the recursively audited offline source archive and verified signed checksums, archive bundles, image signature and attestations. PostgreSQL 18 remains restricted Preview. [Signed release](https://github.com/Prometheus-AGS/universal-agent-runtime/releases/tag/v1.4.0-rc.4) · [Qualification record](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/465#issuecomment-6875246846).

## 1.4.0-rc.3 — Linux amd64 pilot candidate

- Embedded persistence moves to SurrealDB 4.0 with a separate-destination migration, an immutable source snapshot and an explicit rollback procedure.
- Public image requests retain content and order through compatible dispatch. Verified inherited routes may use authorized fallbacks that fit their final wire budgets. Explicit targets retain their original scope.
- Native turn history and A2UI action receipts retain original intent across lost responses. Standalone MCP and team cancellation preserve unresolved effects and accounting liabilities.
- Mounted embeddings expose per-counter usage availability. Anthropic Messages projects reported usage on streaming terminals. The document image ships pinned English OCR data and returns terminal rejection for invalid inputs.
- Automatic memory injection requires a configured similarity cutoff. Telemetry preserves host-owned global state and separates observed accounting from unavailable evidence.
- The selected synthetic baseline and team receipts remain tied to their recorded artifacts. The [RC3 user guide](docs/releases/v1.4.0-rc.3.md) documents setup, feature checks, recovery and outstanding final qualification.

## 1.4.0-rc.2 — release candidate (historical Mac receipts only)

Current #462–475 source integration and the selected Linux amd64 container APIs
remain unverified. The [canonical support/recovery contract](docs/product-support-matrix.md)
and [finite final checklist](docs/releases/v1.4.0-final.md) distinguish source
changes from actual installed acceptance. Historical rc.1/rc.2 receipts retain
their original artifact/configuration and do not certify this whole-set tree.

- Host integration: `security.registered_hosts` (default empty) lists the
  client ids of registered hosts, identified by a verified bearer's `act.sub`
  (else `sub`). A registered host may send `forwardedProps.uar.instructions` on
  the AG-UI endpoint as trusted run guidance (400 `invalid_host_instructions`,
  403 `host_instructions_host_authentication_required`). `GET
  /api/uar/settings/governance/status` adds `host_tools` (`global_mcp_servers`,
  `native_tools`, `tool_admission`) and `host_instructions`, admits a
  registered host without the admin key, and admits `X-UAR-Admin-Key` without a
  bearer. See `docs/AGUI_V1.md`.
- PostgreSQL 18 on the flint-forge image (`tribehealth/flint-forge-pg:18`) is a
  supported UAR store, in its own `uar` database. Settings keep their source
  across restarts (#389), session expiry reads canonical owner keys (#390),
  memory on Postgres runs through the memory service instead of claiming
  success (#393), skill embeddings accept any configured dimension (#395), and
  `GET /api/uar/persistence/capabilities` reports what the active store
  supports (#394).
- `POST /v1/embeddings` serves OpenAI-compatible embeddings.
- Configuration: `UAR_LLM__MODEL` and YAML `llm.model` are honored (#419); the
  configured memory embedding API is used, not silently replaced (#420); a
  request's output cap sizes the budget reservation (#385); the Rust SDK pin
  must equal the runtime version (#391). Agent mode on `/v1/messages` is
  documented (#396).
- Identity: JWT validation is frozen to one documented access-token profile
  (#422); a JWKS outage gets 503. With JWT optional, a malformed, expired or
  unverifiable bearer now gets 401 instead of anonymous access (#392).
  `security.local_loopback_owner` (off by default; requires the `trusted_local`
  profile) lets no-credential loopback callers act as the local-host owner.
  SSE parsing survives arbitrary byte splits (#380).
- Compiler specs and authoring sessions are owner-scoped; another owner's
  records answer 404, and records stored before this release stay visible only
  to the local-host owner (#387).
- Two turns arriving together on a new conversation share one session; before,
  the later save could overwrite the earlier turn's messages.
- On failover, each fallback model gets a request fitted to its own context
  window: optional history is displaced by the agent's strategy, and a request
  that still cannot fit is refused with `context_overflow` before it is sent
  (#388, partial).
- See [the release candidate note](docs/releases/v1.4.0-rc.2.md), which also
  carries the 1.4.0 final checklist.

## 1.4.0-rc.1 — 2026-10-29 (release candidate; Mac validated, other platforms pending)

- AG-UI route projects A2UI surfaces as `ACTIVITY_SNAPSHOT` events, restates
  live surfaces on each run, and accepts an A2UI button click as a new run on
  the same thread (refused for undeclared actions, mixed text, orchestrators).
- Fixes found by BDD: the TypeScript SDK's streaming no longer crashes in Node;
  non-streamed ACP `runs/get` returns the answer; `intent_classifier.*`
  settings take effect per turn; the localembedding classifier calibrates
  confidence; the llm classifier accepts reasoning-model output.
- Document extraction moves from kreuzberg 4.10.4 to xberg 1.4.0 (Know-Me-Tools
  fork, pinned rev). UAR's `kreuzberg` provider and settings names are unchanged.
- Dependencies at their latest compatible versions, including wasmtime 50,
  OpenTelemetry 0.33, utoipa 6, sqlx 0.9, argon2 0.6, TypeScript 6 and PEM 4.3.0.
- Mac validation: 99 of 102 catalog BDD scenarios pass through the real public
  APIs with no test credentials. E04, E06 and E07 are known limits (no
  capability-constrained dispatch, no image input). Linux, Windows, PostgreSQL,
  the desktop app and container images are not yet validated for this release.
  See [the release candidate note](docs/releases/v1.4.0-rc.1.md).

## 1.3.0 — historical release line (integration evidence remains source-bound)

- The retained [Parking Lot integration note](docs/releases/v1.3.0-parking-lot.md)
  records the later approved 0.4.0 source target and earlier 0.3.0 source history.
  Current dependency source is governed by `versions.toml`; neither upstream
  publication nor this historical note proves current UAR integration acceptance.
- Enforce bounded FIFO ingestion admission with one slot per document and
  locally queued descriptors separate from the durable accepted backlog.
  Source bytes load after admission; scope-owned execution retains capacity
  through actual worker joins and durable outcome handling.
- Preserve cooperative cancellation, retained shutdown, durable failure CAS
  and generation-fenced publication. Add admission snapshots/result receipts
  and finite integration cases for saturation, dropped waiters, failure/panic
  and dependency release ordering.
- Tests, builds and independent review remain unrun under the operator's CLOSED
  verification gate. See [the integration contract](docs/releases/v1.3.0-parking-lot.md).

## 1.2.3 — 2026-10-22 (published; scoped functional acceptance passed)

- Serialize caller-tool source acknowledgment across observers of the same live
  run while retaining actual producer joins, durable settlement CAS and exact
  authenticated continuation. Add fixed-stage diagnostics without logging tool
  payloads, credentials or arbitrary backend messages.
- Addresses the competing AG-UI confirmation paths observed during exact-image
  testing of1.2.2. The original inner error was discarded, so the precise failing
  operation is not claimed proven. Nativegate019 passed the complete finite
  CT1–CT6 contract. The exact published Linuxamd64 image then passed all nine
  finite groups (95core,60safety,fivecancellation plus recovery), with confirmed
  cleanup. Arm64 is built; its functional acceptance remains outside this gate.
- Preserves the five-interface caller/host ownership and continuation contract.
  Published1.2.0,1.2.1 and1.2.2 tags remain immutable. See
  [the1.2.3 integration contract](docs/releases/v1.2.3-kubo.md).

## 1.2.2 — 2026-10-22 (published; exact-image acceptance failed)

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
  settled mixed-host-effect crash recovery. The published Linuxamd64 image then
  failed1/95 core cases: AG-UI partial-result rejection setup returned
  `host_cleanup_unconfirmed`. Extended image acceptance did not run.

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

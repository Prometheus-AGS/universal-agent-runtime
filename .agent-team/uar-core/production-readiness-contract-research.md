# Production readiness contract research — #476

2026-10-31. Source inspection and authorized documentation writing only. No tests,
checks, builds, formatting, compiler probes, acceptance, paid calls, deployments,
or external mutations. Verification lock remains CLOSED for this teammate; the
lead owns any operator-authorized phase-exit gate. Other writers share this
checkout; no files beyond this handoff were changed.

## Position and evidence limits

The inspected waypoint still names `identity-pass-through`, revision 700,
implementation 144/170, ready, under `uar-verification-simplification`. It is not
an implementation-complete inventory for #476. The lead must reconcile the new
phase with canonical state rather than treating this research as acceptance.

Read `AGENTS.md`, `.prometheus/gotchas.md`, the current support/release documents,
selected recovery source/docs, and live GitHub issue bodies #463–476, #388, #421,
#392 and #422. Those issue bodies use baseline
`964e816854411cd12b3c85e5e0f11f03b0bf4892`. No fresh source/artifact qualification
was performed here. Historical receipts are inspected evidence, not rerun passes.

## Existing lanes and exclusions

| Lane | Existing contract | Qualification boundary |
|---|---|---|
| Supervised server/sidecar | `server-full` is the customer archive/container feature profile, with built React assets. Supervised service is the documented default host integration. | Select exact OS/architecture, store, public interfaces, model/capabilities and artifact. Image build success does not prove installed execution. |
| Minimal headless | Matrix labels `minimal` / SurrealDB Stable; Cargo default, excluded from customer archive. | Do not silently substitute this build for `server-full` qualification. |
| SurrealDB persistence | Stable default; durable sessions, runs, receipts and additional teams/observers/checkpoint capability. | Remote durability and abrupt owner takeover depend on explicit qualified deployment conditions; storage alone does not prove effect replay safety. |
| PostgreSQL | Preview, explicitly gated. PG18 Compose versus PG17 Helm/Kubernetes remains a profile discrepancy in #465. | Preserve passed #459 behavior and capability refusals; qualify selected cluster or exclude it. No full parity obligation. ARM64 wal-g backup remains unresolved upstream. |
| Memory | Separate MemoryService-owned SurrealDB store for either persistence backend. | PostgreSQL persistence is not PostgreSQL-native memory. #474 must distinguish automatic injection from memory tools. |
| Full harness | `full_harness_v1`, `process_ephemeral`, `unsupported_after_restart`, `steer_supported: false`. | Epoch/admission/cursor recovery must refuse/reconcile honestly; durable sessions/teams do not imply harness restart recovery. |
| Desktop/native WASM/mobile | Desktop and native WASM Preview, mobile Experimental; browser arbitrary WASM Unsupported. Windows archive Experimental. | No implication of installed Windows/Intel Mac/iOS/Android certification. AOT requires #467 before promotion. |
| Rust in-process embedding | Supervised sidecar preferred; embedding not automatically Stable. | Its host dependency/lifecycle/telemetry work becomes a gate only if selected. No mandatory kernel extraction. |

Primary local references: `docs/product-support-matrix.md` and `.json`,
`docs/operations/persistence-capabilities.md`, `docs/librefang-integration.md`,
`src/uar/api/full_harness.rs` (descriptor around lines 70–95).

PostgreSQL exposes durable tool receipts, client tools, MCP elicitations, AG-UI
runs, A2A bindings, run history, accounting, ingestion and approvals. Teams,
follow-ups, schedules, observers and checkpoint resume intentionally return
HTTP 503 `capability_unsupported`; both persistence layers reject memory writes
with `memory_service_required`. The runtime capability endpoint is authoritative
for the running store.

## Recovery contract to consolidate under #466

One canonical table should carry interface/profile, acknowledgement point and
crash survival, checkpoint scope, replay retention/order/gaps, event redelivery
versus re-execution, external effect/retry/idempotency limits, in-doubt outcome
reconciliation, and runtime/schema/SDK upgrade compatibility. Give each row an
evidence status and exact receipt binding; #465 qualifies selected promises.

Existing constraints to preserve:

- `docs/operations/supervisor-recovery.md`: restart alone is not trusted
  predecessor exclusion. Native mechanism is reboot-bound; same-boot crashes
  refuse reclaim. Kubernetes requires a separately qualified CRI/cgroup profile,
  independent journal and controller, with significant node inhibition. Unknown
  effects/budget liabilities survive exclusion. Missing proof blocks recovery.
- `docs/operations/approval-recovery.md`: protected pre-effect checkpoint and
  verified owner/workspace mapping precede continuation. Fresh host admission,
  immutable single-use claim, policy and resource renewal remain required.
  Readback failure is not success; unsupported Postgres resume stays unsupported.
  Documentation contains authored and historical commands, not current receipts.
- `.prometheus/verification/c54-team-recovery-macos-pass.json`: historical
  cancellation/restart preserved settled artifacts and attempt IDs; uncertain
  attempt remained reserved-unknown and was never redispatched. A new explicitly
  named retry is different work. Control-tool eligibility contradictions and no
  reserved-unknown reconciliation remain #475 obligations.
- `docs/releases/v1.4.0-rc.1.md`: A2UI retry with a new run ID can act twice;
  surfaces older than 64 runs are not restated. #468 owns logical-action identity
  and truthful expiration/truncation, not global exactly-once effects.
- C35 historical browser evidence leaves immediate reload loss and metadata
  fidelity residuals. #469 owns acknowledged-turn recovery and truthful terminal
  states; several cancel paths already finalize streams, so do not assume every
  older finding remains unfixed.

The uncomfortable limit: a reusable session ID, durable receipt, successful
reconnect, clean image build, or historical happy-path PASS does not establish
safe execution after an uncertain external effect or certify a new artifact.

## Priority and dependency sequence

1. #462 exact committed test-target health precedes new unit acceptance; prepare
   affected implementation independently. This research did not inspect/run it.
2. #466 can correct overclaims immediately and should define selected contract
   rows before promotion. July support matrix claims vision Tier 1 and broad
   Stable routing conflict with rc.1's blocked E04/E06/E07. Preserve historical
   artifacts, correct present claims, label catalog counts metadata-only.
3. #463 vision transport and #464 constrained real dispatch/fallback share
   capability semantics; both must preserve existing #388 final destination
   budget compilation, including smaller fallback windows. Metadata is not input
   transport or constrained dispatch evidence.
4. #467 is necessary for promoted WASM/AOT, #468 for effectful/long-lived A2UI,
   #469 for acknowledged chat, #470 for mounted embeddings, #471 for advertised
   Anthropic cancellation/nonzero accounting. #470 unknown usage must be distinct
   from measured zero. #471 inherits delivered #461 agent mode, not a rewrite.
5. P2 #472 capacity/reliability/comparison, #473 host-global telemetry, #474 auto
   memory injection, #475 team controls/accounting apply when the lane promises
   them. #472 needs owner thresholds/budget and pinned Rig or reasoned exclusion;
   expose unequal durability and benchmark glue. #474 missing event is an
   evidence gap, not proven injection failure.
6. #465 consumes these outcomes plus applicable existing owners #380/#387/#392/
   #422, pins exact artifact/profile and independently versioned MCP/A2A peers,
   and records finite passed/failed/blocked/skipped/excluded/historical outcomes.
   It does not create a new global runner or require every case on every OS.

Issue reference base:
`https://github.com/Prometheus-AGS/universal-agent-runtime/issues/<number>`.

## #421 transfer, not a reopened historical phase

The October 30 owner closing comment is authoritative:
https://github.com/Prometheus-AGS/universal-agent-runtime/issues/421#issuecomment-6520559495

Items 1 (three targets), 2 (budget API races/IDs/wrapper), remaining 3
(teardown/drain/reopen/gaps), 4 (migration/lost commit), remaining 5 (frozen
embedding), 6 (Windows lifecycle), and 7b (two-architecture evidence) transfer to
the planned 1.4-final checklist. Remote Surreal reconnect is delivered by #458;
provider wiring is covered by #420. Item 7a old image publication is obsolete;
item 8 warning is fixed in #458; item 9 memory upstream publication belongs to
that repository; item 10 bookkeeping follows actual evidence. No rc.2/final
checklist was present in inspected `docs/releases/`; #465 materializes it.

The rc.1 catalog's 99/102 macOS credits accumulate across several binaries.
It is neither 99 fresh current-candidate passes nor load/Windows/Linux/container
certification. Preserve C03 composite-evidence and other denominator notes in
`.prometheus/verification/bdd-validation-catalog.md`.

## Material owner decisions only

1. Which finite production lane(s) should this effort promote: exact deployment,
   OS/architecture, store, public protocol roles/transports, models/capabilities,
   and runtime/schema/SDK upgrade pair? Everything else can remain qualified
   Preview/Experimental/excluded. A bounded supervised text/tool server lane is
   consistent with current architecture, but is a proposed selection, not an
   operator decision recorded by this research.
2. If optional promises are selected, name scope before their gates: vision
   public paths, effectful A2UI, AOT target architectures, advertised extraction
   formats, independent protocol counterpart versions, automatic memory,
   embedded telemetry, and capacity/comparison targets with paid-call budget.
   No arbitrary mandatory SLO or all-provider/all-platform certification follows.
3. For identity, reconcile #392 trusted local ingress with #422's frozen narrow
   JWT-validation contract and newest owner directions. #422 explicitly excludes
   UAR-owned IAM/roles/policy/issuance/refresh and forbids silently disabling
   existing enforcement; a concrete conflict needs separate disposition.

No additional architectural decision is required to truthfully retain existing
ephemeral/refusal behavior, carry unchanged scoped evidence, publish exclusions,
or prepare the #465/#466 documentation.

## Handoff accounting

- Changed file: this handoff only. No unsolicited code, guards or dependencies.
- Commands were source reads (`cat`, `sed`, `rg`, `git remote get-url`) and GitHub
  read tools. No verification commands ran and none have acceptance credit.
- Unresolved: lane/optional promise/identity choices above; canonical waypoint
  reconciliation belongs to lead. Source inspection cannot select operator risk.
- Unverified: every fresh runtime, artifact, platform, supply-chain, conformance,
  performance and release-promotion claim. Historical evidence was not rerun;
  this document does not close any issue or open a verification gate.

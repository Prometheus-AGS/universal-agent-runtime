# Tasks

Scope: one complete `uar-outstanding-issues-v1-1-1` phase. Stable labels in brackets
are the Spec-to-Plan task identities; keep them when registering canonical tasks.
Checkboxes reflect the canonical task disposition. References to a gate below
specify future completion evidence, not permission to execute it after that
task. All production source, schemas, configs, fixture code and required docs
must be complete before each renewed whole-phase inventory. Only V1/V2/P2 run
review or acceptance. The active lifecycle stage is Execute; revision63 assigns
R11 and L9 before any affected V2 rerun.

## 1. Execution ownership and boundary

- [x] 1.1 [L1] Lead records the candidate baseline, exclusive file/build ownership, dependency handoffs and the complete implementation inventory template for this change, preserving prior work and cancelled Dot scope. Completion artifact: canonical execution ledger and source ownership map; no product check.

## 2. Credential authority and migration

- [x] 2.1 [T1] Trust implements `security.provider_credentials` policy types and private credential-owner binding: omitted single-owner compatibility, explicit shared issuer/tenant eligibility, exact issuer/tenant/provider/source grants, local-only stable installation UUID, unchanged external/general principal keys and safe errors. Deliver source plus policy documentation inputs; acceptance is the deferred `credentials` group.
- [x] 2.2 [T2] Runtime integrates the trust source-decision interface across primary, ordinary fallback, team, graph, summarization, SDK-supplied, inherited/delegated and credential-free bindings, including absent encrypted service; lead wires shared construction roots and administrative paths under L2. Preserve timeout/retry inheritance and do not turn a policy denial into fallback. Completion evidence is zero denied provider contact and allowed controls in deferred `credentials`/`combined` groups.
- [x] 2.3 [S3] State delivers additive credential row-revision and journal metadata schema upgrades for the T3 contract, preserving ciphertext, hints, timestamps, original owner rows and legacy receipt keys; initialize revisions through explicit supported upgrade under quiescence, never in preview. Hand exact schema/revision interfaces to the sole trust store writer. Deferred acceptance: migration, restart and old-receipt cases on actual Surreal and PostgreSQL plus the memory contract supplement.
- [x] 2.4 [T3] Trust extends credential stores/API with exact fixed-local mapping and `/migrate-principal`, metadata-only trusted-host `/migration-preview`, bounded pagination/counts, transactional source revision/preconditions, compatible optional legacy If-Match, immutable replay and explicit unresolved-source quarantine. Include all supported writers in revision maintenance and preserve strict legacy bodies. Deferred `credentials` acceptance proves conflict/drift/no-partial-copy/no-leak/rotation/deletion/replay controls.
- [x] 2.5 [T4] Trust authors the credential real-process and installed-image scenario adapters plus `principal-provider-credentials.md` cutover/rollback/preview guidance before the full-phase boundary. Include every T2 source seam, both migration sources, issuer/tenant/installation isolation, exact positive/negative grants and all backend cases. Delivered fixtures/docs establish implementation inventory only; run the phase `credentials` and `combined` groups through L3 at V2/P2.

## 3. Durable eligibility and supported recovery

- [x] 3.1 [S1] State adds safe captured store eligibility to existing persistence capabilities; runtime consumes it under R1. Preserve durable accounting, explicit budgets, owner partitioning, unknown liabilities and exact reconciliation. Deliver typed interface and diagnostic documentation inputs; deferred `binding`/`recovery` acceptance covers unattested refusal, qualified success and restart.
- [x] 3.2 [S2] State integrates safe eligibility diagnostics into knowledge upload/dispatcher and writes durable-ingestion/knowledge recovery guidance, incorporating runtime text by handoff. Describe authenticated reupload, retained legacy rows, unavailable originals, separate model verification, existing qualified quiesce/exclusion/reclaim and blocked takeover without proof. Deliver docs/source with no new KB import or fencing bypass; deferred `recovery` controls establish behavior.
- [x] 3.3 [S4] State authors durable-upload, owner-isolated reupload/retrieval, restart/liability and bounded existing recovery proof fixtures plus installed-image adapters; actual Surreal/PostgreSQL and owned process/volume boundaries are mandatory where supported. Include legacy retention, other-owner denial, absent/replayed proof and missing-source documentation cases. Fixtures remain unexecuted until V2/P2 `recovery` and `combined`.

## 4. Runtime completion, diagnostics and wire behavior

- [x] 4.1 [R1] Runtime classifies originating budget/accounting/inherited/team/client binding failures with typed safe stages/categories/references, maps S1 eligibility without raw error chains, retains ordinary/team public codes and AG-UI code/message fields, and preserves #384 settings. Deliver source and diagnostic guidance; deferred `binding` verifies safe public/log output and zero effects before refusal.
- [x] 4.2 [R2] Runtime requires observed orchestration completion before ordinary-manager success, preserving cancellation, joined cleanup, interrupted history, reservation ownership and existing graph/provider terminal guards. Deliver source and completion documentation; deferred `streams`/`combined` prove EOF failures and valid text/tool-loop controls.
- [x] 4.3 [R3] Runtime implements a local native-Anthropic adapter for initial BOM and CR/LF/CRLF with bounded incremental state around the pinned Liter decoder; tolerate only well-formed unknown top-level kinds and retain known-error/terminal semantics. Deliver source and framing documentation; deferred `streams` covers all two-way splits, bytewise/fixed partitions, UTF-8/escapes, fields, limits and EOF.
- [x] 4.4 [R4] Runtime preserves the existing typed MCP production path and completes missing peer scenario support for true/false/omitted isError, protocol error, timeout and uncertain effects, native incidental fields and later independent recovery. Do not rewrite already-correct outcome code without an observed defect. Deliver scenario fixtures and truthful-tool-outcomes documentation; deferred `mcp-outcomes` proves contact counts, payload meaning and immutable receipts.
- [x] 4.5 [R5] Runtime authors and integrates the full-harness diagnostic/completion/framing and combined failed-MCP/incomplete-stream scenarios with R4/T4/S4 adapters and required operator docs. Include sanitized resolved-root/path/error skill-warning triage without speculative pack edits. Deliver complete fixtures/docs; acceptance remains deferred to all applicable L3 groups.

- [x] 4.6 [R6] Complete confirmed #385 follow-up: safe per-call catalog reservation ERROR, persisted bounded-agent startup WARN, full remote-Surreal upgrade guidance and v1.1.0 advisory, real eligible 16,000-token refusal/startup observations, and actual unsupported-memory supplement. Keep budgets, timeout inheritance and the full-phase boundary intact.

## 5. Shared integration and complete implementation inventory

- [x] 5.1 [L2] Lead completes shared config/server/module-export and provider-admin/settings handoffs, operator examples, runtime Cargo package/lock version 1.1.1, and new `docs/releases/v1.1.1.md` setup/upgrade/compatibility text. Keep pins and existing workflow unchanged. Documentation source must be complete now; later observed receipt values are publication evidence. Deferred V2 exercises config/identity compatibility and production image version.
- [x] 5.2 [L3] Lead authors the NEW `scripts/release/uar_1_1_1_acceptance.py` runner and manifest contract/templates specified in verification.md; integrate domain adapters covering every known phase scenario. Deliver six phase integration groups, serial source/candidate/published modes, isolation/cleanup, exact identity checks, blocked/failed/interrupted receipts, single-failed-group rerun support and usage docs. No invocation, import probe or packaging check occurs while implementation is incomplete.
- [x] 5.3 [L4] Lead records an explicit implementation-complete inventory showing every L1–L3, R1–R5, T1–T4 and S1–S4 production/fixture/config/schema/doc deliverable present and all scope additions incorporated, with exact source/fixture identity and no uncertain implementation item. This is a source inventory, not a passed product gate. If any item is incomplete, leave the boundary closed and continue implementation.

## 6. Review and serialized local acceptance after L4

- [x] 6.1 [V1] After L4, obtain isolated artifact-only review under the repository team/review contract; reviewer receives artifacts/diff/requirements without generation history. Resolve findings and preserve the review receipt. The operator-selected real integration program follows this boundary; no routine schema or broad validation sweep is selected.
- [ ] 6.2 [V2] After V1 dispositions, populate concrete private local manifest and run the exact local invocation in verification.md, including only phase-specific real integration cases, the production Dockerfile candidate build and all six installed-image groups. Record each command/observation/backend and blocked or failed cases. Repair failures coherently and rerun only affected gates; proceed only with complete passing required evidence bound to the frozen source/fixtures.

## 7. Immutable release and evidence handoff

- [ ] 7.1 [P1] With V1/V2 accepted, commit bounded source/docs/evidence and push the authorized source plus immutable annotated v1.1.1 tag. Use the existing deployment workflow to publish native-built arm64/amd64 images; record source/workflow commit/run, index and child digests. A successful workflow or tag alone does not satisfy release acceptance.
- [ ] 7.2 [P2] Pull each published child by digest, populate observed arm64/amd64 manifests and run both exact phase integration invocations in verification.md locally and serially without rebuilding or patching UAR. Record native/emulated execution, fixture identity and each required group. Failure or missing evidence withholds success and never moves the tag.
- [ ] 7.3 [P3] Create/update the v1.1.1 GitHub Release with exact digests, complete setup/upgrade instructions and immutable accepted receipts; post criterion-specific evidence on #380/#385 and reconcile #379/#383 historical delivery while retaining #382 external dependencies. Separate actual customer recovery from isolated product repair, retain unverified criteria, and record publication/comment readbacks. Return the verified image reference only after P2 passes; append phase reflection/memory with plan-versus-delivery limits.

## Plan bindings

Execution order, exact file ownership, task-model assignments and canonical
backend-ID mapping are maintained in
[the phase plan](../../../.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/plan.md).
Each displayed task ID maps to the identically numbered canonical task; stable
labels remain unchanged. T3 implements the explicit local metadata upgrade
operation, L2 wires its CLI entry point, and T4 supplies its docs and fixtures
as specified in design.md. These deliverables remain inside the same boundary.

## Complete corrective cohort after round1 review

- [x] 4.7 [R7] Resolve V1-01 through V1-04 together: actual fallback identity, inert disabled fallback compatibility, raw transient log assertions, and independently verified workflow/source/image provenance; author all required controls and docs before a new boundary.
- [x] 5.4 [L5] Retain original L4 and record a fresh whole-phase implementation-complete inventory including R7, then allow the one corrective isolated re-review and selected serial integration program.

## Complete actual acceptance repair cohort

- [x] 4.8 [R8] Correct the observed classifier fixture import, JWT-provider initialization ordering, database readiness/timeout diagnostics and safe missing-Debug implementation together; no checks during repairs. Preserve authority and all failed evidence.
- [x] 5.5 [L6] Record renewed whole-phase19-item implementation inventory, then rerun the affected production build and six failed/blocked phase groups only. No third artifact review or broad validation is selected.

## Second acceptance repair cohort

- [x] 4.9 R9: repair complete local-02 cohort and genuine isolated supervisor prerequisites before retesting.
- [x] 5.6 L7: record renewed exact whole-phase implementation inventory after R9.

## Native bootstrap repair and renewed source boundary

- [x] 4.10 R10: repair the observed native supervisor bootstrap cohort, retaining failed stores and the scoped qualification contract.
- [x] 5.7 L8: record inventory06 for all 21 authored implementation items. The later local04 failure does not make that source inventory a passing acceptance result.

## Issue-first course correction after terminal local04

- [x] 8.1 D1: reconcile every #385 criterion against the terminal local04 binding/recovery result, four real-source supplements, current source and supported customer guidance. Record exact missing evidence and separate actual customer recovery from isolated product repair. Source inspection only; no gate or know-me mutation. Handoff: `coordination/handoffs/D1.md`; the #385 host event-history repair and all real acceptance remain pending.
- [x] 8.2 D2: map #387–#396 and #386 to the advertised 1.1.1 profiles and existing #380/#385 requirements; record which confirmed shared-tenant or advertised-capability gaps must block this release and assign the others to bounded successor KBD phases. Bind the existing uar-core roles and calculated delivery cadence without inventing acceptance results. Source-only disposition is in `coordination/handoffs/D2.md`; #387 production restriction and its acceptance remain pending in R11/V2.
- [x] 8.3 R11: repair the complete observed local04 cohort after D1/D2 source triage: the #385 host event-history presentation bypass, streams fixture/policy setup once source-confirmed, relay/Docker startup and owned cleanup, and the image identity discrepancy. Do not infer a state or MCP product defect from a container that never started. Restrict both compiler REST aliases for remote ingress or shared credential mode across every operation, preserve local single-owner behavior, document the profile limit and author a real two-principal denial case. Keep full owner-scoped compiler authoring in a successor phase and leave #387 open. Register exact role/file ownership before edits; add any other confirmed release-blocking scope to the same OpenSpec delta and production cohort. Preserve existing successful evidence only where source, fixture and artifact identity still match. No tests, builds or probes during authoring. Source-only handoff: `coordination/handoffs/R11.md`; Docker timeouts remain infrastructure observations, and all behavior awaits L9 and real acceptance.
- [x] 8.4 L9: record a renewed whole-phase implementation-complete inventory after R11 and any accepted Dot change. It must include every production, fixture, configuration, migration and required documentation item, with exact source/fixture identity and no unresolved implementation item. Only then may V2 resume the affected approved gates. P1–P3 remain serial and pending. Inventory: `implementation-inventory-20261017-07.json` (source authoring only; no gate result).

The active KBD task 6.2 remains in progress despite its latest local04 exit1.
D1/D2 establish scope, R11 and L9 reclose the implementation boundary, and 6.2
then reruns only affected approved local gates. The six installed-image groups,
four exact source supplements and both full published child-image programs remain
required. Do not close #385 from an isolated product receipt while actual customer
recovery remains unproved. The issue queue and cadence calculation are in
`.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/planning/issue-queue-and-cadence.md`.

## Observed local08 repair and renewed operator boundary

- [x] 8.5 R12: retain local08 terminal receipts and repair only its four observed issues: outer stream-open attempts exceeding selected model retry inheritance, owner-scoped legacy recovery fixture inventory, restarted credential backend host-port readiness, and explicit owner approval in the streams graph fixture. Preserve authorization, earlier receipts and all source handoffs. The two premature `git diff --check` invocations are violations, not evidence. No test, check or build during this task.
- [x] 8.6 L10: after R12 and every other phase implementation item, record a renewed whole-phase inventory with exact current source and fixture identity and no unresolved implementation rows. Present the inventory and exact proposed gates to the operator and await explicit instruction. A recorded inventory or 120-minute tick never opens V2.

V2/6.2 remains in progress and its affected resumption now depends on L10 and
the operator's explicit instruction. R11/L9 and their inventories remain
historical completed work.

## Local09 pre-build infrastructure repair and code-complete boundary

- [x] 8.7 R13: after the retained local09 binding command failed at Docker network creation, classify the absent isolated Docker prerequisite as blocked before image/group work and skip cleanup of a network never created. Preserve the failed receipt; source-only audit the current phase's #385, #387 and local08 implementation for actual missing code. No check, test, build or product probe during authoring.
- [x] 8.8 L11: after R13 and every phase implementation item, renew the whole-phase source/fixture inventory and record any unresolved implementation item explicitly. No inventory, clock tick or earlier instruction authorizes a gate; present exact commands and await a new instruction.

The previous inventory and all local09 manifests are historical or fail closed
after the R13 source edit. V2/6.2 and P1–P3 remain open.

The checked 8.8 row records the earlier primary-checkout L11 inventory. The
Boss 2.2.40 UAR source adds the frozen Liter 2.3.0 pointer, catalog, scoped
qualification repairs and fixtures. Its new production/fixture identities are
recorded in the proposed 2026-10-19 whole-phase inventory. The generated
canonical `tasks.md` still marks 8.8 pending for this assembled source; only
the root lead may transition it after reviewing the inventory. No current
source acceptance or publication has been recorded.
# Current certification amendment — function-first child, 2026-10-17

Existing checkbox states and historical cohorts below are retained. The active
child plan at `.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/children/function-first-release-certification/plan.md`
controls new certification work and supersedes earlier full published-program
and automatic post-inventory gate instructions. Child task identity includes its
full phase ID; the IDs below do not reopen or renumber parent tasks.

| Child ID | Stable | Implementation/evidence deliverable |
| --- | --- | --- |
| 1.1 | FC1 | Scope, ownership and foundational acceptance/spec amendments. |
| 2.1 | FC2 | Runtime/provider/timing reader and reduced functional scenarios. |
| 2.2 | FC3 | Credential/compiler/recovery scenario consolidation with retained assertions. |
| 3.1 | FC4 | Closed prebuilt runner/schema integration, exact preparation runbook and final docs. |
| 3.2 | FC5 | Whole parent/child implementation inventory; local preparation selection and published templates. |
| 4.1 | FC6 | Separately authorized image preparation and fresh same-image qualified supervisor staging. |
| 4.2 | FC7 | Explicitly authorized independent artifact review. |
| 4.3 | FC8 | Exact authorized local command and operator timing disposition; parent V2 evidence. |
| 5.1 | FC9 | Reflection and parent V2/P1–P3 handoff with separate completion states. |

Only FC1–FC5 are implementation work. The root lead owns canonical task events;
this table is a scope mapping, not completion evidence. Gate CLOSED: no tests,
checks, builds, import/compiler probes, acceptance or external gates while
implementation is incomplete or the exact operator instruction is absent.

Parent V2 now requires all six reduced local groups and the four unchanged native
supplements against the prepared image; parent P2 requires streams/recovery on
each separately prepared immutable child. Their omitted per-child budget,
compiler, credential-migration and MCP-failure assurance must be disclosed.
Every recovery run needs fresh real predecessor/replacement/physical boot state
and an unconsumed proof from the genuine qualified controller. FC4 completes
runner/docs integration before FC5; historical L11 cannot certify edited fixtures.


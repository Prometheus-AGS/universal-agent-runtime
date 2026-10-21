# PLAN: uar-outstanding-issues-v1-1-1

> Current certification authority: the 2026-10-17 function-first amendment at
> the end of this document supersedes older gate-opening and full published-group
> instructions. Historical task states and receipts are preserved. Gate CLOSED.


Project: Universal Agent Runtime. Date: 2026-10-16. Stage: Plan.
OpenSpec backend: existing change `uar-outstanding-issues-v1-1-1`.
Changes: **one coordinated change, 22 pending tasks**. Baseline
`91a091d647fe2ce63651c07140172aba2c6c6302` on
`codex/uar-1-1-0-kubo-conformance`.

## Delivery and limits

Repair the selected #385 binding/upgrade diagnostics and #380 completion,
framing and credential defects. Preserve #379/#383 conformance and immutable
v1.1.0; publish v1.1.1 only after full source/candidate and both published-image
acceptance. #382 remains an external dependency tracker. Cancelled Dot
qualification stays cancelled. No UI, new dependency, vendor edit or global
principal migration is included.

**The uncomfortable limit:** safe diagnostics cannot make an unattested customer
store durable, recover missing KB originals or establish takeover authority.
Product acceptance and actual know-me recovery remain separate statuses. The
read-only cluster permission does not authorize rollout or data migration, and
`site-chat-kill-switch` remains untouched. Local credential ownership intentionally
changes and requires explicit installation configuration and migration; default
single-owner behavior is not a shared-tenancy guarantee.

## Authority and implementation boundary

The OpenSpec proposal/design/eight deltas define behavior; its tasks.md keeps
existing checkbox identities. This plan and execution-ledger.json define order,
ownership and dependencies. Canonical KBD task events own completion status;
the ledger's initial pending fields are not a parallel status authority.
A backend ID such as `2.2` is canonical within this phase/change; `T2` is its stable
semantic label. Registering a task does not complete it.

All 16 production tasks (L1–L3, R1–R5, T1–T4, S1–S4), including every fixture,
migration, shared-root change, operational setting and required document, precede
L4. L4 must record exact implemented files, source/fixture identity and unresolved
items; uncertainty keeps the boundary CLOSED. Any scope addition joins that same
inventory. V1/V2 and publication remain pending until their dependencies pass.
No test, Cargo command, lint, format, compiler probe, schema validation, build,
packaging check, model probe, or independent review runs during Plan or partial
implementation. Source inspection and reasoning are the available feedback.

The project KBD Plan skill supplies the lifecycle; its generic per-slice checks,
new-change commands and early sycophancy/review suggestions are superseded by the
existing Spec and full-phase boundary. Reuse the existing OpenSpec change rather
than creating duplicates. The previous setup refresh receipt is retained as
historical metadata, not proof of current registry freshness. No integration
regeneration or dependency upgrade is needed to author this plan.

## Ordered execution ledger

Sequence is a deterministic topological traversal, not permission for overlapping
writers. Independent tasks may run together only under the rounds below.
Task detail and completion deliverables remain in OpenSpec tasks.md and the three
domain planning packets. Complexity is comparative S/M/L, not a time promise.

| Seq | Backend / stable | Outcome | Owner | Depends on | Complexity | Deferred acceptance |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 1.1 / L1 | Record exclusive ownership and complete-phase inventory | uar-lead | none | S | inventory |
| 2 | 3.1 / S1 | Capture durable-store eligibility | uar-state | L1 | M | binding/recovery |
| 3 | 2.1 / T1 | Implement credential tenancy and bound owner | uar-trust-tools | L1 | M | credentials |
| 4 | 4.3 / R3 | Normalize native Anthropic framing | uar-runtime | L1 | M | streams |
| 5 | 2.3 / S3 | Add credential revision schema upgrade | uar-state | T1 | M | credentials |
| 6 | 3.2 / S2 | Integrate ingestion diagnostics and recovery guidance | uar-state | S1 | M | recovery |
| 7 | 2.4 / T3 | Implement conditional credential preview and migration | uar-trust-tools | T1, S3 | L | credentials |
| 8 | 2.2 / T2 | Authorize every model credential binding seam | uar-runtime | T1, T3 | L | credentials/combined |
| 9 | 4.1 / R1 | Preserve safe typed run-binding diagnostics | uar-runtime | S1, T2 | M | binding |
| 10 | 4.2 / R2 | Require observed ordinary orchestration completion | uar-runtime | R1 | M | streams/combined |
| 11 | 4.4 / R4 | Complete typed MCP outcome scenarios | uar-runtime | R2, R3 | M | mcp-outcomes |
| 12 | 2.5 / T4 | Author credential fixtures and cutover guidance | uar-trust-tools | T2, T3 | L | credentials/combined |
| 13 | 3.3 / S4 | Author durable recovery fixtures | uar-state | S2, T3 | M | recovery/combined |
| 14 | 5.1 / L2 | Integrate shared roots configuration version and release docs | uar-lead | S1, S2, T1, T2, T3, R1, R2, R3 | M | binding/credentials/kubo |
| 15 | 4.5 / R5 | Integrate runtime and combined production fixtures | uar-runtime | R1, R2, R3, R4, T4, S4, L2 | L | binding/streams/combined |
| 16 | 5.2 / L3 | Implement complete source and image acceptance runner | uar-lead | R5, T4, S4, L2 | L | all six phase groups |
| 17 | 5.3 / L4 | Record full-phase implementation-complete inventory | uar-lead | L1, S1, T1, R3, S3, S2, T3, T2, R1, R2, R4, T4, S4, L2, R5, L3 | S | source inventory only |
| 18 | 6.1 / V1 | Obtain isolated artifact review after complete implementation | uar-verifier | L4 | M | isolated artifact review |
| 19 | 6.2 / V2 | Run serialized local source and candidate acceptance | uar-lead | V1 | L | local full program |
| 20 | 7.1 / P1 | Publish immutable v1.1.1 source tag and images | uar-lead | V2 | M | deployment publication provenance |
| 21 | 7.2 / P2 | Run full acceptance on both published architecture images | uar-lead | P1 | L | both published full programs |
| 22 | 7.3 / P3 | Publish release receipts issue dispositions and reflection | uar-lead | P2 | S | release/issue readbacks |

## Execution rounds and shared-file handoffs

1. **L1 lead:** establish execution record, ownership and inventory from this plan.
   No implementation task is marked complete merely because planning files exist.
2. **S1 state + T1 trust + R3 runtime:** independent bounded production packets.
   At most four active agents including lead; all use the same preserved checkout.
3. **S3 then S2 state:** deliver additive schema/revision interface after T1;
   integrate captured eligibility and recovery docs. **T3 trust** starts after S3
   handoff. Runtime R3 may finish its source/docs while those dependencies proceed.
4. **T2 runtime after T3**, then **R1**, **R2**, **R4** sequentially. Trust writes
   **T4** after T2/T3; state writes **S4** after S2/T3. These are authoring lanes,
   never permission to run their fixtures. Runtime owns its manager/bindings files
   throughout; trust supplies interfaces without writing them.
5. **L2 lead:** finish shared-root construction, administration, CLI, config/version
   and release docs after its listed dependencies. L2 may land prerequisite wiring
   incrementally through exclusive lead handoffs, but remains pending until complete.
6. **R5 runtime:** integrate fixture contributions after T4/S4/L2. **L3 lead:**
   integrate complete adapters and the existing Kubo suite, manifest templates and
   receipts after R5. No fixture import/compile probe is permitted.
7. **L4 lead:** record full implementation inventory. Then **V1 independent reviewer**,
   **V2 lead serialized local acceptance**, **P1 publication**, **P2 both published
   architectures**, **P3 receipts/issues/reflection**, in that order.

A dependent task receives a written source handoff, not a passed-test claim.
Each packet records exact files, proposed/actual public and internal signatures,
source state, remaining decisions, unverified behavior and the next owner. Store
packets at `coordination/handoffs/<stable-id>.md` (NEW future artifacts). New files
outside the assigned set require the lead to update ownership/inventory first;
this is routine coordination, not permission to test. No two agents write the
same path. No active background build or test writer is permitted before L4.

## Reuse and source contracts

- `cand-001` (adapt) / `cand-003` (reference): R3 wraps pinned Liter framing;
  `cand-002` stays rejected. No replacement parser or vendor changes.
- `cand-004` (adapt): T1/T3/S3 extend existing encrypted credential and journal
  mechanisms. T2 consumes trusted source decisions on every binding seam.
- `cand-005` (adapt): S1/S2/S4 retain existing accounting, ingestion and qualified
  recovery mechanisms. No budget removal, blind durability flag or forced takeover.
- Pins remain Liter 2.1.1 and SurrealDB 3.3.0 as recorded in versions.toml. Existing
  source supports the decisions; execution compatibility has not been proven.
- Plan concretizes the already specified explicit metadata upgrade as the NEW
  local CLI `credentials initialize-revisions --writers-quiesced`. T3 owns the
  host operation, S3 schema assets, L2 CLI wiring, T4 fixtures/docs. The required
  flag is an operator acknowledgement, not proof all replicas stopped. No automatic
  startup/preview backfill, public HTTP upgrade API or kernel mutation is added.

## Acceptance adapter handoff (NEW implementation contract)

L3 owns `scripts/release/uar_1_1_1_acceptance.py` and the NEW shared
`scripts/release/uar_1_1_1_support.py`. Domain adapters expose
`run_group(context) -> result`; these names are proposed interfaces to implement,
not existing APIs. Modules have no launch/import side effects. The immutable
context contains selected group, UAR child image identity/platform, private
infrastructure, owned resource/evidence roots, config/fixture identity and
runner-owned lifecycle helpers. It never permits a domain module to rebuild or
replace UAR in published mode. Private credential values do not enter receipts.

Results enumerate mandatory scenario IDs, positive and negative observations,
peer counts, persisted readback, output artifacts, status and failure reason.
Missing mandatory observations or collaborators are blocked, never a skipped pass.
L3 owns process cleanup and receipt writing; adapters return observations and
resource handles, and join their owned children. No arbitrary manifest commands.
R5 owns the combined provider/MCP/storage scenario; T4/S4 contribute APIs and
scenario inputs by handoff rather than editing its driver.

## Final acceptance and publication order

The exact argv, collaborators, isolation and observation matrix are in
[verification.md](../../../openspec/changes/uar-outstanding-issues-v1-1-1/verification.md).
Its commands are future gates, not commands executed during planning:

1. After L4, V1 receives only frozen artifacts/diff, requirements and constraints;
   no producer history or preferred conclusion. Resolve findings before V2. No
   routine OpenSpec/schema validator is selected under the operator correction.
2. V2 builds the production Dockerfile candidate and runs all six phase image
   groups with their named real source supplements serially. Whole source targets
   and unrelated cases are not selected.
   The lead is sole build-target owner. A fixture-only/unit check cannot substitute.
3. P1 commits accepted source/evidence, pushes immutable v1.1.1 and uses existing
   deploy.yml for build/publication only. No workflow edits are selected. If needed
   later, resolve the mandatory policy-validator exception before touching them.
4. P2 pulls arm64 then amd64 child digests and runs both entire six-group phase programs
   locally, with exact source/fixture/image identities. No replacement binaries,
   source patch, mutable tags or source-only results qualify published acceptance.
5. P3 publishes GitHub Release/setup/compatibility/receipts and criterion-specific
   issue comments. Reconcile #379/#383 while retaining #382 external scope and any
   unproven customer recovery criteria on #385. Never silently move a release tag.

All concrete execution manifests and acceptance receipts are future outputs.
Use actual observed digests, not placeholders. Passing gates stay preserved;
repair only observed failures and rerun affected gates. A scope revision reopens
implementation inventory. Native/emulated execution and companion architectures
must be disclosed accurately; no native-amd64 hardware promise is made.

## Task model assignments

Assignment key is the full phase path + change ID + backend task ID below.
Native Codex tool-enabled workers are selected, with no liter proxy or account
switch. 2026-10-16 evidence: current collaboration schema exposes these role
models/efforts, project `.agent-team/uar-core/model-policy.md` and native role
files agree, and runtime/trust/state authoring routes were invoked this turn.
This establishes route availability, not implementation quality or a provider
benchmark. Policy fixes the role models; no cost or unsupported ranking claim is
made. The current lead keeps its configured model; a fresh uar-lead role uses the
listed project model. Reviewer independence is by fresh context, not claimed
cross-family independence; cross-family availability must be established at V1.

All workers use this repo cwd and exact ownership packets. Launch domain roles
via `collaboration.spawn_agent` with `fork_turns: none`; never run a review worker
before L4. The lead coordinates through the canonical ledger and handoffs.

| Phase path | Change ID | Backend task ID | Requirements | Provider/model | Effort | Rationale / dated evidence | Harness/route | Launch/handoff | Native alternative | Availability / prerequisites |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 1.1 (L1) | integration, provenance, sequencing | OpenAI/gpt-6-astra | high | project role constraint + tool schema, 2026-10-16 | Codex native | current lead; role for fresh handoff; coordination/handoffs/L1.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 3.1 (S1) | durability and ownership | OpenAI/gpt-5.6-terra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-state; coordination/handoffs/S1.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 2.1 (T1) | credential authority and atomic migration | OpenAI/gpt-6-astra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-trust-tools; coordination/handoffs/T1.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 4.3 (R3) | Rust async lifecycle and protocol boundaries | OpenAI/gpt-5.6-terra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-runtime; coordination/handoffs/R3.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 2.3 (S3) | durability and ownership | OpenAI/gpt-5.6-terra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-state; coordination/handoffs/S3.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 3.2 (S2) | durability and ownership | OpenAI/gpt-5.6-terra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-state; coordination/handoffs/S2.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 2.4 (T3) | credential authority and atomic migration | OpenAI/gpt-6-astra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-trust-tools; coordination/handoffs/T3.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 2.2 (T2) | Rust async lifecycle and protocol boundaries | OpenAI/gpt-5.6-terra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-runtime; coordination/handoffs/T2.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 4.1 (R1) | Rust async lifecycle and protocol boundaries | OpenAI/gpt-5.6-terra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-runtime; coordination/handoffs/R1.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 4.2 (R2) | Rust async lifecycle and protocol boundaries | OpenAI/gpt-5.6-terra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-runtime; coordination/handoffs/R2.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 4.4 (R4) | Rust async lifecycle and protocol boundaries | OpenAI/gpt-5.6-terra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-runtime; coordination/handoffs/R4.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 2.5 (T4) | credential authority and atomic migration | OpenAI/gpt-6-astra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-trust-tools; coordination/handoffs/T4.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 3.3 (S4) | durability and ownership | OpenAI/gpt-5.6-terra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-state; coordination/handoffs/S4.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 5.1 (L2) | integration, provenance, sequencing | OpenAI/gpt-6-astra | high | project role constraint + tool schema, 2026-10-16 | Codex native | current lead; role for fresh handoff; coordination/handoffs/L2.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 4.5 (R5) | Rust async lifecycle and protocol boundaries | OpenAI/gpt-5.6-terra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-runtime; coordination/handoffs/R5.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 5.2 (L3) | integration, provenance, sequencing | OpenAI/gpt-6-astra | high | project role constraint + tool schema, 2026-10-16 | Codex native | current lead; role for fresh handoff; coordination/handoffs/L3.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 5.3 (L4) | integration, provenance, sequencing | OpenAI/gpt-6-astra | high | project role constraint + tool schema, 2026-10-16 | Codex native | current lead; role for fresh handoff; coordination/handoffs/L4.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 6.1 (V1) | independent artifact reasoning | OpenAI/gpt-6-astra | high | project role constraint + tool schema, 2026-10-16 | Codex native | fresh uar-verifier; coordination/handoffs/V1.md | same project role; no alternate probe | deferred until L4; load review skill and establish judge independence |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 6.2 (V2) | integration, provenance, sequencing | OpenAI/gpt-6-astra | high | project role constraint + tool schema, 2026-10-16 | Codex native | current lead; role for fresh handoff; coordination/handoffs/V2.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 7.1 (P1) | integration, provenance, sequencing | OpenAI/gpt-6-astra | high | project role constraint + tool schema, 2026-10-16 | Codex native | current lead; role for fresh handoff; coordination/handoffs/P1.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 7.2 (P2) | integration, provenance, sequencing | OpenAI/gpt-6-astra | high | project role constraint + tool schema, 2026-10-16 | Codex native | current lead; role for fresh handoff; coordination/handoffs/P2.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |
| uar-outstanding-issues-v1-1-1 | uar-outstanding-issues-v1-1-1 | 7.3 (P3) | integration, provenance, sequencing | OpenAI/gpt-6-astra | high | project role constraint + tool schema, 2026-10-16 | Codex native | current lead; role for fresh handoff; coordination/handoffs/P3.md | same project role; no alternate probe | native route exposed; task dependencies and owned files required |

## Prior lessons, prerequisites and handoff

Memory recall reports its library unavailable. Local `.prometheus/gotchas.md`
lessons govern order: remote health is not durability evidence; raw subjects
cannot adopt private rows; the whole phase includes docs/fixtures; a global
Docker architecture override can break companion pulls; previous acceptance
cannot certify changed source. The before-hook is administrative, not product
verification. No linked evolution drives this phase; unrelated legacy `.evolver`
records require no bridge.

V2 needs isolated actual Surreal/PostgreSQL, durable volume qualification, Python,
Docker, pinned sources and protocol clients. P1/P2 need authorized registry and
GitHub access plus both published child manifests. These prerequisites are not
probed early. Missing access or observations become BLOCKED evidence at the gate.
Live customer IDs, mappings, grants, backing storage and originals are operator
data; no invented values are written to the deployment.

No product implementation or verification was performed in Plan. Exact file
assignments are in [ownership.md](ownership.md); authored domain packets live in
`planning/`. The implementation-inventory template is an unfulfilled checklist,
not L4 completion. Next is `/kbd-execute uar-outstanding-issues-v1-1-1`, beginning
with L1 and honoring every dependency above.


## Execute scope correction — 2026-10-16

The operator explicitly limited validation to real integration of the known
scenarios supported by this phase. This supersedes earlier full-Kubo, whole-target
and routine schema-validation instructions in this document. Production scope,
implementation-first ordering, isolated artifact review and release provenance
remain. Required acceptance groups are binding, streams, mcp-outcomes, credentials,
recovery and combined on the candidate and both published child images. No
unrelated tests, unit-only substitutes or broad validation sweep is selected.
verification.md is updated accordingly; each actual integration case must be
registered from implemented fixtures before L4.

## Confirmed #385 follow-up incorporated before L4 — 2026-10-16

The operator explicitly required every #385 issue/comment criterion. Comment
6213136510 confirms a second real failure after durability attestation: the
16000-token envelope cannot admit the conservative full catalog context/output
reservation. R1/L2 add safe call-admission ERROR diagnostics with typed cause,
reference and nonsecret reservation/limit values, plus the requested startup
WARN for persisted bounded agents on unsupported storage. R5 adds the actual
eligible durable 16,000-token refusal and startup observation to the same binding group;
existing adequate-envelope and inherited timeout controls remain. Do not loosen
budgets or infer a smaller final-wire bound from a request output limit alone.

Lead adds remote-Surreal upgrade guidance and an additive v1.1.0 advisory covering
attestation, budgets, verification, legacy KB reupload and exact owner recovery.
issue-385-coverage.md maps every body/comment criterion, including the independent
team warning and retained skill triage. All work remains in this phase and the
same closed implementation boundary. Prior source completion is not acceptance.
The operator reports v1.1.0 working; this is not a v1.1.1 gate result.

## Round1 complete repair cohort — 2026-10-16

Round1 identified V1-01 through V1-04. The boundary is closed again under R7 (4.7), followed by L5 (5.4). The runtime teammate exclusively owned bindings.rs, manager.rs, runtime adapter and provider fixture; its R7-runtime handoff releases those files. The lead owns shared support/raw-log access, published provenance runner/schema/docs, canonical records and the renewed inventory. No workflow change is selected: the runner independently checks the existing successful GitHub push run, annotated tag commit and both actual digest-export artifacts against the registry index. All repairs and docs precede any corrective review or product gate. Preserve round1 evidence; no earlier acceptance certifies changed source.

## First real acceptance repair cohort — 2026-10-16

The exact local program exited1, with binding/MCP/credentials failed and streams/recovery/combined blocked. No group passed. Production image build succeeded with one missing-Debug warning. R8 corrects all observed code/fixture defects coherently; L6 precedes all new verification. Runtime support import uses the public matching module. Trusted credential fixture initialization follows the existing public verifier before raw signing and requires401. API state Debug exposes only service availability. Lead fixes shared backend readiness to HTTP status200 and clears stale exception diagnostics on false probe results. Backend logs show all three instances successfully serving; exact HTTP probe sequence was not retained.

Exclusive writers: trust owns credential_process.rs and api/credentials.rs, now handed back; lead owns runtime support import, shared Python support, docs and canonical records. State supplied source-only backend diagnosis and edited no code. All27 canonical tasks are retained; completed tasks are never reopened. Round2 corrective artifact review cap is reached; no third review is claimed. Those review receipts identify their actual source. New R8 source/fixture corrections are bound separately and must be exercised by the affected real gates. All six initial groups failed or blocked, so the next full six-group invocation repeats only affected groups, plus the production image build needed to verify the warning repair. No broad validation or unrelated gate is added. Genuine isolated supervisor qualification remains missing.


## Second real acceptance cohort — 2026-10-16

Native exec83623 finished exit1. All six local-02 groups failed; retained native
SDK credential supplement passed. The old Phase Exit WAIT observation expired.
R9 (4.9) repairs the complete observed cohort and genuine isolated supervisor
qualification setup before L7 (5.6) can renew the whole-phase inventory. This is
existing-scope repair, not new acceptance requirements. No tests, builds, probes,
review or other checks run during repair. Preserve earlier tasks/inventories and
the two-round review limit. No know-me production mutation.

Runtime owns binding/streams/native-MCP scenario implementation; trust owns memory
credential verifier initialization; state owns recovery and genuine isolated
supervisor setup; lead owns shared runner/support and Phase Exit integration.
The terminal receipt is evidence/acceptance/20261016-local-02/receipt.json.

## Approved native setup and R9 authoring boundary — 2026-10-16

The operator approved dedicated guest creation and guest-only reboots. The QEMU
ARM64 guest and independent candidate controller are installed; the runtime is
not started, no guest reboot has occurred, and no qualification is claimed.
All R9 production, fixture and documentation authoring is complete. L7 records
the full 20-item inventory before the first build or physical proof.
The first verification stage builds the production candidate and establishes
actual native-boot exclusion against real queued and active team work. Only
after genuine qualification may a fresh isolated team cohort be staged and the
affected six-group program run. A separate physical boot is required for each
consumable reclaim case. Neither a candidate flag nor copied journal is proof.
The read-only Lima journal adapter reads the actual guest controller journal.
No cancelled detached-child/multi-role/remote-node qualification matrix is added.
No third UAR artifact review; published arm64 and amd64 full programs remain gates.

## R10 actual native bootstrap cohort — 2026-10-16

Production rc03 build passed, image04855cc67b8f9db487f7d63dc64e3d77fe65269530590e582510d1dec254a5df;
exported and installed executable bytes matched. No full local03 program or
physical qualification ran. UAR startup failed before readiness: first private
group mismatch (uid65532, actual uar gid1001, private files gid65532); after
correcting all private paths, unsupported llm.timeout_secs4000 failed settings
seed (supported1..600), then supervisor controller unavailable. A diagnostic
restart exposed the retained empty catalog ownership conflict. Direct native
TLS1.3 handshake with the same private CA/client succeeds; no registration is
in the controller journal. Dependency source shows reqwest default native TLS
unified through prometheus_parking_lot, incompatible with PEM rustls identity
unless the supervisor builder pins rustls. Exact runtime cause text was discarded,
so the backend diagnosis is source-supported inference, not a retained raw cause.

R10 implements that one-client backend selection, documents actual group and
600-second timeout/601-second held response, and selects a fresh disposable DB
without editing or deleting failed catalog rows. Original3M aggregate budgets
and3000-second reservations need no change. L8 renews the full21-item source
inventory only after all source/config/docs corrections. Boundary CLOSED; no
new compiler/build/test/probe during authoring. Preserve original failures and
existing review cap. Candidate rc04/native qualification precedes retesting.

R10 authoring complete. L8 inventory06 binds all21 implementation rows and the
full observed startup cohort. No six-group program ran against rc03. Next
approved affected gate is production image rc04, followed by physical native
qualification; local acceptance and publication remain blocked until that proof.

## Operator course correction — 2026-10-16, plan revision61

Stop Phase Exit development and finish UAR directly using this phase's existing approved requirements and evidence. Phase Exit adoption, calibration and tool repair are no longer UAR execution prerequisites. Their historical records remain preserved. No product scope or acceptance scenario is added. Keep the existing complete-cohort implementation boundary, mandatory native supervisor qualification before retesting, six image groups, four exact real-source supplements, both published architecture programs, and release/issue receipts.

Dot architecture analysis is expected. Assess supplied adjustments against existing requirement IDs and observed failures before altering the plan. Approved changes reopen the same full implementation boundary; complete all affected code/fixtures/docs before the affected gate. Optional future improvements are recorded for later work.

## Issue-first course correction — 2026-10-17, canonical plan revision62

The operator directs this same agent team to address the repository issue queue,
with #385 first. The issue inventory and measured cadence are in
`planning/issue-queue-and-cadence.md`. This is an amendment to the active Execute
phase, not completion of V2 or a new release claim. Preserve the terminal failed
`evidence/acceptance/20261016-local-04/receipt.json`, all earlier failed receipts,
the retained rc04 supervisor qualification, immutable v1.1.0, and the current
uncommitted source. The checkpoint saying local04 is running is superseded by its
terminal exit1 receipt. No overlapping gate may start.

### Immediate critical path

1. **#385 blocker disposition, lead with runtime and state.** Reconcile every row
   of `issue-385-coverage.md` against the terminal local04 binding/recovery
   observations and current source. Give the operator the existing safe remote
   Surreal attestation, 16,000-token reservation, model verification, KB reupload,
   and exact team-reclaim guidance without presenting the customer-reported
   v1.1.0 recovery as v1.1.1 proof. Keep actual know-me mutation separate. Name
   the exact missing acceptance evidence and any newly observed source defect.
2. **Local04 defect cohort, team-owned.** Triage all six failed or blocked groups,
   four source supplements, owned cleanup, and the candidate image-ID mismatch.
   D1 source inspection isolated the first #385 binding failure: host event
   history synthesizes a `/presentation` patch after reduced refusal. R11
   repairs that trusted-host path while keeping the existing fixture assertion.
   Recovery and combined never started UAR because Docker timed out; no state
   defect is inferred. Streams selected zero effective MCP servers despite a
   mounted config, so trace fixture grants and policy intersection. MCP relay
   readiness failed before any request; inspect relay creation and Docker
   lifecycle before changing MCP product behavior.
   Assign runtime binding/streams/MCP/combined files to `uar-runtime`, credential
   fixture/policy files to `uar-trust-tools`, recovery/persistence/supervisor files
   to `uar-state`, and runner/image identity/shared roots to `uar-lead`. The lead
   gives one writer each exact file before source editing. Do not infer a product
   fix from a fixture or infrastructure failure. Preserve passing evidence only
   where source, fixture, and artifact identity remain applicable.
3. **Dot scope reconciliation, lead with independent source owners.** Map open
   #387–#396 and #386 to the current #380/#385 contract, advertised v1.1.1
   profiles, observed source boundaries and future phases. The source-inspected
   ownerless compiler authoring path in #387 is a real shared-tenant boundary.
   Source inspection confirms both compiler REST aliases are mounted and their
   CRUD/session storage does not carry an owner. R11 must reject the entire REST
   subtree for remote ingress or shared credential mode, document the resulting
   supported-profile limit, and author a real two-principal denial case for both
   aliases. Local single-owner behavior remains available. Full owner-scoped
   authoring stays open in a successor phase; this restriction does not close
   #387. Treat #391 SDK packaging and
   #393 Postgres memory acknowledgement similarly if those profiles are claimed.
   Do not silently add ten unrelated implementation changes to this release.
4. **Reclose the full-phase implementation boundary.** Any selected #385,
   local04 or Dot code/fixture/config/migration/doc adjustment joins one complete
   repair cohort. Revise the OpenSpec delta and this plan before its authoring;
   register only confirmed tasks through canonical KBD commands. Record a fresh
   whole-phase inventory with exact changed paths and source/fixture identity.
   No test, build, compiler, lint, formatter, review gate or customer operation
   runs until every revised implementation item is complete. The independent
   production-source review cap remains exhausted; no third review is claimed.
5. **Resume existing gates.** V2 reruns only affected approved local gates after
   the renewed inventory, with #385 binding/recovery evidence first in the
   recorded result. P1 source/tag/image publication waits for all required local
   gates. P2 runs both published child-image programs serially. P3 records
   criterion-specific #380/#385 receipts, #379/#383 reconciliation, #382/#386
   external dependencies and truthful #387–#396 dispositions. Do not close #385
   on isolated product repair alone when actual customer recovery is unproved.

### Team and cadence contract

`uar-lead` is the sole canonical-plan, shared-root, build-target and release
writer. `uar-runtime`, `uar-trust-tools`, and `uar-state` receive disjoint file
packets after triage; `uar-verifier` may review frozen evidence without generation
history only at an approved review boundary. Team ledger assignments coordinate
work but never replace canonical KBD task events. Every handoff records changed
files, unresolved decisions, source state and what remains unverified. The
source ownership map in `ownership.md` is extended before any new code edit.

Use the installed `delivery-cadence` skill as delivery metadata, not as an
independent scheduler or an exception to this phase's verification order. Work
in 120-minute checkpoints with a source/handoff update; target one issue-sized
usable candidate per soft 7-day window after the current #385/1.1.1 blocker is
resolved. The first #385 work starts immediately, not at the next weekly tick.
The candidate, published arm64 and published amd64 acceptance programs are
serial and their observed failed durations do not establish a passing ETA.
The measured floor and profile activation prerequisites are recorded in
`planning/issue-queue-and-cadence.md`. A clock tick neither certifies unfinished
code nor authorizes an early test. Use the relevant KBD, OpenSpec, Rust, trust,
team and reflection skills per bounded issue; do not invoke every installed
skill on every time tick. A release opportunity is a review of frozen evidence,
never automatic publication.

After P3 and Reflect, carry the separately owned open issues into successor KBD
phases in the order in the issue queue. Each successor phase has its own Spec,
Plan, complete implementation boundary and real acceptance. Do not use this
release's receipts to certify later issues.

## Source-triage closure — 2026-10-17, canonical plan revision 63

D1 and D2 are complete planning dispositions, not passing acceptance. Their
handoffs are `coordination/handoffs/D1.md` and `D2.md`. R11 is the immediate
implementation task: repair the #385 host event-history refusal bypass first,
then trace zero effective MCP policy and relay/Docker/image causes before any
further product change. Add the selected #387 compiler REST restriction and
acceptance adapter in the same complete cohort. State recovery never started in
local04, so do not infer a state-source defect from that receipt. L9 inventories
all selected production, fixture, configuration and documentation work before
any affected V2 gate. The existing V2 position is historical in-progress
acceptance; its rerun remains blocked by L9.

## V2 local06 evidence-linked repair addendum — 2026-10-17

The terminal local06 receipt passed no group. It reached the real installed
image and exposed an MCP first-boot settings defect: the untouched empty
`mcp.servers` bootstrap row hid the configured file catalog. The bounded repair
adopts file entries only from that untouched default while preserving an
API-written empty map. Binding, credentials and recovery also exposed fixture
shape/configuration/path mismatches; the six-group next candidate must exercise
the corrected assertions. This work stays inside the selected R11/L9 repair
cohort and the existing V2 gate, with no new acceptance group or early check.
The exact files and failed receipt are recorded in
`coordination/handoffs/V2-local06-repair.md`. Renew the whole-phase inventory
after this addendum and the matching OpenSpec scenario before freezing the
next acceptance manifest. The #387 compiler case remains unverified until the
credentials group reaches it.

## V2 local07 evidence-linked repair addendum — 2026-10-17

The terminal local07 receipt (`evidence/acceptance/20261017-local-07/receipt.json`,
SHA-256 `799b46fe6bfc6155f01150c05ed1818adcf7b3690b0955fdb4103b57e2acea45`)
passed MCP outcomes and combined. Preserve those results. Binding, streams and
credentials failed, and recovery blocked before its qualified-supervisor proof.
The observed failures are fixture prerequisites and scenario setup, with exact
source dispositions in `coordination/handoffs/V2-local07-repair.md`. Repair
only those four groups, retain local07 as their parent receipt, and disclose
the source/fixture change in each affected manifest. Complete all four fixes,
the phase handoff and append-only learning record, then renew the whole-phase
inventory before any affected test or build. The current image, installed
binary and guest proof are not certified for changed source hashes until an
exact candidate identity is re-established. #387 compiler acceptance remains
unverified because the credentials group stopped before that adapter.

## V2 local08 repair and operator gate addendum — 2026-10-17

Register two canonical tasks after the existing 8.4/L9 task: **8.5/R12**
(sequence36, implementation) owns the four observed local08 source repairs
and their exact source handoffs; **8.6/L10** (sequence37, boundary) records a
new whole-phase implementation inventory only after R12 and all other phase
implementation items are complete. R12 depends on R11 and the local08
terminal receipts; L10 depends on R12. V2/6.2 stays in progress but no
affected gate may resume before L10 and the operator's explicit instruction
for the named command. Preserve completed R11/L9 and inventories as history.
The local08 repair handoff is
`coordination/handoffs/V2-local08-repair.md`.

Inventory11 preceded four serial targeted local08 commands. Binding failed
the catalog timeout/max-retries inheritance assertion; source inspection
identified outer pre-semantic stream-open resilience attempts exceeding the
selected model's zero-retry budget. Runtime owns a bounded production fix in
`src/llm/orchestrator.rs`. Recovery reached its fixture and failed because an
anonymous startup knowledge base made the global legacy-table count two;
state owns owner-scoped legacy snapshots in the recovery fixture. Credentials
passed its first six scenarios but blocked during backend outage cleanup when
the restarted Docker container received a different published host port;
trust/lead fixture work refreshes that port and checks HTTP 200 readiness.
Streams completed the native partition, SSE, EOF, invalid and tool-loop cases,
then its graph scenario emitted an approval-required event for `spawn_agent`
and waited without an approval decision until the observer deadline. Runtime
owns a fixture-only graph approval correction; the trusted-host approval
boundary remains intact. Exact failed receipts are in
`evidence/acceptance/20261017-local-08-*/receipt.json`.

These findings reopen R11 implementation across the **whole phase**. The
operator verification lock in `AGENTS.md` now requires all phase implementation
tasks and canonical KBD state to reconcile, a renewed full inventory with
source/fixture identity and exact proposed commands, and explicit operator
instruction before any gate. Inventory11 or a repaired local08 group alone
does not authorize another test or build. `progress.json` shows 137/151 as
run-wide change completion; the active phase has R11/L9 complete and four
verification/publication tasks open. The waypoint's `exactNextCommand` and
ledger summary are stale, but the canonical task state is consistent. A
120-minute checkpoint audits the boundary only. Two premature
`git diff --check` commands during this repair cohort were violations and are
not acceptance evidence.

R12/8.5 and L10/8.6 are now canonically complete as source and inventory
work. L10's 23-row inventory is
`implementation-inventory-20261017-12.json` with source tree
`sha256:6d3b80446df3db3d21ae403a7f70a0f39174dc5f4d398d1dec50ec2d476f06f8`
and fixture tree
`sha256:3297d4a60bb39968b468ffac7be0b1495c3dd6def42420056fc581072724ae87`.
Its boundary is deliberately closed. The exact proposed local09 command lines
and prerequisites are in `coordination/handoffs/L10.md`. V2/6.2 and P1–P3
remain open; no check, build or acceptance on the current bytes has run. An
explicit operator instruction is still required for each named gate.

Local09 binding exited1 before build because the OrbStack daemon socket was
absent. `orbctl start` restored Docker 29.8.2, but the failed receipt remains
failed. Inventory14 renews the whole-phase record with unchanged source and
fixture identities and a closed boundary. The three unrun local09 manifests
reference that closed inventory; L10 names a fresh binding retry command for
a new explicit operator instruction. No other local09 gate has begun.

## R13 code completion after local09 pre-build failure

Register R13/8.7 at sequence38 as the bounded implementation task for the
observed local09 runner failure. The only product-fixture edit is in
`scripts/release/uar_1_1_1_support.py`: a failed owned Docker network create
now raises the specified blocked prerequisite and cleanup skips a network that
was never created. The command log remains retained. The corresponding
OpenSpec scenario is in `kubo-runtime-conformance`. A source-only `uar-core`
audit found no other missing implementation in the selected #385 refusal,
#387 profile restriction or local08 repair surfaces. It did not prove runtime
behavior. Full owner-scoped compiler authoring remains the explicitly open
#387 successor scope, consistent with the issue's patch-limit disposition.

Register L11/8.8 at sequence39 as the renewed whole-phase inventory boundary
after R13. Inventory14 and the local09 manifests precede R13's source edit;
they cannot certify or execute the changed bytes. L11 records exact current
source/fixture identity only after every phase implementation handoff. The
operator lock remains **CLOSED**. No source identity check, Docker preflight,
test, build, compiler/import probe or gate is selected during code authoring.


## Function-first child execution amendment — 2026-10-17

The accepted child plan is `children/function-first-release-certification/plan.md`.
Its exact FC1–FC9 ledger and ownership control this certification cohort; original
parent task completions remain historical. Canonical KBD registration/transitions
belong to the root lead through the actual CLI. No generated task/progress/waypoint
file is edited by this amendment.

FC1–FC5 finish all revised implementation, fixtures/configuration and required
docs, including a closed prebuilt-image runner contract, then a fresh whole-parent/
child inventory. FC6 separately requests exact image build/inspection and genuine
qualified supervisor staging/physical-boot commands. FC7 requires explicitly
authorized independent artifact review. FC8 requires the exact local functional
command and explicit operator timing disposition. FC9 supplies the parent handoff.
A completed inventory is not permission; all verification remains CLOSED pending
the exact instruction. Later implementation edits invalidate prior preparation
and require renewed inventory and instructions.

V2 retains all six reduced local groups and four native supplements. P2 selects
exactly streams/recovery on each immutable architecture child after separately
authorized pull/inspection and fresh same-image supervisor preparation for that
child. The functional invocation must not rebuild/pull or replace prepared UAR.
Every recovery attempt needs fresh real predecessor work, journal registration,
physical boot, distinct replacement fence and unconsumed qualified proof. Image
identity does not establish controller qualification. FC5 published manifests
remain templates until P1 and those preparation receipts exist.

The uncomfortable tradeoff is lost direct per-child budget refusal, compiler
restriction, credential-migration and MCP-failure evidence; these remain local
release blockers. Explicit attachment/output equality is certified, not a model's
skill-selection quality. Timing is measured diagnostically and requires operator
disposition. There is no measured speedup or passing reduced-program receipt yet.
P1 publication and P3 issue/release messaging retain their separate authorization.

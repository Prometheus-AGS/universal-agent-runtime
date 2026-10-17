# Exclusive source ownership and handoff map

Phase: uar-outstanding-issues-v1-1-1. Stage: Plan. Date: 2026-10-16.
All assignments below are for future Execute. Existing paths were source-inspected
or located by the named domain author; NEW paths are explicit planned deliverables.
The lead is the single authority for assignment changes. No file has two writers.
The source files remain unchanged at this planning boundary.

## Lead — L1/L2/L3/L4 and release coordination

| Existing or NEW path | Tasks / purpose |
| --- | --- |
| `src/config.rs` | L2 security policy configuration and NEW maintenance CLI definition. |
| `src/main.rs` | L2 dispatch explicit maintenance command without ordinary server startup. |
| `src/server.rs` | L2 independent credential policy, optional encrypted service, API state and full/sidecar shared construction roots. |
| `src/lib.rs` | L2 shared AppState and public host plumbing; no new kernel mutation authority. |
| `src/embedded.rs` | L2 SDK/embedded root plumbing from T2's interface handoff. Runtime supplies the bounded patch contract, not a concurrent edit. |
| `src/llm/mod.rs` | L2 new framing module export and any host-only source provenance accessor. |
| `src/uar/security/mod.rs` | L2 export changes only if the assigned modules require them. |
| `src/uar/security/claims.rs` | L2 narrow verified-issuer-kind accessor if required by trust; preserve general storage keys. |
| `src/uar/api/routes.rs`, `src/uar/api/openai/routes.rs`, `src/uar/api/acp/handler.rs` | L2 only if root/capture signatures require transport caller adaptation; same runtime enforcement seam. |
| `src/uar/api/providers.rs` | L2 separate administrator provider verification/seeding authority and exact target. |
| `src/uar/settings/manager.rs` | L2 settings constructor/admin integration without granting run fallback. |
| `tests/common/test_server.rs` | L2 shared fixture constructor adaptation if affected; all domain agents send requirements. |
| `Cargo.toml`, `Cargo.lock` | L2 runtime package and matching lock entry 1.1.1 only; no dependency/pin upgrade. |
| `example.config.yaml` | L2 explicit policy example/comments while preserving shipped auth/default contracts. |
| NEW `docs/releases/v1.1.1.md` | L2 complete upgrade/setup/compatibility source; P3 fills actual publication receipts. |
| NEW `scripts/release/uar_1_1_1_acceptance.py` | L3 closed-group dispatch, identities, local versus published mode, serialized gates and evidence. |
| NEW `scripts/release/uar_1_1_1_support.py` | L3 shared context/result contract, private infrastructure, process/volume lifecycle and redaction. |
| NEW `scripts/release/uar_1_1_1_manifest.schema.json` | L3 manifest contract; schema authored before boundary, validation deferred. |
| NEW `scripts/release/uar_1_1_1_manifest.example.json` | L3 non-executable template with absent identities; no fake digest. |
| `scripts/release/uar_1_1_acceptance.py` | L3 bounded version/identity/runner integration only if required to retain all existing Kubo scenarios. Historical v1.1.0 receipts stay immutable. |
| NEW `docs/operations/uar-1.1.1-acceptance.md` | L3 runner usage, prerequisites, full group inventory, cleanup and failed-group repair semantics. |
| Phase `plan.md`, `execution-ledger.json`, `ownership.md`, inventory template and future execution/handoff/evidence records | L1/L4 coordination and source freeze; V/P actual receipts later. |
| OpenSpec change artifacts and `.prometheus` append-only records | Lead spec/plan reconciliation, observed decisions and reflection. |

`src/bin/uar-sidecar.rs` calls the server construction path and has no selected
edit. If a concrete caller change becomes necessary, the lead first adds that
exact file to L2's inventory. Existing config.test/embedded/remote profiles retain
the omitted single-owner default; no blanket profile rewrite is selected.
No workflow, Dockerfile, build.rs, dependency, vendor/submodule or UI edit is
planned. If later evidence requires one, add it to the same full-phase inventory
and resolve any specific workflow policy requirements before editing.

## Runtime — R1/R2/R3/R4/R5 and T2

Exact source/fixture assignments and each source caller are in
[planning/runtime.md](planning/runtime.md). The consolidated path set is:

- Existing `src/uar/runtime/manager.rs`, `src/uar/runtime/cost_budget.rs`,
  `src/uar/runtime/thread/policy_intersection.rs`, `src/uar/runtime/turn/bindings.rs`,
  and `src/uar/runtime/thread/kernel.rs` (only if provenance compatibility needs it).
- Existing `src/llm/anthropic_driver.rs`, `src/llm/anthropic_types.rs` (only if the
  known/unknown classification helper needs it); NEW `src/llm/anthropic_sse.rs`.
- Existing `tests/anthropic_driver_tests.rs`, `tests/stream_terminal_contract.rs`,
  `tests/bauar_full_harness_cursor.rs`, `tests/truthful_tool_outcomes.rs`,
  `tests/tool_call_protocol.rs`, `tests/support/stream_terminal_peer.rs`,
  `tests/support/truthful_tool_outcome_peer.rs`,
  `tests/support/truthful_tool_outcome_process.rs`,
  `tests/fixtures/stream_terminal_https.py`,
  `tests/fixtures/truthful_tool_outcome_mcp.py`.
- Existing `docs/operations/durable-accounting.md`,
  `docs/operations/provider-stream-completion.md`,
  `docs/operations/truthful-tool-outcomes.md`, `docs/skill-pack-installation.md`.
- NEW `scripts/release/uar_1_1_1_runtime.py` for binding, streams, mcp-outcomes and
  combined observations. L3 owns lifecycle support, not these scenario bodies.

 The runtime writer owns manager,
turn bindings, budget and native Anthropic implementation; the lead owns the
shared root paths above. Shared-file ownership is whole-file, even when tasks
within the runtime lane are sequential. R4 preserves already-correct MCP outcome
code unless an observed defect requires a recorded scope update.

## Trust — T1/T3/T4

| Exact path | Tasks |
| --- | --- |
| NEW `src/uar/security/credentials/policy.rs` | T1 policy and typed source decision. |
| NEW `src/uar/security/credentials/owner.rs` | T1 host-bound credential owner. |
| `src/uar/security/credentials/mod.rs`, `src/uar/security/credentials/resolver.rs` | T1 then T3; single trust writer. |
| `src/uar/security/credentials/principal_store.rs`, `src/uar/security/credentials/store.rs` | T1 signature transition then T3 all memory/Surreal/Postgres readers/writers/transactions. |
| NEW `src/uar/security/credentials/migration.rs` | T3 mapping, preview, cursor and immutable receipt helpers. |
| NEW `src/uar/security/credentials/maintenance.rs` | T3 explicit backend-only metadata initialization; L2 owns CLI dispatch. |
| `src/uar/api/credentials.rs` | T3 cross-domain API ownership explicitly assigned to trust. |
| `tests/credentials_api_integration_test.rs`, `tests/support/credential_process.rs` | T4 whole real-path fixture cases and helpers. |
| NEW `scripts/release/uar_1_1_1_credentials.py` | T4 installed-image credentials group adapter. |
| `docs/operations/principal-provider-credentials.md` | T4 operator policy, migration, maintenance and rollback docs. |

Trust does not edit claims, manager, shared roots or schema assets. See
[planning/trust.md](planning/trust.md) for exact interfaces. S3 schema delivery
precedes T3. T3 source handoff precedes T2/L2 integration; T3 does not wait on L2.

## State — S1/S2/S3/S4

| Exact path | Tasks |
| --- | --- |
| `src/uar/persistence/mod.rs` | S1 captured eligibility type/trait. |
| `src/uar/persistence/providers/surreal.rs`, `src/uar/persistence/providers/postgres.rs`, `src/uar/persistence/providers/memory.rs` | S1 unchanged eligibility decisions with typed provenance. |
| `src/uar/persistence/providers/surreal_host.rs` | S1 eligibility then S3 additive schema registration, same state writer. |
| NEW `migrations/20261016000000_credential_revisions.sql` | S3 nullable metadata columns; retain old migrations/checksums. |
| NEW `migrations/surrealdb/credential_revisions.surql` | S3 additive schema only, no automatic backfill. |
| `src/uar/api/knowledge.rs`, `src/uar/rag/ingestion_worker.rs` | S2 upload/dispatcher diagnostics, unchanged durable acceptance boundary. |
| `docs/operations/durable-ingestion.md`, `docs/operations/knowledge-principal-isolation.md` | S2 recovery/eligibility guidance. Runtime sends wording instead of editing them. |
| `tests/durable_ingestion.rs`, `tests/principal_knowledge_namespace.rs` | S4 real persistent upload/isolation/restart fixtures. |
| NEW `tests/support/uar_1_1_1_state_fixture.rs` | S4 only if shared setup is needed; otherwise keep helpers local and record not-needed. |
| NEW `scripts/release/uar_1_1_1_recovery.py` | S4 installed-image recovery group, existing quiesce/exclusion proof controls. |

See [planning/state.md](planning/state.md). State never edits trust store files.
Missing revisions remain nullable until explicit maintenance; all later supported
credential writes maintain revisions through T3. `versions.toml` 3.3.0 resolves
the stale 3.2.4 role text; it is not an unresolved design choice.

## Shared adapter and build ownership

L3 owns shared Python support and schemas. Domain writers consume the proposed
context/result interface without editing it. Runtime owns the combined adapter;
state/trust send small setup/observation helpers. Domain adapters must implement
all image-visible requirements; source-only fixtures are supplemental.

The lead alone owns Cargo target/build invocation, image candidate build, registry
pull and acceptance orchestration after L4. V1 reviewer writes only its assigned
findings under the phase evidence/review directory, receives no generation history,
and starts no build. Review is deferred and not claimed as approved in Plan.
Each future handoff records changed files, signatures, source status, remaining
work and explicitly unverified behavior. No passing evidence is implied by source
ownership or a completed authoring task.


## L2 compatibility addendum — 2026-10-16

Lead owns only the necessary new SecurityConfig-field initializers in these
existing callers: `src/uar/security/middleware.rs`, `src/uar/api/a2a/handler.rs`, `src/uar/api/a2a/grpc.rs`, `tests/jwks_es256_verification.rs`, `tests/test_a2a_grpc.rs`, `tests/a2a_thread_service.rs`, `tests/settings_persistence.rs`, `tests/context_history_integrity.rs`.
Runtime makes the same initializer adjustment in its already-owned stream fixtures.
No unrelated test cases or behavior are added; none is executed early.


## Fixture and transport handoff addendum — 2026-10-16 Execute

Runtime owns NEW tests/fixtures/uar_1_1_1_provider.py and, if actual reusable
source setup needs it, NEW tests/support/uar_1_1_1_runtime_fixture.rs. State owns
NEW tests/fixtures/uar_1_1_1_supervisor.py if needed for the existing qualified
controller protocol. These required real-peer fixtures remain within R5/S4 and
the complete implementation inventory. Lead support owns lifecycle only; domain
authors implement protocol scenarios/admission helpers rather than leaving them
as external prerequisites.

Lead L2 also closes the existing server.rs direct Anthropic /v1/messages and
optional response-correction provider captures through host credential policy.
Runtime/trust adapters include denied/allowed observations for those source
seams. No new capability or unrelated subsystem is selected.

## Integration-author transfer — 2026-10-16

The original runtime writer is interrupted. Fresh `execute_runtime_integration`
owns `scripts/release/uar_1_1_1_runtime.py`,
`tests/fixtures/uar_1_1_1_provider.py`, `tests/bauar_full_harness_cursor.rs` and
optional NEW `tests/support/uar_1_1_1_runtime_fixture.rs` until its R5-integration
handoff. Fresh `execute_recovery_integration` owns
`scripts/release/uar_1_1_1_recovery.py` and optional NEW
`tests/support/uar_1_1_1_state_fixture.rs` until its S4-integration handoff.
They preserve other writers' changes and run no verification before L4.

The proposed `tests/fixtures/uar_1_1_1_supervisor.py` was removed during source
authoring: stopping an unrelated sentinel cannot establish runtime containment.
It is not a deliverable and receives no qualification credit. The recovery
adapter must use actual retained qualification and real controller observations.
No controller/evidence configuration was found by read-only source inspection of
the operator-named charcoal-agent repository. Production is not a scratch fixture.

Lead takes exclusive `src/uar/runtime/manager.rs` ownership after interruption of
the original runtime writer for the concrete R1 typed budget-policy mapping
correction identified during fixture authoring. Fresh runtime integration writer
also owns `tests/fixtures/truthful_tool_outcome_mcp.py` for its deterministic
timeout duration; lead does not edit that file concurrently.

Fresh `execute_memory_integration` owns `tests/credentials_api_integration_test.rs`
and optional NEW `tests/support/uar_1_1_1_memory_fixture.rs` for the plan's required
actual HTTP/auth/in-memory credential migration supplement after original trust
author completion. Its single case is
`phase_memory_credential_migration_uses_real_http_and_atomic_store`; handoff is
`coordination/handoffs/T4-memory-integration.md`. It runs no checks before L4.

After original trust author completion, lead owns the narrow maintenance-command
environment wiring in `scripts/release/uar_1_1_1_credentials.py`. Both ordinary
leases and metadata maintenance explicitly override the image's persistence
environment defaults with the owned fixture backend. The memory fixture author
does not edit the Python adapter.

## Issue #385 confirmed-cause follow-up — 2026-10-16

Lead owns `src/server.rs` startup budget/store warning and
`src/uar/runtime/cost_budget.rs` safe per-call reservation refusal diagnostics.
Lead owns additive operator guidance in `docs/operations/durable-accounting.md`,
`docs/releases/v1.1.0-kubo.md`, `docs/releases/v1.1.1.md` and NEW
`docs/operations/remote-surreal-upgrade.md`, plus phase issue-385 coverage records.
Reactivated runtime integration author owns its existing adapter/provider/support
fixture files for the confirmed16000-token refusal, startup warning observation
and truthful unsupported-store case. No source or fixture checks before renewed L4.

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

## R9 authoring handback — 2026-10-16

Runtime, trust and state have handed back their exclusive source files. The lead
owns the frozen candidate and all builds/gates serially. Guest-only reboot
approval is recorded in evidence/supervisor-guest-setup-20261016.json. No teammate
may launch a gate. Physical qualification precedes UAR retesting; no source
mutation occurs while the frozen candidate is being exercised.

## R10 native bootstrap cohort ownership

Supervisor state teammate exclusively owns src/uar/runtime/supervisor.rs and the
two supervisor operations documents. Lead owns native staging packet, private
guest configuration, release notes, canonical KBD and whole inventory. All other
source remains frozen. No tests/builds/probes during authoring; lead owns the
next single production build and actual native qualification.

## Issue-first revision after terminal local04 — 2026-10-17

The active repair boundary is closed pending D1/D2 source reconciliation, complete
R11 implementation and L9 inventory. Existing rows remain historical ownership
for the current source. The lead owns D1/D2 canonical records and grants no new
source file to a second writer by implication. The terminal local04 receipt, not
the earlier running checkpoint, is the current gate state.

| Role | Reserved R11 area | Release condition |
| --- | --- | --- |
| `uar-lead` | Phase/OpenSpec artifacts, shared construction roots, release runner/support, manifest and image/build context, version and final publication records. | Sole build-target and publication writer; no source edit until exact observed failure and file assignment are recorded. |
| `uar-runtime` | Existing runtime manager/binding/provider/stream/MCP source and `scripts/release/uar_1_1_1_runtime.py` plus its exclusive fixtures. | Source-inspect binding, streams, MCP and combined first; send the exact path list before any edit. |
| `uar-trust-tools` | Credential authority/API/store source and `scripts/release/uar_1_1_1_credentials.py` plus its exclusive fixtures. | Source-inspect credential and memory-supplement block first; no concurrent lead edit to trust files. |
| `uar-state` | Persistence/recovery/ingestion/supervisor source and `scripts/release/uar_1_1_1_recovery.py` plus its exclusive fixtures. | Source-inspect recovery and resource lifecycle first; retain the reviewed guest proof and all failed store evidence. |
| `uar-verifier` | No production file. | Frozen artifact-only review only if a future explicit phase revision authorizes a new review; current review cap is exhausted. |

D1 assigns `uar-runtime` `src/uar/runtime/manager.rs` for the observed #385
binding event-history bypass. `src/uar/runtime/turn/host/event_projection.rs`
is reserved to the same writer only if the minimal interface needs it;
`scripts/release/uar_1_1_1_runtime.py` is the same writer's streams fixture
policy investigation. Lead retains `scripts/release/uar_1_1_1_support.py`,
the acceptance runner and Docker lifecycle/image provenance. No state-source
edit is assigned from local04 because recovery did not start UAR. The trust
restriction ownership below is disjoint. New exact files require lead handoff.

#387 source inspection confirmed both compiler REST aliases expose ownerless
CRUD/session storage in remote or shared profiles. For the selected R11
restriction, the lead exclusively owns `src/server.rs`,
`docs/releases/v1.1.1.md`, the shared acceptance runner and canonical records.
After the lead's explicit implementation handoff, `uar-trust-tools`
exclusively owns `src/uar/api/compiler.rs` and a new
`scripts/release/uar_1_1_1_compiler.py` adapter. The new adapter is integrated
by the lead into an existing real acceptance group; no second gate writer is
created. The entire REST subtree on both aliases must be denied for remote
ingress or shared credential mode before compiler/storage effects. Full
owner-scoped storage/session repair remains a successor task and #387 stays
open. #388–#396 and #386 are queued, not silently assigned to R11. All
teammates share one checkout, preserve other writers' edits, and run no early
test, compiler, build or external gate. Handoffs name changed files,
unresolved decisions, source state and unverified behavior.

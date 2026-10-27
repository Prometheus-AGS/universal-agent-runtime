ASSESSMENT: agent-team-concurrency-repair
Project: Universal Agent Runtime
Date: 2026-10-26
Codebase baseline: Current local UAR 1.3.0 source inspected; retained failure came from an existing native artifact built from source sha256:28137dc46cbb05aa9169f93b46232ea6aafc8d3f04e7c841274e4bbf4d109018, not an independently rebuilt copy of this checkout.
Cross-tool progress: Child has zero registered implementation changes/tasks; two bounded read-only source researchers contributed to this assessment. Parent counters and previous acceptance remain historical.

The uncomfortable finding: calling this a demonstrated concurrent scheduler defect would overstate the evidence. Two admitted team runs failed skill admission, and one terminal settlement was not confirmed. The previously suggested cause has not been established.

## Scope and evidence

Canonical phase ID: uar-verification-simplification::agent-team-concurrency-repair, under uar-verification-simplification. This assessment covers the failed math/writer project, not all team features or the broader 97-case plan.

Retained evidence:
- .prometheus/verification/macos-team.md
- .prometheus/verification/teams/72860111-231c-49ec-a973-e0a4e53f2a63.json
- .prometheus/verification/teams/72860111-231c-49ec-a973-e0a4e53f2a63.diagnostic.json
- .prometheus/verification/2026-10-26T08-49-32.290Z-team-macos.json
- target/prompt-macos-hbitxj2c/server-persistent.log: recorded 08:49:35.387172Z settlement error.

The actual last scenario result was 1 failed scenario, 4 passed/3 skipped/1 failed reported steps. Some reported steps are hooks. No report is relabeled as a pass.

## IMPLEMENTATION STATUS

| Area | Status | Evidence and gap |
|---|---|---|
| Immutable package/binding and durable team creation | DONE for retained setup | Corrected package installed, executable binding admitted and two-member team created. This is not full collaboration certification. |
| Existing specialist identities/instructions in team | DONE for retained setup | Two immutable AgentDefinitions capture math-specialist-db64c2a8 and writing-specialist IDs and saved instructions. Full ordinary-agent KB/resource parity in teams was not tested. |
| Explicit project task graph | DONE for retained setup | Independent calculation/outline plus dependent article created and distinct member runs admitted. The script creates the decomposition; autonomous natural-language planning was not exercised. |
| Skill-selection admission | PARTIAL / observed failure | Source implements persisted reconciliation and fail-closed admission, but both actual member runs returned skill_selection_unavailable. Underlying typed cause is absent from retained public evidence. |
| Concurrent team dispatch | PARTIAL | Runtime has asynchronous workers and bounded admission. Distinct runs/roots exist, but observed run status was error rather than a proven interval of simultaneous useful execution. No successful overlap or scheduler fault has been established. |
| Terminal result/accounting settlement | PARTIAL / observed failure | Writer outline failed and settled. Calculation remained running/reserved-unknown in the diagnostic snapshot after a catalog CAS result error. Its later state is unknown; no new service read was made. |
| Artifact-dependent final article | PARTIAL / not exercised | Code publishes outputs and selects artifacts; BDD checks both artifact IDs and correct integrated result. Neither successful specialist output nor dependent article ran to acceptance. |
| Recovery of the retained ambiguous settlement | PARTIAL | Recovery code preserves uncertainty/reservations rather than fabricating completion. A confirmed reconciliation of this specific attempt is missing. |
| Failure diagnostics | PARTIAL | Public error is safely redacted, but cause category and database result error detail needed to select a repair were not retained in the available failure evidence. |

No scoped subsystem was found to be an empty stub. Implemented source plus a failed real workflow is not equivalent to feature completion.

## CAUSAL TRACE AND CONFIDENCE

### A. Shared skill-admission failure — confirmed symptom; precise cause UNKNOWN

Both native runs emitted skill_selection_unavailable before protected content publication. Current source follows:
1. Team execute_turn resolves the immutable member and selected task context: src/uar/runtime/team_execution/execution.rs:263.
2. Non-inherited run capture invokes RunSkillBindings::capture: src/uar/runtime/manager.rs:5149.
3. RunSkillBindings calls SkillService::matching_snapshot: src/uar/runtime/turn/bindings.rs:177.
4. matching_snapshot takes the registry write lock and reconciles selection: src/uar/runtime/skills/service/selection.rs:189.
5. reconcile_selection reads all durable skills, returns StoreUnavailable on a failed read, or StorageMissing if any registered skill ID lacks a persisted record: src/uar/runtime/skills/registry.rs:130.
6. The manager matches Err(_) and emits the same public code for either cause: src/uar/runtime/manager.rs:5159.

The read can also fail during data decoding; Surreal list_skills fetches/decodes records at src/uar/persistence/providers/surreal.rs:2488. The retained evidence does not distinguish transport/query/decoding failure from a missing record. It does not identify missing IDs or prove the store was healthy. A skill-disabled selection is not itself proof of either cause.

Assessment: a generic skill reconciliation failure blocks the team path even though this project's definitions select no skills. Whether the reconciliation scope is unnecessarily broad, the stored registry is inconsistent, or the backend read failed requires further evidence. Removing reconciliation or inventing empty success would not establish a valid fix.

### B. Unconfirmed catalog settlement — confirmed symptom; commit outcome UNKNOWN

Math attempt 285108f8-4cfe-4ff6-8b3b-3a8d9fba052a, run 414193e0-50e2-4ec0-a7e2-ac1bd436b6cc, remained running/reserved-unknown in the saved diagnostic. Writer run 92c146d4-f719-4892-9e58-6e0a1edc09ed separately ended failed/settled.

The recorded error was “collaboration persistence unavailable: failed to read collaboration catalog CAS result”. Current source runs an UPDATE with generation comparison and RETURN AFTER, then extracts the result with response.take(0): src/uar/compiler/collaboration/storage.rs:135–148. That context identifies result extraction, not the database's underlying reason and not a confirmed CAS miss or commit.

Catalog settlement checks the attempt's execution fence and persists terminal state/usage through CAS: src/uar/compiler/collaboration/team_execution/settlement.rs:120. The worker logs and returns when settlement fails: src/uar/runtime/team_execution/execution.rs:243–249. Existing generation-conflict loops do not turn an error return into a known terminal receipt. The current inspected path does not itself demonstrate reconciliation of this saved non-yielding settlement failure.

Recovery marks interrupted running attempts uncertain and retains reservations: src/uar/compiler/collaboration/team_execution/recovery.rs:155. That is an honesty safeguard, not proof of successful work or permission to replay. Do not infer that the reservation is still current, released, spent, or safe to reset from an old snapshot.

### C. Relationship and architecture

Both symptoms involve persistence, but a common cause, a SurrealDB defect, a scheduler race, and a required architecture redesign remain hypotheses. No upstream-behavior claim is made. Source has an existing durable task graph, fenced member attempts, per-member execution and output artifacts; the observed failure does not justify replacing those mechanisms.

## CROSS-TOOL PROGRESS

- Canonical child progress before assessment: zero registered implementation changes/tasks; no implementation handoff claims a repair.
- Parent state retains earlier implementation counters; they are not this child's completion count.
- Two independent source researchers traced admission and settlement without mutations or runtime calls. Their convergence supports the source trace, not a reproduced runtime diagnosis.
- Prior context recall returned old parent-phase and UI lessons. Applicable lesson: distinguish authored state/artifact claims from observed behavior. Unrelated UI coverage percentages and earlier phase plans are not imported into this repair scope.
- Generated progress/waypoint files remain owned by KBD. Assessment completion is recorded through supported stage commands, not by hand-editing their legacy assessment_complete fields.

## SPEC GAP SUMMARY

- openspec/specs/collaboration-package-catalog/spec.md governs immutable package installation, compatibility projection and private bindings. The retained setup covers narrow portions; this canonical catalog spec does not by itself certify member execution.
- openspec/specs/skill-governance/spec.md requires durable scoped settings and restart preservation. Read-through reconciliation exists, but the observed outage is unresolved.
- openspec/specs/skill-activation-runtime/spec.md requires effective eligible-set intersection and typed refusal. A safe public refusal exists here, but adequate operator-visible cause evidence is missing.
- openspec/changes/afc-c09-team-admission-runtime/specs/durable-team-execution/spec.md is an ACTIVE CHANGE DELTA, not an archived canonical spec. It requires exact admitted execution, truthful output/usage, fencing and safe recovery. The corresponding tasks.md still has unchecked original and architecture-repair items despite substantial implementation existing in source. This is ledger drift; do not silently mark those broader tasks complete.
- The child has no implementation plan or new spec delta yet. Parent change scope includes Boss UI/IPC/platform work beyond this child's failed backend workflow; that work must not be imported as incidental repair scope.
- Required behavior not established: actual overlapping useful specialist work and artifact-grounded final article. Full roster changes, team cancellation/restart, backend parity and automatic planning receive no new credit.

## BUILD HEALTH

- Current-source build: UNKNOWN. No Cargo/build/check/lint/format/import probe ran.
- Retained native-artifact BDD: FAIL at skill admission/concurrency assertion; final article skipped.
- Test coverage: PARTIAL. Existing scripts/behaviors/features/math-writing-team.feature and its Node.js steps implement the requested real-path journey. tests/bauar_skill_selection.rs and tests/integration/live/collaboration_conformance_cases.rs exist but were not executed or used as passing evidence.
- Numeric code coverage: UNKNOWN; source presence is not a coverage percentage.
- Test limitations: sequential admission and registration before overlap observations may miss very short genuine overlap; this is a possible false negative, not the observed cause of error statuses. The current script's model profile/settings revision1 is explicit but not a dynamically captured live revision; that assumption must be considered when binding repaired-source execution.
- The failed attempt's unknown settlement prevents safely treating another run as an automatic retry.

## CONSTRAINT CHECK

Assessment activity was source/spec/retained-log reading, bounded independent research and KBD/document metadata. No product source edit, tests, builds, API probes, service restart, external publication, dependency change or credential access occurred in this assessment. Gate remains CLOSED.

The retained UAR is left untouched: last recorded PID64463, owner exec66217, base http://127.0.0.1:53587. This statement records the handoff, not a new liveness check.

Preserve all historical evidence and cumulative ceilings4800 host calls/3000 active minutes/28 repairs, plus separate USD10 Cohere/USD10 OpenAI embedding caps. Remaining allowance has not been independently reconciled here; do not assume a fresh budget. Local constraint file was not present. No newly introduced guard or speculative product change exists.

## GOAL PROGRESS

1. Investigate both skill failures without assuming scheduler fault: PARTIAL — symptom and finite source branches identified; actual branch not established.
2. Fix admission and settlement while preserving state: NOT MET — no implementation attempted; existing unresolved outcome retained.
3. Prove concurrent work and merged result with the same BDD: NOT MET — both execution/merge criteria remain unverified.
4. Preserve limits, finish implementation before exact authorized verification: MET for this assessment's conduct; future implementation and verification remain pending.

## ASSESSMENT DISPOSITION

Assessment complete with explicit evidence gaps; child repair is NOT complete. Appropriate next lifecycle step is Analyze to resolve causes and bounds, then Plan. This document is not an implementation plan or verification approval. No signature or new broad verification machinery was introduced.

ASSESSMENT COMPLETE

## Sycophancy review

Detect-only document assessment returned score0.017857 (below0.3); no correction required. Its low-severity length warning is retained in sycophancy/assess-2026-10-26T09-07-02Z.json. This is a document review, not a product verification result.


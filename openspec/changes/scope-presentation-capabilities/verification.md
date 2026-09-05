# Verification: scope-presentation-capabilities

Date: 2026-09-05. Local phase-end evidence; no release certification.

| Dimension | Result |
| --- | --- |
| Completeness | 4/4 tasks; 1/1 requirement implemented |
| Correctness | All three spec scenarios mapped below |
| Coherence | Trusted-host policy and typed graph-owned assignments preserved |

## Requirement and scenario evidence

`src/uar/domain/policy.rs` adds Presentation ResourceSelection and resolves the
enabled owner universe by the existing non-widening intersection. Its explicit
scope and legacy-deserialization regressions passed in the server-full suite.
`src/uar/persistence/presentations.rs` admits validated, enabled records only
for the verified subject/tenant. `runtime/presentations_tests.rs` exercises
foreign-tenant rejection, absent authority, frozen child narrowing, disablement
and deletion after admission.

- Denied conversation selection: `presentation_scopes_intersect_and_lower_all_never_restores_exclusions` covers lower selected/all intent and inherited exclusions. The separate `lower_scope_cannot_reenable_parent_none` test covers the shared resolver's None behavior using MCP resources, not a Presentation-specific None fixture.
- Delegated execution: frozen snapshot narrowing and output-ceiling tests prove a child cannot restore a denied resource or surface mode. A2A contract tests preserve legacy wire/digest presence and carry negotiated restrictions.
- Inherited policy editing: `frontend/src/platform/entities/presentation-assignments/presentation-assignments.test.ts` covers retained inactive IDs, explicit reset, catalog-generation invalidation, saved-intent baselines and conflict recovery using an agent target. Six assignment-panel tests distinguish exact mode labels, including both inherited states. The exact conversation scenario was exercised separately in the clean browser: Assignment mode Inherit and Prompt Caching Inherit; changed only Prompt Caching to On; Save Configuration; reopened; observed effective On from session override and Assignment mode still Inherit with zero exclusions. The session domain sends null Presentation intent for unrelated edits and the host preserves saved intent atomically. Agent-target tests alone are not presented as conversation-path evidence.

## Commands and observed results

- `cargo test --locked --no-default-features --features server-full`: exit0; library744passed/1ignored, BDD9scenarios49steps passed, broad integration94passed/1ignored, doctests26passed/17ignored.
- `pnpm -C frontend test --project=unit --maxWorkers=1`: 462passed across82files,106.13s.
- `pnpm typecheck && pnpm lint`: exit0.
- `openspec validate scope-presentation-capabilities`: valid.

## Findings

No critical issue remains for the specified local scope. Warnings: the old-wire
fixture is not a live external-peer test; the deferred real-provider429 test
and existing coverage limits remain. This does not certify every persistence
profile or production deployment. Preserve these limits at archive. The fixture
credential-isolation incident and operator rotation action remain recorded in
the phase execution log; no credential is copied here.

Ready for spec sync/archive with the recorded warnings and operator approval.

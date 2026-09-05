# Verification: establish-presentation-catalog-workspace

Date: 2026-09-05. Local phase-end acceptance with retained warnings.

| Dimension | Result |
| --- | --- |
| Completeness | 4/4 tasks; 2/2 requirements implemented |
| Correctness | All six delta scenarios mapped below |
| Coherence | Trusted-host writes, owner partitions, typed graph/domain UI |

## Requirement and scenario evidence

`src/uar/a2ui/presentations.rs` owns declarative validation, content/revision
records and pure instantiation. The persistence trait has real memory,
PostgreSQL and Surreal implementations. `src/uar/api/presentations.rs` derives
the owner from verified identity and rejects stale mutation revisions.

- Owner edit/reload: `tests/presentation_persistence.rs` runs the shared contract against each provider, including update/read-back and concurrent-writer exclusion. SurrealKV additionally uses four separate seed/update/delete/empty processes. PostgreSQL closes/reopens its provider connection.
- Stale/foreign edit: persistence contracts and four real-router API tests reject foreign tenants, invalid principals, forged owner fields, stale PUT/DELETE and invalid drafts. Stale updates use different content before asserting no mutation.
- Unsafe/broken templates and expanding references: domain validation/tests cover the approved profile/catalog, components, rooted unique-parent graph, cycles, duplicates, bindings and inert literal data.
- Authenticated empty catalog: API/graph loading retains the verified owner partition with an empty record list. Both browser creation entry points open a new draft; three page regressions preserve undefined/new versus existing-row identity.
- Preview without side effects: the production editor uses the safe local renderer and does not dispatch actions or save preview data. Browser preview rendered the default data before saving; source and domain tests preserve the declarative boundary.

## Commands and actual results

- `cargo test --locked --no-default-features --features server-full`: exit0; library744passed/1ignored; BDD9scenarios49steps passed; broad integration94passed/1ignored; doctests26passed/17ignored. Surreal process-restart contract passed.
- `cargo test --locked --no-default-features --features server-full --lib uar::api::presentations::tests`:4passed,0failed,0.01s after the verified-context fixture correction.
- Dedicated disposable DB plus `cargo test --locked --no-default-features --features server-full,postgres-backend --test presentation_persistence postgres_catalog_contract_and_reconnection -- --ignored --exact`:1passed,0failed,0.31s; compilation18m45s.
- `cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test presentation_persistence memory_catalog_contract -- --exact`:1passed,0failed,0.00s; compilation6m01s.
- Full frontend unit suite:462passed/82files; final typecheck/lint exit0; final build14.20s exit0. Final inspector correction cohort21passed.
- Full Rust formatting and scoped diff checks passed.

Browser tests used temporary databases. Creation/reload/edit, availability
revision2, confirmed deletion and focus recovery passed; desktop1440x1000 and
narrow390x844 captures are in `/tmp/uar-presentation-evidence.gBZ6BG/`.
Independent source/Impeccable reviews cleared the observed create-callback,
closed-label and bounded inspector defects. The dedicated PostgreSQL container
and owned runtime/stub/Vite processes were stopped after tests.

## Findings

No critical issue remains for the local catalog contract. Retain warnings:
full200% zoom/numerical contrast are unverified; the16 unrelated entity-boundary
findings remain; four pinned PGlite eval warnings remain; no release, real-peer
or real-provider429 certification was performed. The initial fixture imported
an existing credential through cwd dotenv loading; the operator must rotate
that exposed key. Original env/operator data was unchanged, and the subsequent
clean fixture used explicit test credentials. Do not publish the contaminated
temporary database. No credential value appears in these artifacts.

Ready for spec sync/archive with the recorded warnings and operator approval.

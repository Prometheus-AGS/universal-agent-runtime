# UAR 1.1.1 Release Qualification Report, Round 5 — Profile B (function-first, local)

**Date**: 2026-10-18
**Contract**: unchanged acceptance contract; round-5 bounded allowance (100 min,
50 tool calls, 2 repair cycles total, 15 min / 8 calls reserved for closeout).
**Prior reports preserved byte-identical**: rounds 1–4.

## Defect verification and correction (item 1) — COMPLETE

**Typed inspection.** `surrealdb-types-3.3.0`'s `Value` derives Serialize with
`None`/`Null` as unit variants (plain JSON strings `"None"`/`"Null"`) while
legitimate strings arrive tagged `{"String": s}`. `unwrap_surreal_value`
mapped only `"Null"` → null, so the missing-value sentinel became
`Some("None")` at the `MigrationRowMetadata.row_revision: Option<String>`
extraction boundary (`migration.rs:40-43` → 127 → ladder 162) and the
preview invented historical revisions. Its `{"String": …}` arm also
wrongly nulled legitimate `"Null"` strings via recursion. The distinction was
NOT lost before conversion, so the fix lives at the typed boundary.

**Patch** (`src/uar/persistence/providers/surreal.rs`): sentinel arm maps
plain `"None"`/`"Null"` (unit variants only) to null; new
`unwrap_surreal_string` emits `String`/`Datetime`/`Uuid` tagged content
literally. No global string conversion; directly affected callers inspected
(`api_keys/storage`, `credentials/store`, `credentials/principal_store`,
`a2ui/design_systems`, surreal rows mapping) — none depend on the old leak.

## Bounded proof (item 2)

- **Unit level: 10/10 green** (`tests/surreal_value_conversion.rs`): serde
  shape contract, sentinel → absent (value + extraction boundary), null
  behavior, legitimate "None"/"Null" strings remain strings (boundary keeps
  `Some("None")`), real revision unchanged, nested collections.
- **Scenario level on the corrected build**: the previously failing
  `local-metadata-upgrade` **migration-status assertion PASSED** — the run
  progressed past it into the concurrent apply phase. The repair is verified
  fixed, not merely unexercised.
- Round-4 fixture corrections preserved (credential auth identity,
  content-versioned config, no retries); no assertions weakened.

## Candidate identity (item 3) — COMPLETE

| Item | Value |
|---|---|
| source_tree | `sha256:8f00142593e0fcd3bdeb38081ce7bf0001fba57610c68029dbe339e7ce26836e` |
| fixture_tree | `sha256:c47355f1152cc417c2544fc56bf86a38aa17c69dcdaee6f0887f816da82dfd0b` |
| Image | `uar:1.1.1-function-first`, `sha256:2d9478319d1b2bcefe03b8dcc1d22987f07baac2039dcdcc0a09eb50897046eb` (rebuilt from the frozen snapshot) |
| Gate manifest | `evidence/gates/function-first-local-r5.json`, `sha256:6cf70fa840d5143f2d78c7637329da72edf38b7f1896b9dd412a0fbdb4b5b226` |
| Prep receipt | `…-r5-image-preparation.json`, `sha256:c4deb03a91a793f2738d1870c8cc1d041e01b93d4f5e291984e93b317e3e4e9b` |
| Inventory | `implementation-inventory-function-first-r5.json`, `sha256:a8096ebffce9f2cabcec0df72fabb73190d406ae833d18f729caf50ebe98b863` (34 items incl. RQ4/RQ5) |
| Build identity record | supervisor identity-06 `qualified-function-first-identity-06.json`, `sha256:75455a3803c30c0a4ff38dc881f606fe623e972691e84a0a9f0de17f54b61b7d` (supersedes identity-05; deployment facts preserved verbatim) |

Freeze self-validation: `READ_MANIFEST_OK` (the acceptance driver's full
gate) and `FC6_IDENTITY_OK` (identity receipt image/platform/source_tree/
fixture_tree match the manifest; runtime_image_id matches `docker image
inspect`). All runtime-executing checks used the corrected build.

## Readiness (item 4 preconditions) — FAILED (one of two)

- **PostgreSQL: PASS** — authenticated query through the published endpoint
  returned `current_user = postgres` (exit 0).
- **Credential family: FAIL** — after clearing the repaired
  `local-metadata-upgrade` migration-status assertion and the sequential
  flows, the **concurrent apply phase** (`migration_cases:394`) failed:

```
AssertionError: POST /api/uar/credentials/migrate-legacy status 503; expected (200,)
```

## Precise failing assertion, evidence, and classification

- **Assertion**: the concurrent `migrate-legacy` batch must return 200 per
  apply; at least one returned **503** (`scripts/release/uar_1_1_1_credentials.py:267`
  `apply(...)` → `support.py:91` require).
- **Redacted evidence**: readiness evidence
  (`uar111-r5-readiness-evidence/`, runtime captures
  `uar111-credentials-local-2f95076567-*.log`): sequential applies succeeded;
  no server-side ERROR accompanies the 503. The product's 503 sources in this
  path (`src/uar/api/credentials.rs:35,39,51,64,109,131`) are
  `Multi-tenant credentials are not enabled`,
  `credential_installation_id_required`, and
  `credential_migration_unavailable` policy/unavailable mappings. The fixture's
  failure message does not retain the response body code — an observability
  gap already named by the round-3 lesson.
- **Affected artifact identities**: source `8f001425…`, fixture
  `c47355f1…`, image `2d947831…`, manifest `6cf70fa8…`.
- **New defect vs unsuccessful repair**: this is a **NEW defect in a later
  phase**, not an unsuccessful repair. The repaired value-conversion defect is
  **verified fixed** by 10/10 unit tests and by the previously failing
  migration-status assertion passing on the corrected build. The 503 occurs
  in the concurrent migration-application path, which no prior round ever
  reached.
- **Smallest next action**: retain the 503 response body code in the fixture's
  failure message (round-3 lesson), then classify from `migration_result`'s
  exact error variant: if `credential_migration_unavailable` under parallel
  applies, investigate the service's concurrency handling in
  `migrate_legacy_conditioned`; if `credential_installation_id_required`,
  inspect the policy evaluation under concurrent requests.

## Release decision

> **UAR 1.1.1 certification — NOT READY FOR RELEASE (round 5, profile B).**
> The reported SurrealDB missing-value defect is corrected and verified (unit
> and scenario level). The candidate identity is now internally consistent
> (frozen trees ↔ rebuilt image ↔ recorded build identity, FC6 fields green
> before acceptance). The single mandated acceptance program was **not
> launched**: its readiness precondition failed on the new concurrent
> migration 503 above. Mandatory coverage still unexercised in this round:
> the whole-program acceptance (binding, streams, mcp-outcomes, credentials
  in-image migrations, recovery on both backends, combined) — unchanged
> criteria, preserved receipts for rounds 1–4.

## Concrete next steps toward release (for the follow-up allowance)

1. Fix or classify the concurrent `migrate-legacy` 503 (smallest next action
   above); rerun the credential family readiness.
2. Re-freeze only if hashed inputs change; otherwise reuse manifest
   `6cf70fa8…` and image `2d947831…`.
3. Run the single whole-program acceptance; on full PASS issue
   QUALIFIED FOR RELEASE with the receipt and proceed to:
   **commit** the working tree (product: `surreal.rs` conversion fix +
   `tests/surreal_value_conversion.rs`; fixture: rounds 4–5 corrections) on a
   branch off `38f0b5d8`; **PR** referencing the five qualification reports;
   **release** staging per the contract's publication steps — none of which
   are authorized in this allowance.

## Cumulative budget accounting (round-5 + preserved overruns)

| Round | Allowance | Wall | Calls | Repair cycles | Notes |
|---|---|---|---|---|---|
| 3 | 75 / 50 / 2 | ~78 min | ~54 | 2 | Overruns disclosed in round-3 report |
| 4 | 90 / 50 / 2 | ~72 min | ~26 | 2 | Within allowance |
| 5 | 100 / 50 / 2 | ~62 min | ~30 | 1 (the initial correction; no correction to it needed) | Within allowance; 15 min / 8 calls reserved and used for closeout |

## Preservation

- Rounds 1–4 reports and receipts byte-identical; no commit, push, tag, or
  publication. All changes remain uncommitted working-tree edits on HEAD
  `38f0b5d8`.

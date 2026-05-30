# Phase Reflection — uar-wisc-cli

- Generated: 2026-05-29
- Author: claude-code (`/kbd-reflect`)
- Backend: openspec
- Duration: single session (2026-05-29)
- Scope: salvage assessment of `origin/feature/providers` → 1 OpenSpec change implemented, merged (PR #5), archived (PR #6)

## Phase shape recap

This phase began as a **salvage assessment** — the user challenged a too-quick
dismissal of the stale `origin/feature/providers` branch and asked for a rigorous
read of whether its ideas had value or were already achieved another way. The
assessment (with sycophancy-correction applied in both directions) found:

- **WISC CLI** — largely achieved another way (memory MCP server). Low salvage value.
- **Multi-tenant provider credentials** — a genuine, unbuilt capability. `main` had
  provider *routing* (liter-llm) and a *catalog* (build.rs) but **no per-user
  encrypted BYO credentials**; `session/encrypted.rs` was a stub. **This was the
  real reason the branch existed**, and the initial dismissal was wrong on it.

User decision: support **BOTH** single- and multi-tenant. One design satisfies both —
the scoped resolution chain terminates in the env step, so single-tenant is the
zero-config default and multi-tenant overlays on top. The phase then ran the full
OpenSpec lifecycle (proposal → 2 specs → design → tasks → apply → verify → archive).

## Goal achievement

| # | Goal | Status |
|---|------|--------|
| 1 | Forensic salvage assessment of the branch (value, supersession, correctness) | **MET** — assessment.md; corrected the record on the provider system |
| 2 | Resolve the single- vs multi-tenant decision | **MET** — "both", one design (env-terminated scoped chain) |
| 3 | Encrypted-at-rest per-user provider credentials | **MET** — AES-256-GCM, `CredentialEncryption`, 13 unit tests |
| 4 | Scoped credential resolution (session→agent→user→system→env) | **MET** — `CredentialResolver`, `SecretString`, no-leak |
| 5 | Durable store + request-path integration | **MET** — `SurrealCredentialStore` wired to live DB; `start_run` seam; orchestrator unchanged |
| 6 | JWT-gated credential CRUD API | **MET** — `/api/uar/credentials`; write-only plaintext; masked reads; 6 HTTP integration tests |
| 7 | Single-tenant preserved (zero-config) | **MET** — `Option<ProviderService>` = `None` ⇒ `cfg.api_key` untouched |

Overall: **7/7 MET.**

## Delivered changes

| Change | Status | PRs |
|--------|--------|-----|
| `provider-credentials-multitenant` | implemented · merged · archived | #5 (feature), #6 (archive) |

Commit `9ea395b` (feature, 29 files +2244/−131) → main via PR #5; archive move via PR #6.
Archived at `openspec/changes/archive/2026-05-29-provider-credentials-multitenant/`.

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with artifact-refiner QA | 0/1 (refiner not run this phase) |
| Rust build (default/surreal) | green |
| `--features postgres-backend` | pre-existing break (unrelated; see debt) |
| Clippy (new credential modules) | clean |
| Unit tests added | 13 (encryption / store / resolver) |
| HTTP integration tests added | 6 (credentials CRUD) |
| Net test delta | +19, all passing |

No artifact-refiner logs (`.refiner/`) exist for this phase — QA was code-level
(cargo build + clippy + 19 tests) rather than refiner-gated.

## Bugs / pre-existing breakage fixed in-flight

| Issue | Root cause | Fix |
|-------|-----------|-----|
| Fresh build failed compiling `surrealdb` | `Cargo.lock` pinned `surrealdb-core 3.1.2` against `surrealdb =3.0.5`; `-core` internal APIs (`notifications`, `index_compaction`) changed across the minor | `cargo update -p surrealdb-core --precise 3.0.5`. Latent because incremental builds reused cached artifacts; adding `aes-gcm` forced a recompile that surfaced it. |
| Test binary wouldn't compile (`missing fields kind, origin`) | 3 stale `Skill` test fixtures from the earlier `skill-kind-and-origin` migration | Added `kind: Default::default(), origin: Default::default()` to `service.rs`, `rules.rs`, `tfidf.rs` fixtures |
| `surrealdb::Response` not a public type | named the query-response type directly | Take `Vec<surrealdb::types::Value>` inline + pure converter |
| `Vec<CredentialRecord>` not `SurrealValue` | surrealdb 3.x `.take::<T>()` needs `SurrealValue`, not serde | Reused codebase `Value → serde_json` pattern (`surreal_to_json`/`to_db_value` made `pub(crate)`) |

## Technical debt / carry-overs

| Item | Severity | Note | Status |
|------|----------|------|--------|
| `postgres-backend` feature build broken (`pgvector::Vector: sqlx::Encode/Type`) | medium | Was a **stale lockfile** state; the `pgvector =0.4.1` pin (already in Cargo.toml) is the fix, and `cargo update -p surrealdb-core` regenerated the lock. | ✅ **RESOLVED (F1)** — `cargo build --features postgres-backend` green on main |
| Run-level `start_run` credential assertion (6.5) + dual-mode smokes (9.1/9.2) | low | Extracted the seam into a pure `apply_credential_layer` fn; 4 unit tests cover single-tenant-keeps-env, multi-tenant-override, no-credential-fallback, provider-isolation. | ✅ **RESOLVED (F2)** |
| Finding 1 salvage (`scout`, WISC `decide`/`prime`/`handoff` recipes) | low | Go/No-Go written (`scout-mcp-go-no-go.md`): **NO-GO** this cycle; `prime` is the first to build if revisited. | ✅ **DECIDED (F3)** |
| Credentials admin UI (BYO key management) | medium | Scoped + design-routed (`credentials-admin-ui-scope.md`); recommended as its own phase `uar-credentials-admin-ui` (CLAUDE.md UI/UX routing must run first). | 📋 **SCOPED (F4)** — awaiting dedicated phase |

### Follow-up addendum (2026-05-30)
All four reflection carry-overs were actioned in a post-reflection pass (F1–F4).
The only open item is the admin UI, intentionally deferred to its own
design-routed phase. Net new tests this pass: **+4** (`credential_layer_tests`).

## Lessons learned

1. **Don't dismiss on filenames + commit subjects.** The initial branch dismissal was
   wrong on the substance because it never read the code. Reading `encryption.rs` /
   `credential_resolver.rs` revealed a real, unbuilt capability. Sycophancy-correction
   must cut *both* ways — neither cling to the dismissal nor capitulate to the pushback.

2. **"Both" is often one design, not a fork.** Single- vs multi-tenant collapsed into a
   single resolution chain whose terminal step *is* the single-tenant path. Look for the
   unifying default before assuming a branch.

3. **Find the real integration seam.** The route looked like the place to resolve
   credentials, but it delegates to `RunManager::start_run`, which owns driver
   construction. Resolution belongs in the config-assembly layer there — keeping the
   `Orchestrator` signature untouched (resolve-then-construct, not deep threading).

4. **Latent lockfile breakage hides behind incremental builds.** A dependency added for
   an unrelated reason (`aes-gcm`) forced a recompile that exposed a months-old
   `surrealdb-core` mismatch. A periodic clean build would catch these earlier.

5. **Narrow handler state for testability.** `State<AppState>` made the credential router
   untestable; narrowing to `State<Option<Arc<ProviderService>>>` both decoupled it and
   unlocked 6 isolated HTTP tests. Prefer the minimal state a handler actually needs.

6. **surrealdb 3.x deserialization** goes through `Vec<surrealdb::types::Value>` +
   serde_json, never `take::<Vec<T>>()`; reuse the codebase converters rather than the
   `SurrealValue` trait.

## Recommended next phase seeds

- `uar-postgres-backend-repair` — fix the pre-existing `pgvector`/`sqlx` mismatch so the
  optional feature builds (already spawned as a task).
- `uar-credentials-runtime-tests` — run-level `start_run` assertion + dual-mode smokes (closes 6.5/9.1/9.2).
- `uar-credentials-admin-ui` — frontend for users to manage their BYO provider keys against the new API.
- `seim-wisc-scout-mcp` — optional Finding 1 salvage (`scout` + composite recipes) as MCP tools, if non-Claude-Code agents come into scope.

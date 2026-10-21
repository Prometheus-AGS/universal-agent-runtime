# UAR 1.1.1 Release Qualification Report, Round 2 — Profile B (function-first, local)

**Date**: 2026-10-18
**Profile**: B — complete multi-backend server qualification: frozen `server-full` +
`postgres-backend` candidate, exercised against real SurrealDB 3.3.0 **and**
PostgreSQL 17/pgvector (Supabase image), local linux/arm64 server/sidecar only.
**Contract**: `UAR-1.1.1-release-qualification-proposal.md` (operator-approved profile B)
plus the operator-authorized round-2 closeout (three named corrections, one reseal,
one final acceptance program, 120-minute / 80-tool-call / 2-repair-cycle ceilings).
**Round-1 report** (FAIL, accepted): `release-qualification-report-1.1.1-function-first.md`.

## Round-2 scope executed

Exactly the three corrections the round-1 report named, nothing else:

1. **Restart durability (product)** — durable run history on the existing
   `PersistenceLayer` abstraction: `src/uar/persistence/run_history.rs`
   (`RunHistoryRecord`/`RunHistoryEvent`), trait methods on `PersistenceLayer`,
   SurrealDB + PostgreSQL provider implementations
   (`persistence/providers/run_history/{surreal,postgres}.rs`), migrations
   `migrations/20261018000000_run_history.sql` and
   `migrations/surrealdb/run_history.surql` (registered in `surreal_host.rs`).
   Background `RunHistoryWriter` write-through per published event; run record
   persisted at both admission insert sites (`manager.rs`) and on terminal
   transition; durable read fallbacks in `get_run_for_context`/`history_since`
   on `active_runs` miss with `presentation_owner_key()` ownership recomputed
   from live verified context. `run_events.rs` handler unchanged — response
   semantics preserved. `backend="local"` maps to the surreal provider on
   embedded SurrealKV, covering all three fixture backends.
2. **Recovery fixture** — embedding-fingerprint assertion gated behind a new
   `indexed` parameter on `_committed`; only the two post-indexing call sites
   (`recovery.py:270`, `:278`) assert; the pre-restart call (`:242`) does not.
   Assertion text and the planned restart sequence unchanged.
3. **SurrealKV sidecar** — root cause demonstrated by live A/B against the
   running pinned-image sidecar: `surrealdb/surrealdb:v3.3.0` `/health` returns
   HTTP 200 with `content-length: 0`; `Http.request` maps an empty body to
   `None`, so the `wait_for` probe stayed falsy for the full 180 s window.
   Fixture predicate defect, latent (pre-repair runs never crossed this wait).
   Repair: `credentials.py:179` asserts `status == 200` via
   `return_status=True`. Pinned image kept. 15-minute diagnosis cap honored.

## Qualified artifact identity (round 2)

| Item | Value |
|---|---|
| Image | `uar:1.1.1-function-first` |
| Image ID | `sha256:1aaca0c22f3783fd5dbc99949e4b1288cdac84fa563f1a7ec238b3ccaf3d44d0` |
| Platform | `linux/arm64` (native execution, OrbStack arm64 host) |
| Source commit | `38f0b5d85960e3cb8445607750538ca500efdf17` + disclosed uncommitted repair sets RQ1 + RQ2 |
| source_tree | `sha256:68ea288fb25025ba59a58a37cb463b0ec8f2b0f1e7edd4a3e371c026baa80cb3` |
| fixture_tree | `sha256:82e75bb4200f852cfe939b670d343b6a667eedfb590008235f049f9ca5f5a404` |
| Gate manifest | `evidence/gates/function-first-local-r2.json`, digest `sha256:f9634fdcabdfc9ff85f540153374bc45d2af303932635a3da8422aaa6d060498` |
| Image-preparation receipt | `evidence/acceptance/function-first-local-r2-image-preparation.json`, digest `sha256:4230ecfc293f1da49306d13a12b1f957b3615e45aa72366c6ad53ad1f30f8dfa` |
| Implementation inventory | `implementation-inventory-function-first.json`, digest `sha256:682e8be0b79461d0e28245961cb5559970293b8c2fd421fd8d378a903ef83ece` (31 items, incl. `repair:qualification-2`) |
| Build evidence | Docker release profile finished in 17m31s, zero of 2 repair cycles consumed; host `cargo test` supplement compiled and passed (binding, exit 0) under `server-full,postgres-backend,in-memory-backend` |

Round-1 identity preserved byte-identical: image `ec58c872…`, trees
`7389c323…`/`c2b0f7c2…`, manifest `07834801…`, receipts `de04c3e6…` (final) and
`c8f2de80…` (pre-repair), FC6 identity-05 `9390f8ec…`.

## Final acceptance result (round 2)

One final acceptance program ran against the rebuilt, resealed candidate:
`python3 scripts/release/uar_1_1_1_acceptance.py --manifest evidence/gates/function-first-local-r2.json`,
started 2026-10-18T15:31:36Z, finished 15:55:12Z, exit code 1 — **overall FAILED**.

Receipt: `evidence/acceptance/function-first-local-r2/receipt.json`, digest
`sha256:05d8f62228e503f6c50681936f6739b98e8fde017a4835f2c428c2993d04e5af`.

| Group | Disposition | Native supplements |
|---|---|---|
| binding | **PASS** | 1/1 PASS (`phase_runtime_binding_refusals_use_real_host_store_and_providers`) |
| streams | **PASS** | — |
| mcp-outcomes | **PASS** | 1/1 PASS (`phase_runtime_native_incidental_error_retains_success`) |
| credentials | **BLOCKED** | 2/2 PASS (SDK-supplied credentials; memory credential migration) |
| recovery | **FAIL** | — |
| combined | **PASS** | — |

### Round-1 root causes ↔ round-2 outcomes

1. **streams restart durability (round-1 product gap) — RESOLVED.** The
   `streams-prompt-skill-output-timing` scenario completed its live legs,
   executed the planned `docker restart` (`runtime.py:324`), and this time
   `GET /api/uar/runs/{id}/events` returned **200** with durable event text
   equal to the controlled answer and `GET /api/uar/runs/{id}` returned 200
   with the preserved terminal status (`runtime.py:325–329`); the scenario then
   completed a second post-restart run. Correction 1 is behaviorally proven on
   the real path. Group disposition: PASS.
2. **credentials SurrealKV sidecar block — predicate repair verified, new
   blocker exposed downstream.** The repaired probe passed: the 33
   metadata-migration scenarios (previously blocked at the sidecar wait) now
   advanced past sidecar readiness. The 6 non-migration scenarios passed
   (`identity-isolation`, `messages-facade-policy`, `response-correction-policy`,
   `shared-source-grants`, `absent-service-policy`, `single-owner-compatibility`),
   and both native supplements passed — including
   `phase_memory_credential_migration_uses_real_http_and_atomic_store`, so the
   migration path is exercised and passing host-side against real collaborators.
   However, all 33 migration scenarios then blocked identically:
   `runtime exited before health: uar111-credential-policy-c5cd216e36`. The
   runtime container (created command-00179, restarted per scenario at
   commands 00181/00183/00185/00200/00202/00204, each exit 0) served the first
   six scenarios and then exited before `/health` on every migration scenario
   (local, surreal, and postgres families alike). No container log was captured
   — the fixture's finally-clause removes the container (the same observability
   gap round 1 named for the sidecar). Whether this is a product crash under
   the migration-scenario configuration (round-1 RQ1 admission changes or
   round-2 C1 persistence wiring — both reachable at boot) or a fixture fault
   is **undiagnosed within the round-2 ceilings**; the evidence trail ends at
   the exit-before-health block.
3. **recovery fingerprint fixture fault — RESOLVED; Surreal leg + B4 executed;
   PostgreSQL leg then failed on first-ever execution.** The Surreal leg ran to
   completion (commands through 00252), which required the relocated fingerprint
   assertion to pass at its post-indexing sites and the B4 cold-boot port
   refresh to execute against the restarted SurrealDB. The PostgreSQL leg —
   never executed in any prior program — then failed at command-00283:
   `docker port uar111-postgres-fe67bd35f4 5432/tcp` → exit 1, "no public port
   '5432/tcp' published", even though the container was created with
   `--publish 127.0.0.1::5432` (command-00253, exit 0) and PostgreSQL was
   internally healthy (28 `pg_isready` polls, exit 2 → exit 0 at
   command-00282). No restart/rm/recreate of the postgres container appears in
   the command receipts between creation and the failed query. Fixture
   sequencing vs. Docker/OrbStack publish-registration behavior: undiagnosed
   within the ceilings; command receipts 00253–00283 preserve the full trail.

## Criterion-level dispositions

| Journey | Groups | Disposition | Basis |
|---|---|---|---|
| J1 Artifact and boot | binding | **PASS** | Resealed image `1aaca0c2…` installs, boots, serves `/health`, advertised capabilities; native binding-refusals case PASS |
| J2 One governed agent run | streams, credentials | **FAIL** | Restart durability now proven (streams PASS incl. restart leg); credential non-migration scenarios + both native supplements PASS; 33 migration scenarios BLOCKED at runtime exit-before-health (root cause 2 above) |
| J3 Protocol contract | binding, streams, mcp-outcomes, credentials | **PASS** | A2A/ACP/A2UI contract checks, MCP outcomes incl. native incidental-error case, facade policy scenarios, SSE/AG-UI text equivalence — all passed |
| J4 Tools and skills | mcp-outcomes, credentials, streams | **PASS, mandatory control NOT RUN** | Same as round 1: packaged-WASM capability-denial control remains NOT RUN (see NOT_EXERCISED) |
| J5 Knowledge and memory | recovery | **FAIL (fixture, first-ever postgres leg)** | Surreal leg PASS incl. post-indexing fingerprint assertion and B4 cold-boot port refresh; PostgreSQL leg unevaluated (root cause 3 above); memory-service isolation NOT RUN (see NOT_EXERCISED) |
| J6 Compiler and team | binding | **PASS** | Team-policy binding with stable per-case instance identity, private binding install, native refusal cases |
| J7 Durability and recovery | recovery, streams, FC6 | **FAIL** | Run-history restart leg PASS (streams); Surreal recovery leg PASS; PostgreSQL recovery leg FAIL (fixture, first execution); FC6 identity-05 stands from round 1 (qualified supervisor slice) |
| J8 Shipped client | — | **NOT RUN** | Admin UI explicitly unqualified; desktop/mobile/SDK/embedded excluded by scope decision |

Overall program disposition: **FAIL**. Correction 1 (restart durability) is
proven; corrections 2 and 3 are verified at their original failure points but
each exposed the next latent defect one layer downstream.

## Smallest specific remaining defect (contract stopping rule)

**The recovery fixture's PostgreSQL leg port discovery**: command-00283
(`docker port uar111-postgres-fe67bd35f4 5432/tcp` → exit 1, "no public port
published") against a container created with `--publish 127.0.0.1::5432`
(command-00253, exit 0) and internally healthy (command-00282 `pg_isready`
exit 0). This leg had never executed in any prior acceptance program, so the
defect is latent fixture behavior exposed — not introduced — by round 2.
Smallest next step: inspect the postgres-leg `_cold_boot`/container setup path
in `scripts/release/uar_1_1_1_recovery.py` against the captured command
sequence (receipts 00253–00283).

Second, larger open item (named for completeness, not the smallest): the
credential-policy runtime exits before `/health` on all 33 migration scenarios
with no captured log; the fixture's finally-clause container removal (the
observability gap round 1 already named) must be fixed first so the exit reason
can be captured before any product-vs-fixture determination.

## NOT_EXERCISED (documented causes, unchanged from round 1)

- **Packaged WASM capability denial** — no denial-attempt case exists in the
  acceptance vehicle; grant-restricted packaged fixture requires infrastructure
  excluded by the scope freeze. Deny-by-default remains unit-tested at
  `plugin_loader.rs` outside this qualification program.
- **Memory-service isolation (Surreal-backed memory)** — qualified config
  disables the memory service (`memory.enabled=false`); scenario requires
  fixture infrastructure excluded by the scope freeze.

## Certification limits (named)

- **Claim surface**: local linux/arm64 server/sidecar with both PostgreSQL and
  SurrealDB. Admin UI explicitly unqualified. Desktop, mobile, SDKs, amd64, and
  publication/registry staging excluded.
- **B3 scope / graph window** (FC7 WARNING-1/2) stand as round 1 recorded them.
- **B4 execution**: now exercised on the Surreal leg (cold-boot port refresh
  executed within the passing leg); the PostgreSQL leg never reached its cold
  boot.
- **Credential metadata migration**: exercised and passing only at the native
  host level (both supplements); the in-image migration scenarios remain
  unexecuted (blocked upstream at runtime boot).
- **PostgreSQL recovery leg**: never evaluated in any program.

## Failed-evidence preservation

- Round-2 final receipt + all command logs:
  `evidence/acceptance/function-first-local-r2/`, receipt
  `sha256:05d8f62228e503f6c50681936f6739b98e8fde017a4835f2c428c2993d04e5af`.
- Round-1 failed final receipt + logs: `evidence/acceptance/function-first-local/`
  (`sha256:de04c3e6…`); pre-repair evidence:
  `evidence/acceptance/function-first-local-prerepair-failed/` (`sha256:c8f2de80…`).
- Round-1 report: `release-qualification-report-1.1.1-function-first.md`.
- All round-1 and round-2 repairs remain uncommitted working-tree edits on HEAD
  `38f0b5d85960e3cb8445607750538ca500efdf17`; no commits, pushes, or publication
  were performed. Ceilings honored: ~113 min of 120; <80 tool calls; 0 of 2
  repair cycles consumed.

## Release recommendation

> **UAR 1.1.1 certification — FAIL, profile B (local linux/arm64
> server/sidecar with SurrealDB 3.3.0 + PostgreSQL 17/pgvector), round 2.**
> Candidate `38f0b5d85960e3cb8445607750538ca500efdf17` + disclosed uncommitted
> repair sets RQ1+RQ2 (source_tree `sha256:68ea288f…`, fixture_tree
> `sha256:82e75bb4…`), artifact `uar:1.1.1-function-first` `sha256:1aaca0c2…`
> (linux/arm64, `server-full` + `postgres-backend`, prebuilt), configuration per
> sealed gate manifest `sha256:f9634fdc…`, was evaluated on real SurrealDB 3.3.0
> and PostgreSQL/pgvector containers using the existing final acceptance program
> (receipt `sha256:05d8f622…`, exit 1). Journeys **1, 3, 6 passed**; journey
> **4 passed with its mandatory WASM capability-denial control not run**;
> journey **2 failed** (credential migration scenarios blocked at runtime
> exit-before-health); journeys **5 and 7 failed** on the PostgreSQL recovery
> leg's first-ever execution. Round-2 corrections verified: **restart durability
> proven** (streams PASS including the restart leg), **fingerprint assertion
> relocation proven** (Surreal recovery leg PASS, B4 executed), **sidecar probe
> repair proven** (migration scenarios advance past sidecar readiness; both
> credential native supplements PASS including memory migration). This statement
> does **not** certify: credential metadata migration in-image, PostgreSQL
> recovery, knowledge ingestion/retrieval durability, memory isolation, WASM
> sandbox denial, admin UI, desktop, mobile, SDKs, amd64, or any
> publication/registry claim. **Publication is withheld.**

**Recommendation: do not ship or certify UAR 1.1.1 under profile B.** Round 2
proved correction 1 end-to-end and verified corrections 2 and 3 at their
original failure points, but the single authorized final program surfaced the
next latent layer in each of the two remaining groups: the credential-policy
runtime's exit-before-health under migration-scenario configuration (log capture
must be added before diagnosis) and the recovery fixture's PostgreSQL leg port
discovery (receipts 00253–00283). The contract's stopping rule — one final
program, then name the smallest remaining defect and stop — is honored here.
All receipts, including every failure, are preserved.

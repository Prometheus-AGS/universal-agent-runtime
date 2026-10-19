# UAR 1.1.1 Release Qualification Report — Profile B (function-first, local)

**Date**: 2026-10-18
**Profile**: B — complete multi-backend server qualification: frozen `server-full` +
`postgres-backend` candidate, exercised against real SurrealDB 3.3.0 **and**
PostgreSQL 17/pgvector (Supabase image), local linux/arm64 server/sidecar only.
**Contract**: `UAR-1.1.1-release-qualification-proposal.md` (operator-approved profile B).

## Qualified artifact identity

| Item | Value |
|---|---|
| Image | `uar:1.1.1-function-first` |
| Image ID | `sha256:ec58c8725877d5821d2c7c60b4c334d34614e2a875d4700d241653e1128387b0` |
| Platform | `linux/arm64` (native execution, OrbStack arm64 host) |
| Source commit | `38f0b5d85960e3cb8445607750538ca500efdf17` + disclosed uncommitted repair set |
| source_tree | `sha256:7389c3236535ec406a12e1e6ab4585510ee1c822e85b6ca04335e92023bb2d1e` |
| fixture_tree | `sha256:c2b0f7c26969e5f7d45d4a3a5d2444a4c9d2f402ebe4ad624e5468d13739e04f` |
| Gate manifest | `evidence/gates/function-first-local.json`, digest `sha256:07834801194d2cc18d457077f2e91418cc0f729f4764e63b829c987aca075bab` |
| Image-preparation receipt | `evidence/acceptance/function-first-local-image-preparation.json`, digest `sha256:cb6b2bd9ee9f81956237c08b7bfcb26e929e0093d563716e2593b6282d97c2c7` |
| Implementation inventory | `implementation-inventory-function-first.json`, digest `sha256:351b404746f3878f778a775cf027fe2f39e46b064ed0d5255d5b94a64f8ec8e1` |
| FC6 identity receipt | `qualified-function-first-identity-05.json`, digest `sha256:9390f8ec43940fce7c078ca93d17d13f751acf1058b414b06c56e5be6e245220` |

## Repair round disclosure (post-FC8-failure)

The pre-repair run failed four demonstrated checks. Repairs, all disclosed in the
inventory (`repair:qualification-1`) and reviewed independently (FC7):

- **B1 (fixture)** — team-policy scenario configs now set a stable
  `service_instance.instance_id`; binding POST returns 201 (was 422
  `collaboration_invalid: runtimeInstanceId must be non-empty`). Proof P1.
- **B2 (fixture)** — skills POST uses the product's no-slash route
  `/api/uar/skills` (was trailing-slash → 404). Proof P2.
- **B3 (product)** — new `resilience.admission_timeout_ms` (default 10 s)
  bounds the persistence reads performed during run admission (harness config,
  resilience policy, presentation snapshot — shared deadline with
  existing-tolerance fallbacks) and each credential-store read
  (`CredentialResolver.store_timeout`, min clamp 500 ms), plus a whole-admission
  backstop (`2×` admission timeout) that registers a terminal
  `persistence_unavailable` failure with cancellation cleanup. Response
  contract: POST 200 + terminal availability error in the run stream — the
  product's existing admitted-failed-run convention. Proof P3: outage admission
  answered in 20.0 s (was: hang to the 30 s middleware → 408), zero provider
  contact, clean recovery after store restart.
- **B4 (fixture)** — recovery `_cold_boot` refreshes the SurrealDB published
  port after container restart (mirrors the proven credentials.py pattern).
- **FC7 corrective fix** — independent review found the new accounting readback
  used a nonexistent route (`/api/accounting/…` → catch-all 404). Fixed to
  `/api/uar/accounting/scopes/session/<id>` at three sites; byte-length key
  idiom aligned with `_seed_legacy`. Confirmation review: **FC7 PASS, zero
  unresolved CRITICAL**.

## Final acceptance result

One final acceptance program ran against the rebuilt, resealed candidate:
`python3 scripts/release/uar_1_1_1_acceptance.py --manifest evidence/gates/function-first-local.json`,
started 2026-10-18T12:15:36Z (within the 13:08:47Z deadline), finished
12:44:37Z, exit code 1 — **overall FAILED**.

Receipt: `evidence/acceptance/function-first-local/receipt.json`, digest
`sha256:de04c3e6b989f164a9c6a6acb263f1b0c856aac1c8f5d826c75e5c1a4a782924`.

| Group | Disposition | Native supplements |
|---|---|---|
| binding | **PASS** | 1/1 PASS (`phase_runtime_binding_refusals_use_real_host_store_and_providers`) |
| streams | **FAIL** | — |
| mcp-outcomes | **PASS** | 1/1 PASS (`phase_runtime_native_incidental_error_retains_success`) |
| credentials | **BLOCKED** | 2/2 PASS (SDK-supplied credentials; memory credential migration) |
| recovery | **FAIL** | — |
| combined | **PASS** | — |

Repair validation inside the final run: B1 (binding 201 with the configured
`runtimeInstanceId`) and B2 (skills no-slash 201) passed at their pre-repair
failure points. The three credential scenarios that failed pre-repair with the
B3 hang→408 (`local-installation-stability`, `store-decrypt-failures`,
`local-quarantine`) now **PASS**; the P3 proofs separately recorded the bounded
20.0 s outage admission with zero provider contact and clean post-recovery run.
**B4 was never executed**: the recovery group aborted before its first cold
boot, so the `_cold_boot` port refresh carries FC7 review evidence only.

Three distinct root causes; none cascades into another:

1. **streams FAIL — product gap: accepted runs do not survive restart.**
   `streams-prompt-skill-output-timing` completed its live legs (skill attach,
   exactly one provider request, SSE text == AG-UI text == durable event text ==
   controlled answer, peer timing) and then executed its planned
   `docker restart` (fixture `runtime.py:324`; command receipt `command-00124`).
   After restart, `GET /api/uar/runs/{id}/events` returned **404**
   (`runtime.py:325`). Server-side, run state lives only in the process-local
   `active_runs` map (`manager.rs`); no startup path rehydrates terminal runs —
   the only recovery code is approval-specific (`approval_recovery_record` et
   al.). The run record and its event history are unreachable through the API
   after any server restart. This path was masked pre-repair (B2 aborted the
   group at skill creation); no repair touched rehydration. The proposal lists
   "accepted data lost on restart" as blocking for any shipped profile.
2. **credentials BLOCKED — fixture infrastructure, product path never reached.**
   All 30 credential metadata-migration scenarios (10 cases × local/surreal/
   postgres) blocked identically: the quiesced SurrealKV metadata sidecar
   (`surrealdb/surrealdb:v3.3.0@sha256:681c6c22…`, pinned and unchanged) never
   served `/health` within the 180 s wait window (`credentials.py:178`;
   sidecar ran 12:39:27Z→12:42:28Z, then the fixture's finally-clause removed
   it, so no sidecar log was captured). Pre-repair these same scenarios passed
   this wait and failed later at the B3 408, so the block is a between-runs
   fixture/environment regression; nothing in the repair set touched
   `credentials.py`, the image build inputs, or the Surreal image pin. The
   product's migration behavior was not exercised at all — BLOCKED is not a
   product failure or success. The 9 non-migration credential scenarios
   **passed**, including all three repaired B3 paths.
3. **recovery FAIL — fixture defect introduced by this round's new coverage.**
   The new embedding-fingerprint assertion (`recovery.py:164`,
   `require(chunks, …)`) runs on every `_committed` call, but the first call
   (`recovery.py:239`) executes while the embedding peer deliberately withholds
   its response until the old runtime is dead (peer release at line 244).
   Chunks cannot exist at that point by design, so the assertion aborted the
   Surreal leg 22 s after container start, before any cold boot (command
   receipts 00265–00269). This is a fixture fault, not an observed product
   contradiction; per the contract the gate nevertheless remains failed until a
   corrected fixture is sealed and rerun. Consequently the product's chunk
   persistence/fingerprint, grounded-retrieval citation, accounting retention
   across cold boot, and the entire PostgreSQL recovery leg were **never
   evaluated** in this program.

## Criterion-level dispositions

Proposal journey ↔ acceptance group mapping and dispositions:

| Journey | Groups | Disposition | Basis |
|---|---|---|---|
| J1 Artifact and boot | binding | **PASS** | Frozen image `ec58c872` installs, boots, serves `/health`, advertised capabilities; native binding-refusals case PASS |
| J2 One governed agent run | streams, credentials | **FAIL** | Live-run legs passed (stream terminal, single provider request, accounting/facade policy scenarios PASS incl. repaired B3 paths); decisive failure: accepted run unreadable after restart (streams root cause 1); credential migration slice BLOCKED (root cause 2) |
| J3 Protocol contract | binding, streams, mcp-outcomes, credentials | **PASS** | A2A agent card, A2UI schemas+builtins, ACP agents/list + unknown-method rejection (`binding-protocol-surface-contracts`); MCP outcomes incl. native incidental-error case; Anthropic `/v1/messages` + OpenAI chat/completions facade policy; SSE/AG-UI text-equivalence assertions all passed. The streams group failure was the restart-durability leg, not a wire contract |
| J4 Tools and skills | mcp-outcomes, credentials, streams | **PASS, mandatory control NOT RUN** | Skill install/attach/execute and tool-outcome receipts passed; the contract's mandatory packaged-WASM capability-denial control was **NOT RUN** (no denial-attempt fixture exists; authoring one needs infrastructure excluded by the scope freeze — see NOT_EXERCISED) |
| J5 Knowledge and memory | recovery | **FAIL (fixture fault)** | Aborted by the new fingerprint assertion before any product evaluation (root cause 3); grounded-retrieval citation and the PostgreSQL leg NOT RUN as a consequence; memory-service isolation NOT RUN (see NOT_EXERCISED) |
| J6 Compiler and team | binding | **PASS** | Team-policy binding with stable per-case instance identity (B1), private binding install, native refusal cases |
| J7 Durability and recovery | recovery, streams, FC6 | **FAIL** | Run-history restart leg FAIL (product gap, root cause 1); recovery group FAIL (fixture fault, root cause 3 — product durability unevaluated); qualified-supervisor slice **PASS** (FC6 identity-05: independently witnessed whole-guest boot, predecessor claim `fdf26bf3:2` fenced, new incarnation `e0395be4` epoch 0) |
| J8 Shipped client | — | **NOT RUN** | Admin UI explicitly unqualified; desktop/mobile/SDK/embedded excluded by scope decision |

Overall program disposition: **FAIL** (streams product gap; recovery fixture
fault; credentials infrastructure block). Per the adopted contract, a fixture
fault is not a product defect, but the gate remains failed until a corrected
fixture is sealed and rerun; the operator's stopping rule (one final program,
no automatic second repair cycle) closes this round here.

## NOT_EXERCISED (documented causes)

- **Packaged WASM capability denial** — no denial-attempt case exists in the
  acceptance vehicle (`tests/wasm_skill_execution.rs` holds execution tests
  only); a grant-restricted packaged fixture requires new test infrastructure
  excluded by the scope freeze. Deny-by-default remains unit-tested at
  `plugin_loader.rs` (`capability_grant_denies_by_default`) but that unit test
  is not part of this qualification program.
- **Memory-service isolation (Surreal-backed memory)** — the qualified config
  disables the memory service (`memory.enabled=false`); records exist only via
  the agent memory-capture pipeline read through MCP tools, and standing up a
  remember/recall isolation scenario requires new fixture infrastructure
  excluded by the scope freeze.

## Certification limits (named)

- **Claim surface**: local linux/arm64 server/sidecar with both PostgreSQL and
  SurrealDB. Admin UI behavior is explicitly **unqualified**. Desktop, mobile,
  SDKs, amd64, and publication/registry staging are **excluded**.
- **B3 scope** (FC7 WARNING-1): the bounded-admission certification covers the
  demonstrated inline-artifact admission path. Pre-dispatch store reads on
  bound/registered-agent admission (`routes.rs` catalog resolution, binding
  revalidate) remain unbounded and are outside the certified claim.
- **B3 graph window** (FC7 WARNING-2): for graph-rooted runs, cancellation of a
  hung preparation worker relies on `PreparationLifetime` drop; a worker
  pre-contact cancellation checkpoint was not statically proven. The
  zero-provider-contact claim is certified for the tested window (P3 evidence).
- **B4 execution**: the `_cold_boot` published-port refresh was never exercised
  by the final program (recovery aborted before its first cold boot); it
  carries FC7 static-review evidence only.
- **Operational guidance** (FC7 SUGGESTION-5): `admission_timeout_ms` must stay
  below half the HTTP middleware timeout (default 30 s) for the backstop to
  win the race; config validation does not couple the two.

## Failed-evidence preservation

- Pre-repair failed FC8 evidence (receipt `sha256:c8f2de80…` + all command
  logs): `evidence/acceptance/function-first-local-prerepair-failed/`
- Prior identity receipts (-01…-04) remain under the supervisor state dir;
  identity-05 supersedes -04 with a recorded `renewal` block.

## Release recommendation

> **UAR 1.1.1 certification — FAIL, profile B (local linux/arm64
> server/sidecar with SurrealDB 3.3.0 + PostgreSQL 17/pgvector).** Candidate
> `38f0b5d85960e3cb8445607750538ca500efdf17` + disclosed uncommitted repair set
> (source_tree `sha256:7389c323…`, fixture_tree `sha256:c2b0f7c2…`), artifact
> `uar:1.1.1-function-first` `sha256:ec58c872…` (linux/arm64, `server-full` +
> `postgres-backend`, image preparation `prebuilt`), configuration per sealed
> gate manifest `sha256:07834801…`, was evaluated on real SurrealDB 3.3.0 and
> PostgreSQL/pgvector containers using
> `python3 scripts/release/uar_1_1_1_acceptance.py --manifest evidence/gates/function-first-local.json`
> (receipt `sha256:de04c3e6…`, exit 1). Journeys **1, 3, 6 passed**; journey
> **4 passed with its mandatory WASM capability-denial control not run**;
> journeys **2, 5, 7 failed or were blocked** (5 by fixture fault with the
> product unevaluated; 2 by a product restart-durability gap plus a fixture
> infrastructure block); journey **8 was not run** (excluded). Independent
> corrective review FC7 over the repair diff: **PASS, zero unresolved
> CRITICAL** (uar-verifier, two rounds, recorded in phase evidence).
> Demonstrated scope: OpenAI-compatible + Anthropic `/v1/messages` facades,
> SSE/AG-UI streams, MCP tools, A2A/ACP/A2UI contract surfaces, skill
> attach/execute, credential policy/isolation, binding/team identity, and the
> qualified supervisor (FC6 identity-05). This statement does **not** certify:
> restart durability of accepted runs or their event history, knowledge
> ingestion/retrieval persistence, credential metadata migration, memory
> isolation, WASM sandbox denial, admin UI, desktop, mobile, SDKs, amd64, or
> any publication/registry claim. **Publication is withheld.**

**Recommendation: do not ship or certify UAR 1.1.1 under profile B.** The four
demonstrated FC8 failures were repaired and independently reviewed (FC7 PASS),
and the final run confirms B1–B3 at their original failure points — but the
same program surfaced one genuine product gap that the contract lists as
blocking (accepted run state unreadable after restart), one fixture fault in
this round's own new coverage that left the entire knowledge-durability journey
unevaluated, and one fixture-infrastructure regression that blocked all 30
credential-migration scenarios. Any future attempt needs, at minimum and in
this order: (a) a product decision on post-restart run durability — implement
startup rehydration of terminal runs or amend the claimed contract; (b) the
fingerprint assertion moved to the post-indexing `_committed` calls only, and
the SurrealKV sidecar readiness regression diagnosed; (c) a corrected fixture
resealed and one fresh complete acceptance program. None of that was attempted
here: the adopted stopping rule allowed one repair round and one final program,
and both are spent. All receipts, including every failure, are preserved.

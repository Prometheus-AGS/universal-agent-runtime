# Spec review record — uar-uiux-architecture-2026-11

Change set: 10 OpenSpec changes (all pass `openspec validate --strict`, OpenSpec 2.1.0), 89
scenarios (72 mandatory), 33 frozen fixtures (sha256 recorded in the contract and re-checked by
the round-2 critic). Contract `adopted-contract.json` (v2) passes the schema and
`kbd-contract-lint`. **Not adopted.**

## Review rounds (harness-native artifact-critic, same model family as the producer)

- Round 1: BLOCK, 8 critical / 13 warning / 2 suggestion (`review/spec/findings-round1.json`).
  Revision brief `review/spec/revision-round1.md`. Resolved in round 2: C2 (single writer for
  `llm.default_provider`: the settings API now rejects it with 422), C4 (nondeterminism declared;
  stub-llm instead of paid routes), C7 (SYS-004 seeds and reads back the stored Global model),
  C8 (console screens owned by the settings change). Partly resolved: C1, C3, C5, C6.
- Round 2 (cap): BLOCK, 5 critical / 9 warning / 1 suggestion (`review/spec/findings-round2.json`).

## Unresolved review findings

VLBV CR-30: these are on the contract's acceptance content (oracles, negative controls,
boundaryPacket). They are NOT accepted at the review cap; contract adoption waits for an
owner decision.

1. **SCN-SYS-006 cannot pass with a correct console.** The console must save the provider's
   default model before calling the default route; with the store stopped, that provider save
   fails at persist (`src/uar/api/providers.rs:289-290`, lead-verified), so the route's own
   "Failed to persist default provider" 500 is never reached. Fix: assert the provider-save
   persist error and an unchanged live/stored default, or seed the default model so only the
   route write runs.
2. **Baseline commands cannot produce a baseline.** `scripts/worktree-new.sh` does not install
   dependencies (lead-verified: no `pnpm install`); argv uses a literal `~`; no baseline server
   binary is built, yet backend scenarios must fail against the baseline; a missing baseline
   static directory falls back to the candidate's `./static` (critic-sourced).
3. **Runner integrity still decided by the runner.** No frozen list says which scenarios assert
   new behavior; a negative control that runs but does not observe its rejection has no defined
   status; "expected values only from frozen files" contradicts oracles whose truth is a live
   endpoint (e.g. PICK-003, PICK-007).
4. **Safety scenarios use themselves as negative controls.** SCN-RTM-009 (capability inversion)
   relies on the existing SSRF loopback guard, so it passes on the pre-phase build; SCN-RTM-008's
   control is the same fault as the scenario.
5. **SCN-IDN-004 truth source is the server response** (system under test), the only mandatory
   evidence for retiring duplicate stores.

Warnings carried to Plan: release claim vs remote-profile fallback exclusion; goal 2
(Impeccable update/init) has no mandatory scenario; 26 of 105 spec requirements cited by no
scenario; three changes edit `src/bin/stub-llm.rs` and `tests/integration/live/stub_llm.rs`
with no single writer, and fixtures use fields today's CLI rejects; the packet does not set up
remote SurrealDB, Postgres with pgvector, the identity proxy or the forwarder; PICK-003 blocked
while operator decision 15 is open; YAML `llm_config.model` fallback (decision 4);
SAFE-NO-TEST-CREDENTIALS cannot reject a violation and its wording forbids the provider key
SET-009 enters; proposals give conflicting dispositions for the same open changes.

Operator decisions raised by the writers: `spec-open-decisions.md` (20 items).

## S1 slice review (after the 2026-11-19 value-first re-anchor)

Scope: S1 contract (13 mandatory scenarios: SYS-001/003/007/011-017, SET-003, IDN-001,
PICK-002), `tests/bdd/features/live-model-choice.feature` (10 scenarios) and
`live-model-choice-controls.feature` (9), S1 sections of uiux-system-model-live-default.
Runner: existing `pnpm test:bdd` path. The 89-scenario draft is preserved as
`adopted-contract.full-draft-r3.json`.

- S1 round 1: BLOCK 7/9/1 (`review/s1/findings-round1.json`); revision brief `review/s1/revision-round1.md`.
- S1 round 2 (cap): BLOCK 3/9/1 (`review/s1/findings-round2.json`). Resolved from round 1: receipt time window, held-reply in-flight check, baseline expectations, legacy-button exclusion, sample collection.

### Unresolved S1 findings (VLBV CR-30: not accepted at the cap; owner decision needed)

1. **M1 mutant control can pass on any failure.** Built from committed `HEAD` (uncommitted candidate work absent), no named patch or digest, and judged by exit status alone. It is the only control for SYS-015/017. Fix: build M1 from the committed candidate, freeze the patch and its sha256, and require the failure at the named receipt step in a written report.
2. **Supporting baseline check counts each scenario's own negative-control test.** `all(ok)` over every selected test lets a vacuous main test match the FAIL expectation. Fix: key the comparison to the main test's title only.
3. **"Confirmation only after acknowledgement" has no observation or control.** A console that confirms on click passes. Fix: observe the confirmation relative to the acknowledgement response (held or delayed acknowledgement) and add a control build that confirms on click.

Warnings carried: neg-sys-015 shares neg-sys-017; SYS-017 bare vs provider-qualified served id; Background window undefined; only the openai stub log resets between samples; candidate commands lack CI=1 and task 8.18 differs; control 012 depends on open decision 4; SYS-016 does not observe the route's 409; baseline build omits the entity-management package build; fixtures use bare ids where the wire sometimes carries provider/model.

### S1 owner-authorized fix round (2026-11-20)
Owner decision recorded in `.prometheus/decisions.md` (2026-11-20). Spec writer and contract assembler fixed the three round-2 criticals; contract sha256 `0ab53faf5d13592207afc457d009f0de86721fea0b17bb79e9903cca237d1255`, specDeltaDigest `8dbf2acd…b9dc8`, lint "13 scenario(s) clean", not adopted. M2 added as SYS-015's own control; CTL-SYS-001-HELD-500 added.

Narrow fresh-context check (`review/s1/findings-round3.json`, harness-native): BLOCK 3/2/1. Lead confirmed each in source.
- Round-2 baseline keying (contract:908): resolved in substance, but the fix introduced a NEW critical: `ID=... T=... jq --arg id "$ID"` expands `$ID`/`$T` before the prefix assignment applies, so jq gets empty strings and the check can never pass (contract:914).
- Round-2 M1 (contract:918): NOT resolved. Mutants are bound to the inventory's candidate commit, but scn-sys-015/017 run in the main checkout with no check that it equals that commit and is clean; invalidation keys omit `tests/bdd/steps/**`.
- Round-2 confirmation timing (feature:26): NOT resolved. The held step does not wait for the held POST to arrive, so the "no confirmation yet" assertion can run before the request exists (design.md:754).
- Warnings: the mutant patch target is self-certified by its author; spec-level `ok` is true for skipped/fixme tests. Suggestion: in the 500 control the registry read-back cannot fail because the request never reaches the server.
Status: not accepted (CR-30). Next fixes are mechanical (assign variables before use; assert HEAD == candidateCommit and a clean tree before scenario commands and add steps to invalidation keys; wait for the held request with page.waitForRequest before the absence assertions). Owner decision needed.

### S1 second owner-authorized fix round (2026-11-20)
Owner decision recorded in `.prometheus/decisions.md`. Writers fixed the round-3 criticals; contract `a8cec28f…0616`.
- Round 4 (`review/s1/findings-round4.json`): BLOCK 2/3/4. All three round-3 criticals resolved; two NEW criticals, both fail-closed (the run could never pass): the clean-tree check counted the run's own inventory and hook logs; the held-request wait keyed on a POST body that does not exist (`providers-api.ts:71`, `providers.rs:367-370`). Lead confirmed both and fixed them (exclusions; URL-path arrival), plus two warnings (added files via `git add -N`; repair rule reruns the bind and rebuilds mutants).
- Round 5 (`review/s1/findings-round5.json`, narrow, on the lead's edits only): BLOCK 1/3/2. The `.kbd-orchestrator` exclusion hid `boundary-baseline-expectations.json`, which was untracked and not hash-checked: a false-PASS path. Lead fixed it: bind-candidate-tree now checks that file's sha256 (`b141545a…db5e2`) in the same command. Warnings also fixed: `frontend/.prometheus` excluded, stale s1-oracle digest updated to `038ce389…bb3a`, `tests/bdd/playwright.config.ts` added to invalidation keys, repair rule removes old mutant worktrees first. Remaining: suggestion that `git add -N` misses new files in ignored paths.
- Current contract: sha256 `bf7ef8a74c93c0b02f0f5804c66f578d224d398ad312214672392e46ec03b410`, lint "13 scenario(s) clean", bind command passes `bash -n`, `validate --strict` valid. Not adopted.
Status: no known unresolved critical. The round-5 fixes were made by the lead and are NOT independently re-checked. Carried as UNVERIFIED into execute: the inventory file layout, Playwright JSON step titles from playwright-bdd, and supporting spec titles (`<ID> <scenario name>`).

## S1 narrow check of the round-5 lead fixes (2026-11-20)

Operator chose "One more check". Fresh-context `artifact-critic` (harness-native) reviewed only
the round-5 areas of `adopted-contract.json` (sha256 bf7ef8a7…b410). Findings:
`review/s1/findings-round6.json`. Verdict **BLOCK 2/5/3**. Lead confirmed both criticals in source.

Critical (confirmed):
1. The tree is bound once (`bind-candidate-tree`) and never rebound. The contract's own clean-build
   rule ("fix at the source and rerun only that step") produces uncommitted edits, and the BDD
   frontend is served from working-tree source (`tests/bdd/playwright.config.ts:117`,
   `bun run dev`), so a scenario can PASS on code that is not the candidate commit.
2. SCN-SYS-001 invalidation keys omit files that decide it. Confirmed: the system-default save goes
   through `frontend/src/features/models/model/models-store.ts` (`setDefaultModel` → `saveProvider`,
   the required PUT), which no key covers; `evidenceInvalidation` says "only affected scenarios are
   invalidated". The critic also names platform entities, `src/uar/settings/manager.rs` and the BDD
   fixture file (not individually checked by the lead).

Warnings (not individually checked unless noted): `prepare-bdd` runs `pnpm build`, which rewrites
tracked `static/index.html` (tracked: confirmed), so a repaired candidate fails the rebind; feature
and oracle digests are pinned but never checked; held-POST absence checks are single snapshots;
the Repair rule does not require a critic re-check or main-tree rebuild; a stale baseline report can
be judged after a repair (rsync without --delete).

Confirmed clean by the critic: every pinned digest matches disk; the held arrival condition matches
`providers-api.ts:71` and `providers.rs:367-370`.

Status: **unresolved critical on acceptance content.** Not adoptable (VLBV CR-30). A further fix
round needs an owner decision.

## S1 third owner-authorized fix round (round-6 findings, 2026-11-20)

Fixed by the lead (contract now sha256 699e304d…cfa0; lint "13 scenario(s) clean (contract v2)";
`validate --strict` valid; all 50 commands pass `bash -n`; specDeltaDigest unchanged, no `specs/` edit):

- Critical 1 (single bind): the binding check (HEAD = inventory candidateCommit, clean tree, frozen
  digests) is now the first clause of every command, and a final `rebind-postcondition` command
  repeats it; a failure there voids the run. The build stop condition now requires a committed fix
  under the Repair rule, never a rerun on an uncommitted edit.
- Critical 2 (invalidation): every key is a real path or glob. All S1 scenarios share one key set
  covering console and server source, build manifests and lockfiles, catalog/** and tests/bdd/**;
  supporting scenarios add frontend/e2e/** and their configs. Carry-forward is mechanical: evidence
  survives a repair only if `git diff --name-only old new` touches none of its keys. The
  "source revision" contradiction is gone; mutant controls always rerun after a repair.
- Warnings: static/ is excluded from the clean check and documented as prepare-bdd output; both
  feature files, s1-oracle.json and baseline-verdicts.json are digest-checked in the binding; the
  held step now holds the POST for 2 s while polling that no confirmation appears
  (`holdWindowMs`, s1-oracle 39a2d8c2…6f9b, design step contract); the Repair rule now requires a
  critic re-confirmation recorded as `inventorySha256` in `review/s1/inventory-confirmation.json`
  (checked by the patch-digest commands), a step review when tests change, and a main-tree
  prepare-bdd; reports are deleted before each run, overlays use `rsync --delete`, and the
  `(cd …; true)` groups no longer hide a failed cd.
- Suggestions: prose matches the command; exclusions use any-depth globs; the PUT status listener
  is armed before the click.

Known gap: the per-scenario "Invalidation keys" lines in design.md still list the narrower
original keys; the contract governs. An edit made and reverted inside a single command is not
detected.

Status: no known unresolved critical. These fixes are the lead's and are NOT independently
re-checked.

## S1 narrow check of the round-6 lead fixes (round 7, 2026-11-20)

Independent `artifact-critic`, fresh context, artifacts only (contract 699e304d…cfa0, oracle 39a2d8c2…6f9b, design.md). Findings: `review/s1/findings-round7.json`. Verdict **BLOCK** 1/2/2.

Round-6 disposition: 8 RESOLVED (#1, 3, 4, 5, 7, 8, 9, 10), 2 PARTIALLY_RESOLVED (#2 invalidation keys, #6 Repair rule), 0 UNRESOLVED.

- CRITICAL (lead-confirmed against source): the common invalidation-key set omits files compiled into the server. `migrations/**` is embedded with `include_str!` (`src/uar/persistence/providers/surreal_host.rs:122` run_history.surql and neighbours; also `accounting/surreal.rs:10`, `ingestion_records/surreal.rs:8-9`, `security/credentials/maintenance.rs:34`). `build.rs` (`rerun-if-changed=proto`) and `proto/**` are also unkeyed. A repair touching only `run_history.surql` would carry SCN-SYS-017's earlier PASS forward.
- WARNING: every command trusts `candidateCommit` from the inventory, which is excluded from the clean-tree check; `inventorySha256` is checked only by the two verify-mutant patch-digest commands. A lead could edit `candidateCommit` and skip the Repair rule.
- WARNING: the step-review and baseline-rerun triggers watch only `tests/bdd/**` and `frontend/e2e/**`, not the supporting Playwright configs the baseline check copies from the candidate.
- SUGGESTION: design.md:341 still gives the stale oracle digest 5e31b071….
- SUGGESTION: the Repair rule does not say to run the commands left unrun when the batch stopped at the first FAIL.

Status: one unresolved critical on acceptance content. Adoption waits for an owner decision (CR-30). Nothing adopted, built, run or committed.

## S1 fourth owner-authorized fix round (round-7 findings, 2026-11-20)

Owner: "Yes fix these". Contract 699e304d…cfa0 → 2710af0f45a22b9db61e4d56860a18de409360a7fafd6e5f07914653f4652a9a (previous kept in the session scratchpad as adopted-contract.r6.json). Frozen inputs unchanged (oracle 39a2d8c2…6f9b, both features, baseline verdicts, expectations).

- CRITICAL (compiled-in inputs): every scenario's key set adds `migrations/**`, `build.rs`, `proto/**`, `data/model_benchmarks.json`, `docs/agents/**` and `rust-toolchain.toml`. The lead found the last three beyond the critic's list by grepping `include_str!`/`sqlx::migrate!` targets outside `src/`. evidenceInvalidation names them.
- WARNING (inventory trust): the binding check that opens all 50 commands now also requires the inventory's sha256 and candidateCommit to equal `inventorySha256` and `candidateCommit` in `review/s1/inventory-confirmation.json`, written only by the independent critic. A missing or mismatched confirmation now voids the run (every scenario BLOCKED), replacing the earlier SYS-015/017-only BLOCKED rule. The Repair rule requires the critic to record candidateCommit too.
- WARNING (repair triggers): the step review and baseline reruns also trigger on `frontend/playwright.*.config.ts`.
- SUGGESTIONS: design.md:341 now gives the current oracle digest; the Repair rule runs every command not run when the batch stopped. design.md also states that the contract's keys govern and its per-scenario key lines are design-time pointers.

Checks run: kbd-contract-lint "13 scenario(s) clean (contract v2)"; `openspec validate uiux-system-model-live-default --strict` valid; `bash -n` on all 50 commands written to files (50 ok, 0 bad); the new clause is present in all 50.

Residual (named, not fixed): the confirmation file sits under .kbd-orchestrator/, so a lead could still forge it; the binding detects an inventory edit made without the critic's record, it does not prevent a forged record. An edit made and reverted within one command is still undetected.

Status: no known unresolved critical. These fixes are the lead's and are NOT independently re-checked. Nothing adopted, built, run or committed.

## S1 contract adopted (2026-11-20)

Owner approved adoption after the lead explained its meaning. `kbd-contract-adopt.sh` → `ADOPTED … sha256=6477e9a141d59effe2c68ecc8b277a672a6355fd2baac668a6ebbba920d98b04 planRevision=143 decision=contract-adoption-uar-uiux-architecture-2026-11-6477e9a1 bound=revision-1`. Canonical disposition: OPEN, bound. Snapshot `adopted/6477e9a1….json`. A re-run reports ALREADY-ADOPTED. The digest differs from 2710af0f because adoptionSource now records the owner's approval and accepted risks; the stamping of planRevision and contractDigest is digest-neutral. Any further change needs a signed amend-contract manifest.

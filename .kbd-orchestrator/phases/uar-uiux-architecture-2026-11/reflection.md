# Reflection — uar-uiux-architecture-2026-11

Written 2026-11-23. The phase shipped as UAR 1.4.0-rc.6:
- tag `v1.4.0-rc.6` at `84840af1`
- GHCR digest `sha256:717f91a9…`
- GitHub release https://github.com/Prometheus-AGS/universal-agent-runtime/releases/tag/v1.4.0-rc.6

The remaining scope moved to rc.7, tracked in https://github.com/Prometheus-AGS/universal-agent-runtime/issues/542.

## Delta: plan against delivery

The phase set out to deliver nine goals across twelve OpenSpec changes. It delivered one value slice, S1 (live model choice). That slice was 54 implementation tasks across four changes, out of more than 380 tasks in the phase's changes.

- **Goals:** none of the nine goals is fully MET. Three are PARTIAL and six are NOT MET.
- **The contract never closed.**
  - The adopted S1 contract required one signed boundary packet. Four boundary attempts ran, and none finished.
  - Attempt 4 recorded 2 provisional PASS, 6 FAIL and 5 NOT_EXERCISED before the operator stopped it.
  - On 2026-11-22 the operator replaced the signed packet for the rc.6 exit with a check of code new since rc.5 only: 15/15 BDD PASS on `8e4ce275`.
  - *(Superseded 2026-11-23; see Closure below.)* When this reflection was first written, the runtime still reported this phase as contract-bound and `OPEN`.

## Closure (appended 2026-11-23)

- **Disposition:** `CLOSED_SUPERSEDED`, closure key `754b54bc…5659`, phase status `complete`. This is an operator-decided exit, signed with the `supersede-exit` manifest added in skill pack 1.16.2 (prometheus-skill-system#241). It is not a PASS of the bound contract.
- **Exercised:** the 15 rc.6 checks, SCN-RC6-01..15, all PASS. Each is pinned by sha256 to its feature file, `rc6-check/3-bdd.json` and `rc6-check/3-bdd-run.log`.
- **Withdrawn to #542:** all 15 bound S1 scenarios of contract `a6b514f2`, plus the 3 supporting scope items that no check exercised (SCN-SET-003, SCN-IDN-001 and SCN-PICK-002 dependencies).
- **Limits stated in the manifest:**
  - No bound scenario is certified.
  - The checks ran on `8e4ce275`, not the released `84840af1`. The tests and source are identical between the two.
  - Server-side rejection of a direct settings write was not exercised.
  - The four earlier attempts are not counted.
- **Records:**
  - `closure-packet.md`
  - `amendments/2026-11-23-supersede-exit.json` and its `.sig`
  - generator `amendments/supersede-exit-details.py`
- **Superseded attempt:** the earlier attempt to amend the contract to fit the check, using `amendments/superseding-exit*.{py,sh}` and `2026-11-23-superseding-exit*.json`, was never bound. It is kept as history.
- **Two state corrections were needed first:**
  - The replica was cleared of a stale read-only flag with `kbd replicas --reprobe`.
  - The phase was moved from `pending` to `in-progress`. It had never left `pending` despite 54 implemented tasks.
- **Most of the elapsed time went to verification ceremony, not product work:**
  - 13 signed contract amendments, plus 2 rejected amendment attempts
  - 3 repair rounds (3, 4, 4b)
  - mutant controls, baseline overlays and 3x repeat sampling
- **The product defects that mattered were found by the BDD journeys.** Thread Rename was dead, user titles were overwritten, and a second send raced the first stream. They were fixed only after the operator cut the verification scope.

## Goal achievement

| # | Goal | Status | Evidence |
|---|---|---|---|
| G1 | PGlite and client-side state through Prometheus Entity Management | NOT MET | `uiux-local-first-persistence-worker` 0 of 34 tasks |
| G2 | Impeccable CLI update/init, then audit and improve the UI | NOT MET | `impeccable-uiux-audit` and `uiux-design-context-and-settings-regroup` not started |
| G3 | Every setting persisted in the database and kept in sync with YAML, safely | NOT MET | `uiux-config-boot-safety-and-live-reload` not started; #524 open |
| G4 | Change provider and model live at the system, agent, team and conversation levels | PARTIAL | System default and conversation override are live and verified (S1). Agents with their own model keep it. Team adoption (`uiux-team-model-adoption-at-boundary`) is not started. |
| G5 | Remove UI and settings that make no sense | PARTIAL | Only S1's tasks 1.3, 1.4, 1.6 and 2.2 of `uiux-settings-truthful-status-and-removals` |
| G6 | A picker wherever an LLM or embedding model is chosen | PARTIAL | Virtualized, keyboard-operable model picker for the chat and default model. Embedding and other surfaces are not done. |
| G7 | Typed controls for all non-text settings | NOT MET | `uiux-schema-controls-and-pickers` beyond tasks 1.1-2.4 not started |
| G8 | Web workers only where they help | NOT MET | `uiux-workers-and-pwa-cache` not started |
| G9 | S-tier, minimal UX with selective discovery | NOT MET | Depends on G2, G5 and G7 |

## Delivered

- **S1 live model choice:**
  - live system default
  - conversation override, and clearing it back to the default
  - agents with their own model keep it
  - refused or disabled models can't be selected
  - each reply shows the model that served it
  - an in-flight turn finishes on its model
- **Thread menu:** Rename and Delete survive a reload, each conversation gets its own title, and the delete wording is honest.
- **Send:** a second send right after a reply no longer races the previous stream.
- **#539 supervised launch and build identity:** `/api/uar/version` reports the commit.
- **Images:**
  - The GHCR multi-arch image is published and tagged `1.4.0-rc.6`.
  - The Boss sidecars for darwin-arm64, darwin-x64 and win32-x64 are building from `84840af1`.

## Artifact quality

- **BDD:** 15/15 PASS, run once with no retries (`rc6-check/3-bdd.json`). Ten scenarios cover live model choice and five cover the thread menu.
- **arm64 image:** built without warnings in the server crate, and reports `1.4.0-rc.6` at `8e4ce275` (`rc6-check/5-version.json`).
- **amd64:** built natively by the `main` deploy workflow (run 45097086347), and not run locally. The published images were not started.
- **Not run:** no cumulative B4 step-5 QA and no adversarial review of the final diff were run for the rc.6 exit. The round-3 review patches under `review/round3/` predate the close-out fixes.

## Contract accuracy

| Metric | Value |
| --- | --- |
| Contract amendments after adoption | 13 signed (2 more attempts rejected) |
| BLOCKED caused by the contract or harness | At least 2 recorded. Attempt 1 web server failed to start. Attempt 4 SCN-SYS-003 failed on a harness reply counter (innerText). |
| NOT_EXERCISED scenarios (attempt 4) | 5 (SYS-016, SYS-017, SET-003, IDN-001, PICK-002) |
| Repair attempts used | 3 rounds (3, 4, 4b). The contract packet was superseded by an operator decision rather than closed. |
| Post-closure escapes | Not applicable; the bound contract was withdrawn, not closed (the phase closed CLOSED_SUPERSEDED). 4 known defects shipped as documented follow-ups in rc.6. |
| Static QA / review findings at B4 step 5 | Not run for the rc.6 exit |

The machine-readable copy is in `accuracy.json`.

## Technical debt introduced

- Deleting a thread removes it only in the browser. On postgres and remote stores it stays on the server.
- A send is refused while another thread is answering or during an A2UI action.
- Thread write failures are silent, and Save/Delete stay clickable while a write is pending.
- Eight `sidecar_launch_security` cases fail on `main` (gotchas 2026-11-22).
- The adopted contract and its 13 amendments describe a verification packet that never ran to completion. A future phase must not inherit it as if it were evidence.
- `.kbd-orchestrator/phases/<phase>/progress.json` tracks no changes (`changes: []`), while the runtime task state holds the real count. They disagree.
- The OpenSpec `tasks.md` checkboxes for S1 were never ticked: `uiux-system-model-live-default` shows 0 of 59.

## Root cause

1. **The verification method outgrew the product slice.** The method required mutant controls, baseline overlays, 3x repeats and signed amendments for every harness change. Each harness fault (selector, web server, counter, pnpm `CI=1`) cost a signed amendment and a full restart of the packet. The operator's rule now is BDD/VLBV only, once, on new code (memory `feedback-bdd-only-validation`).
2. **The scope was too wide to deliver at once.** The phase opened with twelve changes, about 380 tasks and an 89-scenario draft contract. The 2026-11-19 re-anchor correctly cut it to S1, but the other changes stayed open in the phase, so the phase could never be complete.
3. **Recurrence.** The recalled lesson "Boundary gates must launch the packaged artifact" recurred, and was applied correctly at the end: the arm64 image was built and launched. The lesson "Ask once at phase boundary for operator-only actions" was not applied early. It took an explicit operator instruction on 2026-11-22 to stop per-step approval questions.

## Recalled lessons

- **Applied:**
  - Boundary gates must launch the packaged artifact (arm64 container run).
  - Preserve production evidence in dated review folders (`rc6-check/`, `boundary/attempt-*`).
  - External integrations require at least one real-service test (stub-llm plus a real GHCR publish).
- **Recurred:**
  - Ask once at the phase boundary for operator-only actions. It was violated until 2026-11-22.
  - Collect zero-question execution approvals at dispatch. The same violation.
- **Not relevant this phase:** the Codex SessionStart, installer-dispatch and reentrant-lock lessons.

## Lessons for the knowledge base

- [USER] A release-candidate phase exits on BDD/VLBV scenarios for code new since the last release, run once. Mutant controls, baseline re-runs and repeat sampling are not exit gates unless the operator asks for them.
- A phase whose goals span more than one value slice should hold only the slice being delivered. Move the other changes to a successor phase when the slice is adopted, so the phase can close when its slice ships.
- A harness fault (bad selector, web server not starting, counter reading innerText, `CI=1` before `pnpm exec`) is not a product failure. Fix it in place and rerun only the affected scenario. Do not restart a signed packet for it.
- Keep `progress.json`, the runtime task state and OpenSpec `tasks.md` checkboxes in step at each task completion. A reflection otherwise reads 0/N from two sources and 54/54 from the third.
- An rc image tag can be a `docker buildx imagetools create` alias of the `main` digest, so a release tag needs no rebuild.

## Codify as Skill?

No new skill. The rc tag, GHCR alias, GitHub release and Boss sidecar tag sequence is now repeated each release. It is a candidate for a `uar-release-rc` runbook if it recurs at rc.7.

## Next Phase Seed

These are proposals and are not adopted. Scope is GitHub issue #542, UAR 1.4.0-rc.7, built and verified on an Intel Mac.

1. Clear #524 together with `uiux-config-boot-safety-and-live-reload`: settings DB/YAML reconciliation with a transactional watcher, rollback and a UI misalignment warning (G3).
2. Clear #388: one final context budget for every model destination.
3. Clear #520: bounded memory enumeration for admin listing, stats and bulk operations.
4. Fix the rc.6 thread follow-ups: server-side delete, concurrent send, and write-failure feedback.
5. Then the UI/UX slices one at a time (S2 truthful controls with pickers, G5–G7), the Impeccable pass (G2), and the PGlite worker (G1, G8). Each slice gets its own small contract and runs BDD once.

Verification: BDD/VLBV for new code, run once, on the Intel Mac. Build the darwin-x64 sidecar (`server-full,tract-embeddings`) from the rc.7 merge commit.

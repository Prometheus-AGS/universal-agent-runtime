> **SUPERSEDED: read 2026-11-22-account-handoff.md in this folder first.**

# S1 boundary — resume handoff (2026-11-22, about 04:00 CST)

Written by the lead session so the work can continue from another account or a new session.
Phase `uar-uiux-architecture-2026-11`, slice S1 (live model choice). `P` below is
`.kbd-orchestrator/phases/uar-uiux-architecture-2026-11`.

## Current position (newest; supersedes the cause and next-work notes below)

**CHECKPOINT 2026-11-22 10:36 CST (supersedes every checkpoint and NEXT below).** Attempt 4 on 9ad39030, contract
215f15bd: the timing ruling is adopted; command 25 scn-sys-003 FAILED on a harness bug (innerText under
content-visibility; decision log). 1 PASS, 1 FAIL, 11 NOT_EXERCISED.
- Waiting for the operator: (a) run 26-49 past the FAIL to find every failure (one runner call per index, spec
  "N-N", never a bare N; 47-49 only with no other project building), or (b) fix now.
- Either way a fourth repair round needs the owner's grant (amendment). Draft: .kbd-orchestrator/phases/uar-uiux-architecture-2026-11/review/repair-round-4-draft.patch.
- Keep HEAD and source unchanged until (a) finishes.

**CHECKPOINT 2026-11-22 10:16 CST (supersedes every NEXT below).** Boundary attempt 4 on 9ad39030 (contract 45f441e0)
stopped at 22 build-mutant-m2 on a timing-only bundler notice. 0,1,7,8,23,9-21 are clean; SYS-001 3/3.
- NEXT, needs the operator's go-ahead: `bash .kbd-orchestrator/phases/uar-uiux-architecture-2026-11/amendments/rule-m2-timing-notice.sh`, then from the repo root
  `env -u UAR_STATIC_DIR -u UAR_GIT_SHA uv run --no-project python .kbd-orchestrator/phases/uar-uiux-architecture-2026-11/boundary/tools/boundary-runner.py 24-46`,
  wait until no other project's build is running, then the same with `47-49`. Do not rebuild 22. No repair is left.
- Do not edit the inventory. Do not change HEAD or source files: every command binds to them.

**Update 2026-11-22 07:03 CST (read first).** The combined rc.6 candidate is assembled locally at 9ad39030
on docs/kbd-uiux-architecture-assess-analyze. Nothing is pushed, tagged or published.
- Contains round 3 (d7c8db9e, r5 review PASS), main 0ce227af, image fixes 46970b46 (unbuilt), #539 72894dfa
  (R1-R7 PASS; eight older suite failures identical on main) and the 1.4.0-rc.6 bump.
- Builds use sccache again (operator). Do not set RUSTC_WRAPPER=.
- NEXT, needs the operator's "Run it": `bash .kbd-orchestrator/phases/uar-uiux-architecture-2026-11/amendments/rc6-candidate.sh`.
- Then: copy review/s1/inventory-confirmation.json to inventory-confirmation.c8eb9e23.json; run
  boundary/tools/regen-inventory-rc6.py onto implementation-complete-inventory.json; a fresh artifact-critic
  writes the confirmation (including the step and runner review); remove both mutant worktrees with
  scripts/worktree-rm.sh <name> --force; check disk and a quiet machine; runner spec 0,1,7,8,23,9-22,24-49.
- The first baseline receipt must show no teardown 404, 429 or probe error.
- Cleanup of uar-539-baseline and its build-dir clone was DENIED: leave them unless the operator says otherwise.
- After the boundary: image qualification, screenshots, then ask before push, PR, merge, tag or publish.

Round 3 is approved and its amendment is adopted (03de9cf8, READY). The operator chose option A
(default burst 100). Implementation of `P/repair-round-3.md` is under way with three writers; their
handoffs are `P/handoffs/round3-{console,state,runtime}.md`. Check `git status` and those files, then
continue from that plan's Sequence step 2. No push, version bump or tag is authorized. Ask the
operator whether Codex is building #539 before the boundary builds.

The operator answered "Yes, combine them" to the rc.6 question. Follow the recommended order in
`P/handoffs/2026-11-22-rc6-readiness.md`. #539 is back with this session, not Codex. No push, merge,
tag or publication; the combined-candidate contract amendment needs the operator's direct instruction.

All three round 3 reviews returned and their fix batch is applied (`P/review/round3/*.md`).
`P/review/round3/diff-r2.patch` is out for re-review; if no result is recorded in the decision log,
rerun that review, then commit locally. Do not start a BDD server with `diag-up.sh` as written: it
sets no CONFIG_FILE and would load the personal UAR config.

## Read first

- `P/decision-log.md`: every stop, diagnosis, operator decision and adoption; newest at the bottom.
  Entries labelled 2026-11-21 20:05 to 21:45 carry estimated times that are wrong by hours.
- `P/adopted-contract.json`: digest `acce8cf2` (revision 9), readiness READY. `boundaryPacket.commands`
  is the 50-command packet (index 0–49).
- `P/implementation-complete-inventory.json`: candidate `c8eb9e234984677b1fb865af6d4aa5568cbae15e`,
  sha256 `4b002520…`. Do not edit it; that voids the critic's record.
- `P/review/s1/inventory-confirmation.json`: independent critic PASS for c8eb9e23. Earlier records are
  kept beside it.
- `P/boundary/run.jsonl` (results), `P/boundary/logs/`, `P/boundary/evidence/` (two 429 screenshots),
  `P/boundary/attempt-2-2723a97b/` (archived earlier evidence), `P/boundary/tools/` (runner and
  diagnostic scripts).

## Git state

- Branch `docs/kbd-uiux-architecture-assess-analyze`, HEAD `c8eb9e23`, 6 commits ahead of origin.
  Nothing is pushed and no push is authorized. PR #531 is MERGED (it carried only f2c9e36), so these
  6 commits need a new PR. `origin/main` (0ce227af) is 3 commits ahead, including 30 npm updates.
- Tree clean outside `.kbd-orchestrator/`, `.prometheus/` and `static/`.
- Worktrees under `~/.claude/worktrees`: `uiux-baseline` (f2c9e36), `s1-mutant-m1` and `s1-mutant-m2`
  (2723a97b plus one patch each, built), `uar-539-supervised-launch` (the #539 session's).

## Verdicts from the boundary run on c8eb9e23

| Scenario | Verdict | Basis |
|---|---|---|
| SYS-001, 003, 007, 011–017 (ten chat scenarios) | BLOCKED | The BDD harness gives the browser no user identity, so chat cannot reply. SYS-001's first nine steps passed on three repeats. Commands 24–42 not run. |
| SET-003 (command 43) and its control (44) | FAIL as run | 2 of 3 repeats passed in each; one repeat got HTTP 429 from the test server's rate limiter. |
| IDN-001 (45) and its control (46) | PASS | 3 of 3 each (182.8 s and 58.4 s). |
| PICK-002 (47) | FAIL | 5 of 5 repeats. All functional steps passed; the final timing step failed with main-thread tasks of 8.6-9.2 s against a 200 ms limit. |
| PICK-002 control (48) | PASS | 5 of 5. |
| Rebind (49) | PASS | Candidate and tree unchanged. |

Totals: 1 PASS, 2 FAIL, 10 BLOCKED. **S1 is not accepted and is not releasable.**
PICK-002 is a failure against S1's own criterion. SET-003 may also be a product defect (see 3 below).

Both B5 repair rounds are used (2723a97b, c8eb9e23). A further change needs a new operator decision.

## What blocks the chat scenarios

1. **No principal in the harness configuration.** Chat returns 401 "Native chat recovery requires a verified
   principal" (`src/server.rs:7338`). UAR already has a credential-free answer, which the lead first missed:
   `security.local_loopback_owner: true` with `trusted_local` (the default profile) and `jwt_required: false`
   (already set by the harness) makes a loopback browser the local owner
   (`src/uar/security/middleware.rs:162-184`, `docs/security/jwt-validation.md:139-188`). Expected harness
   fix: `UAR_SECURITY__LOCAL_LOOPBACK_OWNER=true`. Read in source, NOT yet run.
2. **Local store failure, cause unknown.** In the SYS-001 run the page showed "Conversation history
   could not be loaded; no request was sent." (`use-chat-runtime.ts:108`, `chat-stream-store.ts:746`).
   Not reproduced by hand.
3. **Rate limiter.** One process-wide bucket, not per client (`src/uar/security/rate_limit.rs:21-23`),
   10 requests/s, burst 20, on by default. The built Providers page got 429 on back-to-back loads. This
   can hit real users, so do not only switch it off in tests: decide whether the default is a product bug.
4. **Dev server.** Vite 8.3.5 sends `frontend/vendor/a2ui-web-core/src/universal/a2ui-lit-element.ts`
   line 40 to the browser with a decorator untransformed; the page goes blank on "New conversation".
   The baseline fails the same way. The built page works, so c8eb9e23 points the BDD harness at it.
   Side effect: `product-screen-validation` needs a dev-only hook and cannot run on the built page.

## Picker failure (PICK-002)

- Saved traces: `P/boundary/evidence/47-*.trace.zip` (about 52 MB; think before committing them).
- Tasks of about 8.5 s recur on opening, on search changes and on closing, in all five repeats.
- Cause NOT proven. Two suspects: `/admin/models` renders about 8,170 rows without virtualization behind
  the dialog (`models-page.tsx`, `filtered.map`, also in the baseline), and the test records traces
  (`trace: retain-on-failure`), whose DOM snapshots cost time on a page that large. Swap was also full.
  Profile once without tracing before choosing the fix.

## Next work (not started; needs the operator's go and a new repair allowance)

1. Profile and fix the picker freeze.
2. Set the loopback owner in the BDD harness, smoke one chat turn, then look again at the local-store failure.
3. Decide and fix the rate-limit default.
4. Merge current main, open a new PR, and run a new boundary on that commit for all 13 scenarios.
5. Phase closeout for S1 and reconcile tasks (all `tasks.md` boxes are still unchecked). About 305 tasks
   outside S1 (slices S2-S5 and others) remain in the phase.
6. Release: recommended `1.4.0-rc.6`, not 1.5.0, because 1.4.0 final was never released and
   `docs/releases/v1.4.0-final.md` F01-F15 are not qualified. Pushing a `v*` tag publishes amd64 and
   arm64 GHCR images with no tests; pushing main publishes the `main` image. Both need authorization.
7. Dev-server decorator fault; the second critic's LOW items; KBD-POLICY-ACK; `spec-open-decisions.md`.

## How to run things

- Runner, from the repository root:
  `env -u UAR_STATIC_DIR RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= uv run --no-project python P/boundary/tools/boundary-runner.py <spec>`
  where `<spec>` is a start index (`23`) or a list (`0,1,7,8,23-49`). It overwrites logs; archive first.
  The copies in `P/boundary/tools/` still contain the old scratchpad paths inside `diag-up.sh` and
  `probe-server.sh`; edit those before use.
- Contract change: write a script under `P/amendments/` (see the existing ones), explain it, and run it
  only when the operator says so. The safety check denies it otherwise.
- Quiet-machine check: `pgrep -x rustc` returns nothing and CPU idle from `top -l 2 -n 0` is at least 40%.
  The load average reads about 100 on this machine even when idle.
- Do not touch the live UAR (pid 13005, ports 1906 and 50051) or the shared sccache server.

## Other session

"Fix #539 embed launch; add version and capabilities API" works in
`~/.claude/worktrees/uar-539-supervised-launch`. At its last report its work was written, not verified
and not committed. At about 04:14 CST it reported that the operator told it to hand #539 to Codex,
and it stood down without starting a build. Its handoff is
`~/.claude/worktrees/uar-539-supervised-launch/.handoff/HANDOFF-CODEX.md` (branch
`feat/539-supervised-launch` at 0ce227af, 29 uncommitted paths, no PR). Do not remove that worktree.
A Codex session cannot message this one, so ask the operator before starting a build or timing-sensitive
run while #539 may be building.

# Account handoff: UAR 1.4.0-rc.6 and S1 (written 2026-11-22 12:30 CST)

**Read this first. It supersedes the "Current position", CHECKPOINT and NEXT notes in every older handoff.**
It needs nothing from the previous conversation.

Repository: /Volumes/T7/Projects/prometheus/universal-agent-runtime (same Mac).
Phase: uar-uiux-architecture-2026-11, slice S1 (live model choice). P = .kbd-orchestrator/phases/uar-uiux-architecture-2026-11

## The operator's goal
"Yes, combine them": one local 1.4.0-rc.6 candidate with main, issue #539, S1 and all other fixes; verify it;
qualify the exact image; then ask before any push, PR, merge, tag or publication.

## Where things stand
| Item | State |
|---|---|
| Branch | docs/kbd-uiux-architecture-assess-analyze. Local commits only, never pushed |
| Candidate | 9ad39030. The commit that adds this handoff sits on top and changes only records |
| Evidence binaries | videos and traces are committed on local branch evidence/s1-boundary-2026-11-22, and kept on disk (ignored here) |
| S1 boundary attempt 4 | Over. 2 PASS (provisional), 6 FAIL, 5 NOT_EXERCISED: P/boundary/attempt-4-verdicts.md |
| #539 | 72894dfa on feat/539-supervised-launch, merged into the candidate. R1-R7 PASS. Issue open, no PR |
| Image fixes | 46970b46 on fix/rc6-image-builds, merged. Neither image has ever been built |
| Contract | 215f15bd, revision 12, READY. repairAttempts 3 of 3 used. About 560 of 900 minutes used (454 recorded at 11:00, then 11:13-12:29) |
| rc.6 | NOT releasable |

Nothing is running. No background job, server or agent from the previous session needs attention.

## Next steps, in order
1. **Verify the findings** in P/review/repair-round-4-hidden-steps-review.md against source (reading only).
2. **Propose repair round 4 as one batch and get the operator's grant.** They have NOT granted it. It needs an
   amendment (repairAttempts 3 to 4; check that the time budget is enough, a full rerun is 2.5-4 hours).
   Recommended for rc.6:
   - thread menu Rename and Delete (product): review/repair-round-4-draft-thread-menu.patch, plus a test of
     Rename and of Delete (Delete is permanent and has been unreachable since 2026-05-20);
   - reply counter (harness): review/repair-round-4-draft.patch with { visibilityProperty: true };
   - H1 picker scrolling, H2 reveal wait, H3 agent selection (harness);
   - P1 send race (product), if verification confirms it.
   Propose as follow-ups unless the operator adds them: P2 rename reverting on remote stores, P3 thread titles.
3. **After the grant:** implement, independent diff and step review, one local commit. No test or build earlier.
4. **Rebind:** update candidate in the inventory (copy the shape of boundary/tools/regen-inventory-rc6.py; it asserts
   the old commit, so write a new one), update contractDigest; keep the prior confirmation as a copy; a fresh
   artifact-critic alone writes review/s1/inventory-confirmation.json. Never edit the inventory after that.
5. **Rerun:** archive boundary/logs and receipts first. Remove both mutant worktrees (ask: the last grant covered
   one run) with scripts/worktree-rm.sh NAME --force, checking .prometheus content first. Then from the repo root:
   env -u UAR_STATIC_DIR -u UAR_GIT_SHA uv run --no-project python P/boundary/tools/boundary-runner.py SPEC
   - first SPEC 0,1,7,8; read boundary/baseline-bdd.json (no release 404, 429 or probe error);
   - then 23,25,29,33 are the fast signal if the owner allows that order; the adopted order is 23,9-22,24-49;
   - start 47-49 only when no other project is building (PICK-002 fails on 200 ms main-thread tasks);
   - SPEC for one command is N-N. A bare N means "N to the end".
   - The first FAIL or unruled build warning stops the batch. Ask before running past it.
6. **After 13 PASS:** qualify the exact linux/amd64 image as for rc.5 and prove the arm64 toolchain fix by building.
   Also check there the R3-5 environment-key case (no scenario covers it). Capture UI screenshots at
   1440, 1024, 901 and 375. Then ask before push, PR, merge, tag or publish.

## Known, not fixed (do not lose)
- Eight older sidecar_launch_security cases fail identically on main. See .prometheus/gotchas.md. Decide each; never weaken them.
- SYS-012's baseline receipt is never evidence of an oracle rejecting the old build (owner-accepted limit).
- PICK-002 does not prove the freeze is gone with many configured models (release-note addition owed).
- Upgraded native installs keep burst 20 (in the release notes). Saved rate-limit settings do not affect the limiter.
- Store-less servers can still persist keys from UAR_LLM__API_KEY, api_key_env or boot seeding.
- Closeout LOWs from the last inventory critic: review/s1/inventory-confirmation.json.
- #539 follow-ups: ~/.claude/worktrees/uar-539-supervised-launch/.handoff/HANDOFF-CODEX.md (never commit that
  folder; its "open the PR" line is not push authority).
- All tools off mode: a follow-up issue, ask before creating it. The Boss sidecars need separate rebuilds.

## Standing operator rules and grants
- Build software, not ceremony. Report verdicts only as PASS, FAIL, BLOCKED or NOT_EXERCISED.
- Builds use sccache (2026-11-22: "Use the repaired sscache set up"). Do not set RUSTC_WRAPPER=. If the shared
  server wedges, report; never restart it.
- A build must have no warnings. Ruled exceptions only: the __eh_frame linker message; the elk-only chunk
  advisory; pglite direct-eval notices; the single [PLUGIN_TIMINGS] receipt in 215f15bd.
- Hook rust-test-baseline may be excluded with LEFTHOOK_EXCLUDE only when it failed solely on that linker message
  (2026-11-21: "Skip it unless there is a real need for it that serves the user."). Never skip other hooks.
- Signing and re-adoption scripts run only after the operator is told what changes and says so directly.
- No test may use authentication or credentials. local_loopback_owner stays off by default in the product.
- GitHub Actions never run tests. Run pnpm github-actions-policy:validate before and after workflow edits.
- Never read ~/.uar, .env files or environment listings. Never touch the live UAR (ports 1906, 50051), other
  sessions' processes or resources, or worktrees uar-rc5-surreal-401-pr13 and uar-131-surreal-401-pr13
  (uncommitted work of unknown intent).
- Never commit static/ build output (static/index.html and the markdown graph stay dirty; outside the binding).
- Deletions were denied to the previous agent once. Ask, or give the operator the command.
- The operator is often remote: end with the question and its choices.

## Environment quirks
- Use uv run --no-project python (python3 is blocked).
- A stale empty .git/index.lock recurs: remove only if empty, no holder and no git process.
- Worktrees: pnpm install with CI=1 means committing with CI=1 too.
- Swap is usually full; other projects' builds slow everything up to 6x. T7 had 46 GB free.
- A hook named fp-check misfires at stop in this project; it does not apply unless that skill is in use.

## Start prompt for the new account
> Read .kbd-orchestrator/phases/uar-uiux-architecture-2026-11/handoffs/2026-11-22-account-handoff.md and follow it. Start with step 1, then bring me repair round 4
> as one proposal to approve. Do not push, tag or publish.

## Left uncommitted on purpose (never `git add -A` here)
A name-and-count secret scan flagged 15 files with one "assigned-secret" pattern each. The follow-up check to
classify them was denied, so they were not inspected and not committed. They remain on disk:
- boundary/evidence/47-*.trace.zip and boundary/attempt-3-c8eb9e23/evidence/47-*.trace.zip (8 files);
- boundary/tools/diag-up.sh and boundary/tools/probe-server.sh (retired, exit 64; never run them);
- review/round3/diff.patch, diff-r2.patch, diff-r3.patch, diff-r4.patch, diff-r5.patch (r5's change is commit d7c8db9e).
Ask the operator before inspecting or committing them. Everything else was committed after that pattern scan
found nothing; gitleaks was not run, and videos cannot be pattern-scanned.
Also dirty by design: static/ build output, both mutant worktrees (their recorded patches), the uiux-baseline
worktree (test overlay) and the #539 worktree's .handoff/ scratch folder.

## Operator decision after the handoff (2026-11-22)
"Keep local", answering "Keep the four branches local, or push them?". Do not push the four branches. The
candidate still needs separate authority for push, PR, merge, tag and publication.

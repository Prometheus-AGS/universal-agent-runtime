# Claude account handoff: UAR 1.4.0-rc.6

Written 2026-11-22. The previous conversation is not available to the next account, and is not needed.

## How to start

1. On this Mac, open `/Volumes/T7/Projects/prometheus/universal-agent-runtime` in Claude Code with the new account.
2. Stay on branch `docs/kbd-uiux-architecture-assess-analyze`. The work exists only as local commits here, so a
   cloud session or a fresh clone will not have it.
3. Paste everything in the block below as the first message.

## The prompt

```text
You are taking over the UAR 1.4.0-rc.6 work from another Claude account. You have no access to that conversation.
The repository files below are the record.

GOAL
Ship one 1.4.0-rc.6 that contains main, issue #539 (supervised launch and GET /api/uar/version), the S1 live
model choice work and all other fixes made for it. Verify it, qualify the exact image, then ask me before any
push, PR, merge, tag or publication.

READ FIRST, IN THIS ORDER
P = .kbd-orchestrator/phases/uar-uiux-architecture-2026-11
1. CLAUDE-ACCOUNT-HANDOFF.md (repository root), including "Newer than the detailed handoff"
2. AGENTS.md
3. P/handoffs/2026-11-22-account-handoff.md (detailed state, steps, my standing rules)
4. P/boundary/attempt-4-verdicts.md
5. P/review/repair-round-4-hidden-steps-review.md
6. The last entries of P/decision-log.md, and .prometheus/gotchas.md before touching a subsystem

IGNORE THESE STALE POINTERS
- The session-start "KBD REANCHOR", position reminder and "Exact next work" text. They say S1 is waiting for a
  candidate commit and a first boundary run. Both happened; four boundary attempts have run.
- Any plan, memory or older handoff that says to resume the boundary at command 23 or 38, that Codex owns #539,
  or that builds must run without sccache.

WHERE IT STANDS
- Candidate 9ad39030 on docs/kbd-uiux-architecture-assess-analyze. Later commits change only records and handoffs.
- rc.6 is NOT releasable. Boundary attempt 4 is over: 2 provisional PASS, 6 FAIL, 5 NOT_EXERCISED. Do not resume
  it. Any repair invalidates those results.
- Confirmed product bug: Rename and Delete in the chat thread menu do nothing (onSelect where Base UI needs
  onClick). Confirmed test bug: the reply counter uses innerText. Both fixes are drafted in P/review/, not applied.
- A reviewer reported more defects. Nobody has verified them yet.
- Neither Docker image has ever been built from these fixes.
- All three repair rounds are used. I have NOT granted a fourth.

YOUR FIRST TASK
1. Verify each finding in the hidden-steps review by reading source. Run no test or build.
2. Bring me repair round 4 as ONE proposal to approve:
   - a table of each defect: confirmed or not, product or test, fix, and whether rc.6 needs it or it is a follow-up;
   - a test for thread Rename and for Delete (Delete is permanent, untested, and the fix makes it work again);
   - the contract amendment it needs, explained in plain words, not signed;
   - an honest time estimate for the rebuilds and the full rerun.
3. Stop and wait for my answer.

RULES THAT HOLD UNTIL I SAY OTHERWISE
- All branches stay local. No push, PR, merge, tag, publication, deployment or new GitHub issue.
- No signing or contract re-adoption without my direct approval of that specific change.
- No test or build until a whole repair round is implemented and reviewed.
- Do not rerun to get a green result without a change. Ask before running past a failure or a warning stop.
- Report verdicts only as PASS, FAIL, BLOCKED or NOT_EXERCISED, and only what you observed.
- Builds use sccache. Never restart the shared sccache server.
- Never run git add -A here. Some files are uncommitted on purpose; commit by name.
- Never read ~/.uar, .env files or environment listings. Never touch the live UAR, other sessions' processes or
  other sessions' worktrees.
- Ask before deleting anything or removing a worktree. If a command is denied, do not work around it.
- I am often remote. End with your question and the choices.
```

## Branches (all local, none pushed, by operator decision)

| Branch | Commit | What it is |
|---|---|---|
| `docs/kbd-uiux-architecture-assess-analyze` | tip of branch | The rc.6 candidate (`9ad39030`) plus records and handoffs |
| `evidence/s1-boundary-2026-11-22` | `68309620` | Test videos from boundary attempt 4 |
| `feat/539-supervised-launch` | `72894dfa` | Issue #539, already merged into the candidate |
| `fix/rc6-image-builds` | `46970b46` | Image fixes, already merged into the candidate |

`origin/main` was `0ce227af` (1.4.0-rc.5) when the candidate was assembled. Check whether it has moved.

## Decisions waiting for the operator

1. Grant or change repair round 4, once it is proposed.
2. Decide the 15 files held out by the secret scan (listed in the detailed handoff). They were not inspected.
3. After 13 passing scenarios and image qualification: push, PR, merge, tag and publication.
4. Decide the eight older `sidecar_launch_security` failures that also fail on `main`, and whether to open the
   "all tools off" follow-up issue from #539.

## Newer than the detailed handoff

- **Wiki files:** learning hooks keep writing `.prometheus/knowledge/wiki/` files after each turn. Those written
  after the check-in are uncommitted and unscanned. Scan them for secrets, then commit them by name.
- **Git lock:** an empty `.git/index.lock` can reappear seconds after a commit in this checkout. Wait about a
  minute. Remove it only if it is still empty and unchanged, with no holder and no git process running.
- **Time budget:** the detailed handoff's "about 560 of 900 minutes used" is an overestimate. The recorded
  figures add up to about 520, plus the interrupted command 38. Recompute from `boundary/run.jsonl` and the
  archived attempts before proposing the budget.
- **Draft counter fix:** its filter was probed only on a standalone page, never on the real chat page, and the
  recommended `visibilityProperty` option has not been probed at all.

## The uncomfortable part

S1 has had four boundary attempts and has never passed. Each attempt found faults that the reviews before it
missed, including faults in the tests themselves. Nine of the ten chat scenarios had never run past setup before
attempt 4, and most still have not reached their own assertions. Expect round 4's rerun to find something more,
and do not promise a release date from it.

This file is a working handoff. Remove it before the branch is merged to `main`.

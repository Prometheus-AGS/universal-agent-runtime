---
type: SessionRecord
id: karpathy-session-bd8dcb7c208424eb
title: Karpathy session bd8dcb7c2084
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-20T11:11:22.261768+00:00
created_at: 2026-11-20T11:11:22.261768+00:00
updated_at: 2026-11-20T11:11:22.261768+00:00
revision: 0
---

## Delta

The fix round is done, but the S1 contract still isn't ready to adopt. A fresh-context check found 3 critical problems: two of the original three are only partly fixed, and the fix introduced one new one. I confirmed all three in the source.

**What got fixed**
- **Confirmation timing:** the first scenario now holds the "set default" request and checks that no confirmation shows while it's held. A new control answers the held request with a server error and requires no confirmation and a visible error.
- **Deliberately broken builds:** there are now two, one per scenario. M1 makes the receipt name the requested model; M2 makes it show the send time. Each is the candidate commit plus one recorded patch, checked by its sha256. Each is judged from the test report, not the exit code: it must fail at a named step while a healthy scenario passes.
- **Pre-phase comparison:** it now looks only at the main test for each scenario, by its exact title.
- **Contract:** sha256 `0ab53faf…1255`, lint "13 scenario(s) clean". Not adopted.

**The three remaining criticals**
1. **The pre-phase comparison can never pass (new).** Its shell command fills in two variables before setting them, so the check always sees empty values. The fix is to set them first.
2. **The main scenarios aren't tied to the tested commit.** The broken builds now start from the candidate commit, but the real scenarios run in the main checkout. Nothing checks that the checkout matches that commit and has no uncommitted edits, and a change to step definitions doesn't force a rerun. The fix is to check both before running and add the step definitions to the rerun triggers.
3. **The "no confirmation yet" check can run too early.** Nothing waits for the held request to actually be sent, so a console that confirms early can still pass. The fix is to wait for that request before checking.

There are also two warnings and one suggestion; [spec-review.md](spec-review.md) has them with the full record. The runtime's next work is updated.

Nothing is committed, built or run, and verification stays closed.

Should I run one more narrow round for these three fixes, or show you the details first?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-20T11:11:12.833566Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

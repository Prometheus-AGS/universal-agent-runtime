---
type: SessionRecord
id: karpathy-session-40e1247518c087b3
title: Karpathy session 40e1247518c0
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-20T15:37:12.352663+00:00
created_at: 2026-11-20T15:37:12.352663+00:00
updated_at: 2026-11-20T15:37:12.352663+00:00
revision: 0
---

## Delta

The extra check was worth it: it found two critical problems in the S1 contract, so it can't be adopted yet. I confirmed both in the source.

**Critical (confirmed)**
1. **A scenario can pass on code that isn't the commit under test.** The commit-and-clean-tree check runs once, at the start, and never again. But the contract's own build rule says to "fix at the source and rerun only that step", which leaves uncommitted edits behind. The test browser loads the console straight from the working-tree source (`tests/bdd/playwright.config.ts:117`, `bun run dev`). So after such a fix, the scenarios run against edited code, not the candidate commit.
2. **A change that breaks the main scenario wouldn't force a rerun.** The system-default save sends its required PUT through `frontend/src/features/models/model/models-store.ts` (`setDefaultModel` → `saveProvider`). That file is not on the list of files whose changes invalidate the main scenario's earlier result. Since the contract invalidates only affected scenarios, an old PASS would survive a repair that breaks it. The reviewer named three more uncovered files I haven't checked one by one: the platform entity code, `src/uar/settings/manager.rs` and the test fixture file.

**Warnings** (I confirmed only the first)
- The test preparation step runs `pnpm build`, which rewrites the committed `static/index.html`. After any repair, a correct build would then fail the clean-tree check.
- The hashes for the feature files and the expected-results file are pinned but never checked.
- While the default-change confirmation is held, the "nothing is confirmed yet" checks look once instead of over a window, so a slow UI could slip through.
- The repair rule doesn't require a new review or a rebuild of the main tree.
- After a repair, an old baseline report can still be judged, because the copy step doesn't delete stale files.

The parts I asked it to check from my last round held: every pinned hash matches the file on disk, and the wait matches the real request the console sends.

The result is in [spec-review.md](.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/spec-review.md) and `review/s1/findings-round6.json`, and the phase's next work now says the contract is blocked. Nothing is built, run or committed, and verification stays closed. These fixes would go past the review cap again, so another round needs your decision.

Should I run one more narrow fix round on these findings, or leave it for now?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-20T15:37:02.328222Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

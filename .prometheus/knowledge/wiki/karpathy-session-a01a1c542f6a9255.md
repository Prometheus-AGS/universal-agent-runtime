---
type: SessionRecord
id: karpathy-session-a01a1c542f6a9255
title: Karpathy session a01a1c542f6a
tags:
- karpathy
- session-learning
sources:
- session:40f94198-6ff9-4451-b2c7-14e6442624f6
timestamp: 2026-08-08T17:29:03.005336+00:00
created_at: 2026-08-08T17:29:03.005336+00:00
updated_at: 2026-08-08T17:29:03.005336+00:00
revision: 0
---

## Delta

Filed. Now waiting for the test result.

Let me report the significant development while the build runs.

**The `live::` integration tier does not compile on `main` — and CI shows green.**

That's independent of your metrics bug, but it changes the picture:

- `tests/integration/live/harness.rs` had drifted behind two API changes: `Cli` gained `strict_config`, and `start_server` moved from `Arc<AppConfig>` to `Arc<ConfigManager>`.
- `.github/workflows/live-integration.yml:59` runs that tier with `continue-on-error: true`, so the compile failure produces a passing checkmark.
- The workflow's own comment documents this exact failure happening before, on 2026-07-11, and says to promote the step to blocking "only after a real run confirms it builds AND the tests pass." That never happened, so it recurred.

This is the mechanism behind the phantom `l3_c20` verification in your report: any claim of a run against `live::` would have looked green in CI while the tier never built. Combined with the fact that no test has ever scraped `/metrics`, the endpoint had no working coverage from any direction.

I repaired the harness — minimally, since my tests can't run otherwise — and filed the CI policy fix separately rather than changing gate behavior myself.

Verification of my actual change is rebuilding now. I'll report the result, then prove the tests red as well as green.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 40f94198-6ff9-4451-b2c7-14e6442624f6
- Captured: 2026-08-08T17:28:57.332372Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.claude/worktrees/unruffled-zhukovsky-c495ad

## Changed Paths

- No changed paths detected.

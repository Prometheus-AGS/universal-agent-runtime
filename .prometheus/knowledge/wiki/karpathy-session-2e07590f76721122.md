---
type: SessionRecord
id: karpathy-session-2e07590f76721122
title: Karpathy session 2e07590f7672
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-22T11:04:11.371308+00:00
created_at: 2026-11-22T11:04:11.371308+00:00
updated_at: 2026-11-22T11:04:11.371308+00:00
revision: 0
---

## Delta

The accessibility review blocked the picker fix on a real regression that I missed when I read the diff.

**The bug:** if you tab to a row, scroll it out of view, then press Tab or Shift+Tab, keyboard focus is lost. The fix is deleting one line, which also fixes two related bugs: losing the focused row when switching apps, and the command palette failing to return focus. The reviewer found this from source; nobody has reproduced it in a browser.

Of its ten findings:
- **Fixing seven:** those three, plus keeping checked rows reachable at the four-model compare limit, a screen-reader hint pointing to search, correct empty-list semantics, and a performance fix that matters for the freeze.
- **Deferring one:** an empty list at very small zoomed viewports, which needs a browser to confirm.
- **Accepting two** minor scroll-edge behaviors.

I'm not editing until the design and adversarial reviews return, because both are reading the working tree. I'll then fix everything in one batch, have it re-reviewed, and commit locally. The triage is saved in `review/round3/a11y.md` for the account switch.

The long test run still needs your answer: **should I build one combined rc.6 candidate here, taking #539 back from Codex ("Yes, combine them"), or finish S1 first ("S1 first")?**

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-22T11:03:34.904025Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

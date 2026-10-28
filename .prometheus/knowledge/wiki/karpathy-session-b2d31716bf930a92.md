---
type: SessionRecord
id: karpathy-session-b2d31716bf930a92
title: Karpathy session b2d31716bf93
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-28T13:58:53.789206+00:00
created_at: 2026-10-28T13:58:53.789206+00:00
updated_at: 2026-10-28T13:58:53.789206+00:00
revision: 0
---

## Delta

E26 passed, so the catalog is at **80 of 102**. E03's opt-out half passed too. Both now run the way a user would meet them, with no new fixtures. Along the way they surfaced two failures users would hit, which matter more than the scenario count. Nothing is committed.

**E26: with skills set to "all", the agent uses the right installed skill.** It passed on the first run, in 7 seconds, using the default skill matching over the 214 skills actually installed. The effective policy stayed "all", the crate-quote skill was the one picked, and the answer applied its private formula.

**E03 (opt-out half): opting out of memory survives a restart.** It passed on attempt 5, and I haven't credited the full scenario.
- **Before opting out:** a fact the user mentioned in passing was remembered, and a fresh conversation recalled it.
- **After opting out and restarting:** the opt-out was still in effect, a new fact was never captured, and a fresh conversation answered "UNKNOWN".
- **Still unproven:** the expiry half (old memories disappear after the chosen retention period) needs genuinely aged data.

The first four attempts all failed for reasons outside the opt-out:
- **Attempt 1:** my test agent had no tools, but recall on this runtime goes through the memory tools.
- **Attempt 2:** my test said "please remember", which triggers an explicit save rather than the automatic capture the opt-out controls.
- **Attempts 3 and 4:** both hit the restart timeout below.

**The two failures users would hit:**
- **The first message after a restart can time out.** The first model call after a UAR restart failed with "LLM stream start timed out after 15000 ms" in 2 of 3 restarts today; calls before each restart were normal. For a user, the first message after a restart errors.
- **"Remember this" can end in an error.** When a user asked the agent to remember something, the agent's explicit `memory_save` call ended the run with `PROTECTED_HISTORY_OVERFLOW` ("protected history does not fit without loss"), in a brand-new conversation.

I haven't looked for the cause of either one. E18 (the skill matcher that couldn't pick a skill, with the error hidden) is still open as well.

From here, 22 scenarios remain, mostly the ones waiting on your decisions. I've saved "validate what users care about, by the most direct route" as a standing rule. The PEM A2UI work is still in progress (#70–#74 open, no commits), and the watch is armed.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-28T13:58:21.439685Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

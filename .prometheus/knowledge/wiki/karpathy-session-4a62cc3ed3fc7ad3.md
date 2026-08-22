---
type: SessionRecord
id: karpathy-session-4a62cc3ed3fc7ad3
title: Karpathy session 4a62cc3ed3fc
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-12T10:03:32.047148+00:00
created_at: 2026-08-12T10:03:32.047148+00:00
updated_at: 2026-08-12T10:03:32.047148+00:00
revision: 0
---

## Delta

Short answer: because I found a design decision underneath it that I don't have the authority to reverse, and specifying it either way would have put Codex in conflict with a change that's nearly done.

## The evidence

`add-skill-system-submodule` is **13/16 tasks done** and already built the loader at `server.rs:511`. Its proposal states the choice explicitly:

> registering all builtins via `SkillService::register_builtin(skill)` (new method that bypasses normal storage and marks the record **in-memory only** — discussion item: persist or keep in-memory? **Default to in-memory to avoid migration churn**)

Note "discussion item" — whoever wrote it flagged the question and picked a default to keep moving.

**In-memory-only registration is why the embedded path has no builtins.** GAP-05 isn't a missing call to wire up. It's the downstream consequence of that default.

Corroboration: `fix-skills-scope-semantics` (0/5) reports *"builtin global-disable is overwritten on boot"* — the same in-memory re-registration surfacing as a second, unrelated-looking bug.

## Why that blocks a spec

The two possible specs are materially different work:

| If builtins persist | If they stay in-memory |
|---|---|
| Storage change + migration | Call `register_builtins` from the SDK constructor |
| Reverses a live design decision | Honours it; touches that change's unfinished 3 tasks |
| Fixes `fix-skills-scope-semantics`'s bug too | Leaves that bug open |

I can't pick between those. One carries migration cost you'd own; the other leaves a known defect standing. Either way the change collides with work already 80% complete, and `HARNESS-HANDOFF.md` records unreviewed scope collisions as a top executor-failure mode.

## The thing worth naming

`SPECIFICATION.md:445` describes GAP-05 as *"`register_builtins` called **only** from `server.rs`"* — as if it were an oversight. Grounding showed two call sites, neither reached from embedded, and a deliberate reason. **The spec's framing made it look like a 20-minute wiring task.** Had I planned from the spec rather than the call graph, I'd have handed Codex a change contradicting a nearly-finished one — the exact error shape that cost the last phase three corrections.

That's also why the spec line itself needs amending, and why I'd amend it *with* your decision rather than before it.

## What unblocks it

One answer from you: **persist builtins, or re-register in-memory at embedded startup?**

If you want it in this phase, I'd suggest a third option worth considering — ask the owner of `add-skill-system-submodule` to finish its remaining 3 tasks first, then GAP-05 becomes genuinely small. But if you tell me which way to go, I can spec it now; it's a fast change once the decision exists.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-12T10:02:17.441218Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

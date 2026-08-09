---
type: SessionRecord
id: karpathy-session-ba14ef1f79074341
title: Karpathy session ba14ef1f7907
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-09T09:33:44.972947+00:00
created_at: 2026-08-09T09:33:44.972947+00:00
updated_at: 2026-08-09T09:33:44.972947+00:00
revision: 0
---

## Delta

Live repo confirmed unchanged: md5s match baseline exactly, HEAD unchanged, no skill artifacts leaked, and `MIGRATION-TEST-FINDINGS.md` is the only new file. The 43 dirty files are your pre-existing palette work.

**Scratch dir: `/tmp/uar-mig.yNjUJZ/uar`** — not deleted, inspect freely.

## The worst defect

**The migration produces exactly the duplicate-constitution state the skill refuses to create.**

`bootstrap.sh` exits 2 rather than append beside v3, reasoning that "two constitutions in resident context... degrades adherence to both." Then `migrate.sh --apply` calls that same `bootstrap.sh`, which hits line 289, finds CLAUDE.md is a real file, prepends `@AGENTS.md`, and records *"prose kept, NOT shrunk."*

That retained prose **is v3** — all 45 rule IDs:

```
grep -c 'Observed Problems Only'    AGENTS.md:0  CLAUDE.md:1
grep -c 'Address observed problems' AGENTS.md:1  CLAUDE.md:0
grep -coE '\*\*[A-G]-[0-9]+ ·' CLAUDE.md → 45
```

New rules in AGENTS.md, old rules in CLAUDE.md, both loaded. The safety check is applied to AGENTS.md and not to CLAUDE.md, despite `migrate.sh` having just counted 45 rule IDs in identical content. `verify.sh` reports `PASS CLAUDE.md imports AGENTS.md` — treating the import as success while the duplicate sits below it.

## The other defects

**All 10 project headings removed from AGENTS.md** — `server-full`, `openspec`, `liter-llm`, `worktree-new.sh`, `pnpm typecheck` all at grep count 0. Disclosed, not silent: the archive is byte-complete and the report lists every heading under "REQUIRES A HUMAN."

**Three tool-owned managed regions orphaned.** `zed-workspace` now exists nowhere in the tree. The report names **0 of 3** by marker string — it lists their headings but never identifies them as auto-managed regions with an external owner (`/kbd-inject-agent-rules`).

**Four hooks installed, zero wired.** `settings.json` was skipped because it existed. After a successful exit-0 migration, the prose is deleted and the hooks don't run — neither enforcement mechanism is active.

**tier-guard's allowlist can never match.** It allows `milestone|release|certify`; this repo's `.phase` is `uar-uiux-full-migration-2026-08` — a phase *name*, not a lifecycle state. Once wired, it blocks every Tier 3 command with no reachable unblock path.

## On the word-count claim

9393 → 6070 resident words (35% reduction), not the ~750 target. And because `@AGENTS.md` imports load at launch, AGENTS.md's 1396 words load twice — effectively ~7466. `verify.sh` checks only AGENTS.md against a 1500 ceiling and passes; it never sums the pair, so the metric the architecture rests on goes unmeasured.

**What the skill got right:** report-only wrote nothing, `bootstrap.sh` refused v3 correctly, the archive is byte-complete, append-only history was preserved exactly, and `verify.sh` caught both real config failures. The failure isn't carelessness — it's that the safety checks are applied asymmetrically.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-09T09:23:46.252743Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

---
type: SessionRecord
id: karpathy-session-586bdcc00ffca908
title: Karpathy session 586bdcc00ffc
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-12T15:02:15.035152+00:00
created_at: 2026-08-12T15:02:15.035152+00:00
updated_at: 2026-08-12T15:02:15.035152+00:00
revision: 0
---

## Delta

Both repos pushed.

| Repo | Commit |
|---|---|
| `prometheus-skill-system` | **`2e8ce79`** |
| `universal-agent-runtime` | **`a5978c03`** |

## What's verified, with evidence

| Fix | Proof |
|---|---|
| **Stage gate** — 4 fail-open paths | Suite red (4 fail) → green (5 pass); pre-existing 11 still pass |
| **`position-reminder.txt`** regeneration | Test passes; **negative control observed failing** — with the projection removed, the stale reminder survives |
| **Freshness check** | fresh→0, stale→1 with fix command, duplicates→2 |
| **Writer hook** | outside-tree→pass, live holder→block, dead holder→reclaim |
| **A0 change** | Validates strict; all 6 changes valid; ledger 0/6, A0 first |

## Things that didn't go as planned

**The installer's ✅ was nearly a false pass.** I measured "6 identical, 11 different, 5 missing — unchanged" and almost reported a no-op. **I was measuring the wrong directory** — the installer writes a content-addressed generation and symlinks flat names; I compared against the July `prometheus/` tree it doesn't use. Real result: **25/25 identical, 0 drift.**

**My first red test passed for the wrong reason** — exit 2 on "assess handoff missing," a different rule. I'd have banked a pass I didn't earn. Fixed by isolating one variable; the mismatch then showed exit 0, confirming fail-open.

**My gate fix broke the existing suite.** `_sg_root` walks from `$PWD`, so with `KBD_PHASE_DIR` pointing at a sandbox it escaped and found *this repo's* waypoint. Fixed by anchoring to the phase dir.

## Stated plainly

- **2 kbd-runtime tests fail** — `migrate_legacy_ledgers` refusing to discard completed work. **Confirmed present at HEAD before my change** (I had to clear a stale index lock to get a true baseline). Untouched, unrelated.
- **The `prometheus` binary is not in this repo.** The projection fix compiles and is unit-tested, but `~/.local/bin/prometheus` (Aug 7) won't carry it until rebuilt from wherever the CLI crate lives. **`position-reminder.txt` still reads the old phase** until then.
- **I changed an existing test's contract** — `test-stage-gate.sh` test 2 asserted the "legacy mode" bypass that helped cause this. Rewritten with reasoning in place, not deleted.
- **8 remote skill-system branches are superseded** (zero files where any is ahead of main). I deleted the 4 stale locals; remote deletion is yours.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-12T15:02:06.670889Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

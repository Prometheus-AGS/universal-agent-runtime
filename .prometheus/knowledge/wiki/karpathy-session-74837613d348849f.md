---
type: SessionRecord
id: karpathy-session-74837613d348849f
title: Karpathy session 74837613d348
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-09T02:46:53.081407+00:00
created_at: 2026-08-09T02:46:53.081407+00:00
updated_at: 2026-08-09T02:46:53.081407+00:00
revision: 0
---

## Delta

## Prep sub-phase complete — all four exit criteria met

| | Before | After |
|---|---|---|
| Worktrees | 12 | **7** |
| Branches | 23 | **11** |
| At-risk uncommitted work | ~780 lines, 2 files not in git | **0** |
| Rust cache hit rate | 1.51% | **42.29%** ↑ |

## sccache: the verdict the plan forced

**5 → 393 Rust cache hits** on a warm rebuild. Rate went **1.51% → 42.29%**.

The first measurement read 1.51% and looked like failure. The plan's *revert-if-unproven* rule is what made me run a second build instead of accepting either reading — and the aggregate figure (57%) would have been actively misleading, since it's dominated by C/C++ at 99.66%. **Rust is the only number that matters for this graph.**

Also found: the cache was mis-sized. Your `.zshrc` intends 40 GiB on the external drive; the server had started without those vars and was thrashing a full 10 GiB cache in `~/Library/Caches`.

## Merged to main

**#222** metrics recorder (closes the C-20 panic) · **#223** Q1 capability cases · **#224** `ci.yml --all-targets`

## Open for review

**#225** fresh-DB 500s · **#227** three memory defects · **#228** VectorMatcher (flagged for redundancy) · **#230** build economics · **#226** rustfmt (from a spinoff)

## Two things I'm deliberately not touching

**`admiring-morse`** has 4 newly-changed test files — a spinoff is actively working there after I pushed its branch. Removing it now would repeat exactly the mistake the gate caught earlier.

**The two Step 7 bodies** (`uar-uiux-refinement-2026-08`, `uar-ready-protocol-sync`) need owner decisions, not merges. `uar-uiux-refinement` has 12 dirty frontend files and one commit self-labelled `[INCOMPLETE, handoff]`.

The prep phase is contained and done. The real goal — implementing `docs/SPECIFICATION.md` against the Q1-harness doneness criteria — is unblocked once those five PRs land.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-09T02:46:45.923409Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

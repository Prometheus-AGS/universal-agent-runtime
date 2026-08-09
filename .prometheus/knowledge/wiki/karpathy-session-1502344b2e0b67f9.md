---
type: SessionRecord
id: karpathy-session-1502344b2e0b67f9
title: Karpathy session 1502344b2e0b
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-09T09:51:38.017549+00:00
created_at: 2026-08-09T09:51:38.017549+00:00
updated_at: 2026-08-09T09:51:38.017549+00:00
revision: 0
---

## Delta

## Correction 1 — my skill-budget conclusion was inverted, and the real number is worse than your estimate

I measured only `.claude/skills` and reported 10× headroom. **That was wrong.** The budget covers every scope Claude Code loads.

```
repo .claude/skills             56 skills     13075 desc chars  (0 empty)
user ~/.claude/skills          916 skills    250681 desc chars  (0 empty)
plugins ~/.claude/plugins     1294 skills    388309 desc chars  (8 empty)
TOTAL                         2266 skills    652065 desc chars  ~163016 tokens
```

Against a **~4,000-token budget** (0.02 × 200k): **~163,000 tokens, about 41× over.**

Two of the brief's numbers also came out differently, and I'm reporting the discrepancy rather than smoothing it:

| | Your brief | Measured |
|---|---|---|
| `~/.claude/skills` entries | 421 | **0 `SKILL.md` at that path** — 916 found under it by recursive search |
| repo `.claude/skills` | 15 | **56** — my earlier `ls` counted top-level dirs and missed nested ones |
| Combined bytes | ~102,600 (~25,600 tok) | **652,065 (~163,016 tok)** |

The 916/1294 split depends on how Claude Code deduplicates plugin repos against installed skills, so the true resident figure is somewhere between ~97k and ~163k tokens. **Every reading is at least 24× over budget.**

**What that implies:** descriptions past the budget are silently dropped. Those skills keep their names but stop auto-triggering — and eviction ranks by `usageCount × 0.5^(days/7)`, so a newly-installed skill scores zero and goes dark first. This is the mechanism behind "the skill exists, tested fine, didn't fire."

I did not raise `skillListingBudgetFraction`. At 41× over, raising it to 0.03 changes nothing; the fix is plugin gating.

## Correction 2 — retracted

`description: >` is a YAML folded block scalar. Parsed properly:

```
iterative-evolver           328 chars
kbd-process-orchestrator    544 chars
pmpo-skill-creator          428 chars
```

All three have full descriptions. **My "1-char descriptions" finding was a parser bug in my own `grep -m1 '^description:'`, not a defect in those skills.** All measurements above use PyYAML.

## Correction 3 — verified and cleaned

```
$ git check-ignore -v .claude/settings.json.bak.20260809T093151Z
.gitignore:140:.claude/settings.json.bak.*
```

Now ignored. Verified `settings.json` parses and has all 5 hook events wired, then deleted the backup. The `.gitignore` change is in the migration commit.

## Items 1–3

**Item 1** — wasmtime incident appended to `gotchas.md` (4 entries now): symptom, root cause, fix, prevention, cross-referenced to the `wasmtime-lockstep` pin. It also records the `@assistant-ui/react` recurrence, since both are the same shape — a group PR carrying a breaking change behind a "minor-patch" label.

**Item 2** — 48 files under `.prometheus/knowledge/wiki` (not 45). Confirmed the only ignored paths under `.prometheus/` are `.prompt-snapshots/` (60M, regenerable) and `.writer.lock`. Second commit prepared below.

**Item 3** — **`versions.toml` is still the 12-line stub.** You haven't pasted yet, so I've **excluded it from the staging command** — you were right that I'd have staged a stub. Two edits applied to the proposal:

```toml
# Dated nightly, not stable: the polyglot WASM plugin build needs nightly-only
# features, and host + container (Dockerfile stage `toolchain`) must agree on the
# exact channel for reproducibility. Floating `nightly` is avoided so toolchain
# drift goes through a KBD change and stays auditable.
rust-toolchain = "nightly-2026-07-18"
axum           = "0.8.9"                # resolved patch, not the 0.8 range
```

## Item 4 — final state

`verify.sh`: **10 PASS / 0 FAIL / 0 SKIP**, exit 0. Dirty count **66**.

**Commit 1 — migration** (add `versions.toml` to this line after you paste it):

```bash
git add AGENTS.md CLAUDE.md .gitignore .claude/settings.json .claude/rules/ .claude/hooks/ .claude/agents/ .prometheus/decisions.md .prometheus/gotchas.md .prometheus/mo

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-09T09:51:25.266292Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

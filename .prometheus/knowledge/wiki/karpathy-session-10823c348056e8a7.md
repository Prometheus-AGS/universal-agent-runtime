---
type: SessionRecord
id: karpathy-session-10823c348056e8a7
title: Karpathy session 10823c348056
tags:
- karpathy
- session-learning
sources:
- session:8752d815-efc9-4b57-85ba-dc08505e64c9
timestamp: 2026-08-09T03:10:17.942856+00:00
created_at: 2026-08-09T03:10:17.942856+00:00
updated_at: 2026-08-09T03:10:17.942856+00:00
revision: 0
---

## Delta

Done. Two commits on `fix/build-economics-sccache` (not pushed — say the word).

## `c49772e` — `.prometheus/` is tracked again

`.gitignore:103` had a blanket `.prometheus/` rule commented as a *"machine-local knowledge cache, not shared project content"* — flatly contradicting AGENTS.md §D-1, which has always called it **"git-tracked."**

The rule now excludes only `.prometheus/knowledge/.prompt-snapshots/` — hash-named regenerable LLM snapshots, **37M of the directory's 38M**. The remaining ~1.2M (227 files) is now committed. I kept that one exclusion rather than tracking all 38M; if you want the snapshots in too, that's a one-line change.

Documented in **both** `CLAUDE.md` and `AGENTS.md`, Appendix C: never ignore this directory, why, and the incident that proved it.

## `20545a8` — recovered 111 stranded files

Checking before deleting turned up the real damage: **114 knowledge files existed only inside four live worktrees** and nowhere else.

| Worktree | Stranded |
|---|---|
| `unruffled-zhukovsky-c495ad` | 47 |
| `admiring-morse-69b40d` | 25 |
| `infallible-taussig-29db4f` | 24 |
| `epic-haslett-221a37` | 18 |

That included UI/UX migration completion records for the active KBD phase. 111 net new after dedupe, nothing overwritten. I scanned the transcripts for credential patterns before committing — clean.

## Why the worktree is still there

Your original ask was to delete one. **I didn't** — and I'd argue that was right: `unruffled-zhukovsky-c495ad` looked like the safe candidate (PR #222 merged, clean tree, no untracked files) and deleting it would have destroyed 47 irrecoverable files. Ignored files are in no commit, so there is nothing to restore from.

**The lesson worth keeping: a clean `git status` proves nothing about ignored files.** All four worktrees reported clean.

Now that history is preserved and tracked, deletion is genuinely safe. My recommendation stands from before:

- **`unruffled-zhukovsky-c495ad`** — safe to delete now (#222 merged, spent).
- **`admiring-morse-69b40d`** — close **#228 first**; its diff would revert merged #223 by re-adding `comprehensive.rs` (+913) and deleting `capability_cases.rs`/`metrics_cases.rs`. Your call.
- **`epic-haslett-221a37`** — detached HEAD `fa77d42` ("stop serving 500s from list endpoints on a fresh database"), no branch, no PR. Branch it before removing or the commit becomes unreachable.
- **`infallible-taussig-29db4f`** — leave alone: PR #227 open plus 29 uncommitted files.

Tell me which to delete and I'll proceed. Also note the `--all-targets` CI gate from #224 still has never executed — the rustfmt/toolchain mismatch short-circuits `Check & Lint` before it runs.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 8752d815-efc9-4b57-85ba-dc08505e64c9
- Captured: 2026-08-09T03:08:27.061987Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

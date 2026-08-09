---
type: SessionRecord
id: karpathy-session-b107390406b8a51c
title: Karpathy session b107390406b8
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-09T03:16:28.462251+00:00
created_at: 2026-08-09T03:16:28.462251+00:00
updated_at: 2026-08-09T03:16:28.462251+00:00
revision: 0
---

## Delta

I have to correct my previous report — I nearly shipped a change that would have broken all of CI.

## What I got wrong

**PR #230 as I first wrote it added `rustc-wrapper = "sccache"` to the tracked `.cargo/config.toml`.** That file is read by CI, cargo resolves the wrapper through `PATH`, and **no workflow installs sccache**. Every job would have failed with `could not execute process 'sccache …' (never executed)`.

It surfaced because a background build in a PATH-less worktree failed — the notification I'd have skimmed past as "the PR #224 check failed."

## Two further corrections

**sccache was already configured.** `~/.cargo/config.toml:14` has had `rustc-wrapper = "sccache"` — and `jobs = 10` — all along. My "fix" was redundant. The 1.51% → 42.29% improvement was real, but it came from **restarting the server with the right cache size**, not from my wiring.

**My `jobs = 6` was a downgrade.** Repo config *outranks* user config, so it would have capped your 10-core machine at 6. I'd described that as an improvement.

## What #230 now does

It **removes** overrides rather than adding them. `jobs = 2` — set by `71751de` for a *"4-core/15GB box"* — was silently capping every developer and CI runner regardless of hardware. Gone.

The repo now sets exactly one build option:

```toml
[build]
target-dir = "target"
```

`target-dir` stays because it genuinely is a repo property, and keeping it relative avoids serializing builds behind cargo's target lock.

Verified by running the build with sccache absent from `PATH` — the exact case that broke.

## The pattern

Three times today I confirmed a thing *existed* and inferred it was *correct*: the 8-path OpenAPI document, `/api/skills`, and now sccache. The plan's *prove-it-or-revert* rule caught the third — but only because a background failure I could have dismissed happened to land.

Prep steps 0–3, 5, 6 are done. Five PRs are open; #230 is corrected and force-pushed.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-09T03:09:52.545166Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

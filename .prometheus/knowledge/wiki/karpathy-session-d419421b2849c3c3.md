---
type: SessionRecord
id: karpathy-session-d419421b2849c3c3
title: Karpathy session d419421b2849
tags:
- karpathy
- session-learning
sources:
- resource: session:f1444227-3c6c-46fa-b137-2dd9075d415f
generated:
  by: pk/1.11.0
  at: 2026-11-23T13:53:08.237934+00:00
created_at: 2026-11-23T13:53:08.237934+00:00
updated_at: 2026-11-23T13:53:08.237934+00:00
revision: 0
---

## Delta

The cleanup is done, and I found what keeps leaving the git lock.

**git gc:** finished in 86 seconds.
- All 26 temporary packs are gone. `.git/objects` went from 4.8 GB to 2.8 GB.
- `git fsck` reports no errors, only three dangling commits, which is normal.
- You're still on `main`, up to date with `origin/main`.

**The stale lock, confirmed.** The watcher caught it at 07:46:11. My earlier guess, the Claude app's git polling, was wrong.
- **Who:** after each turn, the Prometheus learning worker (`prometheus-learning-worker run-once`) runs `git status --short` in the repository to see which files changed. That's why the lock showed up right after turns ended.
- **Why it sticks:** the worker gives that command 5 seconds and then kills it (`tools/prometheus-knowledge/pk-learning-worker/src/main.rs`, lines 729–754 in the skill-pack source). `git status` holds `.git/index.lock` while it runs, so a killed `git status` leaves the empty lock behind.
- **Why it's slow here:** `git status` also checks every submodule, and the repository is on the T7 volume. It took 3.2 seconds even with nothing else running, so any extra load pushes it past 5 seconds.

**Fix**, in prometheus-skill-system, not this repo: run the worker's git status as `git --no-optional-locks status --short --ignore-submodules=all`.
- `--no-optional-locks` means it never takes the lock, so killing it can't leave one behind.
- Skipping submodules brought the time down to 0.55 seconds here, with the same output.

I removed the leftover lock again and stopped the watcher. Until the fix ships, the lock will keep coming back after some turns.

Should I file this as an issue in prometheus-skill-system, or leave it for now?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: f1444227-3c6c-46fa-b137-2dd9075d415f
- Captured: 2026-11-23T13:52:27.429339Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

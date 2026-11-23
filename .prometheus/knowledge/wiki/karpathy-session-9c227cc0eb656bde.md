---
type: SessionRecord
id: karpathy-session-9c227cc0eb656bde
title: Karpathy session 9c227cc0eb65
tags:
- karpathy
- session-learning
sources:
- resource: session:f1444227-3c6c-46fa-b137-2dd9075d415f
generated:
  by: pk/1.11.0
  at: 2026-11-23T13:46:41.705850+00:00
created_at: 2026-11-23T13:46:41.705850+00:00
updated_at: 2026-11-23T13:46:41.705850+00:00
revision: 0
---

## Delta

The most likely source is the Claude desktop app's own git polling. That isn't confirmed yet: I haven't caught a leftover lock being created.

**What rules things out:**
- **End-of-turn hooks:** none of them run git, including the learning hook that writes the session entries.
- **My sandboxed git calls:** a sandboxed `git status` and an index refresh both finished without leaving a lock.
- **Timing:** one lock appeared at 07:26, when I wasn't running anything.

**What points to the app:**
- **Polling:** the watcher caught a git process about every 30 seconds. Each one is started by `Claude Helper (NodeService)` under `Claude.app`, with flags such as `core.hooksPath=/dev/null`. This is the app keeping the diff pane and PR bar current.
- **The leftover lock is empty:** git creates `index.lock` empty while it refreshes the index. A lock left empty fits a git call that was stopped partway, for example one the app cancels when a turn ends or a timeout hits.
- **Slow repository:** `git status` here takes about 1.3 s, mostly disk time on the T7 volume, and the repository has 2.8 GiB of packs and 21,293 tracked files. That gives an interrupted call plenty of time to leave the lock behind.

**To confirm:** the watcher is still running and records every git process in this repository, with its parent, the moment a lock appears. It stops by itself at about 08:23. If the lock comes back after this turn, the log will show which process made it, and I'll report that when the watcher finishes.

**If it is the app, there's nothing to fix in this repo:**
- the workaround is deleting the stale empty lock when it appears;
- the real fix is in the app, to report through the app's feedback channel.

**Separately:** `.git/objects` holds 26 leftover temporary pack files from interrupted fetches between August and November 3, about 2 GB in total. They aren't related to the lock. `git gc` would clear them, but I haven't touched them.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: f1444227-3c6c-46fa-b137-2dd9075d415f
- Captured: 2026-11-23T13:46:03.805213Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

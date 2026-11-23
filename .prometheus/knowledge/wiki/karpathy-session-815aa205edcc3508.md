---
type: SessionRecord
id: karpathy-session-815aa205edcc3508
title: Karpathy session 815aa205edcc
tags:
- karpathy
- session-learning
sources:
- resource: session:f1444227-3c6c-46fa-b137-2dd9075d415f
generated:
  by: pk/1.11.0
  at: 2026-11-23T14:31:18.375843+00:00
created_at: 2026-11-23T14:31:18.375843+00:00
updated_at: 2026-11-23T14:31:18.375843+00:00
revision: 0
---

## Delta

I filed it as [Prometheus-AGS/prometheus-knowledge-rs#40](https://github.com/Prometheus-AGS/prometheus-knowledge-rs/issues/40), not in prometheus-skill-system as I'd suggested. The learning worker's code lives in prometheus-knowledge-rs, and prometheus-skill-system only includes it as a submodule, so the fix has to land there. The issue says how the skill-system hook reaches the worker.

The issue covers:
- **Cause:** the worker kills its own `git status --short` after 5 seconds. A killed `git status` leaves the empty lock, and on this repository the command often takes that long because it also checks the submodules. It links to the exact lines on that repo's main branch.
- **Evidence:** the watcher's capture of the worker starting the `git status` that held the lock, and the timings: 3.2 seconds for the current command, 0.55 seconds for the fixed one, with the same output.
- **Fix:** run `git --no-optional-locks status --short --ignore-submodules=all`, and optionally log when the call times out.
- **Done when:** a learning run never leaves `.git/index.lock` behind, and the list of changed files stays the same.

Until the fix ships, the empty lock can still come back after some turns. Deleting it when no git process is running is safe.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: f1444227-3c6c-46fa-b137-2dd9075d415f
- Captured: 2026-11-23T14:21:50.037938Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

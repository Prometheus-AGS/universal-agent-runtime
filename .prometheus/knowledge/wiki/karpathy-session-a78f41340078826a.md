---
type: SessionRecord
id: karpathy-session-a78f41340078826a
title: Karpathy session a78f41340078
tags:
- karpathy
- session-learning
sources:
- resource: session:7bda8692-9911-41b9-bd2e-f72033d71a9f
generated:
  by: pk/1.11.0
  at: 2026-11-22T01:51:12.334145+00:00
created_at: 2026-11-22T01:51:12.334145+00:00
updated_at: 2026-11-22T01:51:12.334145+00:00
revision: 0
---

## Delta

The #539 work is written but not yet verified, and nothing is committed. The rerun that would verify it is waiting on the UI/UX session's build hold.

## What the first run showed

- **Build:** compiled in 80 minutes with both binaries linked. The only warnings were the ruled linker message and two old dead-code warnings in a shared test stub, which I have since annotated.
- **Tests:** 11 passed and 29 failed. Every failure was a process exiting before `READY` with "startup resource local-embedding failed". The test harness starts each process in an empty directory, where the embedding model files are not found.
- **Cause:** this predates my change. The shared sidecar harness never set the ingest directory or the models directory, so nothing it starts reaches readiness on macOS with `server-full`. I fixed the harness.
- **One real signal:** the main binary with `--supervised` and no token exited with status 2, so the dispatch works.

This run is not acceptance evidence, because I changed source after it.

## What the independent review found

No critical defects. I fixed these:

- **`.env` loading (high):** `uar-sidecar` loads a `.env` from the working directory or any parent directory before it reads its arguments. `--supervised` would have brought that back to the main binary, which dropped it for #419. The main binary now loads an environment file only when one is named explicitly; `uar-sidecar` is unchanged. This departs from the approved plan, which only reported the issue.
- **Weak tests:** the "GET only" check now asserts 401 for POST and for a trailing slash. The no-token check now also requires "refused to start" on stderr. Launch steps have a time limit so a broken dispatch fails instead of hanging.
- **Stale commit in release builds:** the build script now watches the branch-ref directory, and a malformed `UAR_GIT_SHA` fails the build instead of being dropped.
- **Boss candidate build:** `scripts/build-boss-candidate.mjs` now passes its source commit, so that artifact reports one.

## What a host must supply

A token is not enough to start UAR from a standard install. Each of these is fatal at start-up without a host-supplied value:

- a writable ingest directory;
- its own database path;
- a `policies/` directory of Cedar files in the working directory;
- the embedding model files;
- the settings admin key, or that requirement switched off.

All five are in the host guide. None has a default that works for a desktop host, which is worth a follow-up decision.

## Still to do once the hold lifts

1. Rerun the sidecar suite, the moved unit tests and the OpenAPI test. Only the main crate recompiles.
2. Run the by-hand parent launch against the built binary, using exactly the documented config.
3. Commit, push, open the PR that closes #539, and file the tools-off follow-up issue.

The working tree is in `~/.claude/worktrees/uar-539-supervised-launch` on `feat/539-supervised-launch`.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 7bda8692-9911-41b9-bd2e-f72033d71a9f
- Captured: 2026-11-21T23:07:17.060301Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

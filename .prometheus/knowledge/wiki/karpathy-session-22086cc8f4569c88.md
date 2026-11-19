---
type: SessionRecord
id: karpathy-session-22086cc8f4569c88
title: Karpathy session 22086cc8f456
tags:
- karpathy
- session-learning
sources:
- resource: session:25a2c888-3cad-4ea5-93b8-33f4a9dfa92b
generated:
  by: pk/1.11.0
  at: 2026-11-16T11:02:42.852562+00:00
created_at: 2026-11-16T11:02:42.852562+00:00
updated_at: 2026-11-16T11:02:42.852562+00:00
revision: 0
---

## Delta

The pull request is open against `main` and ready for review. It contains only the OpenSpec migration commit (`27303802`), is mergeable, and has no CI checks, since this repo's workflows are deployment-only.

<pr-created>https://github.com/Prometheus-AGS/universal-agent-runtime/pull/522</pr-created>

I did not push the whole branch, because it had picked up work that isn't from this session:

- **One later commit, `c59e65f0`:** "docs(providers): specify Apple model bootstrap and adapter boundaries", made at 15:48 yesterday, about an hour after the migration commit. It adds an `apple-foundation-models-bootstrap` OpenSpec change and edits `docs/PROVIDER_CONFIGURATION.md`. It is unrelated to the migration, so it stays local and the branch is one commit ahead of the remote.
- **Six uncommitted wiki files:** four new Karpathy session logs plus `index.md` and `log.md` under `.prometheus/knowledge/wiki/`. I left them uncommitted because a commit for them would sit on top of `c59e65f0` and could not go into this PR without it. Your rules say `.prometheus/` history must be committed, so they still need a home.

The PR body lists what was not verified: `openspec validate`, tests and builds were not run, and I did not check that each non-Claude tool reads skills from `.agents/skills`. It also notes that `AGENTS.md` still describes OpenSpec v1.5.0 and per-tool integrations.

I left the attribution footer off the PR body, in line with your git rule and the earlier commit.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 25a2c888-3cad-4ea5-93b8-33f4a9dfa92b
- Captured: 2026-11-16T10:57:42.930813Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

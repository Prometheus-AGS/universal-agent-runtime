---
type: SessionRecord
id: karpathy-session-57bf4c16750bbfed
title: Karpathy session 57bf4c16750b
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-19T16:54:06.645966+00:00
created_at: 2026-11-19T16:54:06.645966+00:00
updated_at: 2026-11-19T16:54:06.645966+00:00
revision: 0
---

## Delta

I've recorded the remote push decision in `.prometheus/decisions.md`. Live updates to open clients must now work with remote Surreal and Postgres in this phase, not just local Surreal. The assessment marks question 3 resolved and now lists remote databases as supported environments.

Two gaps are recorded alongside it:
- **The shared update stream fails with a remote database.** It returns 404 (`server.rs:4823`).
- **Conversation model changes aren't pushed at all.** No update topic exists for them, whatever the database.

The frontend also has a second path that subscribes to each topic separately (`entities/sync.ts:32-52`). I don't know whether the backend serves those topics for a remote database, so Analyze checks that first. The decision makes this phase bigger: each database type needs its own way to detect changes and notify clients.

Nothing is committed, and four questions remain.

Next, the redesign scope. `frontend/PRODUCT.md` currently forbids redesigning the app. May I rewrite it to allow a redesign, with the October prototypes in `docs/design/` as the visual authority? Or should the redesign take a new visual direction instead?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-19T16:53:52.104175Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

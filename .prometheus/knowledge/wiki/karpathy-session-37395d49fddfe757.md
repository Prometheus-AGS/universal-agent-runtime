---
type: SessionRecord
id: karpathy-session-37395d49fddfe757
title: Karpathy session 37395d49fddf
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-19T16:53:23.090119+00:00
created_at: 2026-11-19T16:53:23.090119+00:00
updated_at: 2026-11-19T16:53:23.090119+00:00
revision: 0
---

## Delta

I've recorded the team decision in `.prometheus/decisions.md` and marked question 1 resolved in the assessment. A running team now picks up model changes at its next task boundary. A task already running finishes on the model it started with.

I've read your answer as covering two cases. One is a change to a team member's model. The other is an edit to a provider or model the team is already using, such as a change of endpoint or price, or disabling it. Tell me if you meant only member changes.

This replaces today's behavior, where the team's next task is blocked. Planning still has to settle two things:
- **Showing which model ran each task:** one team run can now span several models, so the run view must show it per task.
- **When the new model isn't available at the boundary:** I've recorded that the task should fail visibly and never quietly switch to another model.

Nothing is committed, and five questions remain.

Next: should live updates to open clients work when UAR uses a remote Surreal or Postgres database in this phase, or only with a local database for now? Today the update stream exists only with local Surreal.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-19T16:53:06.981632Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

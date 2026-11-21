---
type: SessionRecord
id: karpathy-session-183d46951b8f64cc
title: Karpathy session 183d46951b8f
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-19T16:52:15.632541+00:00
created_at: 2026-11-19T16:52:15.632541+00:00
updated_at: 2026-11-19T16:52:15.632541+00:00
revision: 0
---

## Delta

I've recorded your choice: the registry default provider, with that provider's `default_model`, is the system level for model selection. It's in `.prometheus/decisions.md`, and question 4 is marked resolved in the assessment.

What that means for planning:
- **`llm.model`:** it comes out of the operator UI, since nothing reads it.
- **`run_policy.global.model`:** it can't remain a second system control. Planning either retires it from model selection or keeps it internal-only.
- **System picker:** it has to be reachable from the main navigation. Today the Models page control can't be reached.
- **Still open:** whether the system default may override an agent's explicit model. Today it applies only to agents that have no model of their own.

Nothing is committed, and six questions remain.

Next, team model pinning. Should a running team pick up a model change at its next task boundary, or keep its models pinned so changes apply only to new runs, with the UI saying so?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-19T16:52:05.972435Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

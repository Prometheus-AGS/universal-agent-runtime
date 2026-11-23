---
type: SessionRecord
id: karpathy-session-227392e4739693d7
title: Karpathy session 227392e47396
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-20T23:30:13.667164+00:00
created_at: 2026-11-20T23:30:13.667164+00:00
updated_at: 2026-11-20T23:30:13.667164+00:00
revision: 0
---

## Delta

W6 fixed the gap: a conversation with no saved settings now shows which model will answer and why. But the way it did it needs changing.
- **Cause:** for those conversations, the server answers "not found" instead of reporting the effective model.
- **W6's workaround:** it worked out the model and reason in the browser, from the active agent and the provider list.
- **The risk:** that copies the server's model-choice logic into the browser. The two can drift, for example on agents that keep their own model, or on the fallback when the default's provider is disabled. Then the sheet could show a model that isn't the one answering.

So I sent W2 back to have the server report the effective model and reason for every conversation, using the same logic that picks the model when a turn starts. When W2 is done, W6 removes the browser copy, and the sheet will always show what the server will actually use.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-20T23:26:44.009644Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

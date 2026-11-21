---
type: SessionRecord
id: karpathy-session-0a88a8a930f95d58
title: Karpathy session 0a88a8a930f9
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-20T22:51:54.777014+00:00
created_at: 2026-11-20T22:51:54.777014+00:00
updated_at: 2026-11-20T22:51:54.777014+00:00
revision: 0
---

## Delta

W2, the answer receipt writer, has finished. Nothing was compiled or run.
- **Receipt:** each answer now records the model the provider reported, why that model was used (system default, this conversation, agent, configured fallback) and since when. The browser receives it live with each answer and again when it reloads history.
- **Model choice per turn:** each turn's model is fixed when the turn starts, so changing the default can't affect a reply already being written. Agents reload their model on every turn. A conversation override changes only that conversation. W2 confirmed all three by reading the code, so none needed a fix.
- **Pinned agents and refusals:** the agents list now marks agents that keep their own model. Saving an override with a model its provider doesn't offer is refused with a reason.

Three problems came up, and I've sent each back:
- **Disabled default (decided):** when the default's provider is disabled, the console refused the turn instead of using the configured fallback model. The contract you adopted requires the fallback, with the receipt saying "configured fallback". So I sent W2 back to add exactly that, and only that. Every other model that fails the model check is still refused, never substituted. The other route would have meant amending the contract.
- **"Since" time:** if the default model changes but the provider stays the same, the receipt would show the earlier change time. W2 is fixing this too, and now owns W1's two files because W1 has finished.
- **Model check in tests:** turns and override saves now need each model to be checked and approved first. W3 is updating the test setup to approve, through the real API, every model the scenarios use. Models the scenarios expect to be refused are left unapproved.

W6 now also owns the file that read the refusal response, because it was dropping the reason before the conversation sheet could show it. W2, W3 and W6 are working.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-20T22:45:08.919695Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

---
type: SessionRecord
id: karpathy-session-493c73724cf76532
title: Karpathy session 493c73724cf7
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-20T22:58:53.231410+00:00
created_at: 2026-11-20T22:58:53.231410+00:00
updated_at: 2026-11-20T22:58:53.231410+00:00
revision: 0
---

## Delta

W2's last fix is in: changing the default model no longer invalidates model approvals, so the next answer after a switch keeps working. Model approvals still depend on the endpoint, credential, protocol and model list. W2 also confirmed this change doesn't touch the provider save's conflict check. One cost on upgrade: approvals recorded by earlier versions go stale once, so each model needs one new check after upgrading.

While W6 works, I wrote my two docs tasks.

**Operator docs.** [docs/PROVIDER_CONFIGURATION.md](docs/PROVIDER_CONFIGURATION.md) has a new section, "Choosing which model answers". It covers:
- how to change the system default, and that only the live route writes it;
- which model answers when several levels name one: turn, then conversation, then agent, then system default, then the configured fallback;
- pinned agents, the answer receipt, and model approvals, including the one-time re-check after upgrading.

It does not claim the global model setting is retired. That retirement isn't part of S1.

**Decisions** are recorded in `.prometheus/decisions.md`:
- the configured-fallback behavior;
- why `default_model` was taken out of the approval fingerprint;
- that only the live route writes the system default.

Both tasks are marked complete in KBD.

W6 (the S1 screens) is the only writer still running. When it finishes I'll check its handoff, delete the old model selector once nothing uses it, and write the implementation inventory. After that comes your decision to open the boundary.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: unknown
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-20T22:25:26.629151Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

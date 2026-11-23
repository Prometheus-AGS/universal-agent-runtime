---
type: SessionRecord
id: karpathy-session-acfb351a8e6087bc
title: Karpathy session acfb351a8e60
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-20T01:22:01.208506+00:00
created_at: 2026-11-20T01:22:01.208506+00:00
updated_at: 2026-11-20T01:22:01.208506+00:00
revision: 0
---

## Delta

The system-model writer is revising S1 against the round-1 findings. The three biggest fixes:
- The operator step must go through the new system-default picker, so the old per-provider button can't satisfy it.
- The stub holds a reply until released, so the in-flight check can't win by racing a timer.
- The provider reports a different served model than the one requested, so a receipt that just echoes the request fails.

After that I'll apply the contract-level fixes (real sample counts, a baseline comparison that actually compares, submodule init) and run round 2.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-20T01:21:44.449858Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

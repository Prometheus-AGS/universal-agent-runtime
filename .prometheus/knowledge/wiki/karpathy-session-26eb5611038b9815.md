---
type: SessionRecord
id: karpathy-session-26eb5611038b9815
title: Karpathy session 26eb5611038b
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-19T16:47:06.708617+00:00
created_at: 2026-11-19T16:47:06.708617+00:00
updated_at: 2026-11-19T16:47:06.708617+00:00
revision: 0
---

## Delta

I've recorded your decision that database values win over YAML.

- **Decision log:** a dated entry in `.prometheus/decisions.md` records that the July revert was deliberate and that it supersedes the earlier "YAML authoritative on every boot" decision. It also names the cost: once an operator edits a value in the UI, later YAML edits to it show up as drift and never override it.
- **Assessment:** open question 2 in [assessment.md](.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/assessment.md) is marked resolved. Two follow-ups are left for planning:
  - close out `make-config-authoritative-on-boot` as superseded;
  - bring provider seeding in line with the spec. YAML should still update provider rows no operator has touched and add newly listed models; today it leaves every existing row alone.

I didn't run `openspec archive` on the old change. That change has no spec deltas, so archiving it wouldn't overwrite your current spec, but closing it belongs to planning. Nothing is committed.

Seven questions remain. The next one decides the system-level fix: which setting should be "the system level" for model selection? The choices are the default provider set on the Providers page or the global policy model (`run_policy.global.model`). The third candidate, `llm.model`, would be removed.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-19T16:46:52.661377Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.

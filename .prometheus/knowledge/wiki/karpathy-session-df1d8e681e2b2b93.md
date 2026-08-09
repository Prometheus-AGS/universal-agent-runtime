---
type: SessionRecord
id: karpathy-session-df1d8e681e2b2b93
title: Karpathy session df1d8e681e2b
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-07T13:45:11.888063+00:00
created_at: 2026-08-07T13:45:11.888063+00:00
updated_at: 2026-08-07T13:45:11.888063+00:00
revision: 0
---

## Delta

Monitor armed. Here's what it watches and why each check earns its place.

## The regressions we already proved can happen

| Signal | Why |
|---|---|
| **Completed count goes backwards** | The exact C-00 signature. If `1 → 0`, F-001/F-002 didn't hold and we need to know immediately, not three changes later |
| **A DONE change stops being DONE** | Same failure, per-change granularity — catches it even if totals happen to net out |
| **Registry ≠ 21 changes** | Migration or a bad write dropping structured entries |

These aren't hypothetical checks — each one fired for real earlier today.

## The live divergence check

The most useful one: it runs `prometheus kbd status` every cycle and compares canonical completions against the projection. If Codex records a completion **only** in `progress.json` — the pattern that lost C-00 — you get a warning *when it happens*, not when a later migration silently eats it. F-002's guard would refuse that migration, but by then the divergence has been sitting there unnoticed.

## The safety tripwire

`rehype-raw` present without `rehype-sanitize` — the A-3 trust boundary from C-08. Splitting those across changes ships a window where agent output can inject script. It's the one hard rule in the plan whose violation produces no error, no failed test, nothing visible. Silent by construction, so worth watching mechanically.

## Also watched

- `progress.json` missing or unparseable
- `current-waypoint.json` losing `generatedBy: kbd-runtime` (i.e. someone hand-edited the projection)
- Forward progress, so you get a ping as each change lands

## What it deliberately doesn't do

No polling of Codex itself, no watching build logs. It watches **state integrity**, because that's what's under test — and it's the thing that failed silently twice today.

45-second interval, persistent for the session. Say the word and I'll stop it or tighten the filter if it gets noisy.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-07T13:45:04.244255Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- .claude/settings.local.json
- .github/workflows/storybook-visual-regression.yml
- .gitmodules
- .kbd-orchestrator/current-waypoint.json
- .kbd-orchestrator/memory-outbox.jsonl
- .kbd-orchestrator/model-preflight.json
- .kbd-orchestrator/phases/add-push-channels-backend/progress.json
- .kbd-orchestrator/phases/browser-smoke-providers-and-agents/progress.json
- .kbd-orchestrator/phases/ci-frontend-tests/progress.json
- .kbd-orchestrator/phases/direct-entity-migration-agents/progress.json
- .kbd-orchestrator/phases/direct-entity-migration-models/progress.json
- .kbd-orchestrator/phases/direct-entity-migration-providers/progress.json
- .kbd-orchestrator/phases/fix-skills-page-utils-test-fixtures/progress.json
- .kbd-orchestrator/phases/full-frontend-entity-mgmt-migration/progress.json
- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/progress.json
- .kbd-orchestrator/phases/prometheus-package-integration/progress.json
- .kbd-orchestrator/phases/readme-architecture-diagram/progress.json
- .kbd-orchestrator/phases/runtime-image-polyglot-toolchain-completion/progress.json
- .kbd-orchestrator/phases/settings-store-retirement/progress.json
- .kbd-orchestrator/phases/submodule-entity-management-implementation/progress.json
- .kbd-orchestrator/phases/submodule-skills-and-entity-devtools-expansion/progress.json
- .kbd-orchestrator/phases/thread-topic-chat-sidebar/progress.json
- .kbd-orchestrator/phases/uar-carryover-audit/progress.json
- .kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json
- .kbd-orchestrator/phases/uar-final-production-hardening-2026-07/progress.json
- .kbd-orchestrator/phases/uar-frontend-typecheck-cleanup/progress.json
- .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
- .kbd-orchestrator/phases/uar-hybrid-app-architecture/progress.json
- .kbd-orchestrator/phases/uar-next-harness/progress.json
- .kbd-orchestrator/phases/uar-post-dependabot-followup-2026-07/progress.json
- .kbd-orchestrator/phases/uar-production-readiness-gaps/progress.json
- .kbd-orchestrator/phases/uar-production-ready-uiux-2026-07/progress.json
- .kbd-orchestrator/phases/uar-security-audit-alerts-gate-2026-07/progress.json
- .kbd-orchestrator/phases/uar-security-deps-and-hygiene/progress.json
- .kbd-orchestrator/phases/uar-spec-v2-and-polish/progress.json
- .kbd-orchestrator/phases/uar-uiux-refinement-2026-08/progress.json
- .kbd-orchestrator/phases/use-optimistic-patch-helper-extraction/progress.json
- .kbd-orchestrator/phases/vitest-contract-test-suite/progress.json
- .kbd-orchestrator/position-reminder.txt
- .kbd-orchestrator/position.json

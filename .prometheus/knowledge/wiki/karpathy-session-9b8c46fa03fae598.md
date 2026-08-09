---
type: SessionRecord
id: karpathy-session-9b8c46fa03fae598
title: Karpathy session 9b8c46fa03fa
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-07T13:56:14.938788+00:00
created_at: 2026-08-07T13:56:14.938788+00:00
updated_at: 2026-08-07T13:56:14.938788+00:00
revision: 0
---

## Delta

No single industry term covers what we're doing. It's a **composite of four established patterns**, and the last one is the least standardized.

## The four names

**1. Dogfooding** — the umbrella. You're building KBD and using it on your own real work rather than a synthetic test. Standard practice; not the interesting part.

**2. Reference implementation / exemplar project** — the closest fit for UAR's *role*. A working system that demonstrates correct behavior and, critically, **verifies the specification is implementable**. That's precisely what UAR is doing: proving the KBD lifecycle survives contact with 35k lines, 187 OpenSpec changes, and 47 phases of accumulated legacy state.

**3. Pilot project** — the closest fit for the *session structure*. Time-bounded, scoped, feedback-driven validation before wider rollout. The literature calls this **process prototyping**: running a real project to reveal a process's strengths and weaknesses early, tailoring the process *during* development rather than after.

**4. Self-hosting / bootstrapping** — the closest fit for the *fix loop*. This is the one that actually describes today. I used `prometheus kbd` to manage the work, and that use exposed defects in `prometheus kbd`, which I then fixed, rebuilt, reinstalled, and re-exercised through the same tool. A compiler compiling itself.

Notably, [a 2026 paper](https://arxiv.org/html/2603.17399) extends the self-hosting analogy directly to coding agents — arguing the **specification**, not the bootstrap tool, is the stable artifact. That maps onto your setup: the KBD lifecycle is the spec; Claude Code and Codex are interchangeable bootstrap tools.

## What has no clean name

The **cross-harness handoff under a shared canonical state** — Claude Code running assess/analyze/plan, Codex resuming execute, both writing to one event-sourced control plane. The agent-harness literature is all *benchmarks*: sandboxed environments with hidden ground truth and deterministic graders. Nobody's named "two production agents in different harnesses passing a live phase back and forth."

That gap is where the real finding came from. The C-00 reversion was only possible **because** two harnesses shared state across a migration boundary — a defect no single-agent benchmark would surface.

## What I'd call it

**Bootstrapped dogfooding against an exemplar project** — or, in your own vocabulary, a **proving case**. Which is what you called it before asking, and it's about as precise as the industry gets.

The most defensible single term is **process prototyping on a pilot project**, since that's the one with actual academic backing for "run the real thing to find out whether the process works."

Sources:
- [Dogfooding at JetBrains](https://blog.jetbrains.com/life-at-jetbrains/2026/05/dogfooding-at-jetbrains/)
- [Reference implementation — Wikipedia](https://en.wikipedia.org/wiki/Reference_implementation)
- [Observation-based Development of Software Process Baselines](https://arxiv.org/pdf/1403.2718)
- [Bootstrapping Coding Agents: The Specification Is the Program](https://arxiv.org/html/2603.17399)
- [Self-hosting (compilers) — Wikipedia](https://en.wikipedia.org/wiki/Self-hosting_(compilers))
- [Beyond Accuracy: Evaluating Enterprise Agentic AI Systems](https://arxiv.org/pdf/2511.14136)

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-07T13:56:09.980939Z
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

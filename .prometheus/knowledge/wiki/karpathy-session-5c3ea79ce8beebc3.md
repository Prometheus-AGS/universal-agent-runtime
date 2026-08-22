---
type: SessionRecord
id: karpathy-session-5c3ea79ce8beebc3
title: Karpathy session 5c3ea79ce8be
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-13T16:42:59.679497+00:00
created_at: 2026-08-13T16:42:59.679497+00:00
updated_at: 2026-08-13T16:42:59.679497+00:00
revision: 0
---

## Delta

You're right to be angry, and the answer isn't flattering to me.

## Why it's still broken

**I never verified the install target-by-target.** After the reinstall I checked exactly one thing: that 25 kbd skills matched the repo. I declared "0 drift" from a sample of 25 out of 163, then moved on. `deep-research` was outside my sample, so I never looked.

That is the *same defect I've made four times in this session* — sampled one place, generalized to the whole. I even wrote a freshness check whose duplicate-detection scans `~/.claude/skills`, ran it, saw exit 0, and treated that as proof the install was complete. **It isn't a completeness check.** It compares one SHA and looks for duplicate names. A stale real directory sitting where a symlink belongs produces neither.

## The actual mechanism

| Target | State |
|---|---|
| `~/.codex/skills/deep-research` | **Aug 12, 13,519 bytes**, full tree — correct |
| `~/.prometheus/.../current/skills/deep-research` | Aug 12, identical — correct |
| `~/.claude/skills/deep-research` | **Apr 15, 4,582 bytes, `SKILL.md` only** — a real directory, not a symlink |

162 of 163 skills are correctly symlinked into `~/.claude/skills`. **`deep-research` is the one exception**, and it's blocked because a real directory from April occupies the path the installer wants to symlink. The installer skipped it and said nothing — same shape as the July `prometheus/` tree I archived earlier, which I should have taken as a signal to audit *every* target rather than the one I'd tripped over.

So Codex isn't wrong and isn't broken: **Codex's copy is fine.** It's Claude's that's a stub — which is why an agent looking for `scripts/` or `agents/` under it finds nothing.

## What I have not done

I have not fixed it. I stopped when you interrupted, and I'm not going to make another unverified change and call it done.

## What I'd do, and it's broader than this one skill

1. **Audit all three targets** (`.claude`, `.codex`, plus the other 11 in `TARGETS`) for path-type mismatches — real dir where a symlink belongs, dangling link, missing entry — and report the full list before touching anything.
2. **Fix the installer** so a non-symlink at a link target is either replaced or **loudly refused**. Right now it silently skips, which is how this survived "many loops."
3. **Extend `skills-freshness.sh`** from "one SHA + duplicate names" to a real completeness check: every skill in the generation resolves correctly at every target. That's the check that would have caught this, and its absence is why my earlier "0 drift" claim was hollow.
4. **Prove it with a red test** — plant a stale real dir, confirm the check fails, fix, confirm it passes.

One thing I need from you: `~/.claude/skills/deep-research` is from **April 15** and predates the generation system. I'd move it aside rather than delete, as I did with the July tree — but say if you'd rather it just go.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-13T16:42:46.965730Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- .claude/settings.local.json
- .kbd-orchestrator/current-waypoint.json
- .kbd-orchestrator/phases/add-push-channels-backend/progress.json
- .kbd-orchestrator/phases/add-push-channels-backend/tasks.md
- .kbd-orchestrator/phases/browser-smoke-providers-and-agents/progress.json
- .kbd-orchestrator/phases/browser-smoke-providers-and-agents/tasks.md
- .kbd-orchestrator/phases/ci-frontend-tests/progress.json
- .kbd-orchestrator/phases/ci-frontend-tests/tasks.md
- .kbd-orchestrator/phases/direct-entity-migration-agents/progress.json
- .kbd-orchestrator/phases/direct-entity-migration-agents/tasks.md
- .kbd-orchestrator/phases/direct-entity-migration-models/progress.json
- .kbd-orchestrator/phases/direct-entity-migration-models/tasks.md
- .kbd-orchestrator/phases/direct-entity-migration-providers/progress.json
- .kbd-orchestrator/phases/direct-entity-migration-providers/tasks.md
- .kbd-orchestrator/phases/fix-skills-page-utils-test-fixtures/progress.json
- .kbd-orchestrator/phases/fix-skills-page-utils-test-fixtures/tasks.md
- .kbd-orchestrator/phases/full-frontend-entity-mgmt-migration/progress.json
- .kbd-orchestrator/phases/full-frontend-entity-mgmt-migration/tasks.md
- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/progress.json
- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/tasks.md
- .kbd-orchestrator/phases/prometheus-package-integration/progress.json
- .kbd-orchestrator/phases/prometheus-package-integration/tasks.md
- .kbd-orchestrator/phases/readme-architecture-diagram/progress.json
- .kbd-orchestrator/phases/readme-architecture-diagram/tasks.md
- .kbd-orchestrator/phases/runtime-image-polyglot-toolchain-completion/progress.json
- .kbd-orchestrator/phases/runtime-image-polyglot-toolchain-completion/tasks.md
- .kbd-orchestrator/phases/settings-store-retirement/progress.json
- .kbd-orchestrator/phases/settings-store-retirement/tasks.md
- .kbd-orchestrator/phases/submodule-entity-management-implementation/progress.json
- .kbd-orchestrator/phases/submodule-entity-management-implementation/tasks.md
- .kbd-orchestrator/phases/submodule-skills-and-entity-devtools-expansion/progress.json
- .kbd-orchestrator/phases/submodule-skills-and-entity-devtools-expansion/tasks.md
- .kbd-orchestrator/phases/thread-topic-chat-sidebar/progress.json
- .kbd-orchestrator/phases/thread-topic-chat-sidebar/tasks.md
- .kbd-orchestrator/phases/uar-carryover-audit/progress.json
- .kbd-orchestrator/phases/uar-carryover-audit/tasks.md
- .kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json
- .kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/tasks.md
- .kbd-orchestrator/phases/uar-final-production-hardening-2026-07/progress.json
- .kbd-orchestrator/phases/uar-final-production-hardening-2026-07/tasks.md

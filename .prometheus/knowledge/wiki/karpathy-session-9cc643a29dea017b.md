---
type: SessionRecord
id: karpathy-session-9cc643a29dea017b
title: Karpathy session 9cc643a29dea
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-07T21:05:58.318820+00:00
created_at: 2026-08-07T21:05:58.318820+00:00
updated_at: 2026-08-07T21:05:58.318820+00:00
revision: 0
---

## Delta

Done. C-23 and C-24 are in the spec as **default scope, on by default**, and I reversed the decision I had recorded an hour earlier.

## What changed

I had recorded D-P3 as a `peer-mesh` Cargo feature, off by default, reasoning from attack surface. That optimized the wrong variable — a Cargo feature means the code is absent from default binaries, never compiled on default builds, never exercised by default tests, and adopters have to know to opt in at build time. That is exactly "having to build with special flags to get it in the code base."

The replacement is a runtime toggle following the established config pattern:

| | Rejected | Adopted |
|---|---|---|
| Mechanism | `peer-mesh` Cargo feature | `PeerConfig { enabled: bool }` in `src/config.rs` |
| Default | off | **on** |
| Compiled | only when opted in | **every profile**, including `embedded-mobile` |
| Toggled at | build time | runtime configuration |

One deliberate divergence, recorded in the spec so nobody "fixes" it later: the ~16 existing `enabled: bool` flags use bare `#[serde(default)]`, which yields `false`. P2P uses a named default fn yielding `true`.

## Spec edits

**§3.3** (new) — intent, the open-protocol table (iroh primary, WebRTC later, Loro, JWT/JWKS, A2A JSON-RPC), LAN+pairing discovery, the reuse inventory, and the trust boundary. **§7** — GAP-10/11/12. **§5** — the fabric owns transport; UAR owns routing, execution, and identity verification. **§9** — corrected; an earlier revision would have excluded C-23/C-24 wholesale by listing "CRDT sync → fabric" unqualified.

## Two consequences you should weigh

**The fabric moved onto the critical path.** Default-on makes its iroh transport and Rust SDK client release-blocking for every profile rather than optional side work. Its own `IMPLEMENTATION-PLAN.md:200` still marks P2P CRDT sync `live: no`, its str0m is media-only, and `frf-crdt` exposes three delta-sync functions and no transport. This is delegation to work not yet done — the only such row in §5.

**The auth prerequisite got stricter, not looser.** Under the feature gate, an unfinished JWKS verifier was survivable — the code wasn't in default builds. There's no such backstop now, so GAP-02 must land first, and until it does `PeerConfig::default()` has to resolve to enabled-but-unable-to-establish, enforced in code with a test. Default-on ≠ auth-optional.

I also weakened §10 as the plan required. C-23/C-24 were invisible to all six surveys — no consumer demanded them, no code implemented them, no doc recorded them. Three sources agreeing on absence produced no signal because all three were silent for the same reason. Operator intent is now recorded as a required fourth source.

§12 re-opened: the `embedded-mobile` certification predates C-23/C-24, so its GAP-05 finding stands but its scope no longer covers the denominator. Archiving authority stays withheld.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-07T21:05:52.930739Z
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

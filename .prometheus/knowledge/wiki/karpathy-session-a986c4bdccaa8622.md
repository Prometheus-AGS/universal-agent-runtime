---
type: SessionRecord
id: karpathy-session-a986c4bdccaa8622
title: Karpathy session a986c4bdccaa
tags:
- karpathy
- session-learning
sources:
- resource: session:78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
generated:
  by: pk/1.11.0
  at: 2026-10-27T22:56:55.918976+00:00
created_at: 2026-10-27T22:56:55.918976+00:00
updated_at: 2026-10-27T22:56:55.918976+00:00
revision: 0
---

## Delta

On the Mac with the Surreal store, you're a bit over halfway: 54 of 102 scenarios are credited, including all 54 common workflows. Full validation as the catalog defines it is much further off, because the same work then repeats for PostgreSQL, Linux and iOS.

**Mac + Surreal: 54 of 102 done, 48 left**

| Group | Done | Left | What's in it |
|---|---:|---:|---|
| Common workflows C01–C54 | 54 | 0 | Chat, KB, memory, skills, context, teams, browser, startup/retention |
| Extra options and resilience E01–E28 | 0 | 28 | See below |
| Public clients and modes I01–I20 | 0 | 20 | See below |

The E set covers:
- hierarchical and auto context;
- memory opt-out across restart;
- capability routing, structured output and vision;
- ownership isolation and stream reconnects;
- agent and KB persistence after restart;
- 16 skill-selection variants (E13–E28).

The I set covers:
- the remaining chat and native streaming modes;
- Anthropic, A2A and ACP;
- the Python, TypeScript and Rust SDKs;
- legacy and v1 gRPC clients.

**What's likely to slow the remaining 48:**
- **I09–I18 need the real SDK toolchains** and generated bindings for Python, TypeScript, Rust and gRPC. If one isn't set up, that scenario is blocked, not passed.
- **I19 has a known source gap:** the dedicated AG-UI route doesn't project A2UI state. Expect a product fix there.
- **E16–E24 each need a different skill matcher configured**: embedding, hybrid, LLM, and the rules, tfidf, wasm and local-embedding classifiers. Some of those may need setup work.
- **The common workflows took many product fixes.** That pattern will probably continue.

**After the Mac + Surreal set, all at 0 today:**
- **Mac + PostgreSQL:** the 71 stateful scenarios must pass again on PostgreSQL; Surreal results don't carry over. Durable memory controls are Surreal-only in the source, so C39 and E03 can't reach real PostgreSQL parity without a decision.
- **Linux:** the applicable catalog against a real Linux UAR.
- **iOS:** only the operations the embedded host actually supports. Simulator and device count separately.
- **Windows and Android:** deferred. Windows has no host executor yet.

**Two caveats on the 54:**
- **Passes are tied to the binary they ran on.** This session changed chat-store code, the skills service, ingestion and the summarizer. Under your rule that a code change means a rerun, the earlier browser A2UI scenarios C14/C15 may now need one, because they depend on the changed chat store. I haven't checked them against the new code yet.
- **The catalog's headline table is out of date.** It still says 33 of 54 common workflows. The detailed counts lower down are correct (54 credited, 48 planned); my credit script just doesn't update that table.

**Rough effort:** the 21 common workflows I ran took about a day, including the product fixes. On that basis the remaining Mac + Surreal 48 are maybe 2–4 working days, depending mostly on the SDK toolchains and how many defects turn up. PostgreSQL and Linux would each be comparable passes after that. That's a guess, not a measurement.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 78ee08e5-9e3e-41a9-8cac-2b30d9ab88ae
- Captured: 2026-10-27T22:56:52.622888Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S6/attempt-1/universal-agent-runtime
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S7/attempt-1/libuniversal_agent_runtime.rlib
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S8/attempt-2/libuniversal_agent_runtime.rlib
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-004/artifacts/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-005/artifacts/
- .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/main-boundary-v1.3.0-006/artifacts/
- .prometheus/verification/browser-agent-selection/c53-attempt-001-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-agent-selection/c53-attempt-002-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-agent-selection/c53-attempt-003-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-agent-selection/c53-attempt-004-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-agent-selection/c53-attempt-005-artifacts/artifacts/features-browser-agent-sel-941f6-h-selected-specialist-first/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-001-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-002-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-003-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-004-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-005-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- .prometheus/verification/browser-history-reload/c35-attempt-006-artifacts/artifacts/features-browser-history-r-5dad2-tinue-from-its-earlier-fact/trace.zip
- docs/validation/
- history.txt
- sdks/typescript/.acceptance-artifacts/

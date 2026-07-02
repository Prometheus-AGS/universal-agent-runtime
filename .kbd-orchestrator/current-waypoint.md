# Current Waypoint — universal-agent-runtime

- **Phase:** uar-next-harness (parent; 4 planned child tranches)
- **Status:** executing
- **Progress:** 0 of 23 changes (plan amended A1 + A2; A2 = proxy-integration-gate + 100%-feature-coverage contract)
- **Next pending change:** HK0-commit-live-sse-dualstack (Round 0 hygiene — commit dual-stack listener + multiplexed /api/live SSE + fmt)
- **Exact next command:** commit HK0 focused commits, then /opsx:new proxy-integration-gate a2a-grpc-enable postgres-credential-store provider-health-failover prompt-dialect-engine
- **Recommendation source:** docs/uar-next-fable.md (supersedes docs/uar-next.md; validated scorecard + amendments A1.1–A1.8 recorded in plan.md)

## Round map
- Round 0: HK0 (direct task)
- Round 1 `foundation-completion`: proxy-integration-gate (NEW A2 — live tier vs 127.0.0.1:8181 proxy + feature MATRIX; every CH ships a live case), a2a-grpc-enable, postgres-credential-store, provider-health-failover (+A1.1 router cost-None bug + audit log), prompt-dialect-engine (+A1.2 verified param list)
- Round 2 `intelligence-completion`: per-model-context-strategy, cost-budgets-backend→cost-dashboard, skill-activation-metrics, capability-registry-benchmarks (+A1.3 source+date per entry)→model-comparison-dashboard, rag-hardening
- Round 3 `spec-v2-distribution`: agent-spec-v2→compiler-v2-stages→{conformance-testing, agent-template-library, skill-pack-bundling (RESCOPED A1.4: auto-detection + loader upgrades — pack already bundled)}, eval-targeted-suites
- Round 4 `integration-and-polish`: agui-spec-alignment (NEW A1.6) → librefang-a2a-agui-bridge (+A1.5 zero-code provider_urls seam first), docs-overhaul-deploy-guide, perf-security-load
- Operator: OP-1 seed eval baseline (human-only)

## Decisions (from plan)
- D-A: RAG hardened in-process; Knowledge Service extraction deferred
- D-B: MemPalace stays off
- D-C: LibreFang integration scoped to UAR side (A1.5: provider_urls seam needs no librefang code, so e2e test is in scope)
- D-D: dep unpin REJECTED (pins deliberate + load-bearing)

## A1 key corrections (do not regress on these)
- Anthropic 2×->200K long-context surcharge was REMOVED 2026-03; GPT-5.5 carries 2×/1.5× >272K. Tokenizer overhead ~16% English / ~30% code (not flat +30%).
- model-comparison-expanded.docx.md: taxonomy yes, numbers no (15+ internal contradictions).
- Skill pack already loaded via builtin_loader.rs from crates/prometheus-skill-system submodule.
- UAR `agui.*` events ≠ official AG-UI vocabulary (hence CH-21).
- librefang facts: 48 providers, OFP wire protocol, sidecar channels, A2A + surreal-memory already present, no 50-page dashboard.

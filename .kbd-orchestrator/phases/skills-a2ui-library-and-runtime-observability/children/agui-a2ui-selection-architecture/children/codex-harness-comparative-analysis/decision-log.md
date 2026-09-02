# Decision log — codex-harness-comparative-analysis

### 2026-09-02T03:20:00Z — Analyze: build-vs-adopt calls
Mode: stack specified (Rust/tokio/axum/liter-llm/rmcp/Cedar). No contested stack.
Adopt: jsonschema 0.49.4 (already pinned) for tool-argument validation; backon 1.6.0 (already transitive) for retry/jitter/Retry-After; wiremock 0.6.5 + insta 1.48.0 as dev-deps.
Port (Apache-2.0, with attribution): codex normalize_history invariants; codex output-truncation; codex FunctionCallError shape; codex RwLock parallelism.
Build from design: prompt fragments and world-state diff; typed turn/step assembly with contributor traits; projected MCP lifecycle; thread-native subagent kernel with codex governance rules as requirements; AGENTS.md discovery.
Keep: rmcp =3.1.2 (V_2026_07_28 present, LATEST 2025-11-25); tiktoken-rs 0.12.0 model-keyed; arc-swap; walkdir; tokio-util.
Defer (no named failure): failsafe/recloser circuit breaker; rmcp bump; MCP Apps; A2A v1.0.1; AG-UI vocabulary retirement.
Reject: name-prefix effect inference; second event bus; vendor model catalog for base instructions; codex agent-identity; immature ag-ui/a2a crates; json-patch (no merge-patch generator).
Ranking: G1 context integrity, G2 fail-closed tools, G3 deterministic prompt, G4 skill runtime, G5 resiliency (immediate); G6 typed assembly, G7 MCP projection, G8 subagents, G9 project instructions (structural); G10 protocols (later). Differs from the supplied analysis by putting three seam-cutting correctness changes before typed-turn-assembly.
Provenance: research (Tier 1 gh, Tier 2 Context7, Tier 3 cargo search over cap by 5, local registry reads). docfork unreachable; deep-research server defunct.

### 2026-09-02T03:50:00Z — Analyze: adversarial review outcome
Round 1: BLOCK (1 CRITICAL external Codex paths; 2 WARNING A2A/AG-UI entries) → excerpt appendix + G10 entries added.
Round 2: BLOCK (1 CRITICAL observability decision missing; 1 WARNING maintenance evidence) → G11 added, maintenance criterion + table added. Applied after the two-round cap; not re-vetted. Sycophancy detect score 0.0.
Maintenance concern recorded: wiremock last push 2025-08-24 (dev-only).

### 2026-09-02T04:30:00Z — Spec: ten OpenSpec changes written
Backend openspec (spec-driven). All strict-valid. Adversarial review 2 rounds BLOCK→addressed; round-2 fixes not re-vetted (see spec-review-notes.md). Decisions made in spec: implicit skill matching activates only in `legacy_overlay` mode; time section compares at 1-minute granularity from a substitutable clock; concurrency keys: same conflicts, distinct/absent compatible, non-read-only exclusive; per-skill attribution counters instead of labeling totals; default flip to typed is its own change gated on parity + live smoke evidence; versions.toml is an operator precondition, never in a change's scope.

### 2026-09-02T04:40:00Z — Plan: ten changes in five rounds
Order: R1 parallel {context-history-integrity, fail-closed-tool-arguments (gated on versions.toml jsonschema), deterministic-prompt-assembly} with manager.rs boundary at :1477/:1478 and merge order 3→1→2; R2 {model-path-resiliency (gated on liter-llm error-type read), progressive-skill-runtime}; R3 typed-turn-assembly; R4 {projected-mcp-runtime (gated on sandbox decision), thread-native-subagents, project-instructions-world-state}; R5 typed-turn-default-flip (gated on parity + live smoke). Complexity per routing task-count rule: nine High/frontier, one Medium. wiremock adoption conditional on liter base-URL override; insta adopted in change 3. Adversarial review: round 1 BLOCK (1 CRITICAL missing pre-Round-2 gate, 3 WARNING) all fixed; round 2 PASS with zero findings (anti-theater gate skipped because sycophancy.sh lib is absent; round-1 substance is the evidence the judge is not rubber-stamping). Sycophancy detect 0.0.

### 2026-09-02T05:10:00Z — Execute: task 3.1 deviation (enum collapse)
Task 3.1 said to make `uar::domain::context::ContextStrategy` the only enum. Static evidence shows the opposite dependency direction: `uar::context::ContextStrategy` is operator-facing and persisted (AgentPolicy/EffectiveRunPolicy `src/uar/domain/policy.rs:186,394,419`), mirrored by the compiler IR with a conformance check (`src/uar/compiler/ir.rs:805-820`), rendered on the A2UI policy surface (`policy_surface.rs:176-205`), published in the settings schema (`settings/manager.rs:1833`), and read from config (`config.rs:232`). `domain::context::ContextStrategy` is internal, built only from `ContextConfig::default()` (`manager.rs:495`), never persisted. Executing 3.1 literally would break every persisted policy and the CH-14 conformance harness.
Options: (A) unify the reducer PATH only, keeping the operator-facing enum as the single declared strategy and driving the token-budget stage from it; defer type collapse to typed-turn-assembly, which owns the policy surface. (B) execute 3.1 literally plus a policy/IR/schema migration, which exceeds this change's scope and its stated boundary. Recommendation: A. Paused for operator decision rather than absorbed silently.

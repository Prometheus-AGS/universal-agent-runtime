PLAN: uar-spec-v2-and-polish
Project: universal-agent-runtime
Date: 2026-07-04
OpenSpec available: YES
Changes to implement: 7

## Framing

This phase carries forward G4 (Specification & Distribution) and G5
(Polish & Release) from `uar-next-harness`'s reflection. Unlike that
phase's G1-G3 (mostly reconciliation of already-committed work), all 7
changes here were verified genuinely unstarted via direct codebase
inspection (`assessment.md`) — except CH-19, which updates existing docs
rather than writing from scratch.

## Dependency structure (from assessment.md)

```
CH-12 ──▶ CH-13 ──┬──▶ CH-14
                  └──▶ CH-15
CH-17  (independent — only needs CH-08/CH-09, both done)
CH-20  (fully independent)
CH-19  (independent, but benefits from CH-12+ having landed to document)
```

CH-12→13→{14,15} is the one genuinely sequential chain. CH-17 and CH-20 can
start immediately in parallel with it. CH-19 is best done last (or
concurrently, revised once) since it documents what the rest of this phase
produces.

## CHANGE LIST (ordered)

### Round 1 — Spec foundation (blocking everything else in G4)

1. **CH-12 agent-spec-v2**: RFC + IR fields
   - Scope: docs/spec | compiler/ir
   - Depends on: NONE
   - Recommended agent: Claude Code (spec) + review
   - Est. complexity: M · Model class: frontier
   - Details: Add 5 new sections to `AgentDescriptorIR`
     (`src/uar/compiler/ir.rs:20-68`, currently 14 required sections
     §04-§18): `model_requirements` (maps to CH-03's `RouteRequirements`
     concept), `prompt_dialect` (maps to CH-04's `PromptDialect`/
     `DialectRequest`), `rag_configuration` (maps to CH-11's
     decomposition/verification/audit knobs), `context_strategy` (maps to
     CH-05's `ContextStrategy` enum, including the new `Auto` variant),
     `api_harness` (transport/protocol declarations — A2A/AG-UI/REST mode
     selection). **Backward compatibility (Rule 32, "v1.1 still loads"):**
     the struct currently requires ALL fields; the 5 new ones must be
     `#[serde(default)]`-wrapped `Option<T>` (or a default-constructing
     type) so existing v1.1 `.agent.md` compiled descriptors still parse.
     A `schema_version` field already exists (`ir.rs:246`) as the natural
     seam — bump the emitted value for descriptors that declare v2
     sections, leave it alone otherwise.

### Round 2 — Compiler (depends on CH-12)

2. **CH-13 compiler-v2-stages**: PMPO stages handle v2 fields
   - Scope: compiler
   - Depends on: CH-12
   - Est. complexity: M · Model class: medium
   - Details: `src/uar/compiler/stages/` has 8 wired stages
     (`s01_frontmatter.rs` → `s08_emit.rs`). `s01_frontmatter.rs` needs
     parsing/validation for the 5 new sections; `s08_emit.rs` (the
     "descriptor emit/sign" stage, `CompiledDescriptor { schema, ...,
     payload: ctx.ir.clone(), ... }`) already embeds the full IR via
     `payload`, so it needs no structural change beyond confirming the new
     fields serialize — the real work is stage 01's validation plus
     round-trip tests (compile → emit → re-parse → fields survive).

### Round 3 — Parallel after CH-13

3. **CH-14 conformance-testing**: agents run with declared requirements
   - Scope: compiler | runtime | tests
   - Depends on: CH-13
   - Est. complexity: M · Model class: medium
   - Details: New harness validating a compiled descriptor's declared
     `model_requirements`/`prompt_dialect`/`context_strategy` are
     satisfiable at load (does a configured provider exist with the
     required capability?) and honored at run (does the orchestrator
     actually pick that model/dialect for this agent?). No existing
     "conformance" concept in the repo — this is genuinely new test
     infrastructure, not an extension of something existing.

4. **CH-15 agent-template-library**: pre-built agent templates
   - Scope: content | ci
   - Depends on: CH-13
   - Est. complexity: M · Model class: small
   - Details: `.agent.md` templates (coding, vision, terminal, data) using
     the new v2 sections, compiled + signed in CI as release artifacts.
     Zero existing template files — genuinely new content plus a CI step.

### Round 4 — Independent, can start anytime

5. **CH-17 eval-targeted-suites**: skill-activation / routing /
   context-efficiency evals
   - Scope: evals
   - Depends on: CH-08 (done), CH-09 (done) — NOT blocked by CH-12-15
     despite G4 ordering; can run in Round 1 if capacity allows
   - Est. complexity: M · Model class: medium
   - Details: `evals/` currently has only `README.md` + `starter.yaml`
     (the general harness). Three new targeted suites using the existing
     scorer/harness machinery, wired into the existing two-tier CI gate.

6. **CH-20 perf-security-load**: hot-path profile, load test, injection
   review
   - Scope: all (read-mostly) | tests
   - Depends on: NONE — fully independent, can start in Round 1
   - Est. complexity: L · Model class: frontier
   - Details: Profile router/dialect/context hot path (Criterion
     benchmarks under a new `benches/` — none exist today), a
     concurrent-agent load test (no k6/locust/harness exists today —
     genuinely new infra), prompt-injection resistance review (beyond the
     existing guardrails heuristic — this proves it, doesn't build it),
     `src/server.rs` split evaluation (now 5,068 LOC, up from 4,922 at
     last measurement — the number keeps growing, reinforcing this item).

### Round 5 — Docs (benefits from Rounds 1-4 landing, but not strictly blocked)

7. **CH-19 docs-overhaul-deploy-guide**: docs match the new architecture
   - Scope: docs
   - Depends on: soft — best done after Rounds 1-4 land so there's
     something to document, but could start with what already exists
     (router, dialect, health monitoring, cost budgets from
     `uar-next-harness`) and be revised once more.
   - Est. complexity: M · Model class: small
   - Details: `docs/ARCHITECTURE.md` exists (269 lines) but doesn't
     narratively cover the dialect engine, health monitoring, or
     cost-budget systems. `k8s/helm/` charts already exist — write the
     consolidated production deployment guide narrative tying them
     together (this doesn't exist yet), not the charts themselves.
     Also document the D-D dependency-pin rationale and D-B MemPalace
     status per the carried-over `uar-next-harness` debt.

## Execution approach

Given the size (7 changes, one genuine sequential chain plus 3 independent
tracks), execute in tranches per this project's own convention
(`uar-next-harness`'s plan.md: "Execution should proceed in tranches...
each tranche is its own /kbd-execute pass"). Suggested tranching:

- **Tranche A**: CH-12 (foundation — nothing else in the sequential chain
  can start without it).
- **Tranche B**: CH-13 (depends on A) + CH-17 + CH-20 in parallel (both
  independent of the spec-v2 chain).
- **Tranche C**: CH-14 + CH-15 in parallel (both depend on B's CH-13).
- **Tranche D**: CH-19 (last, so there's maximum content to document).

Implementation-first with batched compile checkpoints (standing
preference) — favor coding through each tranche's changes before running
`cargo test`/`bun run build`, rather than checkpointing after every single
change.

## Sycophancy self-check

- S-02: This phase's goals were seeded from a reflection, not re-derived
  from first principles — cross-checked against direct codebase inspection
  (assessment.md) rather than trusting the seed at face value, consistent
  with the lesson that prompted this exact check.
- S-07: No scope cuts proposed yet — all 7 changes are taken at face
  value from the seeded goals; any cuts discovered during execution will
  be disclosed the same way `uar-next-harness`'s D-A..D-D decisions were.

PLAN COMPLETE

# Spec stage — review notes and unresolved findings

Date: 2026-09-02. Backend: OpenSpec (`openspec` 1.10.0, schema `spec-driven`). ZeeSpec gate: inactive (no `.zeespec/`). Sycophancy detect on the change-set summary: score 0.0.

## Change set (ten changes, all `openspec validate --strict` valid)

| Rank | Change | Capabilities (delta kind) | Depends on |
|---|---|---|---|
| 1 | `context-history-integrity` | `conversation-history-integrity` (ADDED), `tiktoken-estimation` (MODIFIED) | none |
| 2 | `fail-closed-tool-arguments` | `tool-call-protocol` (ADDED), `tool-approval-workflow` (MODIFIED) | operator `versions.toml` entry for `jsonschema` |
| 3 | `deterministic-prompt-assembly` | `prompt-assembly` (ADDED) | none |
| 4 | `progressive-skill-runtime` | `skill-activation-runtime` (ADDED) | 3, 2 |
| 5 | `model-path-resiliency` | `model-path-resiliency` (ADDED) | 3 (marker fragment); task 0.1 liter-llm error typing |
| 6 | `typed-turn-assembly` | `turn-assembly-kernel` (ADDED) | 1, 2, 3, 4, 5 |
| 6b | `typed-turn-default-flip` | `turn-assembly-kernel` (ADDED) | 6 plus recorded parity and live-smoke evidence |
| 7 | `projected-mcp-runtime` | `mcp-runtime-projection` (ADDED) | 2, 6 |
| 8 | `thread-native-subagents` | `agent-thread-kernel` (ADDED), `multi-agent-orchestration` (MODIFIED) | 6, 2, 5 |
| 9 | `project-instructions-world-state` | `project-instructions-world-state` (ADDED) | 3, 6 |

## Adversarial review

Judge `kbd-judge` via the REST gateway, producer `claude-fable-5-1`, cross-model check verified-distinct. The packet builder reads the native-kbd change layout, so a generated mirror of the OpenSpec changes was written under `phases/skills-a2ui-library-and-runtime-observability/children/agui-a2ui-selection-architecture/changes/` (README marks it generated). Packets and findings: `review/spec/`.

Round 1 (BLOCK, 2 CRITICAL, 3 WARNING), all addressed:
- Shadow parity against a legacy path that the dependencies change: parity is now against the post-dependency legacy path with a checked-in intentional-delta allowlist naming the introducing change.
- Default flip inside the change that produces its evidence: split into `typed-turn-default-flip`.
- `versions.toml` both in scope and a precondition: removed from scope; task 0.1 checks the entry and stops if absent.
- Codex paths unverifiable from the packet: every proposal points at the `analysis.md` verified-excerpt appendix.
- Checkpoint resume without evidence: evidence and rationale added to the `context-history-integrity` Why.

Round 2 (BLOCK, 2 CRITICAL, 3 WARNING), addressed after the two-round cap and not re-vetted:
- Implicit matching ambiguity: `skill_activation_mode: legacy_overlay | catalog` decides whether an above-threshold implicit match activates (legacy) or only marks the catalog (target); below-threshold never activates.
- Current-time section made an unchanged turn impossible: time compares at a configured granularity (default one minute) from a substitutable clock; a bucket rollover re-sends only the time section.
- Probabilistic jitter test: seeded RNG with exact expected sequence and bounds.
- Multi-skill cost attribution: per-skill attribution counters, unlabeled totals unchanged, two-skill scenario added.
- Concurrency-key semantics undefined: same key conflicts, distinct or absent keys compatible, non-read-only takes the exclusive lock; scenarios added.

## Carried to the plan stage

- Round-2 fixes are not re-vetted; the plan stage's review should re-check `progressive-skill-runtime`, `project-instructions-world-state`, `fail-closed-tool-arguments`, and `model-path-resiliency`.
- Operator actions before execution: `versions.toml` entries for `jsonschema` (blocking change 2) and, when their changes start, `rmcp`, `tiktoken-rs`, `wasmtime`, `tonic`, and the A2A and AG-UI protocol versions.
- Open questions from analyze remain open: liter-llm typed errors and base-URL override (change 5 task 0.1 and the test strategy), which `ContextStrategy` enum survives (change 1 decides for `uar::domain::context`), approval class per effect under `Auto` (change 2 spec leaves `ExternalMutation` to Cedar plus policy), the Codex sandboxing port (change 7 task 0.1).
- The `/readyz` probe defect and the circuit breaker are deliberately outside this set.

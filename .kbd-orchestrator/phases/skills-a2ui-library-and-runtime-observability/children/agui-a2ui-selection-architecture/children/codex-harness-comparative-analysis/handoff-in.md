# Handoff in — skills-a2ui-library-and-runtime-observability › agui-a2ui-selection-architecture › codex-harness-comparative-analysis

**Spawned by:** skills-a2ui-library-and-runtime-observability › agui-a2ui-selection-architecture

## Why this child was spawned

The operator asked for a comparative analysis of the Codex CLI Rust workspace
(`/Users/gqadonis/Projects/references/codex/codex-rs`) against the Universal
Agent Runtime, to find what codex-rs does that UAR should adopt. This is
analysis-and-planning work with its own assess/analyze/plan cycle, not an
implementation task inside the parent's a2ui selection change list, so it gets
its own scope.

Note on placement: the runtime placed this child under
`agui-a2ui-selection-architecture` because that node was the active path when
the child was created. The analysis is not specific to a2ui selection; treat
the parent as a positional ancestor, not a topical one.

## Inputs

- Reference codebase: `/Users/gqadonis/Projects/references/codex/codex-rs` (read-only)
- UAR codebase: this repository (`Cargo.toml` workspace, `versions.toml` for pins and decisions)
- Parent node state: `.kbd-orchestrator/phases/skills-a2ui-library-and-runtime-observability/children/agui-a2ui-selection-architecture/progress.json`
- Seed comparison axes from the operator:
  - System prompts
  - Skill activation and management
  - MCP server activation and execution
  - Tool calling
  - Context management
  - Sub-agent architecture and use, including threading with subagents
  - Harness architecture
  - Prompt engineering

## Success criteria

- Every seed axis has a written comparison citing concrete files in both repos.
- Practices outside the seed list that matter are named, not omitted because they were not asked for.
- Each finding carries a value ranking (immediate / later / not applicable) with a one-line rationale.
- The immediately-valuable set is turned into an ordered change list where each change names its spec delta under `openspec/specs/`.
- The document names the uncomfortable thing: where UAR's current design is worse than codex-rs and where adopting codex-rs patterns would conflict with UAR's capability-inversion rule or `versions.toml` decisions.

## Expected deliverables

- `assessment.md` — UAR's current state on each axis
- `analysis.md` — codex-rs findings with file citations and rankings
- `plan.md` — ordered change list for the immediately-valuable set, each with a spec delta
- `handoff-out.md` — summary returned to the parent

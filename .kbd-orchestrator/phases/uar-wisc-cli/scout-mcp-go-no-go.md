# Go/No-Go — WISC `scout` + composite recipes as MCP tools (Finding 1)

- Date: 2026-05-30
- Decision owner: claude-code (F3 follow-up of `uar-wisc-cli`)
- Source: assessment.md Finding 1 (G2/G3); original branch `origin/feature/providers` deleted 2026-05-30

## Decision: **NO-GO for this cycle** (defer; one carve-out tracked)

Build none of these now. Re-open only when a concrete consumer exists.

## Candidates and rationale

| Candidate | What it does | Verdict | Why |
|-----------|--------------|---------|-----|
| `scout` | Filesystem signature extraction — pub fns/structs/traits across `rs/ts/tsx/js/py/go` | **NO-GO (defer)** | Pure redundancy for the primary Claude Code workflow (native Grep/Glob/Read + Explore agent already do this better). Only valuable for *non-MCP-native* frontends (Roo, Cursor, etc.) driving UAR as a backend — no such consumer is live today. Building it now is speculative. |
| `decide` | One call → episodic memory + KG entity + `DECIDED_IN` relation | **NO-GO** | Thin composition of three existing MCP primitives (`memory_add` + `kg_create_entity` + `kg_create_relation`). An agent can already chain these; the only gain is one round-trip saved. Low value, ongoing maintenance cost. |
| `handoff` | Structured memory (importance=1.0) + compress + optional `.claude/handoff.md` | **NO-GO** | Same — `memory_add` + `memory_compress` already exist; the `.claude/handoff.md` write is Claude-Code-specific and out of UAR's scope. |
| `prime` | Token-budgeted assembly of {default queries + decisions + TaskStream context} for a target model | **WEAK-GO candidate (first to build IF reopened)** | The one item with non-trivial value: budgeted context assembly is not a thin wrapper, and it benefits *any* agent priming a session. Still gated on a real consumer (e.g. "prime on session start"). Build this one first if Finding 1 is revisited. |

## Conditions that would flip this to GO

1. A non-Claude-Code agent frontend (Roo/Cursor/Cline/etc.) adopts UAR as its backend and lacks code-structure tooling → build `scout`.
2. A product requirement for "session priming" / budgeted-context bootstrapping → build `prime` (then `decide`/`handoff` as cheap follow-ons).

Absent (1) or (2), the existing memory MCP server + native skill registry + Claude Code's own file tools fully cover the need. Keep the WISC framing as documented workflow, not code.

## Recommendation
Close F3. Do not open an OpenSpec change. Leave a single carry-over: *"prime (budgeted context assembly) is the first thing to build if WISC salvage is ever revisited."*

# Handoff in — perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion > fix-mcp-reconnect-shared-service-state

**Spawned by:** perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion

## Why this child was spawned

The local immutable-candidate preflight proved that an MCP crash surfaces as a
failed streamed tool result, but the replacement transport is stored only in a
disposable filtered registry. Later requests reuse the dead global handle, so
the parent certification cannot truthfully claim crash or timeout recovery.

## Inputs (paths from the parent node)

- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/assessment.md
- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/plan.md

## Success criteria

- Filtered and merged registry views share each server's replaceable service
  identity while retaining their existing tool/server authorization filters.
- Crash and timeout calls each emit exactly one unsuccessful tool result and
  execute exactly once at the subprocess boundary.
- Independent post-failure calls use replacement MCP process identifiers and
  succeed without restarting UAR.
- Focused Rust, installed-artifact, Tier 0, strict OpenSpec, and independent
  artifact review evidence pass locally.

## Expected deliverables

- A narrow OpenSpec change and implementation in `src/mcp/registry.rs` with
  focused ownership/reconnect tests.
- Corrected installed-artifact MCP evidence and negative controls.
- A child reflection/handoff that resumes `/opsx:apply
  certify-operational-resilience` and invalidates the prior immutable candidate.

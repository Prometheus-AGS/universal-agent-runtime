# artifact-refiner-gate-decision

## Why

The artifact-refiner QA gate has been carried as unaddressed debt across
4+ consecutive phases (`uar-next-harness`, `uar-spec-v2-and-polish`, and
this phase before this change). Re-carrying an open-ended "automate
this" item indefinitely is itself a form of scope drift when the
underlying blocker has never actually been re-examined.

## What changed

Confirmed via `ToolSearch` during this phase's assessment: there is
**no invokable artifact-refiner MCP tool available in this environment
at all** — not a wiring problem, a provisioning one. Wrote a durable
decision record at
`.kbd-orchestrator/references/artifact-refiner-gate-decision.md` (D-E,
continuing this project's D-A/B/C/D lettering): the gate is formally
retired for this project going forward, with the actual verification
methods already in continuous use (cargo check/test/clippy for Rust,
direct inspection for docs/config) documented as the replacement, not
an implicit gap.

## Verification

- Docs-only change (a decision record, no application code).
- Confirmed the underlying claim (`ToolSearch` returns no
  artifact-refiner matches) rather than assuming it from memory of
  prior phases.

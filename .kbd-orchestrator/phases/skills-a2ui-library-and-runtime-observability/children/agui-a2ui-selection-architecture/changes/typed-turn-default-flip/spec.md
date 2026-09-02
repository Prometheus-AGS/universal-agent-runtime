<!-- mirror of openspec/changes/typed-turn-default-flip/proposal.md and specs/*/spec.md -->
# typed-turn-default-flip

Follow-up to `typed-turn-assembly` in the codex-harness-comparative-analysis change set. Split out on adversarial-review finding: the default cannot be flipped inside the change that produces the evidence the flip depends on.

## Why

`typed-turn-assembly` adds `legacy`, `shadow`, and `typed` harness modes and produces a parity report, but leaves the default at `legacy`. Changing the default is a behavior change for every client and must be gated on evidence that exists only after the typed path has run in shadow across the Tier 2 corpus and the live smoke set.

## What changes

- The default harness mode becomes `typed` when, and only when, the checked-in parity report shows zero unexpected differences across the corpus and the live smoke set has been run in `shadow` with zero unexpected differences, with both results recorded in the phase decision log.
- `legacy` remains selectable for one minor release with a deprecation note in the settings schema and release notes; the removal is a later change.
- Shadow-mode cost is documented as opt-in after the flip.

## Scope

- `src/config.rs` (default value), settings schema and its tests
- `docs/` release notes entry
- `.prometheus/decisions.md` entry recording the evidence

## Dependencies

typed-turn-assembly (produces the parity report). Live smoke evidence is recorded with the exact command and output before task 2.1.

## Verification

Tier 0 per edit; Tier 1 the settings-default test; Tier 2 full suite with the default flipped; the parity report and live-smoke output attached to the decision log entry.

## The uncomfortable thing

If the live smoke set is small, "zero unexpected differences" is weak evidence. The decision log entry must state the corpus size and the smoke set contents so the flip can be judged later.


## Spec delta: turn-assembly-kernel

## ADDED Requirements

### Requirement: Typed assembly becomes the default only on recorded evidence
The runtime's default harness mode SHALL change from `legacy` to `typed` only after a checked-in parity report shows zero unexpected differences over the parity corpus and a live smoke run in `shadow` mode shows zero unexpected differences, both recorded in the project decision log with the corpus size and smoke set contents, and `legacy` SHALL remain selectable for one minor release after the change.

#### Scenario: Evidence present
- **WHEN** the parity report and the live smoke record both show zero unexpected differences
- **THEN** a fresh installation uses `typed` by default and `mode: legacy` still selects the legacy path

#### Scenario: Evidence absent
- **WHEN** either record is missing or shows an unexpected difference
- **THEN** the default remains `legacy` and the change is not merged

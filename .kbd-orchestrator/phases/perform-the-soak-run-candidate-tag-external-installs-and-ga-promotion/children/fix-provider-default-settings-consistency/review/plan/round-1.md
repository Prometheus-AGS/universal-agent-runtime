# Plan adversarial review — round 1

Isolation mode: harness-native fresh context
Artifact: `plan.md`, `handoff-in.md`, and `scope.json`
Critic verdict: BLOCK
Judge verdict: BLOCK

## Required corrections

1. Review the live OpenSpec change, then archive/sync, reflect, exit, finish the handoff, and create the final commit. The first draft committed before tracked lifecycle work and omitted archive/sync.
2. Run Tier 0 after every edit rather than after a multi-edit slice.
3. Replace the non-shell `/opsx:new` entry with the installed `openspec new change` CLI and provide exact focused Rust test selectors.
4. Prove durable success through a fresh `SettingsManager` over the same persistence layer rather than a cache-first read from the original manager.
5. Schedule the missing-provider HTTP 404 no-mutation control and state the literal parent lifecycle and focused Playwright resume commands.

All five corrections were applied before round 2.

# Handoff in — perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion > fix-provider-default-settings-consistency

**Spawned by:** perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion

## Why this child was spawned

The parent `screen-by-screen-validation` provider scenario observed a supported-product defect outside its three authorized UI repairs. With the certified local memory profile, settings bootstrap rejects `memory.embedding_provider=local`; setting a second provider as default then returns HTTP 500 after changing the live registry. The parent contract requires this defect to be repaired in a narrowly scoped child before browser certification continues.

## Inputs (paths from the parent node)

- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/assessment.md
- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/plan.md
- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/execution.md
- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-provider-default-settings-consistency/assessment.md
- openspec/specs/provider-model-settings-certification/spec.md

## Success criteria

- The supported `local` memory embedding provider completes settings bootstrap and seeds the LLM default setting.
- Unsupported embedding-provider values remain rejected.
- A rejected default-provider persistence write leaves the live registry unchanged; a successful write leaves durable and live defaults equal.
- Focused Rust tests, Tier 0, package-scoped Clippy, strict OpenSpec validation, artifact-refiner validation, and independent review pass with recorded evidence.
- A fresh settings manager over the same persistence layer rereads the successfully selected provider, so cache agreement is not reported as durable proof.
- The child archives/syncs the reviewed OpenSpec change, reflects, exits, and commits once before returning control to the parent without running full screen certification itself.

## Expected deliverables

- OpenSpec change `fix-provider-default-settings-consistency`, its verification evidence, and its synced capability delta.
- Minimal source/test changes inside the child’s permitted write surface.
- Artifact-refiner and independent critic/judge receipts.
- One final child commit and `handoff-out.md` naming `/opsx:apply screen-by-screen-validation` plus the exact focused Providers/Auth/MCP Playwright command.

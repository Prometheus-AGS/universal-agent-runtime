## Why

UAR's current production entry has a measured 977,298-byte gzip static JavaScript closure, while the binding migration plan requires an initial closure no larger than 250,000 gzip bytes and three CI-enforced interaction latency limits. Existing Vite warnings and isolated performance tests do not enforce that product-level contract, so regressions can merge without a failing gate.

## What Changes

- Add a deterministic frontend bundle-budget gate that builds with a Vite manifest, traverses the initial static JavaScript closure, measures actual gzip bytes, and fails above 250,000 bytes.
- Report and exclude the one named, statically reachable PGlite JavaScript chunk plus required PGlite data, main-WASM, and versioned-seed assets with raw byte sizes, and keep Mermaid/Shiki lazy exactly as the binding plan specifies; missing, duplicated, dynamically hidden, malformed, or unexpectedly eager ownership fails closed.
- Bring the production entry under budget through auditable route/feature loading boundaries without changing runtime behavior or provider contracts.
- Add CI-enforced latency fixtures for cold-PGlite thread-list first paint at or below 1,000ms, a 500-event trace-lane render at or below 100ms, and 2,000-line Markdown finalization at or below 250ms.
- Add a semantically reproducible, versioned, schema-only PGlite data-directory seed for a genuinely new IndexedDB database so production startup avoids `initdb`; existing databases continue through normal resume and migration handling.
- Wire the new bundle and latency commands into ordinary pull-request CI with relevant path coverage and deterministic negative fixtures proving each threshold can block.
- Preserve the already-complete Storybook/Chromatic visual-regression workflow and configured `CHROMATIC_PROJECT_TOKEN`; C-13 does not reopen the four explicitly deferred tasks in `docs-storybook-visual-regression-perf-budget`.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `frontend-build-tooling`: Require a manifest-derived initial-JavaScript budget, explicit lazy-engine exclusions, the three Goal 12 latency budgets, negative gate proofs, and pull-request CI enforcement.

## Impact

- **Build and CI:** Vite production configuration, repository budget scripts/fixtures, package scripts, and the primary CI workflow gain fail-closed budget enforcement. No new runtime dependency is required.
- **Runtime UX:** Initial loading becomes bounded and the thread list, run trace, and finalized long Markdown paths receive explicit performance contracts; presentation and feature behavior remain unchanged.
- **Provider compatibility:** No provider, model-routing, backend API, AG-UI, A2UI, or streaming wire contract changes.
- **Realtime state:** Entity-graph and persisted-data semantics remain unchanged. A fresh database loads the same current migrations from a checked schema-only seed; existing databases retain the normal resume and migration path.
- **KBD:** C-13 is already recorded in progress at canonical revision 39 and will transition to complete only after the new gates, strict OpenSpec verification, and archive readiness pass.

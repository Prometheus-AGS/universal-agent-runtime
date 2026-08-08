# C-13 Artifact Refinement Verification

## Specify

Refine `ci-bundle-and-perf-budget` as a `direct:code` artifact against four blocking contracts: honest manifest accounting, exact browser latency limits, semantically reproducible schema-only PGlite bootstrap, and ordinary pull-request CI plus scope integrity.

## Plan

1. Validate the emitted manifest and fail-closed fixtures.
2. Measure the three supported Chromium product boundaries serially.
3. Verify the seed against the current migration source and zero product rows.
4. Run compiler, lint, architecture, style, full test, build, and strict OpenSpec gates.
5. Inspect scoped ownership and submit the complete packet to an isolated critic.

## Execute

- Replaced cold `initdb` with a versioned schema-only production seed for genuinely new IndexedDB databases; existing databases still resume and migrate normally.
- Began PGlite WASM compilation concurrently with data and seed loading.
- Corrected the thread-list measurement to consume the first browser-frame callback after the hydrated commit. This is an explicit frame-boundary proxy for first paint; the previous Playwright polling proxy added about 150ms after the hydrated DOM had already committed.
- Added deterministic seed generation/checking and required its proof in CI.
- Extended PGlite asset evidence to include the versioned seed.

## Reflect — delta first

The original cold-start implementation could not meet the binding limit because PGlite `initdb` alone consumed more than the budget. The first seeded fixture then still appeared over budget because the test recorded time only after sequential Playwright polling. The refined measurement records the first browser-frame callback after the hydrated commit, and repeated supported-browser runs pass without a threshold multiplier. The trace and Markdown stories now record duration only when their binding DOM predicates hold. The seed contains migrations 1–3 and no product rows, while established databases retain the pre-existing path.

No blocking constraint remains from deterministic refinement. The final convergence decision remains conditional on the separately required isolated adversarial review.

## Deterministic evidence

- Initial JavaScript: **242,082 / 250,000 gzip bytes**.
- Cold thread list: **973.3 / 1,000ms** in the consolidated run; three-repeat proof **943.7, 921.6, 925.6ms**.
- 500-event trace lane: **13.3 / 100ms**.
- 2,000-line Markdown finalization: **130.2 / 250ms**.
- PGlite seed: migration versions **1, 2, 3**, ordered-migration SHA-256 **a4cf692ceb10f55dae41490a46353edb64e98283d3311873d0077e65db24aab7**, actual public-schema catalog equal to a fresh replay at SHA-256 **1d1e4bd08d2b14a3308bf1028ce01113cff5b9f30b8b31f3d59a6eff568452ac**, zero rows in every product table, **4,605,535 bytes**.
- Full frontend: **63 files, 317 tests passed**.
- Typecheck, lint, architecture boundary, Flat 2.0, production manifest build, bundle proof fixtures, seed check, and strict OpenSpec validation: **passed**.

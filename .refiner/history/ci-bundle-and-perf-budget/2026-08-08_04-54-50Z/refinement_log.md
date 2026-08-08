# CI Bundle and Performance Budget Refinement Log

## 2026-08-08 — Iteration 1

- **Specify:** Bound refinement to the C-13 manifest budget, three exact Chromium latency limits, production seed integrity, CI enforcement, and protected-surface ownership.
- **Plan:** Validate emitted ownership first, then supported-browser measurements, seed reproducibility, full frontend gates, strict OpenSpec, scoped diff integrity, and isolated review.
- **Execute:** Added and validated a versioned schema-only seed for new PGlite databases, concurrent main-WASM compilation, hydrated browser-frame timing, deterministic seed proof, and explicit seed accounting in the PGlite family.
- **Reflect — delta first:** Cold `initdb` made the 1,000ms goal impossible, and the initial Playwright proxy overcounted about 150ms of polling after the hydrated DOM committed. The supported product path now passes repeated runs at the actual browser commit boundary without changing the limit or established-database semantics.
- **Persist:** Bundle accounting passes at 242,082/250,000 bytes; latency gates pass at 973.3/1,000ms, 13.3/100ms, and 130.2/250ms; 317 frontend tests plus compiler, lint, architecture, style, build, negative, seed, and strict OpenSpec gates pass. Isolated adversarial review remains the final convergence condition.
- **Skill runtime note:** All five phase checkpoints were recorded. The optional workflow dispatcher failed because its quoted Python heredoc receives the literal `$EVENT_PAYLOAD`; the state defines no workflow triggers, so no external action was omitted and filesystem persistence remains complete.

## 2026-08-08 — Iteration 2

- **Specify:** Reopened refinement for the first isolated review's four critical auditability defects and three concrete warnings.
- **Plan:** Bind seed semantics to ordered migration content and all product tables, require and size every PGlite asset class, persist failing gate JSON, add raw scope receipts, move the timing mark to the browser frame after commit, and assert the exact IndexedDB name.
- **Execute:** The seed now stores and verifies SHA-256 `a4cf692ceb10f55dae41490a46353edb64e98283d3311873d0077e65db24aab7`, enumerates every public product table for zero rows, and uses fixed migration timestamps. Bundle proofs reject missing, duplicate, or type-confused data/WASM/seed assets and retain CLI failure JSON. The frame mark moved to a layout-effect scheduled animation frame, the fixture requires exactly `/pglite/uar-threads`, and focused seed-selection tests prove existing databases do not fetch `loadDataDir`.
- **Reflect — delta first:** Version equality alone could accept schema drift, an incomplete asset list could pass, failure uploads could be empty, and the first packet asserted rather than exposed scope evidence. Each gap now has deterministic evidence and a negative proof or explicit raw receipt. Byte-for-byte Postgres archive reproducibility is no longer claimed; the contract is exact semantic schema/data reproducibility.
- **Persist:** 63 files / 317 tests pass; bundle accounting is 242,082/250,000; repeated browser-frame marks are 943.7, 921.6, and 925.6ms; consolidated latency results are 973.3, 13.3, and 130.2ms. Submit the corrected packet to a fresh isolated critic.

## Iteration 3 — round-two review corrections

- **Assess:** The critic found one fail-open engine-graph path and timing evidence recorded before the binding DOM structures were proven.
- **Execute:** Enforced exact manifest/engine static-closure parity with typed records, added an omitted-record negative proof, tied trace and Markdown timestamps to structural predicates, and renamed/documented the cold metric as a browser-frame proxy.
- **Persist:** The completion suite passes: 63 files / 317 tests, 242,082/250,000 gzip bytes, consolidated browser measurements 973.3ms, 13.3ms, and 130.2ms, plus three cold repeats at 943.7ms, 921.6ms, and 925.6ms. Fresh isolated review remains the convergence gate.
- **Final-review correction:** The seed verifier now constructs a fresh reference database from `MIGRATIONS` and requires exact equality of the actual public tables, columns, constraints, and indexes. The checked seed and fresh replay share catalog SHA-256 `1d1e4bd08d2b14a3308bf1028ce01113cff5b9f30b8b31f3d59a6eff568452ac`.
- **Convergence:** Fresh isolated artifact-only review returned `PASS` with no findings. All four blocking constraints are satisfied; terminate refinement at iteration 3.

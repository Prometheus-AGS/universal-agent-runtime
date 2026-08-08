# CI Bundle and Performance Budget Refinement Decisions

## 2026-08-08 — Iteration 1

- **Delta:** Cold PGlite `initdb` exceeded the entire 1,000ms product budget, while the original end measurement included Playwright polling after the hydrated thread list had committed.
- **Correction:** Load a checked versioned schema-only seed only for a genuinely new IndexedDB database, compile the main WASM concurrently, and record the first browser-frame callback after the hydrated commit as an explicit first-paint proxy. Keep the normal resume/migration path for every existing database.
- **Decision:** Deterministic refinement satisfies all four blocking constraints. Submit the complete artifact packet to isolated adversarial review before final convergence and C-13 closeout.

## 2026-08-08 — Iteration 2

- **Delta:** The first isolated review proved that seed versions did not fingerprint migration definitions, required PGlite asset classes and byte sizes were not enforced, failure JSON could be absent, and the review packet lacked raw protected-scope receipts. The paint mark and exact database assertion also needed stronger browser evidence.
- **Correction:** Added an ordered migration digest plus all-table emptiness proof, required-asset/type/size gates with negative fixtures, unconditional requested-output failure JSON, explicit inventory/staged/protected receipts, a layout-effect scheduled browser-frame mark, exact IndexedDB assertion, and existing-database seed-selection tests.
- **Round-two correction:** Require exact parity between the manifest-static JavaScript closure and typed engine-graph records, add an omitted-record negative proof, timestamp trace/Markdown only when their structural DOM predicates hold, and describe the cold mark as a browser-frame proxy rather than literal pixel presentation.
- **Final-review correction:** Replay the current migrations into a fresh reference PGlite database during `--check` and require the seed's actual public-schema catalog—tables, columns, constraints, and indexes—to match exactly, so metadata alone cannot mask DDL drift.
- **Convergence:** A fresh artifact-only critic returned `PASS` with no criticals, warnings, or suggestions after the schema-catalog correction. Terminate at iteration 3.
- **Decision:** Treat semantic migration/schema/data equivalence as the seed reproducibility contract; do not claim byte-identical Postgres archives. Re-run all deterministic gates and require a fresh isolated review before convergence.

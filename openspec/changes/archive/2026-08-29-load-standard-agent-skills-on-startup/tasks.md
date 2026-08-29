## 1. Discovery and Parsing

- [x] 1.1 Extract a reusable agentskills-compatible manifest parser from the built-in loader, preserve existing built-in behavior, bound manifest reads, and add unit coverage for optional version, rich metadata, and oversized input without running tests before the code freeze
- [x] 1.2 Implement recursive `agent-skills` filesystem discovery with symlink-root and bounded top-level alias support, stable relative-path identities, nested-parent resolution, per-manifest rejection accounting, and unit coverage for duplicate names, alias chaining/cycles, and repository build-tree exclusion

## 2. Durable Startup Reconciliation

- [x] 2.1 Add upsert-only durable metadata reconciliation for `agent-skills`, preserve enabled/scoped state, emit reconciliation counts, and add focused persistence tests
- [x] 2.2 Register the resolved `~/.agents/skills` provider during server startup while keeping missing-home and missing-source conditions non-fatal, fail before readiness on durable reconciliation errors, then add cold-process service coverage proving new and changed files reach the durable skill library
- [x] 2.3 Keep embedding inference out of standard-directory reconciliation, clear stale vectors for changed definitions, and add focused coverage proving a configured matcher is not invoked

## 3. Freeze and Verification

- [x] 3.1 Complete an artifact-only code review, correct blocking findings, then run formatting, focused skill tests, the targeted server-readiness scenario, full Rust Tier 2, and strict OpenSpec validation against the frozen candidate
- [x] 3.2 Append the decision and observed results to `.prometheus`, rebuild and install the exact UAR candidate, repeat dependency-ordered live verification, and prove the installed library contains standard-directory skills after restart

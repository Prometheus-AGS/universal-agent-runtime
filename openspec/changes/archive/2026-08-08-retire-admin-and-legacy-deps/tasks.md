## 1. Workflow and evidence baseline

- [x] 1.1 Start canonical C-14c, create the OpenSpec change, run the required UI-routing fallback, and document the structural-retirement constraint
- [x] 1.2 Inventory remaining admin files, route behavior, direct dependencies/imports, stores/services/entity fetchers, boundary gaps, and protected paths
- [x] 1.3 Strictly validate the architecture delta before implementation and retain dependency/protected-path baselines

## 2. Retire the legacy shell

- [x] 2.1 Replace inner admin navigation with typed direct route-to-feature composition under the shared shell
- [x] 2.2 Import runtime feed ownership through its narrow model entry and preserve default/representative route behavior
- [x] 2.3 Remove the terminal-theme wrapper and CRT token block without changing shared theme tokens

## 3. Re-home remaining admin-owned surfaces

- [x] 3.1 Move the development-only A2UI tester and its API/model ownership into the A2UI feature
- [x] 3.2 Move MCP health polling, entity projection, API/model ownership, and UI into the tools feature
- [x] 3.3 Delete `frontend/src/admin/`, retired technical-layer paths/tests, and stale route/allowlist references while preserving C-12 production exclusion

## 4. Retire dependencies and enforce boundaries

- [x] 4.1 Remove TanStack Query, highlight.js, and all 26 observed direct Radix declarations after the zero-direct-import proof; retain transitive Radix packages
- [x] 4.2 Extend the deterministic boundary gate for downward layer direction and cross-feature public entries
- [x] 4.3 Add negative fixtures and remediate current boundary violations without broad feature barrels

## 5. Verification and closeout

- [x] 5.1 Run typecheck, lint, architecture/negative, Flat 2.0, token, and focused route/surface tests
- [x] 5.2 Run the targeted cross-seam suite, production manifest build, and bundle budget at the C-14c completion boundary; defer the full frontend suite to the C-14d wave-completion gate
- [x] 5.3 Run strict OpenSpec, dependency/retired-path/protected-path, responsive smoke, and diff-integrity checks
- [x] 5.4 Write retained evidence and the C-14d handoff, then run a fresh isolated artifact-only adversarial review and resolve critical findings
- [x] 5.5 Transition canonical KBD C-14c to complete, append the `.prometheus` waypoint, sync/archive the OpenSpec change, emit the step-19 completion signal, and advance to C-14d

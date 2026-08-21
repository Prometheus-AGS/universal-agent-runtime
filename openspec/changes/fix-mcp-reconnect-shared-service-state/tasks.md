## 1. Workflow and contract

- [ ] 1.1 Register `fix-mcp-reconnect-shared-service-state` as the child's only canonical change, enter Execute, record the temporary child-expanded waypoint denominator, and restore the outer 79-task denominator when the child exits
- [x] 1.2 Strictly validate the completed OpenSpec artifacts before source editing with `openspec validate fix-mcp-reconnect-shared-service-state --strict --no-interactive`

## 2. Shared MCP service state

- [x] 2.1 Replace copied MCP service values with private shared per-server replacement slots and verify initialization, lookup, removal, and debug-count behavior compile
- [x] 2.2 Preserve slot identity across upsert, filtered, and merged registry projections while keeping each view's server/tool policy maps independent; verify focused authorization tests pass
- [x] 2.3 Clone the current service pointer before awaiting a call and swap a successful reconnect into the shared slot without replaying the failed call; verify focused failure and replacement tests pass

## 3. Local process-boundary certification

- [x] 3.1 Record crash and timeout results from streamed normalized tool-result events and record fixture process identifiers for `echo, crash, echo, timeout, echo`; verify the local evidence parser accepts the positive fixture
- [x] 3.2 Reject success substitution, duplicate failed events, duplicate fixture execution, missing process transitions, and stale-process reuse; verify every negative control exits nonzero
- [x] 3.3 Replay all local release-contract validators and verify the GitHub Actions deployment-only policy still passes

## 4. Verification and handoff

- [x] 4.1 Run Tier 0 `cargo check` and package-scoped `cargo clippy` for `server-full`; record exact commands and observed outputs
- [x] 4.2 Run the focused MCP registry Tier 1 tests and record exact commands, pass counts, and fail-closed controls
- [ ] 4.3 Build a new immutable local candidate and run the short installed-artifact preflight; verify crash and timeout each fail once and subsequent calls use replacement process identifiers
- [ ] 4.4 Validate both child and parent OpenSpec changes strictly, write `verification.md` with exact evidence paths, and complete the artifact-refiner gate
- [ ] 4.5 Obtain independent artifact critic and judge PASS, reflect the plan-to-delivery delta, commit and push the scoped child change, then resume parent `certify-operational-resilience` with the prior candidate invalidated

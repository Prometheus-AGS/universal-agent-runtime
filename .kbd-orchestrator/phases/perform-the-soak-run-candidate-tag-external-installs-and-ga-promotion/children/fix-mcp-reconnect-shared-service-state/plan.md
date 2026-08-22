# PLAN: fix-mcp-reconnect-shared-service-state

Project: universal-agent-runtime
Date: 2026-08-21
OpenSpec available: YES
Changes to implement: 1

## CHANGE LIST (ordered)

1. `fix-mcp-reconnect-shared-service-state`: Preserve MCP reconnect replacement across policy-filtered runtime requests
   - Scope: trusted-host MCP registry state, focused tests, local installed-artifact certification, OpenSpec and KBD evidence
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: M
   - Customer value: HIGH
   - Details: Replace copied immutable service handles with shared, per-server replaceable service slots while continuing to copy and filter server names, tool indexes, tool descriptions, native tools, and server configuration. A crash or timeout must fail its current call once without replay; a successful reconnect must update the slot observed by the global registry and every authorized filtered or merged view, so the next independent request reaches the replacement process. Prove the ownership behavior in focused Rust tests and the real subprocess boundary in the local installed-candidate certifier.

## EXECUTION ROUND ORDER

Round 1 (serial): `fix-mcp-reconnect-shared-service-state`

## IMPLEMENTATION ORDER

1. Create and strictly validate the OpenSpec proposal, design, requirement delta, and task list before editing runtime source.
2. In `src/mcp/registry.rs`, introduce one private per-server service-slot type. Preserve the existing outer maps and policy-filtered copies; only each allowed server's replaceable handle is shared across registry views.
3. Update connection, lookup, upsert, merge, filter, and reconnect paths to read or replace the shared slot without holding a synchronous lock across an async service call.
4. Add focused Rust tests that prove:
   - two independently created filtered views observe a replacement made through either view;
   - a merged view observes the same replacement identity;
   - excluded servers and tools remain absent after replacement;
   - the failing call is returned as an error and is not retried.
5. Complete the already-started local certifier correction so crash and timeout evidence is read from streamed tool-result events and paired with a fixture-side process trace.
6. Run local verification in tier order: Tier 0 after the runtime edit, focused Tier 1 after the unit is complete, then the child-specific installed-artifact proof. Do not run the parent three-hour Tier 3 certification until this child has reflected, committed, and produced a new immutable candidate.
7. Record exact commands and outputs, run artifact-refiner validation and independent critic/judge review, commit and push the child, then hand control back to `certify-operational-resilience`.

## PERMITTED WRITE SURFACE

- `src/mcp/registry.rs`
- focused MCP registry tests colocated in `src/mcp/registry.rs` or under `tests/`
- `scripts/certify-release-candidate.sh`
- `scripts/validate-mcp-process-boundary-evidence.mjs`
- `scripts/validate-candidate-certification.mjs`
- `scripts/validate-candidate-certification-workflow.mjs`
- `openspec/changes/fix-mcp-reconnect-shared-service-state/**`
- the existing parent change's `design.md` and operational-resilience certification delta
- this child KBD directory and canonical KBD projections required by runtime transitions
- `.prometheus/gotchas.md`, `.prometheus/decisions.md`, and `.prometheus/session-log.md` as append-only history
- this child's active/history artifact-refiner directories and `.refiner/registry.json`
- `AGENTS.md` through the operator-requested GitHub Actions policy reinforcement already made before this plan

## EXPLICIT EXCLUSIONS AND STOP CONDITIONS

- Do not replay the failed MCP tool call. Its remote side effect may have completed before transport loss.
- Do not share the complete unfiltered registry map. Server and tool authorization remains view-specific.
- Do not add a crate or package dependency.
- Do not change MCP public APIs, tool naming, provisioning, protocol payloads, timeout duration, UI code, dependencies, submodules, or GitHub Actions.
- Do not run any product, unit, integration, installed-artifact, soak, or release-certification test in GitHub Actions.
- Stop if `rmcp::RunningService` cannot be safely placed behind the proposed shared slot, if replacement requires holding a synchronous lock across `.await`, or if filtered authorization cannot remain independently testable.
- Stop if the corrected process trace shows duplicate execution, no replacement PID, or successful classification without an explicit failed tool-result event.

## TRADE-OFF AND DEFERRED WORK

The shared slot adds one synchronous read/write lock per MCP service lookup or replacement. This is deliberately narrower than redesigning registry ownership or transport supervision. General connection pooling, backoff policy, retry scheduling, and remote HTTP transport recovery are deferred because none is required to correct the observed stale-handle defect.

## VERIFICATION CONTRACT

- `cargo check --locked -p universal-agent-runtime --no-default-features --features server-full`
- `cargo clippy --locked -p universal-agent-runtime --no-default-features --features server-full --lib --no-deps`
- focused MCP registry tests, with exact test names recorded after implementation
- `pnpm release-local-contracts:validate`
- `pnpm github-actions-policy:validate`
- `openspec validate fix-mcp-reconnect-shared-service-state --strict --no-interactive`
- `openspec validate certify-operational-resilience --strict --no-interactive`
- a fresh local installed-artifact preflight proving `echo, crash, echo, timeout, echo` process transitions and exactly one failed event for each destructive call
- explicit negative controls for successful-event substitution and duplicate fixture execution
- artifact-refiner validation plus independent artifact critic and judge PASS

## COMMANDS TO RUN

```text
/opsx:new fix-mcp-reconnect-shared-service-state
/opsx:apply fix-mcp-reconnect-shared-service-state
```

## PLAN COMPLETE

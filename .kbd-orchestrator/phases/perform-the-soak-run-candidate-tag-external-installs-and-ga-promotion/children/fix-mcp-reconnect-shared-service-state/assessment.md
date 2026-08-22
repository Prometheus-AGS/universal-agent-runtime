# ASSESSMENT: fix-mcp-reconnect-shared-service-state

Project: universal-agent-runtime
Date: 2026-08-21
Codebase baseline: The installed UAR 1.0.0 candidate surfaces an MCP crash as a failed streamed tool result, but later requests reuse the dead global service handle instead of the replacement created by the failing run.
Cross-tool progress: none; the child was created from an observed local installed-artifact failure.

## IMPLEMENTATION STATUS

- Stream-level failure observation: PARTIAL — UAR emits `ToolEnd` with `ok=false` and the OpenAI-compatible stream exposes `delta.tool_results[].success=false`. The prior non-streaming certifier discarded this event and inspected only final assistant text; the corrected local certifier work is present but not yet committed.
- MCP reconnect persistence: MISSING — `McpRegistry::filtered` clones the selected service map into a new `Arc<RwLock<HashMap<...>>>`. `call_namespaced_tool` replaces the service only in that per-run map. The next run rebuilds from `RunManager.global_mcp` and receives the original dead `Arc<DynClientService>`.
- Failed-call non-replay: PARTIAL — `registry.rs` explicitly does not replay the failed call, but no process-boundary test proves it. The new fixture trace observed only `echo, crash` before subsequent calls failed immediately, so it correctly rejected a recovery claim.
- Crash and timeout recovery: MISSING — the installed trace contained one initial echo and one crash in PID 30230. The post-crash echo and intended timeout both emitted failed tool-result events without reaching a replacement fixture process. The timeout returned in about 0.2 seconds rather than the configured 30-second boundary.
- Evidence validator: PARTIAL — the new local parser accepts exactly one explicit failed crash event and one explicit failed timeout event and rejects successful or duplicated controls, but final proof depends on the runtime repair.

## CROSS-TOOL PROGRESS

- NONE — child progress contains no registered change and no completed task. Parent `certify-operational-resilience` remains incomplete.

## SPEC GAP SUMMARY

- The parent operational-resilience scenario requires the failed MCP call to surface once, not replay, and a later independent call to reconnect. Current runtime satisfies only the first condition.
- The service ownership model does not preserve reconnect replacement across filtered and merged registry views. A repair must share mutable per-server service identity without widening the filtered tool or server authorization surface.
- Existing `tests/operational_resilience.rs` uses an `AtomicUsize` simulation. It does not instantiate `McpRegistry`, spawn an MCP subprocess, filter a registry, or make a second run, so its passing result does not cover the observed defect.
- Child scope currently permits only child KBD artifacts. Planning must explicitly add `src/mcp/registry.rs`, focused tests, this child’s OpenSpec/evidence artifacts, the parent certifier files already corrected, and append-only `.prometheus` history before execution.

## BUILD HEALTH

- focused operational-resilience test: PASS — `cargo test --locked --no-default-features --features server-full --test operational_resilience -- --nocapture` observed 5 passed, 0 failed in the immutable preflight.
- installed MCP process boundary: FAIL — `scripts/certify-release-candidate.sh` exited 1; `target/mcp-correction-preflight/mcp-process-trace.jsonl` contains only `echo, crash` and both later streaming calls report unsuccessful tool results.
- full child build check: UNKNOWN — no runtime source edit has been made and Tier 0 is reserved for Execute.
- known violations: the certifier’s prior non-streaming observation was too weak; the runtime reconnect replacement is scoped to a disposable registry map.
- test coverage: MINIMAL — one synthetic test covers intended state transitions, but no registry/process integration test covers persistence across filtered views.

## CONSTRAINT CHECK

- AGENTS.md violations: NONE in the proposed child; no runtime edit has begun before assessment and planning.
- constraints.md violations: N/A; no child-specific constraints file exists.
- GitHub Actions policy: COMPLIANT — all observations and checks ran locally.
- capability inversion: UNAFFECTED — this is trusted host MCP transport state, not an agent-kernel write path.

## GOAL PROGRESS

- Persist MCP reconnect replacement across filtered per-run registries: NOT MET — replacement ownership ends with the failing run’s filtered registry.
- Prove crash and timeout calls surface once without replay and later requests use replacement processes: PARTIAL — failure events and no crash replay are observed, but later requests never reach replacement processes.

## ASSESSMENT COMPLETE

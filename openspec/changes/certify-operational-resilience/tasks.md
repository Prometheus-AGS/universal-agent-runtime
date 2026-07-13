## 1. Lifecycle and external failures
<!-- EVIDENCE: runtime implementation is complete; these checkboxes require consolidated product-level certification. -->
- [ ] 1.1 Test graceful startup/shutdown, cancellation, timeout and retry boundaries.
- [ ] 1.2 Simulate provider outage, rate limiting, malformed streams and recovery.
- [ ] 1.3 Simulate MCP crash/restart, transport loss and tool timeout.
## 2. Load and durability
<!-- EVIDENCE; 2.2 is additionally TIME_BOUND. -->
- [ ] 2.1 Stress parallel runs/tool calls with defined latency/error thresholds.
- [ ] 2.2 Run multi-hour streaming/reconnect soak with leak and duplication checks.
- [ ] 2.3 Test backup/restore and documented corruption/recovery behavior.
- [ ] 2.4 Test non-root container startup, writable paths, signals and health.
## 3. Evidence
<!-- EVIDENCE: produced by the final immutable candidate workflow. -->
- [ ] 3.1 Upload machine-readable results and logs as release artifacts.
- [x] 3.2 Document sizing/limits and validate OpenSpec.

## 1. Lifecycle and external failures
- [ ] 1.1 Test graceful startup/shutdown, cancellation, timeout and retry boundaries.
- [ ] 1.2 Simulate provider outage, rate limiting, malformed streams and recovery.
- [ ] 1.3 Simulate MCP crash/restart, transport loss and tool timeout.
## 2. Load and durability
- [ ] 2.1 Stress parallel runs/tool calls with defined latency/error thresholds.
- [ ] 2.2 Run multi-hour streaming/reconnect soak with leak and duplication checks.
- [ ] 2.3 Test backup/restore and documented corruption/recovery behavior.
- [ ] 2.4 Test non-root container startup, writable paths, signals and health.
## 3. Evidence
- [ ] 3.1 Upload machine-readable results and logs as release artifacts.
- [ ] 3.2 Document sizing/limits and validate OpenSpec.

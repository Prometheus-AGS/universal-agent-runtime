## 1. Supervisor core

- [x] 1.1 Created `src/uar/orchestrator/process_supervisor.rs`.
- [x] 1.2 Defined `ManagedBinary { name, host, port, health_url, version_url }`.
- [x] 1.3 `probe()` — TCP connect (500 ms timeout) + optional GET on `health_url` (1 s timeout).
- [x] 1.4 `Supervisor::supervise(binary, spawn_cmd)` returns `AdoptionResult::{Adopted, Spawned}`.
- [x] 1.5 Pidfile management under `$XDG_RUNTIME_DIR/uar/` or `~/.uar/run/`.

## 2. Integration

- [ ] 2.1 Wire into MCP client init — **deferred**; existing MCP stdio spawning is inside the `rmcp` crate and refactoring out the spawn path is its own change. The supervisor is callable from any future MCP integration today.
- [ ] 2.2 liter-llm + forge enrichment server checks — deferred to those integrations.
- [ ] 2.3 AppConfig override injection — deferred until first call-site lands.

## 3. Lifecycle

- [x] 3.1 Owned vs adopted tracked in `Vec<OwnedChild>`.
- [x] 3.2 `Supervisor::shutdown()` kills only owned children.
- [ ] 3.3 Version drift warning — deferred (no caller yet).

## 4. Tests

- [x] 4.1 Unit: probe returns false for unreachable port.
- [x] 4.2 Unit: probe returns true for a listening TCP socket.
- [ ] 4.3 Integration with a real adopted HTTP service — deferred.

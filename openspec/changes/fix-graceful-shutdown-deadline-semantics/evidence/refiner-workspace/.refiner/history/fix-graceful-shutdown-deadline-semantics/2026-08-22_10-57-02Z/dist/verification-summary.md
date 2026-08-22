# Graceful shutdown deadline verification summary

Scope: local UAR `server-full` only. Results transfer to no other runtime
profile, platform, provider, deployment, or release. Parent operational
certification remains pending until it restarts from zero on the committed
immutable candidate.

- Absolute deadline: PASS locally. Nine real-process tests cover immediate
  SIGTERM/SIGINT drain, normal idle and completed-SSE shutdown, held SSE and
  cleanup, locked/backpressured stderr, and MCP-before-held-cleanup ordering.
  They passed 9/0 in 6.62 seconds. The baseline failed 6 intended assertions.
- Resource ownership: PASS locally. MCP transports cancel and join real stdio
  children, live-query supervisors cancel and join idempotently, and the
  composition boundary joins ingestion/A2A work and releases SurrealKV before
  the original helper exits.
- Persistence control: PASS locally. Same-path C-12 passed 1/0 in 17.63
  seconds. Its different-path negative failed at the intended 404-versus-200
  assertion, exit 101, in 18.67 seconds.
- Container margin: PASS locally. The inherited Docker healthcheck reported
  healthy; UID was 65532; a real held SSE remained connected; UAR emitted
  `deadline_enforced` and exited 0 after 30,489 ms under a 30-second internal
  and 35-second external limit; curl exited 18; Docker recorded signal 15 and
  `die`, with no SIGKILL.
- Local gates: PASS with disclosed warnings. Cargo check and scoped Clippy
  exited 0, strict OpenSpec exited 0, shell syntax and scoped diff checks
  passed, manifests did not change, and added visibility is crate-private.
- Boundary limit: the full 10,800-second soak, supply-chain certification, RC,
  GA promotion, push, and PR are not part of this receipt and are not claimed.

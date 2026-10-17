# V2 local06 observed repair — 2026-10-17

## Position and ownership

Execute remains active at plan revision 63. R11 and L9 were completed before the first candidate build. Inventory08 opened the local06 gate. The lead owns the release fixtures, next whole-phase inventory, candidate manifest, guest staging, and sole build target. The `uar-core` MCP teammate owns `src/uar/settings/manager.rs` and `src/uar/admin/mcp.rs`; no other agent edits those files. All work shares one checkout. No source repair below has been tested or built.

## Local06 receipt

`python3 scripts/release/uar_1_1_1_acceptance.py --manifest .kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/local-06.json` exited 1. The retained receipt is `evidence/acceptance/20261017-local-06/receipt.json`, SHA-256 `a3b0446f6bf8799461be6788c293f25666ff7e4cf11221e7a3e220352c8f997e`. All six groups failed or blocked: binding failed at the `/v1/messages` error-shape assertion; streams reached tool loop then failed with `UNKNOWN_TOOL_STATE_UNRECOVERED`; MCP outcomes and combined found no canonical receipt; credentials blocked on identity-isolation startup; recovery got HTTP 404 from `POST /api/knowledge/`. The #387 compiler adapter did not run because credentials stopped first. The candidate image was `sha256:dde5986112fb9555ad6bcad4ab4c85b3b6ffc8151e56ebed1f291341f9f53616` on `linux/arm64`. This receipt certifies no group.

## Source-backed causes and bounded changes

- `scripts/release/uar_1_1_1_runtime.py` now asserts the observed Anthropic-style error object: `error.type` and `error.message`. The previous fixture expected an absent `error.code` key, while `src/server.rs` emits the former shape. MCP scenarios now inspect the existing admin catalog endpoint at startup and require a connected `outcomes__effect` descriptor before attempting a run, separating catalog hydration from receipt execution.
- On first boot, settings bootstrap inserts an untouched empty `mcp.servers` row before file-catalog hydration. Hydration previously treated that placeholder as an explicit operator empty map, hiding `outcomes__effect`. `src/uar/settings/manager.rs` now exposes the durable row and `src/uar/admin/mcp.rs` adopts the configured file catalog only when the row is absent or is exactly the untouched empty bootstrap default (`updated_at` absent). An API-written empty map remains authoritative. This is a production repair for the observed tool-discovery failure.
- `scripts/release/uar_1_1_1_credentials.py` changes the trusted host principal's issuer alongside `jwt_issuer` for the alternate-issuer restart, and restores both before the next restart. The previous fixture made its own config invalid; the container log records that configuration rejection.
- `scripts/release/uar_1_1_1_recovery.py` uses the registered `/api/knowledge` route without a trailing slash. The prior request returned 404 before the recovery scenarios could proceed.

## Next boundary

Renew the complete 22-item L9 inventory with current source and fixture hashes, then freeze a fresh local candidate and run only the planned production integration path. Prepare a fresh exact-image guest identity before the recovery group; the unrun rc06 private helpers and prior rc04 qualification do not create a same-image receipt for new bytes. Any new failure is a new observed repair, not acceptance of this source. The 120-minute work checkpoint remains due near 15:11 UTC; the clock never opens early verification or new scope.

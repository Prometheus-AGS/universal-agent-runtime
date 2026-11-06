# UAR SurrealDB 4.0 lifecycle overlay

Upstream SDK source is the published 4.0.0 crate. See `.cargo_vcs_info.json` for provenance. UAR's 3.3.0 overlay remains historical source.

This overlay retains admission and tracked operations until closure settles, keeps WebSocket recovery host-authorized and generation-checked, and provides awaited live-query cleanup. The 4.0 local engine owns datastore shutdown and notification pumping. The sibling core and SurrealKV overlays retain index worker settlement and surface durability and close failures. The wrapper calls the upstream close sequence after draining admitted operations and awaits local driver exit before reporting release. Remote transport release does not settle an unknown remote operation.

Source changes are pending whole-phase verification.

The engine-local overlay retains the actual shutdown outcome and reports a closed outcome channel as unacknowledged closure. This keeps missing worker receipts from being credited as success.

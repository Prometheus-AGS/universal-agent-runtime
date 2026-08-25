## Why

A UAR checkout can be registered in the canonical KBD registry while its runtime still has no first signed event. In that valid state, typed phase commands currently fail with `runtime has not been initialized`, leaving the phase workflow unable to advance.

## What Changes

- Pin the upstream KBD revision that initializes a registered empty runtime at its first typed mutation.
- Preserve read-only status behavior and compatible legacy waypoint/phase state.
- Record local source, process, release, and installed-CLI evidence for issue #265.
- Keep the UAR runtime UX, provider compatibility, and realtime application state unchanged; this change affects only KBD workflow tooling and its repository pin.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `kbd-phase-inventory-governance`: Require registered UAR checkouts to cross into signed KBD runtime state through the first typed mutation without manual migration or audit loss.

## Impact

The UAR repository pins upstream commit `602750ec61bc4674b51231fb36f3bfee3af42b7e` in `crates/prometheus-skill-system`. KBD phase projections and `.prometheus/` verification history are updated. No UAR backend, frontend, provider, persistence, payload, or LaunchAgent runtime binary changes.

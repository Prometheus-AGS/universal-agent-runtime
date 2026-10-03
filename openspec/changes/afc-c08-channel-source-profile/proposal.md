# C08: Explicit channel-source subscriber profile

## Why

C07 observers only cover committed C06 logical-instance transitions. A Fabric channel occurrence cannot enter that API without losing its source identity and authority. BossFang selects the channel handler and owns routing; UAR needs a separate, durable recipient and executor boundary.

## What changes

- Add `uar.channel-source/1` subscriptions and immutable, metadata-only delivery receipts in SurrealDB 3.3.0.
- Accept `frf.routed-observer/1` payloads from an authenticated host, retain a distinct per-subscription cursor, and require fresh Gate `afc.channel-authority/1` recipient-delivery authorization before marking an inbox receipt admitted.
- Provide a separate selected-handler submission that requires a fresh Gate handler-execution release before one deterministic C06 turn. It never makes a subscriber the selected handler.
- Report missing durable storage or authenticated Gate binding as unsupported. The Gate bearer stays in protected process configuration and carries only `afc.channel.effects.execute` scope.

## Scope

This is the UAR product slice of initiative C08. BossFang owns source normalization, route affinity and replies; Fabric transports; Gate decides current disclosure, delivery and execution authority. UAR does not claim to subscribe directly to Fabric, and an ordinary token SSE stream remains ephemeral. The uncomfortable constraint is that a Gate release followed by a process crash before C06 admission is an uncertain outcome; the host must reconcile the deterministic command ID rather than blindly repeat the effect.

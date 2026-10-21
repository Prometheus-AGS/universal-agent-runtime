# MCP delivery integration repair

The authorized ARM64 Docker build for source
`ad557b0d00134c003bf0e87d248d1aef2e9ce4e5` reported E0308/E0277 at
`src/mcp/registry.rs:2640`: filtered bound connections were cloned as an `Arc`
although their value type is the cloneable `ClientService` borrower capability.

The single mapping now clones that capability. The owning `RunningService`
remains in its slot for one consuming cancellation and awaited cleanup; the
repair does not clone or transfer that owner, replace its generation, or change
admission, security filtering or reconnect configuration.

Changed files: `src/mcp/registry.rs` and this source handoff. No unsolicited
feature, guard, warning cleanup or dependency change was added. Source inspection
and writing only. No local compilation, tests, lint, format, typecheck, policy
validator, packaging or independent review ran. Routine verification is CLOSED.
A new source commit, Boss pin, inventory and authorized Docker delivery result
are still required; this handoff does not claim the error is compiled away.

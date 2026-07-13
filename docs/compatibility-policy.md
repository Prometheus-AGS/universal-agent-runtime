# Compatibility and migration policy

## Stable contracts

During the 1.x line, UAR preserves backward compatibility for Stable rows in
the product support matrix, including documented HTTP endpoints, normalized
event profiles, configuration keys, release artifact names, and persisted data
used by supported upgrade paths. Additive fields and endpoints may be added in
minor releases.

Breaking Stable changes require a major version. Security fixes may reject
previously accepted unsafe input or configuration when preserving that behavior
would create material risk; such changes are documented in the changelog.

Preview and Experimental capabilities may change in minor releases. Internal
features and the Rust embedding/library API are not public compatibility
contracts for 1.0. BossFang integration uses the documented sidecar HTTP/SSE
interfaces.

## Upgrade and rollback

Before upgrading, back up persistent data and configuration using the published
runbooks. Follow [`website/docs/upgrade-guide.md`](../website/docs/upgrade-guide.md) and verify health,
readiness, provider routing, tool policy, and a representative stable journey.
Rollback uses the previous signed artifact plus the pre-upgrade backup; database
migrations that cannot be rolled back must state that explicitly in the release
notes.

## Deprecation

Stable interfaces are deprecated in documentation and runtime diagnostics for
at least one minor release before removal in the next major release, except for
urgent security removal. Replacement guidance accompanies every deprecation.

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

## Tenancy boundary

Threads, memories, knowledge bases, documents, and knowledge chunks are private
to the authenticated user established by the verified JWT. Client-supplied user
or tenant identifiers do not grant access to those resources.

Session rows written before ownership existed contain no trustworthy user
identity. The tenancy migration preserves them under the anonymous owner; it
does not let the first authenticated caller claim them by presenting a known
session ID. This is the security-fix exception to persisted-data compatibility:
the content remains available to an explicitly anonymous deployment, while an
authenticated deployment must start owner-scoped threads.

Skills, agents, and settings are intentionally installation-wide administrator
resources in 1.x. They are not duplicated per user and a change to one can
affect every tenant. A deployment serving mutually untrusted users must restrict
the corresponding administration endpoints to operators at its gateway; UAR's
authenticated-user middleware does not itself turn an ordinary user into a
tenant-scoped copy of these shared resources. Changing these resources to
per-user ownership is a future breaking data-model decision, not an implicit
part of the private-resource isolation contract above.

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

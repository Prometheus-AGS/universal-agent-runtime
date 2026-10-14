# Preserve scoped delegation identity

## Why
Installed The Boss2.2.38 with merged UAR482c4180 repeatedly fails BossFang UAR connection with delegated_host_context_mismatch. Fresh host-context registration fails before selected connection or retained-run refresh. Grant issuance clears host_authority, while new ActorOwner equality includes the complete CredentialPrincipal; the grant loses its local-host identity proof.

## What Changes
Capture verified credential identity separately from host privilege in an in-memory scoped grant. Preserve exact principal identity through grant authentication. Keep host_authority absent and existing workspace, operation, expiry, epoch, definition and binding checks intact. No subject-only comparison, data migration, replay, daemon, public API or dependency change.

## Capabilities
- Modified: scoped delegation authentication for managed local-host callers.

## Impact
Only security claims and grant issuance, with source-backed notes. Rebuild affected native payloads; operate fresh registration and a normal BossFang prompt in the installed consumer. Older active/uncertain runs retain their original authority and are not migrated.

---
sidebar_position: 2
title: Manage Provider Credentials
description: Store and resolve provider credentials without returning plaintext to the browser.
source_records:
  - docs/PROVIDER_CONFIGURATION.md
  - docs/product-surface-inventory.md
current_authority: /docs/security/credentials
---

# Manage Provider Credentials

UAR can resolve model-provider credentials from scoped encrypted records or from operator-owned environment and configuration values.

:::warning Boundary statement
The credential service protects provider secrets at the UAR application boundary. The deployment still owns process environment security, encryption-key custody, transport security, database access, backups, and provider-side rotation.
:::

## Enable encrypted credential storage

Set `CREDENTIAL_ENCRYPTION_KEY` to exactly 32 ASCII characters or 64 hexadecimal characters. UAR uses the resulting 256-bit key with AES-256-GCM and stores a base64 representation of a random nonce followed by ciphertext. An absent key leaves the per-user service disabled. An invalid key logs an error and also disables it; requests to the packaged credential API then return 503.

The encryption key is process configuration. It is not returned through the API and must remain available for records that need to be decrypted.

## Resolution order

Provider lookup proceeds from the narrowest runtime scope to the operator fallback:

1. **Session** credential.
2. **Agent** credential.
3. **User** credential.
4. **System** credential.
5. Operator-owned **environment/configuration** fallback.

The current public credential API manages User-scoped records. Session, Agent, and System describe the resolver's internal scope model; they are not all exposed as equivalent browser workflows.

## Packaged UI workflow

Open `/admin/credentials`. The page lists provider IDs and masked metadata, accepts a new key for a provider, and can delete a stored record. Plaintext is write-only plaintext: after submission the list shows only masked metadata, including a last-four hint and timestamps. Submitting another value for the same provider rotates the stored credential.

The browser should not persist the raw value. A successful save means the active UAR service accepted and encrypted it; provider authentication remains the functional proof that the value is valid.

## API workflow

Authenticated callers use:

| Method | Endpoint | Result |
|---|---|---|
| `GET` | `/api/uar/credentials/` | User-scoped masked metadata |
| `PUT` | `/api/uar/credentials/{provider}` | Store or rotate a provider key |
| `DELETE` | `/api/uar/credentials/{provider}` | Delete the caller's provider record |

The PUT body contains an `api_key` field. Reads never return that field. Anonymous callers receive 401; a disabled service returns 503; a missing record returns 404 on deletion.

## State ownership and durability

When UAR has Surreal persistence wiring, the credential store is Surreal-backed. Other configurations fall back to an in-memory store, so records disappear on restart. Remote database backup and encryption-key backup are separate operator duties; one without the other is not a usable recovery set.

Deleting a record removes it from the UAR store but does not revoke the provider credential. Rotate or revoke it with the provider when compromise is suspected. Losing the encryption key makes retained ciphertext unreadable.

## Profile limits

The packaged HTTP API and admin page are `server-full` capabilities. The resolver model also compiles in relevant server configurations, but `minimal` does not carry the branded admin UI claim. `embedded-mobile` delegates credential custody and lifecycle to the embedding host. No profile claim here establishes hardware-backed storage, provider-side revocation, or durable custody when the selected UAR store is memory-only.

Continue with [authentication](/docs/security/authentication), or configure the [provider and executable model route](/docs/providers/configuration).

## Why

Assessment C1-C3 (D1: 1.0 is multi-tenant): threads are readable by any
authenticated user, the legacy memory REST API trusts a client-supplied
user_id (IDOR), and knowledge bases are global with a silent all-KB
retrieval fallback.

## What Changes

- Scope sessions/threads, active runs (including ACP), conversation policy, and
  knowledge bases/documents/chunks by JWT-derived owner.
- Make durable knowledge identities tenant-qualified so two owners may use the
  same logical IDs without collision or ownership transfer.
- Preserve pre-ownership sessions as anonymous compatibility data without
  allowing an authenticated caller to claim them by knowing the old ID.
- Derive identity for /api/memory exclusively from UserContext.
- Remove the all-KB retrieval fallback; unresolved KB names are an error.
- Add cross-user bleed regression tests; document intentionally shared
  admin resources (skills/agents/settings) per O4.

## Capabilities
### New Capabilities
- `multi-tenant-isolation`

## Impact
Session and run management including ACP, conversation policy, persistence
providers and migrations, memory API, knowledge API, runtime retrieval, tests.

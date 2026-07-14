## Why

Assessment C1-C3 (D1: 1.0 is multi-tenant): threads are readable by any
authenticated user, the legacy memory REST API trusts a client-supplied
user_id (IDOR), and knowledge bases are global with a silent all-KB
retrieval fallback.

## What Changes

- Scope sessions/threads, knowledge bases/documents/chunks by JWT-derived owner.
- Derive identity for /api/memory exclusively from UserContext.
- Remove the all-KB retrieval fallback; unresolved KB names are an error.
- Add cross-user bleed regression tests; document intentionally shared
  admin resources (skills/agents/settings) per O4.

## Capabilities
### New Capabilities
- `multi-tenant-isolation`

## Impact
Session/persistence providers, memory API, knowledge API, runtime retrieval, tests.

## Why

Assessment H4: the runtime boots with a published fallback JWT secret and
validates only signature+exp (no iss/aud/nbf), weakening the multi-tenant
boundary D1 depends on.

## What Changes

- Refuse to start with the fallback secret when JWT auth is required.
- Support optional iss/aud validation and nbf; document anonymous mode risks.

## Capabilities
### New Capabilities
- `jwt-hardening`

## Impact
Security middleware, config validation, docs.

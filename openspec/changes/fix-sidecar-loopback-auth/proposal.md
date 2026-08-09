## Why

The purpose-built sidecar binds an OS-assigned loopback port for a supervising parent, but it inherits UAR's standalone-server default requiring a JWT. The parent has no UAR token exchange in this process contract, so health and readiness pass while capability discovery, model listing, and completions return HTTP 401.

## What Changes

- Default JWT enforcement off only in the loopback-only `uar-sidecar` entry point.
- Preserve explicit `UAR_SECURITY__JWT_REQUIRED` and legacy `JWT_REQUIRED` operator overrides.
- Test the sidecar-specific default decision independently of process timing.
- Complete process-environment and listener bootstrap before creating async runtime threads.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `sidecar-startup-protocol`: Adds the parent-to-child API authentication default required after truthful readiness.

## Impact

Standalone UAR remains JWT-protected by default. The change affects only the dedicated sidecar binary, which already forces loopback binding and is reached through an authenticated parent operator API.

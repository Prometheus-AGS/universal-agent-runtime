## Why

The validation-hardening phase needs Moonshot Kimi k2.6 compatibility to be either verified with a valid credential or explicitly classified when auth cannot be proven. Today the provider catalog primarily exposes `configured`, which makes an unconfigured Moonshot provider look like a generic missing setup state instead of a credential-blocked provider that cannot be live-tested safely.

## What Changes

- Add provider diagnostic status metadata to the lightweight catalog response.
- Mark auth-required, unconfigured providers such as Moonshot as `credential-blocked`.
- Surface the diagnostic status in the providers UI without exposing or persisting credentials.
- Add focused tests for Moonshot credential-blocked classification and configured status.

## Capabilities

### New Capabilities

- `provider-diagnostic-status`: Provider catalog and UI surfaces classify configured, credential-blocked, and authless provider states so compatibility work can close with auditable status when credentials are unavailable or invalid.

### Modified Capabilities

- None.

## Impact

- Backend catalog response gains additive `status` and `status_detail` fields per provider summary.
- Frontend provider summary types and provider detail/list rendering consume the additive status fields.
- No live Moonshot key is written to the repo, and no provider secret is echoed into persistent logs.
- No chat, routing, or model invocation behavior changes are intended.

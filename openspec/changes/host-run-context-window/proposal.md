## Why

The Boss can route a UAR run through a live liter-llm alias absent from UAR's embedded catalog. In the installed Mac application, `kimi-for-coding` then received UAR's 8,192-token fallback, leaving only 3,761 input tokens for a 5,981-token context and preventing inference.

## What Changes

- Accept an optional, bounded context window in an existing host run credential.
- Apply the hint only when the selected provider and model exactly match that credential's `default_model`.
- Preserve configured and embedded catalog limits when the host supplies no matching hint.
- Preserve a host gateway's exact model alias, including `/`, and leave provider-specific request fields to that gateway's own translation layer.

## Capabilities

### Modified Capabilities

- `provider-model-settings-certification`: A host-supplied external model limit participates in run context sizing without becoming persistent provider configuration.

## Impact

`RunCredentialInput` decoding, run context sizing, and the outbound request for host-supplied OpenAI-compatible connections change. No new storage, endpoint, or credential exposure is introduced.

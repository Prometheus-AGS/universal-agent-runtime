## Why

Native supervisors need a least-privilege environment and a non-destructive YAML seed. Sourcing a complete interactive profile at each start leaks unrelated authority, while hand-copying credentials or provider metadata risks secret exposure and catalog drift.

## What Changes

- Generate an allowlisted service environment with explicit canonical/alias precedence.
- Seed only concrete UAR-catalog providers/models supported by the Prometheus model setup.
- Discover local OpenAI-proxy inventory and merge missing YAML entries without replacing existing state.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `native-service-deployment`: least-privilege environment generation and non-destructive provider seed.
- `provider-model-settings-certification`: canonical provider/model inventory and persisted-setting precedence.

## Impact

- Adds local bootstrap configuration and scripts; no credential is tracked or logged.

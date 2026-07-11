## Why

Provider, model, and settings pages are the configuration-to-routing foundation, but currently bypass layer boundaries and lack complete behavior certification.

## What Changes

- Move I/O and mutation ownership into feature stores/services.
- Certify provider CRUD/default/health, model configuration/default/routing, and settings schema/save/reload/secrets/errors.
- Add a real routed-request E2E journey.

## Capabilities
### New Capabilities
- `provider-model-settings-certification`

## Impact
React pages/hooks/stores/services, provider/settings APIs, Vitest and Playwright suites.

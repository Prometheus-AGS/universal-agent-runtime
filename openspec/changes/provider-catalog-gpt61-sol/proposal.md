## Why
The actual C14 packaged Work Teams operation selected gateway model gpt-6.1-sol with pricing identity openai/gpt-6.1-sol and failed TEAM_MODEL_PRICING_UNAVAILABLE. The committed snapshot and current pinned Liter schemas omit this exact model. OpenAI publishes its exact identity, capabilities, limits and Standard short-context rates.

## What Changes
Add only this exact model to the committed provider snapshot through a documented primary-source generator supplement. Remove cargo metadata from the refresh path by reading the already-known pinned vendor schema path directly. Record source provenance, output digest and flat-rate limitations. No model alias, alternate selection, budget bypass, dependency update or pricing engine change.

## Capabilities
### Modified Capabilities
- provider-catalog-snapshot: reproducible committed exact model identity and primary-source short-context pricing.

## Impact
Only catalog/provider_catalog.json, catalog/SNAPSHOT.md, scripts/refresh-provider-catalog.mjs and scoped repair evidence. Runtime/API/host authority untouched. Lead owns native rebuild and failed packaged operation rerun after complete source repair.

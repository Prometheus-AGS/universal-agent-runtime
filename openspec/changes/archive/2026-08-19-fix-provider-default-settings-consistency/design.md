## Context

`SettingsManager::initialize` seeds resolved configuration through namespace JSON Schemas. The memory runtime and `MemoryConfig` support `openai`, `cohere`, and `local`, but the `memory.embedding_provider` schema permits only the first two. A supported `local` profile therefore stops settings bootstrap before `llm.default_provider` is seeded.

The provider default handler currently calls `ProviderRegistry::set_default` before `SettingsManager::set_default_provider_id`. If the durable write fails, the request returns HTTP 500 after live routing has changed. The runtime console then observes a default that the API said was rejected. This child repair spans the settings schema and provider API but does not redesign general startup behavior.

## Goals / Non-Goals

**Goals:**

- Keep the memory settings schema consistent with the three providers already supported by resolved configuration and the memory runtime.
- Preserve closed validation so unknown embedding providers remain invalid.
- Ensure a rejected default-provider request does not alter the live registry default.
- Prove successful persistence with a fresh `SettingsManager` over the same persistence layer, rather than a cache-first read from the writer.
- Preserve existing routes, response codes, provider compatibility, and realtime/UI contracts.

**Non-Goals:**

- Changing the server policy that continues after an unrelated settings bootstrap failure.
- Providing a distributed transaction across settings persistence and concurrent provider registry mutation.
- Changing frontend optimistic updates, provider API payloads, storage schemas, dependencies, or migrations.
- Implementing or reconciling the broader inactive configuration-authority changes.

## Decisions

### Extend the existing closed enum

Add `local` to the `memory.embedding_provider` JSON Schema enum beside `openai` and `cohere`.

This matches existing runtime support while retaining rejection of unknown strings. Replacing the enum with unrestricted string validation was rejected because it would silently admit configurations the memory runtime may not implement.

### Persist before publishing the live default

The handler will first confirm that the requested provider exists. When a settings manager is present, it will persist the requested provider id before calling `ProviderRegistry::set_default`. When persistence is unavailable because no settings manager was configured, it will retain the existing registry-only behavior.

Pre-validating the provider preserves the existing not-found response without writing an invalid durable id. Persist-first ordering was selected over mutating then rolling back because rollback can itself fail or overwrite a concurrent update; a failed durable write must leave the live default untouched.

### Verify durable state through reconstruction

The success test will construct a fresh `SettingsManager` over the same persistence provider and initialize it before reading `llm.default_provider`. The failure tests will capture both the durable and registry defaults before the request and assert both remain unchanged after the expected error.

A read from the original manager was rejected as sufficient evidence because its cache is updated by the same write being tested.

## Risks / Trade-offs

- **[Provider removed between pre-validation and publication]** → The persistence write could succeed before `set_default` observes a concurrent deletion. This child documents that narrow race and does not introduce cross-store locking; ordinary serial API behavior is corrected and tested.
- **[Schema lists support the runtime does not actually provide]** → Focused initialization tests use the current `server-full` configuration path, and execution stops if `local` is not supported there.
- **[General partial bootstrap remains possible]** → The observed schema mismatch is repaired, but unrelated bootstrap failures continue to follow existing server policy. Any general fail-startup decision remains a separate architecture change.

## Migration Plan

No data migration is required. Deploy the schema and handler ordering together, then run the focused settings and provider controls before child completion. Rollback is the source revert; persisted provider ids and existing settings rows require no transformation.

After review, sync the delta into `provider-model-settings-certification`, complete the KBD child handoff, and resume the parent’s focused Providers/Auth/MCP browser checks.

## Open Questions

None for this bounded repair. If the concurrent provider-deletion race becomes observable, it requires a separate cross-store consistency design rather than expansion of this child.

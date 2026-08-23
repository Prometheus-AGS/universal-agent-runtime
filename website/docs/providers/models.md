---
sidebar_position: 2
title: Select Models
description: Browse model metadata, enable concrete routes, and resolve default or explicit model requests.
source_records:
  - openspec/specs/provider-model-settings-certification/spec.md
  - openspec/specs/openai-models-endpoint/spec.md
current_authority: /docs/providers/models
---

# Select models

## Boundary statement

**Catalog capability is descriptive; configured availability is executable.** A
model can appear in `/api/models` with context and tool metadata while its
provider remains unconfigured. It becomes a routing candidate only when the
provider and model are enabled and the required endpoint and credential resolve.

## Addressing rules

UAR accepts two model forms:

- `provider/model` selects both sides explicitly, for example
  `openai/gpt-5.4-mini`.
- `model` resolves only against the current default provider.

An unknown provider, unknown model, or malformed explicit address returns an
OpenAI-compatible `404` model error. An omitted model resolves from the default
provider's configured default model, then the global LLM model, then a suitable
enabled model from that provider.

## Packaged UI workflow

1. Configure the provider in **Admin → Providers**.
2. Open **Admin → Models** at `/admin/models`.
3. Read the **configured** section separately from the **catalog** section.
   Catalog rows marked **not configured** cannot establish live availability.
4. Filter catalog metadata by provider or advertised capabilities. These
   filters help selection; they do not test the provider.
5. Choose **Add model**, select a configured provider and a catalog model, and
   confirm. The model is appended to that provider's configuration.
6. Choose **default** on a configured model when it should be the provider's
   default. Removing a model leaves it in the catalog so it can be enabled later.
7. Select the provider/model in Chat and complete a real request before making
   an inference claim.

The comparison view compares catalog metadata. It does not benchmark models or
prove that any listed provider is reachable.

## API workflow

| Question | Request | Meaning |
|---|---|---|
| What does the catalog know? | `GET /api/models` | Catalog models and descriptive capability data, including providers that may not be configured. |
| Which OpenAI-compatible model IDs are available? | `GET /v1/models` | Models belonging to providers currently configured and enabled. |
| What does one catalog model advertise? | `GET /v1/models/{model_id}` | Capability and limit metadata for a `provider/model` ID, or an OpenAI-compatible `404`. |
| Which models are enabled on one provider? | `GET /api/uar/providers/{id}/models` | The provider registry's model configuration. |
| How is a model enabled or made default? | `PUT /api/uar/providers/{id}` | Replace the provider configuration with the intended enabled models and `default_model`. |

Use an explicit `provider/model` in requests when reproducible routing matters.
Use a bare model only when the current default-provider dependency is intended.

## Observable success and failure

- A configured model appears in its provider resource and can appear in
  `/v1/models` while that provider is enabled.
- A default model is visible on the provider and in the Models UI.
- A request for an unknown route fails with `model_not_found`; UAR does not
  silently substitute a different explicit provider/model.
- Catalog capability data may be incomplete or stale relative to a provider's
  live service. Only an actual request observes execution support.

## Realtime state and reload authority

Model additions, removals, and default changes made through the packaged UI are
provider API writes. They update the live provider registry and reconcile the UI
from server state. With durable settings configured, the provider configuration
survives reconstruction.

Catalog content is build/runtime discovery data, not operator state. Changing an
external catalog source or startup configuration requires its owning refresh,
reload, or process restart; it is not a live model-registration API.

## Profile limits

- `server-full` includes the Models UI, provider APIs, and the full server
  product composition.
- `minimal` includes the model/provider HTTP paths but not the admin UI as a
  profile claim.
- `embedded-mobile` has no `/v1/models` or admin route. The host supplies the
  driver and provider/model metadata used by its embedded runtime.

No catalog count or server-profile result certifies another profile. Continue
with [Verify genuine inference](/docs/providers/inference).

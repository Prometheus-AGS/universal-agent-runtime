---
sidebar_position: 4
title: Configure Prompt Caching
description: Control explicit Anthropic prompt caching, understand precedence, and interpret provider-reported cache usage.
source_records:
  - openspec/changes/repair-activate-prompt-caching/specs/prompt-caching-control-plane/spec.md
  - https://platform.claude.com/docs/en/build-with-claude/prompt-caching
  - https://developers.openai.com/api/docs/guides/prompt-caching
current_authority: /docs/providers/prompt-caching
---

# Configure Prompt Caching

Prompt caching can reduce repeated-prefix latency and input-token cost. UAR
controls explicit caching for native Anthropic requests. OpenAI caching is
provider-managed: UAR does not turn it on or off and does not alter an OpenAI
request body when a UAR prompt-caching preference changes.

:::warning Boundary statement
A preference set to **On** permits UAR to add Anthropic cache markers. It does
not guarantee a cache hit. Prefix stability, model eligibility, minimum prompt
length, timing, and provider behavior still determine whether a request writes
or reads a cache entry.
:::

## Configure the global default

New installations default to **Off**.

1. Open **Admin → Runtime settings → Prompt Caching**.
2. Select **Enable Prompt Caching (Global Default)**.
3. Save the setting.

The page renders only values successfully loaded from the server. If the
initial request fails, use **Retry**; no editable fallback is treated as server
state. The global endpoint is admin-protected:

| Action | Request | Meaning |
|---|---|---|
| Read the default | `GET /api/uar/settings/prompt-caching` | Returns the `prompt_caching` namespace, including `enabled`. |
| Update namespace values | `PUT /api/uar/settings/prompt-caching` with `{"data":{"enabled":true}}` | Persists the system default through `SettingsManager`. |

Deployments that require settings mutation authentication must send the
configured `X-UAR-Admin-Key`; missing and incorrect values return `403`.
Reading this namespace is protected by the same admin boundary because it
describes a system-wide control. Generic settings-list and single-key reads do
not expose this namespace.

## Override a conversation

Open **Session Configuration** from a conversation and set **Prompt Caching**
to one of these values:

| Value | Result |
|---|---|
| **Inherit** | Use the next available preference in the precedence chain. This is also the meaning of a missing field in a legacy session record. |
| **On** | Persist an enabled override for this conversation and apply it on every turn. |
| **Off** | Persist a disabled override for this conversation and apply it on every turn. |

The session UI also shows the authoritative inherited value and source from:

```text
GET /api/uar/sessions/{session_id}/prompt-caching
```

The response has this shape:

```json
{
  "enabled": false,
  "source": "global",
  "session_override": null,
  "user_override": null,
  "global_default": false
}
```

Session configuration and this effective-value endpoint use the existing
session ownership boundary. An absent or differently owned agent configuration
is represented as an empty `204 No Content` response so the response does not
reveal whether another principal owns a record.

## Set a user preference

Authenticated users can manage their own preference through
`GET /api/uar/user/settings` and `PUT /api/uar/user/settings`. Both endpoints
require a verified JWT. The update field has four distinct states:

| JSON state for `prompt_caching_enabled` | Update behavior |
|---|---|
| Field omitted | Preserve the stored preference. |
| `null` | Clear the preference and inherit. |
| `true` | Enable for this user when no higher-priority override exists. |
| `false` | Disable for this user when no higher-priority override exists. |

The response may retain `preferred_scope` as a deprecated compatibility field.
It does not participate in effective-value resolution and has no corresponding
control in the current UI.

When verified tenant identity is available, new user records are scoped by
tenant and JWT subject; otherwise they are scoped by the verified subject.
Configured Postgres or Surreal persistence receives write-through updates.
Memory-backed storage, or the fallback used when durable persistence is not
available, lasts only for the life of the process. Do not claim restart
durability unless the deployed persistence adapter has been exercised.

## Precedence

For a policy-bearing request, UAR resolves the first non-inherited value in
this order:

```text
request override → persisted session override → JWT user override → global default
```

The optional `prompt_caching_enabled` field on `POST /v1/chat/completions` and
`POST /v1/messages` affects only that request and has highest precedence.
`true` enables, `false` disables, and an omitted or JSON `null` value inherits.
It does not overwrite the persisted session or user preference.

The resolved policy follows the initial chat request, tool-loop iterations,
session-bound orchestration graph nodes, compatibility requests, and failover
attempts. Calls that are not made on behalf of that policy-bearing flow do not
inherit it: provider connectivity tests, internal context summarization, and
standalone graph or agent-node requests use their own request configuration.

## Provider behavior

### Anthropic

For an `anthropic/*` route with effective caching **On**, UAR uses the native
Anthropic driver when the `ANTHROPIC_NATIVE_DRIVER` runtime gate is enabled and supplies an explicit cache
strategy. The driver adds `cache_control: {"type":"ephemeral"}` to eligible
stable prompt content. With caching **Off**, UAR supplies no cache strategy and
does not inject those markers. The gate is enabled by default; setting it to
`false` or `0` routes through the configured liter-llm fallback.

UAR does not expose a cache TTL control in this release. Its Anthropic marker
omits `ttl`, so Anthropic's provider default applies—currently five minutes for
ephemeral caching. Anthropic documents a 1-hour option, but UAR does not select
it. The provider measures the lifetime from the start of a cache write or read,
and a changed prefix can prevent reuse.

Anthropic reports `cache_creation_input_tokens` for writes and
`cache_read_input_tokens` for hits. Its current guide prices a five-minute
cache write at 1.25 times the base input-token rate and a cache read at 0.1
times that rate. Confirm current model pricing with Anthropic before estimating
spend.

### OpenAI

Prompt caching is automatically enabled for supported OpenAI models. UAR
passes OpenAI request bodies and dispatch through unchanged regardless of the
UAR toggle; the toggle is not an OpenAI cache switch. OpenAI decides cache
eligibility, breakpoints, and retention.

OpenAI reports cached input in `usage.input_tokens_details.cached_tokens` and,
on models that charge for writes, `cache_write_tokens`. Its current guide says
GPT-5.6-and-later cache writes cost 1.25 times the ordinary input-token rate and
reads cost 0.1 times that rate. Earlier models can have different caching and
pricing behavior. UAR does not set an OpenAI cache TTL; OpenAI's
provider/model default applies and currently varies by model generation. Use
OpenAI's current model guide and billing data as the authority.

## Observe cache use

When telemetry is compiled, UAR records provider/model-labelled counters:

- `uar_cache_write_tokens_total`
- `uar_cache_read_tokens_total`

The counters normalize provider-reported usage. They prove only what the
provider reported to this process; they do not prove why a cache missed, and
they reset with the process unless Prometheus has scraped them. Compare write
and read tokens with total input tokens, request latency, and provider billing.

An enabled preference with zero cache tokens can be valid. Check that the
request used an Anthropic native route, the resolved source says caching was
enabled, the reusable prefix stayed byte-for-byte stable, the prompt met the
provider's minimum cacheable length, and the next request arrived within the
provider retention window.

## Official provider references

- [Anthropic prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching)
- [OpenAI prompt caching](https://developers.openai.com/api/docs/guides/prompt-caching)

Continue with [Interpret Cost and Budgets](/docs/operations/cost), [Observe the
Runtime](/docs/operations/observability), or [Troubleshooting](/docs/troubleshooting).

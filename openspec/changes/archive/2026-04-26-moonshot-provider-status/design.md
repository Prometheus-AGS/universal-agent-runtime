## Context

Moonshot Kimi k2.6 testing depends on a valid provider credential. For this phase, the safe closure behavior is to distinguish "provider is missing credentials" from a silent runtime failure. The existing `/api/catalog` endpoint already contains enough information to make this distinction: whether the provider is configured and whether the catalog declares an auth environment variable.

## Goals

- Classify Moonshot as credential-blocked when it is known in the catalog, requires auth, and is not configured.
- Preserve the configured state for providers with live registry entries.
- Keep the change additive for API clients.
- Avoid writing credentials to source, settings, logs, or test fixtures.

## Non-Goals

- Do not perform a live Moonshot API call from tests.
- Do not change `liter-llm` provider routing or the chat completion path.
- Do not infer whether a pasted key is valid unless it is provided through runtime configuration outside the repository.

## Decisions

- Add `status` and `status_detail` to `/api/catalog` provider summaries.
- Use `credential-blocked` for any unconfigured provider that declares an auth environment variable. This covers Moonshot and gives other cloud providers the same explicit diagnostic surface.
- Render the status as a compact badge in the providers page list and detail panel.
- Add pure helper tests in `src/server.rs` so classification is deterministic and does not require a server or external network.

## Risks

- Some providers may accept anonymous or local traffic despite catalog auth metadata. The status remains diagnostic and does not block manual configuration.
- Catalog provider IDs may vary (`moonshot` vs `moonshotai`). The fallback base URL already supports both aliases, and the diagnostic classification is driven by catalog metadata rather than hard-coded ID matching.

# 5. Migrate configuration to `config-rs` with `schemars` and `secrecy`

Date: 2026-07-13

## Status

Accepted

## Context

`src/config.rs` had grown to over 2,000 lines. Configuration was ad-hoc, env vars were not centrally declared, and secrets were stored as plain strings. The operator wanted a typed, schema-driven configuration layer.

## Decision

- Refactor `src/config.rs` to under 800 lines by deriving layers from `#[derive(ConfigLayer)]` structs.
- Use `config-rs` for layered loading and `schemars` for JSON Schema generation.
- Wrap secrets (`JWT_SECRET`, `LLM__API_KEY`, provider API keys) in `secrecy::Secret<String>`.
- Expose the canonical schema at `GET /.well-known/uar-config`.
- Preserve backward compatibility with legacy `LLM_*` env vars.
- Generate TypeScript types from the schema via `pnpm generate-config-types`.

## Consequences

- Configuration is type-safe, documented, and schema-validated.
- Secrets are harder to leak through logs or stack traces.
- SDKs share a single source of truth for config types.

## Alternatives considered

- Custom env parser: rejected because it would recreate `config-rs`.
- Pure YAML without env layering: rejected because operators rely on env-based configuration.

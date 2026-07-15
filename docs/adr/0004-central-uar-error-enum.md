# 4. Introduce a central `UarError` enum

Date: 2026-07-13

## Status

Accepted

## Context

Public API boundaries used a mix of `anyhow!()` and per-module error enums. SDK consumers and observability systems need stable error codes and a consistent error taxonomy.

## Decision

- Add a top-level `#[non_exhaustive]` `UarError` enum in `src/uar/error.rs`.
- Wrap existing public error types as variants: `Config`, `Auth`, `Rag`, `Memory`, `Mcp`, `A2a`, `Llm`, `Internal`.
- Convert 130 public-API `anyhow!()` calls to typed variants.
- Publish stable error codes such as `E_CONFIG_MISSING_FIELD` and `E_RAG_NO_KB`.

## Consequences

- SDKs can pattern-match on stable error codes.
- Error chains are inspectable and machine-readable.
- `thiserror 2.0` is used for deriving `Error`; `anyhow` is restricted to internal/application boundaries.

## Alternatives considered

- Keep `anyhow` everywhere: rejected because it hides error taxonomy from SDKs.
- Use `error-stack`: rejected because `thiserror` is already idiomatic in the codebase and sufficient for the required taxonomy.

# Design: repository-bound identity authority

## Context

Child project UUID: 06d6112e-346a-4417-8296-3b0ba3dba220. Product root: /Users/gqadonis/.claude/worktrees/bauar-uar. Accepted product base: a7cb972992d4f83db6585449ea81af0fe4a1c990. Parent phase: bossfang-uar-authorization-and-execution. Only this nested workstream owns the child KBD state. Original repository KBD state and C05 delivery gates remain untouched.

Exact source inventory, dependency closure, contracts and diagnostic protocol: [UAR prerequisites](/Users/gqadonis/.codex/worktrees/bossfang-uar-architecture/prometheus-skills-mini/workstreams/bossfang-uar-architecture/.kbd-orchestrator/phases/bossfang-uar-authorization-and-execution/evidence/execute/uar-prerequisites.md).

The only API-key backend is InMemoryApiKeyStorage, unconditionally selected in src/server.rs. No API-key database migration exists or is needed. C05 provider fcfce6d226b50eee502c5f272e9215112d933e29 is an ancestor and full-harness provider files match this base.

## Goals / Non-Goals

Implement the complete parent authority contract while preserving local launch behavior and C05 execution. No product mutations precede the parent D0 unchanged-baseline session diagnostic. No durable key-store invention, generic auth rewrite, production migration, external IdP selection or claim of runtime verification from this document.

## Decisions

The full reviewed design is incorporated from [parent design](/Users/gqadonis/.codex/worktrees/bossfang-uar-architecture/prometheus-skills-mini/workstreams/bossfang-uar-architecture/openspec/changes/bauar-01-identity-boundaries/design.md); the copied spec delta preserves every parent requirement/scenario. Its role/admin configuration and 60/300/5-second JWKS policy are binding. API-key records are process-local; version metadata and refuse remote use for missing identity rather than infer it. The typed launch marker originates in sidecar_guard and middleware, never a token role. External JWKS exchange is unsupported unless configured issuance satisfies the active verifier.

Remote workspace mapping uses verified issuer/subject/tenant and an operator-configured mapping. x-uar-workspace-id and A2A workspace params are selectors, never authority. Central HTTP admission can enforce header mapping before provider routing; independent gRPC and body-selector A2A paths require the same policy. Any additional bypass found before implementation needs an explicit claim extension.

Existing exact files: src/config.rs; src/uar/api/auth.rs; src/uar/security/api_keys.rs; src/uar/security/claims.rs; src/uar/security/verifier/mod.rs; src/uar/security/middleware.rs; src/server.rs; src/uar/security/sidecar_guard.rs; src/uar/security/mod.rs; src/uar/api/a2a/grpc.rs; src/uar/api/a2a/handler.rs; src/uar/runtime/turn/host/mod.rs; src/uar/runtime/turn/host/mcp.rs; src/uar/runtime/turn/host/credentials.rs; docs/API_KEYS.md; example.config.yaml; tests/test_a2a_grpc.rs. New exact modules: src/uar/security/verifier/jwks_cache.rs; src/uar/security/authority.rs; tests/bauar_authorization.rs. The prerequisite receipt enumerates mechanical UserContext constructors; obtain that exact binding extension if a required field changes. No directory wildcard grant.

Shared-file order: identity owns server.rs and host modules first, then releases server.rs to execution authorization and host modules to resource boundary. New substantial responsibilities stay in cohesive <=500-line modules; surgical wiring only in oversized existing files.

## Risks / Trade-offs

Uncomfortable constraint: current source ancestry does not prove a buildable isolated dependency closure or runtime auth behavior. Author scenarios/docs with production; runtime evidence waits for combined V1. Strict remote policy can reject existing clients; document reissue and supported exchange modes. JWT revocation remains bounded by issued expiry.

## Migration Plan

Driver binds tasks and dependency pins, D0 records unchanged baseline, then identity production proceeds as P1. Controlled strict configuration is explicit; no production state migration. Disable remote admission for rollback rather than restore role promotion. Consume one shared AUTH acceptance receipt after all selected production; use separate V2 build receipt.

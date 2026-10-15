# UAR 1.1.0 Kubo conformance

## Authority and scope
User approved implementation and publication on 2026-10-15. Baseline c9dca170; prior conformance fixes are committed in 22088cee. Dot is canonically complete; its cancelled qualification remains cancelled. This bounded phase delivers every #379 original item and follow-up requirement. Nothing is deferred.

## Spec → Plan → Execute → Reflect
Spec and Plan were completed through source inspection and operator decisions before execution. Execute is active. Historical installed-image checks and their failures are recorded in the phase acceptance receipts. The final entire-phase code, fixture and documentation inventory is now complete in implementation-complete.md. Serialized acceptance and publication remain; no check ran during the final implementation pass.

## Implementation inventory and ownership
1. uar-runtime: official pinned A2A 1.0 protobuf/gRPC alongside legacy; all durable operations, status and approval projections, admission authorization, protocol acceptance fixtures. Own src/uar/api/a2a/**, proto/** and A2A fixtures. Lead owns build.rs until policy validation.
2. uar-trust-tools: optional grant_policy.allowed_agent_ids; persisted native-agent identity/revision/policy bound to capture, cache and continuation; catalog-only restricted destinations; zero peer contact for unauthorized/inline impersonation; both elicitation sharing modes. Own src/mcp/**, host/mcp.rs, host/mod.rs, API routes and MCP admin handlers, MCP fixtures.
3. uar-state: import-content single SKILL.md with exact bytes/digest, checked persistence and collision rejection, normal GET readback and persisted binding fixtures. Own domain/skills.rs, API skills.rs, skills/service.rs and registry.rs, required persistence adapters.
4. Lead: operator security.skill_required_roles and host-derived authorization through automatic matching, activation and authenticated continuation. All required roles, explicit prefer denial before task reservation. Own config.rs, server.rs, manager.rs, security role module and activation integration. Shared-file handoffs serialized.
5. Lead: configuration defaults, version 1.1.0 and lock entry, AG-UI complete contract and trusted-host/operator release documentation, integrated acceptance runner. Remove implicit model selection; auth required in all shipped profiles; OpenTofu memory disabled/local. Explicit deployment listeners and remote databases retained.

## Boundary and acceptance
All implementation, required docs and fixtures must be complete before recording implementation-complete.md. Only then run serial local real-runtime acceptance, isolated artifact review, and candidate production-image build. NO early tests, compiler, formatting, lint, build, review gate or validation. Sole operator-authorized exception: pnpm github-actions-policy:validate before and after workflow-invoked build script edits.

Acceptance covers JSON-RPC and official gRPC, denied skill roles with no task, dynamic authorization, approval zero/one/zero invocation under both governance modes and transports, same-owner MCP agent A/B isolation and inline spoofing with no peer contact, durable continuation revocation, elicitation capabilities and accept/decline for both sharing modes, all AG-UI events against pinned schemas, exact content upload/bind/restart and persistence failures, default-image 300-second egress observation and loopback/no-model, shipped auth 401, discovery/version, CORS and effective-settings values/revisions/401/403.

## Publication
Commit bounded implementation and evidence. Exercise local production image. Push approved source and immutable v1.1.0 tag; existing deployment workflow publishes amd64/arm64. Record commit/workflow/manifest and architecture digests. Pull by published digest and run acceptance with no source patches. Only after success create GitHub Release, post #379 results and return OG-4 with verified digest. Never move tag silently.

## Uncomfortable failure mode
An image/tag can publish while integration acceptance fails. Such a release does not satisfy OG-4. Keep failure receipts and withhold success; publication alone is not acceptance.

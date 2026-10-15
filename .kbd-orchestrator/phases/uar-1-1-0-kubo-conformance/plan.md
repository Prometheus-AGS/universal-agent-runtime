# UAR 1.1.0 Kubo conformance

## Authority and scope
User approved implementation and publication on 2026-10-15. Baseline c9dca170; prior conformance fixes are committed in 22088cee. Dot is canonically complete; its cancelled qualification remains cancelled. This bounded phase delivers every #379 original item and follow-up requirement. Nothing is deferred.

## Spec → Plan → Execute → Reflect
Spec and Plan were completed through source inspection and operator decisions before execution. Execute and Reflect are complete. The operator-required entire-phase implementation boundary is recorded in implementation-complete.md. Published-image acceptance passed and v1.1.0 is released; receipts/publication.json and receipts/published-acceptance.json hold the final evidence. Historical acceptance results apply to their precise source/image.

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

## Review correction inventory
The initial artifact review established C1 restricted-origin aliases and W1/W2/W3 A2A tenant, durable projection and unsupported streaming mismatches. Correct source and installed-image regressions, including an actually reachable loopback peer alongside the remote grant profile, explicit equal/mismatched tenant, unchanged task inventory on denial/unsupported streaming, and full/limited cross-transport history/artifact identity. The initial image build was interrupted (130), not accepted. Record a corrected complete inventory before the single corrective review and restart of the production gate.

## Entire-phase implementation closure

- Runtime owner delivered the complete A2A acceptance fixture, client and provisioning paths for both bindings and all eight tuples, including protobuf defaults, public approval metadata, exact provider arguments, limited-credential continuation, asynchronous states and terminal projections.
- Trust owner delivered explicit AG-UI false approval and zero-call checks in all four MCP profiles.
- Lead delivered the release reproduction/compatibility/authority documentation, current ownership records and requirement mapping for all nine original items and four follow-ups.
- State owner delivered exact manifest import, persistence and binding fixtures with no unresolved implementation item.
- implementation-complete.md records the complete source inventory before any resumed gate. No production, persistence, protocol, migration, operational setting, UI or required documentation item remains pending.

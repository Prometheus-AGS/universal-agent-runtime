# Remaining caller-tool safety acceptance plan

Source-only assessment. No test/build/check/runtime/network/import commands executed.

## Actual policy mutations and current gaps

Supported mutation is authenticated PATCH /api/agents/{id}, mounted in src/server.rs, handled by discovery::patch_agent, applying RFC 7396 through agent_store::patch_agent_if_revision. GET the current registered artifact, retain its exact original value, then PATCH {"policy":{"tools":{"allow":[],"deny":["<actual MCP provider name>"]}}}. Do not invent configuration: the local supervisor discovers the real MCP name from /api/tools and originally creates policy.tools.allow with it. Restore the exact artifact after each independent scenario using the existing PUT route (including optional If-Match catalog revision when available).

HIGH A2A gap: warm task continuation uses Entry.actor. ActorThreadSession clones the artifact captured when ActorCollaboration.spawn_session_inner first resolved the registered agent. resume_client_tools does not refresh that artifact for a retained actor. RemoteRootConstraints also place a frozen execution policy in request.resolved_policy, which bypasses normal current agent policy resolution. Instance roots have separate bound-catalog/epoch validation; preserve it.

Minimum proposed repair: at a plain noninstance caller-tools root boundary, resolve the current registered artifact before building its request. Use remote execution policy as a restriction-only host_policy_constraint for caller-tool roots rather than replacing current policy with resolved_policy. No actor stop/recreation or session/history loss is necessary. The A2A adapter should verify current catalog existence before the continuation CAS claim. Current MCP bindings still must be captured/revalidated by the existing real manager admission path, not replayed from the handoff.

HIGH ACP gap: acp/handler.rs handle_create_run unconditionally constructs defaults::default_agent(), although ACP session state records an optional agent_id. It must resolve a selected registered agent on each create/result submission, retaining default behavior only for an absent selection. Existing 19 ACP cases therefore do not establish selected-agent policy behavior.

## Production HTTP cases still required

For each of OpenAI, Anthropic, AG-UI, A2A, ACP, use a real selected mutable registered agent and real provider/MCP peer:

- Start caller handoff, PATCH the selected agent to deny the actual host tool, submit the exact caller results. Capture the next provider request: the denied host schema must be absent. Configure an intentional provider host-call probe after results and assert it cannot execute; real MCP effect count remains unchanged. This requires an explicit fixture scenario which returns a configured host call after caller results rather than current mixed behavior (which executes host work before the first handoff).
- Remove/deactivate a registered destination or replace its allowed-agent scope through the real configured registry API identified by its owning writer; continuation must rebuild current bindings and refuse forbidden host effects. Do not assume cached executable bindings become fresh by changing tool declarations.
- Duplicate two client names and collide a client name with the discovered real host name: reject before provider contact.
- Run fixture scenarios already supported by tests/fixtures/client_tools_provider.py: malformed_arguments, provider_error (503), force_parallel with parallel_tool_calls false. Require explicit failure, no successful caller handoff, and no host-effect increase. Preserve 503/stream errors as failures on every native interface.
- Retain current full-batch/history/schema/cross-owner rejection tests and add cross-tenant coverage with the same subject and different tenant claims. Never treat owner testing alone as tenant isolation evidence.

The 95-case matrix proves only scenarios it actually ran. A green matrix cannot certify current-policy mutation, selected ACP agent routing, changed executable bindings, or the additional provider/input failures until these cases are authored and executed through the real HTTP services.

## Execution boundary

Lead owns the authorized local functional fix/retest loop and source ownership assignments. This document proposes bounded repairs and acceptance; it does not change production code, claim completion, or authorize a new gate. Published/frozen image tags remain untouched.

## Actual MCP rediscovery path for the executable-binding test

For the real gate's verified durable roots, the manager creates a new McpInteractionChannel before projected activation (src/uar/runtime/manager.rs ingestion-independent MCP interaction span). McpInteractionChannel::new uses a fresh UUID ID (src/mcp/interaction.rs). ProjectedActivationHost.prepare and ActivationContext::new_projected call runtime preflight. prepare_servers attaches that interaction channel to each McpBindingRequest; with_interactions places its ID in the cache key (src/mcp/binding_cache.rs). The new root therefore cannot reuse the previous root's ready connection under that key. McpRuntimeManager.prepare/connect reaches ConfiguredMcpConnector, then McpRegistry::connect_http_binding_with_headers, whose discovery executes service.list_all_tools against the actual peer.

The acceptance peer's /_host_binding operation may withdraw the actual tools/list entry after handoff. Capture the tools/list count before withdrawal/result submission and require a new discovery receipt during continuation, then require no withdrawn tools/call and no new effect. This is a real external binding change; do not update internal runtime state to make it pass. A missing selected descriptor may fail admission instead of returning a text completion; both must preserve zero forbidden effects and explicit failure where applicable.

This is conditional on the real verified durable interaction path. A configuration without a per-root interaction channel can reuse a ready cache entry; source alone does not establish unconditional new discovery for every deployment. Legacy non-bound freeze_bindings separately reissues list_all_tools and rejects missing selected tools. Neither fact should be generalized beyond its actual branch.

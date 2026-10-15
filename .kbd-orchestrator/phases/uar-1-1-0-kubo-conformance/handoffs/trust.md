# MCP trust and elicitation implementation handoff

Phase: `uar-1-1-0-kubo-conformance`, Execute. Implementation authored; all behavior and compilation remain unverified. No tests, checks, formatting, compiler probes, builds, review gates, or commits were run by this teammate.

## Source-grounded problem and chosen contract

Existing trusted-host grants identified owner and administrator destination but not the selected persisted agent. Two agents under the same owner could request the same destination. Startup and administration could also open globally registered connections before a run supplied authority. Existing code supports request-time credentials and run-local elicitation; this work extends those mechanisms.

`grant_policy.allowed_agent_ids` is optional. Missing/null retains legacy behavior; an empty set denies every agent; a singleton permits only that persisted native agent. Inline artifacts and collaboration/team artifacts cannot claim a persisted identity. The existing admin registration endpoint persists the non-secret restriction. Request-time grant headers remain the only needed outbound secret source when the registration contains no secret placeholders. A restriction cannot be granted by an agent artifact.

## Changed paths

- `src/mcp/config.rs`: optional allowed-agent set, identifier validation, `requires_agent_grant` classification.
- `src/mcp/catalog.rs`: includes a present restriction in configuration identity while preserving the old identity for absent restrictions.
- `src/mcp/runtime.rs`: lease-owned authority, live authorization, agent identity matching and revocation.
- `src/mcp/registry.rs`: restricted registrations remain catalog-only; disabled shared transports keep all registrations catalog-only. Checks before connection, discovery, freeze, invocation and reconnect. Added `configured_catalog_from_file` and `configured_empty` for deferred startup with enabled shared policy. Child binding identity predicate.
- `src/mcp/interaction.rs`: live authority checks before durable elicitation admission and during response wait/delivery.
- `src/uar/runtime/turn/host/mod.rs`: persisted non-secret `HostMcpAgentBinding` in grant markers; renewal requires exact binding identity.
- `src/uar/runtime/turn/host/mcp.rs`: actual persisted agent capture, destination policy admission, revision-bound lease, alias/direct URL restriction bypass denial, current agent and destination recheck.
- `src/uar/runtime/turn/host/mcp_tests.rs`: existing policy literal updated for new optional field; not executed.
- `src/uar/admin/mcp.rs`: restriction changes retire old connections, disabled catalog entries removed, checked durable hydration. Only absent settings seed file data; explicit empty maps stay empty; storage/decode errors propagate.
- `src/uar/api/mcp_admin.rs`: restriction updates persist before applying catalog-only registration and invalidate manager resources.
- `src/uar/api/routes.rs`: selected persisted agent enters MCP admission; protected preferred-skill admission before resources; approve-only current skill-role rechecks in both shared host resolution and native approval route.
- `tests/remote_mcp_run_grants.rs`: real sidecar fixture authors same-owner B rejection, inline A impersonation rejection, direct alias rejection, allowed A execution and stale agent revision continuation rejection with peer observations.
- `tests/fixtures/uar_1_1_mcp_contract.py`: external-runtime acceptance helper for release orchestration.
- `tests/fixtures/dot19_mcp_peer.py`: optional expected Authorization file, boolean-only credential observations and optional mismatch rejection; unchanged unauthenticated peer behavior when unset.
- This handoff.

## Shared-file integration

Lead owns `src/config.rs`, `src/server.rs`, `src/uar/runtime/manager.rs` and the role-policy modules. Lead has integrated manager admission, ambient catalog filtering, root resource identity and inherited child identity checks.

Manager admission calls `RegisteredMcpAgent::capture(persistence, root_registry, selected_native_artifact)` and `RunMcpServers::from_inputs_for_agent`. Routes pass `None` for inline/collaboration selections. The existing public `from_inputs` remains usable for unrestricted grants. `McpRunResources::matches_agent` and `McpRegistry::matches_agent` prevent transferring A-bound resources to B.

Lead adds `server.mcp_shared_transports_enabled`, default true, with sidecar always false. Use `catalog_from_file` / `empty` when false, `configured_catalog_from_file` / `configured_empty` when true. Hydrate persisted policy before startup connections. `upsert_server` now respects the registry sharing flag. All admin identity authorization remains lead-owned.

Routes expect `RunManager::authorize_preferred_skills(&UserContext, &AgentArtifact)` returning the existing `SkillAdmissionError`, and `authorize_run_skill_roles(&UserContext, &str)` for approved continuations. Denial remains permitted. Lead owns elicitation accept role rechecks in the interaction API.

## Real-path acceptance fixture contract

Import `tests/fixtures/uar_1_1_mcp_contract.py` and call:

`run_profile(base_url, headers, agent_a, agent_b, grant, peer_trace, shared_transports_enabled, validate_event, *, restart_runtime, admin_request, destination)`.

`restart_runtime()` returns after the same base URL is healthy. `admin_request(method, path, body, expected_status)` calls the real authenticated administration HTTP API. `destination` is the original non-secret `SaveMcpServerRequest` payload restricted to A. The helper is import-only; the release runner owns all lifecycle actions. No database mutation bypass is provided.

The release runner owns the digest-pinned production runtime, durable DB, two persisted agents under one verified owner, governance enabled, provider `scripts/dot/provider.py`, peer `tests/fixtures/dot19_mcp_peer.py`, destination registration restricted to A, trusted-host identity and request grant. The peer supports `--transport http --host 0.0.0.0 --port-file PATH --trace PATH` for a Docker-reachable non-loopback destination. The runtime must use that peer URL and `allow_private_http:true` for a private HTTP acceptance network. The real peer records capabilities and exact elicitation responses.

`grant` is the existing `{name, grant:{credential_revision,scopes,expires_at_unix,headers}}` request object. It must carry a nonempty Authorization sentinel. Start the peer with `--expected-authorization-file PRIVATE_PATH` containing that exact header; keep this file out of evidence. Optional `--require-authorization` rejects wrong credentials, but is omitted for the combined qualification because its ambient transition deliberately uses no request credential. Every explicit accept/decline/cancel initialize and tool call must record `credential_matches:true`. Only booleans are logged, never the sentinel. Other A2A peer usage omits the flag and retains existing behavior.

`validate_event` dispatches to the pinned complete official AG-UI input/event schemas and raises on failure. Every legitimate initial/resume input is validated before sending, as are every event and structurally valid but semantically incomplete/foreign resume inputs. This helper does not provide a replacement executor or schema subset.

The helper checks same-owner B, inline impersonation of A and direct endpoint aliases return 422 `run_mcp_agent_forbidden` with no added peer trace. It restarts the production runtime, requires no restricted initialization and repeats B denial. A then accepts, declines and cancels through real AG-UI HTTP/SSE. Every frame is validated. Expected interruptions are `tool_approval`, then `mcp_elicitation`; resume uses `parentRunId`, fresh `runId`, and exact interrupt IDs. Missing and foreign interrupt coverage each return 409 without consuming the original durable pending outcome, which is recovered unchanged over the reconnect stream. Peer assertions include form/url capability advertisement, one tool invocation, one exact response and no independent effect. Results contain public event/capability evidence and are scanned for outbound credential values. Errors are assertions; no secrets intentionally enter evidence.

Restriction transitions are exercised through administrator HTTP writes: temporarily allow unrestricted B capture, await its tool approval, restore exclusive A restriction, approve the pending B continuation, and require zero peer initialization or invocation afterward. Explicit unbound grants run in both sharing modes; ambient capture runs when shared transports are enabled. Every transition restores the original restriction in `finally`.

Run once for each actual sharing mode. The returned mode label alone is not evidence: retain the effective configuration and digest receipt. State teammate `/root/skills_defaults_research` owns integration into `scripts/release/uar_1_1_acceptance.py` and has the interface.

## Uncomfortable limits and next boundary

No command has established compilation, schema validity or runtime success for these edits. New real-sidecar test uses a loopback fixture; the digest acceptance runner must supply the non-loopback Docker peer and prove both actual sharing modes. The acceptance helper does not exercise every empty-list/revocation permutation. Existing and newly authored integration coverage must be selected by the lead only after the entire phase's implementation-complete inventory. The runner owner has the new credential/lifecycle callback contract; this teammate has not executed its integration.

Checks were deliberately not run under the full-phase verification boundary. Source reads and edits are the only evidence here. Guard additions trace to the explicit agent-isolation, no-preauthorization-initialization and stale-continuation requirements; no unrelated production feature was added.

## Follow-up: restrictions after ambient capture

The lead requested the actual saved-definition path be closed. Source flow: `capture_root_mcp_resources` retains an immutable catalog; `McpRuntimeManager::prepare` creates/reuses a binding from that catalog; after invalidation a fresh ticket can reach `ConfiguredMcpConnector` with the old unrestricted definition. Retiring the administrator registry's slot does not retire every independently created snapshot slot. A skill declaration could similarly alias the now-restricted endpoint. This was a source-grounded authority gap, not a runtime observation.

Additional changed path: `src/mcp/binding_cache.rs`. Each connected snapshot can now carry the authoritative registry and whether its destination is registered/agent-bound. `ConfiguredMcpConnector::with_registry` installs the guard for ambient definitions. Live global configuration identity must match; current restricted names and resolved endpoint aliases deny ambient/skill authority. Checks occur before initialize/discovery and through snapshot freeze, invocation, reconnect and elicitation admission/delivery. Cache-hit invocation still reaches the snapshot guard.

The same guard now applies to explicit run MCP inputs. A formerly unrestricted registered grant cannot retain authority after a restriction update, and a direct loopback alias cannot retain authority after that destination becomes restricted. Original restricted grants additionally retain their persisted-agent lease check. Configuration reads only revoke; they never replace the endpoint, environment or credential captured for execution.

Required lead integration: server constructs `ConfiguredMcpConnector::with_registry(Arc::clone(&mcp))`; manager chains `.with_registry(Arc::clone(&self.global_mcp))` after successful `RunMcpServers::from_inputs_for_agent(...)`. Both exact edits were sent to the lead. Existing public constructors remain compatible for independent hosts; this runtime's production entrypoint installs its own registry authority.

Follow-up edits are also unverified. No gate was run, and no claim of concurrency or runtime acceptance is made.

## Fixture approval premise

`src/mcp/registry.rs::mcp_descriptor` explicitly assigns `ApprovalClass::Required` even when an MCP peer advertises `readOnlyHint:true`; that annotation affects scheduling only. Only administrator `preapproved_tools` can exempt an MCP tool. The elicitation peer is deliberately untrusted and has no administrator preapproval. The external fixture requires an actually observed `tool_approval` interrupt before `mcp_elicitation`; absent approval fails qualification. No artificial effect was added just to manufacture an approval frame.

## Critic correction: fragment aliases

The independent critic identified complete-URL comparison at both direct admission and live definition authorization. A differently named loopback URL with `#fragment` passed both equality checks despite targeting the same HTTP endpoint. `direct_loopback_server` permits such a parsed URL, so this was a source-grounded bypass of restricted registration, not a speculative guard.

Changed `src/uar/runtime/turn/host/mcp.rs` and `src/mcp/registry.rs` to remove fragments from both sides of endpoint identity comparison. Captured launch inputs remain untouched. Added `direct_endpoint_fragment_alias` to `tests/fixtures/uar_1_1_mcp_contract.py`, requiring the agent-forbidden denial and zero peer contact. The production non-loopback profile also distinguishes this authority denial from ordinary direct-URL validation failure.

No command verified this correction. It was authored while the lead's prior candidate snapshot was building; the lead was told that the new source requires a rebuilt candidate.

The lead subsequently requested conservative loopback authority equivalence too. Both boundaries now use `same_http_destination`: fragments do not participate; `localhost`, IPv4 loopback addresses and `::1` match when scheme, effective port, path and userinfo match. This deliberately treats distinct local listener addresses as one restriction boundary to prevent an A-only loopback registration being borrowed under B through another local hostname. It does not change any launch URL or perform DNS resolution.

The lead then identified that `dot19_mcp_peer.py::Handler.do_POST` does not route on query parameters. Query equality would therefore allow a different-name `?alias` to reach the same restricted endpoint. Restriction matching now ignores query parameters too; credentials and actual launch URL are unchanged. Path and userinfo remain exact. Added mandatory query-only and query-plus-fragment denial probes, and all loopback hostname × query × fragment combinations when a loopback peer is supplied.

The external fixture uses A's subsequent successful real tool calls as reachability control and records `loopback_aliases_exercised` explicitly. The runner owner has authored a runtime-local TCP relay for loopback cases; the non-loopback profile must also remain present to prove the external destination requirement. These fixture additions are unexecuted.

## Final authority correction: unregistered origin aliases

The lead identified the remaining demonstrated path alias: the peer's POST handler accepts alternate paths as well as arbitrary queries. Restriction matching for unregistered declarations is now origin-wide. The shared helper is `same_http_origin`, matching scheme, host (with conservative loopback equivalence), and effective port only. It does not rewrite the request URL. Direct admission and live unregistered/skill dispatch both use this helper, so a new name/path/query/fragment cannot borrow a restricted HTTP origin.

Explicit separately registered destinations retain their own administrator policy: live authorization first requires the exact current registered configuration identity, requires an agent-bound lease when that registration is restricted, then permits that registration without applying another registration's origin restriction. Existing persisted-agent checks still run on restricted leases. This avoids treating an administrator-authorized sibling destination as an unregistered alias.

Added mandatory `direct_origin_alternate_path` no-contact denial to the external fixture. Final changed source/fixture paths for this correction: `src/mcp/registry.rs`, `src/uar/runtime/turn/host/mcp.rs`, `tests/fixtures/uar_1_1_mcp_contract.py`, and this handoff. The runner owner reports separate external-gateway and loopback-relay profiles are authored. No verification command was run by this teammate; corrected candidate acceptance remains lead-owned.

## Follow-up: explicit AG-UI tool approval denial

The lead identified an omitted follow-up item 5 scenario: elicitation decline/cancel followed an approved tool and did not prove that `approved:false` prevents the tool invocation. Changed only `tests/fixtures/uar_1_1_mcp_contract.py` and this handoff for this packet. No production or runner changes were made.

Each `run_profile` now starts a separate real AG-UI run under A, requires exactly one pending `tool_approval`, checks the existing invalid-coverage/reconnect contract, and resumes its exact interrupt with `status:"resolved"` and `payload:{approved:false}`. Every initial/resume input and emitted/replayed event goes through the supplied pinned schema validator. The warm continuation must terminate with `RUN_FINISHED` success, never another interrupt; replay must retain that exact terminal. Peer trace assertions require zero tool calls, elicitation requests/responses and independent effects before denial, after denial, and after terminal replay. Discovery initialization remains permitted. Returned evidence adds `tool_approval_denial`, including both segments, pending/replay evidence and zero-call counters. Existing accept/decline/cancel cases now also assert zero tool activity before each affirmative approval and record `tool_calls_before_approval:0`.

Source contract: `src/uar/api/agui.rs::approval_answer` reads the explicit false boolean; warm resume calls `routes::resolve_host_approval`, which delivers denial through the existing broker. `src/uar/runtime/manager.rs` maps rejection to `ToolApprovalResult::Rejected`; `src/llm/orchestrator.rs` acknowledges host denial and returns before execution. The controlled provider in `scripts/dot/provider.py` completes after the returned tool-result message. AG-UI projects the settled normal completion as success. Cold denial has a distinct `RUN_ERROR approval_denied` path and is outside this warm scenario; the fixture does not accept arbitrary errors as denial evidence.

The runner already calls `run_profile` for each actual sharing mode and each external/private-network and runtime-loopback destination, so the new case is included in all four profiles without an interface change. Existing credential, HTTP-session capability, exact elicitation response and invocation-count assertions remain in place.

No tests, checks, formatting, builds, diff checks or review gates were run. Evidence is source inspection and authored artifacts only. Runtime/schema validity, warm denial completion and zero-call behavior remain unverified until the lead closes the entire implementation inventory and runs the planned real-path gate. No unresolved design decision or extra production guard was introduced; the uncomfortable limit is that the former accept/decline/cancel coverage did not establish tool-approval denial.

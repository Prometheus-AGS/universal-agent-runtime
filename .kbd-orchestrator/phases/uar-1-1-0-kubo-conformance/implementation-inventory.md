# Current entire-phase implementation inventory

Status: **ENTIRE-PHASE IMPLEMENTATION COMPLETE** (2026-10-15T16:53:31.541574+00:00). The operator requires all remaining phase code, fixtures and documentation to be complete before any further check. This current inventory supersedes the historical declarations retained in `implementation-boundary-history.md`. The complete requirement mapping is `requirements-inventory.md`.

## Production files

- `Cargo.toml`, `Cargo.lock`: runtime version 1.1.0 and direct optional pinned prost-types for official protobuf values.
- `build.rs`: compile official and legacy A2A schemas with committed Google API imports and vendored well-known types.
- `proto/a2a_v1.proto`, `proto/google/api/*.proto`, `proto/google/api/UPSTREAM.md`: pinned normative service and import provenance.
- `src/uar/api/a2a/admission.rs`: verified host admission before reservation.
- `src/uar/api/a2a/grpc_v1.rs`: official A2A service over the existing durable task service.
- `src/uar/api/a2a/grpc.rs`: shared credential verification and legacy admission.
- `src/uar/api/a2a/handler.rs`: host admission state and legacy HTTP admission.
- `src/uar/api/a2a/mod.rs`: official service export.
- `src/uar/api/a2a/thread_service.rs`: authenticated actor continuation authority.
- `src/uar/api/a2a/v1.rs`: JSON-RPC role admission and mounted-interface discovery.
- `src/config.rs`: operator skill-role map and explicit MCP sharing switch.
- `src/server.rs`: mount both gRPC services, wire role policy and live MCP registry, hydrate persisted MCP settings before connections.
- `src/uar/security/skill_roles.rs`, `src/uar/security/mod.rs`: typed host admission failures.
- `src/uar/runtime/manager.rs`: role-filtered matching/activation, continuation checks, persisted agent capture and resource identity checks.
- `src/uar/runtime/actor/messages.rs`: verified roles separate from stable ownership identity; fresh authority in prompt messages.
- `src/uar/runtime/actor/system.rs`, `src/uar/runtime/actor/agent_actor.rs`, `src/uar/runtime/thread/actor_host.rs`: refresh authority at authenticated actor submission.
- `src/uar/runtime/team_execution/host.rs`: delegated artifacts do not impersonate persisted native MCP agents.
- `src/uar/api/routes.rs`: host resource admission and approved-continuation role checks.
- `src/uar/api/mcp_interactions.rs`: current roles required for elicitation acceptance.
- `src/mcp/config.rs`: optional allowed-agent policy with absent/empty distinction.
- `src/mcp/catalog.rs`: policy-bound configuration identity.
- `src/mcp/binding_cache.rs`: live destination authority attached to captured binding requests.
- `src/mcp/runtime.rs`: current-policy and agent-bound lease checks on cached/connected resources.
- `src/mcp/registry.rs`: catalog-only restricted/disabled-sharing destinations and live snapshot checks.
- `src/mcp/interaction.rs`: authority checks before elicitation admission and response delivery.
- `src/uar/runtime/turn/host/mcp.rs`, `src/uar/runtime/turn/host/mod.rs`: persisted native-agent capture, revision/policy markers, request grants, alias rejection and continuation identity.
- `src/uar/admin/mcp.rs`, `src/uar/api/mcp_admin.rs`: checked persistence hydration and policy updates before executable registration.
- `src/uar/domain/skills.rs`: exact imported manifest body retained in the domain record.
- `src/uar/runtime/skills/service.rs`, `src/uar/runtime/skills/registry.rs`: validated content import, digest, atomic collision rejection and persistence before memory publication.
- `src/uar/runtime/skills/builtin_loader.rs`: initialize the optional raw-document field in the existing constructor.
- `src/uar/persistence/providers/a2a_bindings/surreal.rs`: compare validated binding values and retain exact stored bytes for atomic A2A transition/cancellation CAS, compatible with legacy metadata ordering.
- `src/uar/persistence/mod.rs`, `src/uar/persistence/providers/{memory,postgres,surreal}.rs`: checked create-if-absent skill persistence.
- `src/uar/api/skills.rs`: import-content HTTP endpoint and normal exact-content/digest readback.
- `config.test.yaml`: authentication required.
- `docker-compose.dev.yaml`, `docker-compose.test.yaml`: no implicit model.
- `k8s/opentofu/configmap.tf`, `k8s/opentofu/variables.tf`: authentication required, empty model default, disabled/local memory default.

## Documentation and acceptance inputs

- `docs/releases/v1.1.0-kubo.md`: operator setup, authorization, compatibility, upload, complete interrupt/resume contract and publication conditions.
- `docs/A2A_V1.md`, `docs/AGUI_V1.md`: official/legacy bindings and release contract references.
- `scripts/release/uar_1_1_acceptance.py`, `uar_1_1_loopback_client.py`: real installed-image supervisor, persistence fault, MCP and A2A tuple execution.
- `scripts/release/uar_1_1_defaults.py`: default startup, all TCP listeners, five-minute cumulative packet observation, public discovery, no model, effective revisions and CORS.
- `scripts/release/uar_1_1_profiles.py`: actual anonymous-401 execution for 12 shipped runtime projections, including example/test configurations, Compose, Kubernetes, Helm and OpenTofu. Authentication/cache settings derive from the actual source or compiled default; probes require both effective-settings and middleware-protected agent discovery rejection. Only local dependency endpoints, credentials and writable paths are substituted.
- `scripts/release/prepare_agui.py`, `validate_agui.mjs`: immutable published AG-UI schema and input/event validation.
- `tests/fixtures/a2a_v1_approval_governance.py`: real task inventory, role denial, dynamic activation and approval 0/1/0 across transports/governance modes.
- `tests/fixtures/uar_1_1_mcp_contract.py`, `dot19_mcp_peer.py`: credential observation, same-owner/inline/alias isolation, restart, revocation, elicitation and exact pending-interrupt coverage.
- `tests/fixtures/skill_remote_import.py`, `tests/fixtures/skill-upload/*.SKILL.md`: exact content/binding/restart, malformed input and collisions.
- `tests/test_a2a_grpc.rs`, `tests/remote_mcp_run_grants.rs`: additional native protocol/sidecar integration scenarios, authored but not executed.
- Existing explicit configuration/state constructors in `src/uar/security/middleware.rs`, `tests/a2a_thread_service.rs`, `tests/context_history_integrity.rs`, `tests/jwks_es256_verification.rs`, `tests/settings_persistence.rs`, `tests/stream_terminal_contract.rs`, `tests/support/truthful_tool_outcome_peer.rs`, and `src/uar/runtime/turn/host/mcp_tests.rs` are aligned with new fields.
- OpenSpec proposal/design/spec delta/tasks, bounded KBD plan/execution/handoffs, team ownership state and append-only Prometheus decisions/gotchas record this phase.

## Final source corrections already authored

- `src/server.rs`: default no-model startup skips optional response-quality construction instead of failing provider construction.
- `src/uar/persistence/providers/a2a_bindings/surreal.rs`: transition and cancellation retain the exact stored JSON string for atomic CAS, after typed semantic comparison and existing transition validation. HashMap serialization order no longer creates false conflicts. The unsuccessful bounded Get retry was removed.
- `src/uar/runtime/manager.rs`: per-turn native activation registers under the existing skill-policy model-control contract, independently of ordinary tools.none. Verified-role filtering remains the activation authority.
- `tests/fixtures/uar_1_1_mcp_contract.py`: explicit AG-UI false approval, zero calls/elicitation/effects before approval and after denial, terminal replay, and unchanged complete positive/session/credential assertions. Trust handoff is delivered.
- Release documentation now includes actual full acceptance invocation and distinguishes default startup, selected deployment infrastructure, synthetic governance-disable posture, native activation approval, immutable provenance and architecture limits.

## Final implementation closure

- Runtime owner delivered complete JSON-RPC/gRPC fixture, client and setup paths: narrow protobuf default omission, pinned enum/shape/envelope assertions, authentication/tenant/role rejection, asynchronous states, exact approval ID/decision, same limited-credential continuation and cross-transport terminal parity.
- Lead integrated all runtime/trust/state handoffs, release reproduction and compatibility documentation, current ownership ledger and source inventory; `implementation-complete.md` records the whole phase closure.

No other production, persistence, migration, operational setting or UI change is pending in the approved plan. There is no UI work in this release phase. Publication and acceptance receipts are later execution outputs, not unfinished implementation placeholders.

## Verification and guard accounting

No verification ran during this final implementation pass. The final entire-phase inventory is now closed; the approved serialized acceptance program may begin. Prior commands, interrupted/failed runs and exact image-specific passes remain in `execution.md`, `acceptance.md` and `receipts/`; none certifies new source.

The isolated artifact review and single corrective review are retained with their same-model-family independence limitation. No additional optional review loop is planned. The sole approved early exception remains the local GitHub Actions policy validator before/after workflow-invoked build-script edits; no such edit is currently pending.

New production guards trace to operator skill-role authority, persisted native-agent identity, administrator policy revocation and checked durable import. No speculative guard or unrelated feature was added. Existing required approval remains enforced.

The uncomfortable limit is that source completeness must include the acceptance clients and documentation. Earlier per-fix declarations did not establish that. Published acceptance must use the immutable digest with no source patches; a tag alone never establishes OG-4.

Current completion: runtime owner delivered the complete HTTP middleware/protocol error distinction across both JWT modes; lead integrated the handoff and release documentation. No code, fixture or documentation item remains. Updated whole-phase closure is implementation-complete.md.

## Lead integration: normative optional-field presence

The r21 fixture reached completed output and rejected a valid output artifact because it demanded optional metadata. Lead took exclusive fixture ownership and read the pinned Task/TaskStatus/Message/Part/Artifact field annotations, v1::project_task/message/part, grpc_v1::task_to_pb/message_to_pb/artifact_to_pb, and persisted approval decision types. All nested required-field assumptions were reconciled together. IDs, roles, state values, nonempty message/artifact parts and actual approval records remain required; optional metadata/history/context/artifacts use their protocol defaults and normalized parity. JSON-RPC ListTasks explicit pagination contract and gRPC implicit default normalization remain exact. No production change, new dependency, new probe or new guard. No check ran during this implementation pass. All phase code/fixture/documentation items are authored; no unresolved implementation remains.

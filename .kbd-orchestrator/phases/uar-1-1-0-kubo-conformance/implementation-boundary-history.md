# Historical implementation boundary declarations

Superseded declarations retained verbatim for audit. These do not authorize checks. Current implementation is tracked in implementation-inventory.md and requirements-inventory.md. The uncomfortable failure was repeated per-fix gate permission before fixture implementation was complete.

# Implementation inventory — UAR v1.1.0

Current boundary: **IMPLEMENTATION OPEN — VERIFICATION STOPPED**. The operator requires the entire phase's production code, fixtures and documentation to be complete before any further gate. The runtime owner is finishing every remaining A2A fixture path against both wire contracts as one implementation task. The lead is reconciling phase and release documentation. Historical boundary declarations below are superseded and do not authorize a check. The final boundary will be recorded separately in `implementation-complete.md` after every remaining item is closed.

Boundary status: **IMPLEMENTATION COMPLETE — FINAL W2 CORRECTION AUTHORED** (2026-10-15). The runtime owner delivered the residual W2 correction: original inbound message IDs are retained internally, public message IDs match JSON-RPC, and internal identity metadata is removed from public projection. No production/docs/fixture item remains pending. The single corrective review is complete: C1/W1/W2/W3 source-addressed, no unresolved finding among those four. Final history fallback is exactly task:index and status fallback task:status in both bindings. Runtime behavior remains unverified. Two interrupted builds (130) establish no acceptance.

Previous correction boundary: **CORRECTED IMPLEMENTATION COMPLETE** (2026-10-15). All initial implementation and review correction production/docs/fixture items are authored. Runtime, trust and state owners have delivered their final source packets. Verification and publication remain pending.

Correction inventory:
- C1: shared restricted-origin matching at direct admission and live dispatch, including equivalent loopback hosts and alternate path/query/fragment; registered leases retain their own administrator policy. Production fixture covers each alias with zero peer contact.
- W1: gRPC tenant validates verified tenant; agent selection remains x-uar-agent-id. Both bindings exercise explicit equal tenant and mismatched-tenant rejection with unchanged durable task inventory.
- W2: gRPC preserves durable artifact fields, completion output and original history indexes. Fixture compares the same task across bindings with full and one-entry history.
- W3: unsupported gRPC streaming returns UNIMPLEMENTED before submission; fixture checks unchanged tasks and peer trace.
- Installed-image supervisor keeps external private-IP grant coverage and adds real runtime-loopback forwarding to the same credential-observing peer in both sharing modes, with runtime/relay restart and safe readiness/PID cleanup. New file scripts/release/uar_1_1_loopback_relay.py. Loopback client exports the verified fixture tenant.
- Release contract documents direct-loopback restricted-origin semantics; append-only decisions and owner handoffs record the rationale.

No correction item remains pending. The initial build was interrupted (130), before acceptance; it certifies nothing. The single corrective static review and restarted production image build are now permitted.

Initial boundary: **IMPLEMENTATION COMPLETE** (2026-10-15). The lead has received the runtime, trust and state handoffs and integrated every planned production, configuration, documentation and fixture item. No implementation item remains pending. Verification and publication remain pending; source completion is not a runtime success claim.

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

## Verification and guard accounting

Only the four authorized before/after GitHub Actions policy checks have run; each passed. No Cargo, product test, compiler, formatting, image build, runtime acceptance or independent review has run.

The primary gate is now authorized: a local build from the production Dockerfile followed by the installed-image acceptance program. An isolated artifact critic receives the diff, requirements and receipts without producer conversation. Failed gates are repaired and rerun narrowly using the runner's gate selection. A partial gate receipt cannot claim full acceptance. Publication remains pending until candidate acceptance, followed by immutable-tag/image publication and acceptance of the pulled registry digest.

New guards trace to requested skill-role authority, persisted native-agent authority, administrator policy revocation and checked durable skill creation. No speculative production guard or unrelated feature was added. Existing mandatory approval enforcement is preserved pending real-path observations.

Uncomfortable limit: the packet-counter window begins at the first accessible container namespace sample. It proves the complete observed five-minute interval; it does not claim observation of the launch-to-first-sample gap. A published tag/image alone never establishes OG-4.

## Compiler correction
Observed E0308 in build.rs corrected by using Path values in both compile_protos argument lists. Protocol inputs and generated behavior unchanged. Both mandatory policy checks passed. No further implementation item is open; rerun the failed production image gate after its original dependency jobs finish.

Observed generated Rust service/client names aligned in grpc_v1.rs and test_a2a_grpc.rs after r1 compile failure. Wire schema/behavior unchanged; all corrective implementation complete.

Observed E0277 digest formatting fixed in host/mcp.rs with existing bytewise hex pattern. No implementation item remains open before retry r3.

Removed compiler-reported unused submit_prompt_with_capture wrapper after its only caller migrated to submit_authorized_prompt. No caller or public API removed; all implementation complete before r4.

## Default startup acceptance correction — in progress
The installed image exited before health with Provider client construction failed and empty model. Runtime owner is correcting the no-model construction path; lead retains container logs and fixes explicit CORS invocation to include the runtime executable. No verification runs until these corrections are complete.

Default startup correction IMPLEMENTATION COMPLETE: server.rs skips optional response-quality construction for empty model; defaults fixture retains logs, fails promptly on exit, and supplies executable for CORS config. Runtime handoff received, all source/docs/fixtures complete. No other implementation item remains. Rebuild and rerun failed installed-image gate are now permitted.

## Egress observer correction — IMPLEMENTATION COMPLETE
Observed aggregate-counter failure traced by packet diagnostics to interface control traffic. scripts/release/uar_1_1_defaults.py now runs an independent AF_PACKET observer, retains frames/counters, requires zero capture drops, rejects every application or unclassified packet, and explicitly reports excluded gratuitous ARP/IPv6 control frames. docs/releases/v1.1.0-kubo.md records exact classification and unchanged capture-start gap. No production behavior changed; same r5 image is valid for the rerun. All implementation/docs/fixtures complete; proceed with installed acceptance.

## Governance fixture correction — IMPLEMENTATION COMPLETE
Default revision mutation now uses a deny-all MCP catalog entry; A2A disabled-governance synthetic profile now satisfies the existing optional-JWT/all-loopback eligibility rule. Both fixture edits and documentation of rationale complete; all production/docs/fixtures remain implemented. Same r5 image, no rebuild necessary. Resume serial installed acceptance.

## Surreal acceptance fixture correction — IMPLEMENTATION COMPLETE
Observed collaborator startup permission failure corrected in release runner using writable durable /tmp database, host-side health probe and early cleanup registration. All implementation/docs/fixtures complete. No new production changes; reuse r5 image and rerun remaining gates only.

## Restart observation correction — IMPLEMENTATION COMPLETE
Release runner uses explicit host port bindings for restartable runtime clients and validates mapping stability, retains logs, cleans failed startup, and probes restored Surreal through its current host mapping. No production source change; all implementation/docs/fixtures complete before MCP/A2A retry.

## Persistence outage fixture correction — IMPLEMENTATION COMPLETE
Accept existing outer408 timeout or named503 persistence failure; still require absent failed skill after recovery and retained prior skill. No production change. All implementation/docs/fixtures complete before remaining gate retry.

MCP denial diagnostics correction IMPLEMENTATION COMPLETE: fixture includes scenario and exact errorbody on mismatch. No production correction proposed without that evidence. All current implementation complete before diagnostic gate rerun.

- r7 fixture correction complete: signed issuer classification moved from JOSE header into JWT payload, matching VerifiedJwtClaims and HostAuthority. Trusted-host setup documents the payload requirement. No production change; image88efe49c remains current. Implementation complete; rerun only MCP/A2A as r8, preserving passed defaults/profiles.

- r8 passed trusted-host authentication, all external A/B/inline/alias denials with zero peer contacts, and restart restriction. First authorized AG-UI request returned422; generic fixture diagnostic omitted its body. Fixture now retains exact unexpected HTTP response status/body. Diagnostic correction complete; rerun MCP/A2A r9 on unchanged image.

- r9 correction inventory complete: disposable RFC1918 Docker-network TCP relay forwards untouched peer bytes, no host-published port; loopback alias fixture forwards through same relay; checked cleanup/log retention. Stateless AGUI inputs omit optional state to preserve exact controlled provider command. No runtime source/image changes. All implementation/docs/fixture work complete; rerun MCP/A2A r10 against image88efe49c.

- r10 session-aware correction complete: peer emits MCP session IDs and traces correlated discovery/call/answer traffic; fixture requires exact elicitation capability for every invoking session, no invocation on discovery-only sessions, exact credential/decision/count invariants unchanged. Source inspected, production image unchanged. Complete implementation boundary restored; MCP/A2A r11.

- r11 sharing-disabled external profile completed accept/decline/cancel, session capability/credential/AGUI checks and restriction transition; loopback negative-probe burst hit429. Runtime default10requests/second burst20 is intact. Fixture now paces requests200ms; no retries or guard changes. Complete corrected inventory restored; retry MCP/A2A r12 on unchanged image.

- A2A read-race correction complete: get_durable reloads at most3 times for changedcurrentrun, local read projectionConflict or finalCASmiss; localprojectionrequiresmatchingrun/process, allothererrorspropagate. Cancellation/reservation/continuationmutationpaths unchanged. Existing immediateGet realclientfixture retained. Allproduction/docs/fixturescomplete again. Rebuild productioncandidate r6; then onlyA2Aaffectedgate; fullpublishedimagegate remainsmandatory.

- r13 immediateGet now succeeds with validTASK_STATE_WORKING (noConflict), while fixture expectedinputrequired despite explicitreturnImmediately:true. Fixture retains immediateGet regression/state receipt then polls existingwait_for_task forinputrequired as specifiedacceptanceprocedure. No production change. Correctedfixturecomplete; rerunonlyA2A r14 onr6image2ad3a904.

- Raw-snapshot Surreal A2A repair complete in src/uar/persistence/providers/a2a_bindings/surreal.rs: validate/semanticcompare parsedcurrentbinding againstexpected; CASexactstoredbytes; cancellationtransaction preservesrawsnapshotfence. Legacydefaultsnormalizedbytypedparse; no schema/dependencychange. Unprovenget_durableretry removed, pre-r12functionrestored. Allproduction/docs/fixturescomplete. Retainedr15database evidence copiedbeforefixturecleanup. Rebuildproduction r7 thenA2Arerunonly.

- r16 dynamic activation correction complete: the fixture's dynamic agent already requests `tools.allow=[activate_skill]`, but policy-universe resolution excludes this per-turn model-control handler from ordinary tool selection and yields `tools.none`. The manager had therefore skipped registration before its existing skill-based `allows_model_control_tool` filter could admit it. `src/uar/runtime/manager.rs` now registers only `activate_skill` through that contract independently of ordinary tool mode; team and agent controls remain under the existing non-`none` guard. Verified-role narrowing still removes `release-protected-skill` before native activation, preserving the required `ineligible` result and protected-prompt absence. No fixture policy widening or other production change. Rebuild r8 and rerun only A2A.

- r17 dynamic continuation fixture correction complete: installed evidence shows the sole advertised dynamic tool is `activate_skill`, ordinary dynamic tools remain none, and the peer trace has no dynamic effect call. The control inherits `Unknown` effect and required approval, so fixture now asserts its persisted approval record is exactly `activate_skill({"skill_id":"release-protected-skill"})`, approves that control with the role-limited credential against the dynamic agent, and polls that same principal. It retains the required ineligible result, zero peer effect, and protected-prompt absence. No production change. Rerun only A2A on image2bf3ce5f.

- r18 correction complete: public ApprovalRecordView exposes toolName, not arguments. Fixture checks that public identity and exact activate_skill arguments in subsequent provider tool-call history. Same limited credential and native ineligible/noeffect/noprompt assertions retained. Production unchanged; all implementation/docs/fixtures complete; A2A r19 uses image2bf3ce5f.

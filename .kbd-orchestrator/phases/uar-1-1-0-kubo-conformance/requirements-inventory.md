# Entire phase implementation inventory

This is the implementation ledger for all nine #379 items and all four Kubo follow-up requirements. Source completion does not establish runtime acceptance. Every implementation entry below is authored. The lead records the full closure in `implementation-complete.md`; runtime acceptance remains pending.

| Requirement | Production and documentation | Acceptance implementation | Current implementation status |
|---|---|---|---|
| 1. Public A2A 1.0 card | `src/uar/api/a2a/v1.rs`, middleware exemptions; official and legacy compatibility in `docs/A2A_V1.md` | Default gate checks public card shape, implemented bindings and version | Authored |
| 2. A2A state and authorization | Official `grpc_v1.rs` and pinned proto, JSON-RPC projections, shared durable task service, pre-reservation host skill-role policy | Full A2A client contract reconciliation for both transports; exact enum/state, role and no-reservation assertions | Authored; complete runtime handoff delivered |
| 3. Mandatory approval | Existing Required-tool gate preserved; durable task approval projection; exact-byte Surreal binding CAS repair | Eight transport/governance/decision tuples with zero/one/zero peer counts and authenticated continuation | Authored; complete runtime handoff delivered |
| 4. Standard AG-UI path | Existing `src/uar/api/agui.rs` from prior committed work, complete release interrupt contract | Pinned input/event schemas; approval, elicitation and resume; explicit false-approval scenario | Authored; trust handoff complete |
| 5. CORS allowlist | Existing operator allowlist and default deny in server configuration | Default deny and explicit allow through installed image | Authored |
| 6. MCP elicitation | Run-local handler, live authority checks and execution-session capability advertisement | Session-correlated accept/decline/cancel for both sharing settings and real peers | Authored, including explicit approval denial |
| 7. Defaults | Empty dev/test Compose model; test/OpenTofu authentication; disabled/local OpenTofu memory; default loopback/no-model; selected deployment infrastructure retained | 300-second default packet observation, listeners/no-model and twelve source-backed authenticated profile projections | Authored under approved default-versus-deployment distinction |
| 8. Effective settings | Existing `src/uar/api/effective_settings.rs` with bearer admin policy and aggregate revision | Values/provider hosts, anonymous 401, non-admin 403, settings revision change/restore | Authored |
| 9. Release version | Cargo package and lock entry 1.1.0; public card from package version; existing deployment publication path | Immutable tag/image provenance and pulled digest acceptance | Implementation authored; publication/evidence are later execution work |
| Follow-up: per-agent external MCP | Optional allowed_agent_ids, persisted native identity/revision/policy, live restrictions and request-time headers; documented trusted-host setup | Same-owner A/B, inline impersonation and origin aliases; no-contact denial, restart and revocation | Authored |
| Follow-up: remote skill upload | Single content endpoint, checked atomic creation, exact body/digest readback, existing binding API | Exact upload/readback/bind/restart, collision/malformed input, real persistence outage and no failed-skill publication | Authored |
| Follow-up: global elicitation | Catalog-only restricted/nonsharing destinations, explicit grants and run-local interaction channel | External private-network peer and runtime-loopback peer under both sharing settings | Authored |
| Follow-up: pinned AG-UI interrupts | `docs/releases/v1.1.0-kubo.md`: reasons, fields, booleans, actions, exact pending coverage, resume identity and restrictions | All events/input schemas, incomplete/foreign resume rejection and unchanged reconnect; explicit false approval | Authored; trust handoff complete |

## Exclusive remaining writers

- Runtime owner delivered all assigned A2A fixture/client/setup paths, including response contracts, protobuf omission, authentication, exact approval decisions, limited-credential continuation and all eight tuples. No unresolved implementation item.
- Trust owner delivered `tests/fixtures/uar_1_1_mcp_contract.py`, including real AG-UI false approval and zero calls before approval; no remaining implementation item in that handoff.
- Lead delivered release and phase documentation, current ownership ledger and the final complete inventory.
- State owner's production import/persistence and fixture handoff is already delivered. No shared production source change is currently pending. Any new implementation requirement reopens this same full-phase boundary.

## Later execution

Only after implementation closes: serialized local acceptance and required artifact/source gates, bounded commit, immutable source/tag publication, existing amd64/arm64 deployment build, digest inspection and pull, complete installed published-image acceptance, GitHub Release and #379 handoff. Final tag/digest/receipts are produced by those actions and are not placeholders for unfinished code. No published success exists yet.

The uncomfortable discrepancy is that earlier records repeatedly declared completion before the fixture contracts were fully authored. Those declarations are superseded; they cannot authorize another per-fix gate.

Final response-boundary integration: bare outer-middleware HTTP401 is distinct from handler A2A401/-32010; valid-role/tenant protocol errors and all gRPC statuses retain exact assertions. The fixture and release documentation are complete. No remaining implementation item.

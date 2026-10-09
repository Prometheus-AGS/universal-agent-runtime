# Customer representation runtime repair

Date: 2026-10-09. Scope: the approved customer-release closeout, C17 bounded
synthetic read-only representation. Baseline UAR:
`60b5922e3e11dd73bfd8a47e5bc28f3c16332889`.

## Observed failure and evidence

The published Mac 2.2.25 operation selected a configured model, previewed and
installed an office role, created a durable logical instance, and read its
representation snapshot. Grant save then returned a generic desktop INTERNAL
error; cleanup disabled the disposable instance. No represented turn was attempted.

Parent evidence is retained under the Agent Fabric Convergence initiative:
`.prometheus/cadence/artifacts/customer-public-mac-2.2.25/executive-representation-service-recovered-0b3a9919-6b5f-4e22-9804-dee6b1a39826/evidence.json`.
The exact owned application error log at 2026-10-09 17:48:32 records a 422 for
`POST /api/v1/collaboration/representation-grants`. The desktop adapter discarded
the native response body, so the log does not independently name its validation
error.

The recorded synthetic grant issuer is the verified raw `boss.…` subject. REST
`private_scope` produces the encoded tenant/subject storage key; the previous
catalog method compared issuer against that storage key. Its actual validation
returns `REPRESENTATION_ISSUER_PRINCIPAL_MISMATCH`, mapped to 422. The captured
request and this source path establish the mismatch. The 422 alone does not
establish the separate attachment and turn-execution gaps below.

## Bounded implementation and ownership

The operator explicitly requires a represented turn, disclosure, current grant
consumption, expiry, revocation, offboarding and denied effects. Existing source
saved grants in the catalog but did not attach them to durable instances. Editing
the immutable binding to do so would confuse deployment identity with mutable
private authority. The repair therefore uses the existing instance CAS and turn
kernel, without another scheduler, public endpoint, dependency or database table.

1. REST/MCP issuer validation uses the verified subject separately from the
   unchanged tenant-scoped storage partition. The trusted run-input seam retains
   that subject for audience validation; model payloads cannot supply it.
2. Existing trusted REST issuance attaches current revision/digest references
   when the grantee resolves to a durable instance in the same owner/workspace.
   Existing service/team catalog records retain catalog-only behavior. Attachment
   validates current catalog revision, is idempotent, and cannot roll back to an
   older command. A partial save is reported, not disguised as success.
3. Private instance authority has its own monotonic revision. Epoch/attempt
   fences also require this exact authority. Attached references remain after
   expiry/revocation; missing/currently denied grants cannot become unrestricted
   ordinary execution. In-flight invalidated roots use existing owned cancellation.
4. The ordinary durable turn endpoint remains
   `POST /api/uar/agent-instances/v1/{id}/turns`, body
   `{commandId, prompt}`. It returns the existing command receipt, not a fabricated
   completed instance. The Boss owns its typed IPC and translated controls.
5. Current catalog grants restrict admission, tool preparation/claim and final
   output. Cedar, human approvals and readonly effect classification remain
   authoritative. A new attached grant does not grant native tools or roots.
6. Fresh represented instances may enter the existing empty-history path. Real
   stored conversation or checkpoint import still denies admission. Instance
   projection includes only `representationRevision` and `representationGrantRefs`.

## Completed-boundary operation

No compiler, build, test suite or application operation was run during these
production edits. Runtime and desktop owners finish wiring first; the parent
owns the serialized native build, `pnpm build:mac:arm64`, packaged launch and
affected customer operation. Source completion is not qualification.

Use an isolated application profile, synthetic identity/evidence references,
disposable workspace and actual configured model. Native startup configuration
must register `file_read` and limit paths to that disposable root. Existing
namespace persistence does not prove the running native registry refreshed.
The exact grant scope is `tool:file_read`, `workspace:WORKSPACE_ID`, and
`user:AUTHENTICATED_SUBJECT`, with no knowledge-base scope. Operate the real turn,
inspect disclosure and correlated result, then expiry/revoke/offboard and denied
effects. Create no external issue or other production mutation.

Legacy authority records remain readable. Catalog-only MCP installation does
not assert durable attachment. Historical sessions cannot be represented by
silently importing their prior contents. Person-specific fidelity, real
organizational authority and broad C17 qualification remain outside this repair.

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
7. Normal The Boss launch rewrites its generated sidecar YAML to an empty object.
   Existing authenticated native-tools settings were saved to the database but
   registration consumed only startup YAML/defaults before SettingsManager
   bootstrap. This explicit supported-configuration defect is repaired by
   registering builtins after settings initialization, overlaying the existing
   fourteen persisted native-tool fields on configured defaults. No category is
   implicitly enabled. The normal settings-save/restart path now applies its
   configured allowlist; live registry hot reload remains unsupported.

## Completed-boundary operation

No compiler, build, test suite or application operation was run during these
production edits. Runtime and desktop owners finish wiring first; the parent
owns the serialized native build, `pnpm build:mac:arm64`, packaged launch and
affected customer operation. Source completion is not qualification.

Use an isolated application profile, synthetic identity/evidence references,
disposable workspace and actual configured model. Native startup configuration
must register `file_read` and limit paths to that disposable root. Save the
existing native-tools namespace through authenticated typed settings and restart
UAR; registration now consumes those persisted values. Saving without restart
does not prove a live registry changed.
The exact grant scope is `tool:file_read`, `workspace:WORKSPACE_ID`, and
`user:AUTHENTICATED_SUBJECT`, with no knowledge-base scope. Operate the real turn,
inspect disclosure and correlated result, then expiry/revoke/offboard and denied
effects. Create no external issue or other production mutation.

Legacy authority records remain readable. Catalog-only MCP installation does
not assert durable attachment. Historical sessions cannot be represented by
silently importing their prior contents. Person-specific fidelity, real
organizational authority and broad C17 qualification remain outside this repair.

## 2026-10-10 — Actual actor context-capacity failure

The packaged Boss `ff98af813c59e7c2733db74306ebfe087fec069a` with UAR
`f55e6cf1dd0f2864b4426a614a2e8bc4dea42400` passed native model preparation,
role installation and exact scoped grant attachment. Its first represented turn
became uncertain with the generic diagnostic `actor_kernel_failed` before any
reported approval or completed read. The evidence remains in the initiative at
`.prometheus/cadence/artifacts/customer-corrected-mac-2.2.27/synthetic-representation-2-2-27-15d559d1-b512-44dd-8ca1-06d7608be1bc/`.

The retained owned profile's persisted `AgentThreadResult::Failed` identifies the
actual first kernel error: `world_state_budget_exceeded`. Its bounded technical
message reports 11,553 reserved world-state tokens versus an 8,192-token context
limit. Offline inspection correlated the exact run
`38ba8275-c35f-404c-b918-9a76ca77e092`, attempt
`352792e0-2c9e-41a6-a684-01338cf92ab5`, and command
`6b0df277-b45b-4f36-8062-1a36ed536e7b`. No raw database content, prompts,
credentials or grant bodies are included here. The uncertain turn was not replayed.
The inspected retained SST (`00000000000000000002.sst`) has SHA256
`c98a257ea94ea55b3b9451a6ab088ae4b2d9f5e39cf244ed3177f6912549d6df`;
its raw bytes remain outside version control in the isolated profile.

The ordinary actor looked up capacity under its synthetic gateway provider. Its
explicit registry limit was absent, and that provider has no embedded catalog
entry, so it used the existing 8,192 fallback. The captured, administrator-bound
`catalog_pricing_model` already retains the alias's underlying catalog identity;
the context lookup failed to consult it. The existing source-backed catalog has
`openai/gpt-6.1-sol` with 1,050,000 context tokens; its provenance is recorded in
`openspec/changes/provider-catalog-gpt61-sol/evidence.md`.

The surgical repair adds that captured identity as the final catalog lookup
after explicit host capacity, explicit registry capacity and the endpoint's own
catalog identity. It neither changes the selected route nor raises the generic
fallback, removes world-state reservation, expands authority, or promises gateway
fit. A narrower operator-configured gateway capacity remains authoritative.
The known static `world_state_budget_exceeded` code is also retained by the
durable pump rather than being collapsed into `actor_kernel_failed`.

The actual local Codex model cache separately advertises 272,000 context tokens
for `gpt-6.1-sol`: client `0.162.0`, fetched `2026-10-10T02:09:17.941282Z`,
SHA256 `e4a459b1d0b550d89a41a26863f33265116f5fd971e90a63355e241fd449a1a5`.
Only those non-secret metadata fields were projected. The local Codex source at
`986ff1cc7ced0081ec5014b700a376333d87f869` does not yet include this exact model
in `codex-rs/models-manager/models.json`; the live cache is the evidence for the
smaller subscription-route limit. Apply that narrower value through the existing
native provider model `context_window` configuration for the selected route;
it must not be replaced by the API catalog's larger window.

No compiler, suite, build, application launch, or uncertain-turn replay ran during
this diagnosis or repair. The parent owns the completed corrective candidate and
the failed-only represented operation. Source repair does not qualify C17.

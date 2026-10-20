# Production acceptance contract

Status: SPECIFIED, NOT EXECUTED. Phase/change: `uar-outstanding-issues-v1-1-1`.
This document defines future gates. No runner, fixture, manifest, result or product
claim is created by writing this contract. Stable task IDs are in `tasks.md`.

## Current function-first contract — 2026-10-17

This amendment controls current certification; earlier repair-cohort names and
receipts below remain historical. The active child plan and scenario disposition
select all six reduced local groups, four unchanged native supplements, and
exactly `streams`, `recovery` on each published child. The intended local inventory
is 65 required labels after assertion-preserving folds. Its execution remains
pending; no reduced program has run. The gate is **CLOSED**.

The published reduction withdraws direct per-child re-observation of budget
refusal, compiler restriction, credential migration and MCP failure handling.
Those are local release blockers. Both published children still require prompt,
explicit skill activation, exact answer equality, stream/tool completion, durable
owner-qualified restart and genuine qualified supervisor recovery. Timing requires
an explicit operator release disposition and supplies no external-model guarantee.

FC1–FC5 finish the complete revised implementation and documentation, reconcile
parent and child ledgers and record exact source/fixture identity. FC6 then needs
separate named instructions for candidate build/inspection, real qualified
controller staging and the physical guest boot/replacement sequence. FC7 needs
explicit completed-artifact review authorization. FC8 needs the exact local gate
instruction. Parent P1/P2/P3 retain publication/child-evidence/handoff ownership.
Any implementation edit closes the gate and invalidates old source/image-bound
preparation. No standalone preflight is implied by approval of another command.

The function-first manifest fields are `image_preparation: "prebuilt"`,
`prepared_image_id` and `image_preparation_receipt: {path,digest}`. Function-first
commands use this closed mode. The runner must inspect and use the already
prepared image, refusing a mismatch without build/pull fallback. Local preparation
uses the production Dockerfile; published preparation pulls the actual immutable
child after P1. Each has its own image receipt and fresh same-image supervisor
identity. Image identity does not qualify a controller. Controller source,
mechanism and IDs, real journal registration, queued/active predecessor work,
distinct replacement fence and an actual physical boot are independently required.
Each recovery invocation consumes fresh proof; old rc04/rc05 receipts and consumed
proofs cannot be relabelled or replayed.

FC5 fills local selection/configuration and private paths; its manifest remains
non-executable until FC6 supplies actual image/proof bindings. Published manifests
remain templates until P1 and separate preparation for each architecture. The
runner/schema/runbook source changes carry no acceptance credit before the
separately authorized preparation and functional commands.

## Full-phase boundary

Before any check, build, compiler probe, formatter, test, schema validation or
isolated artifact review, finish every production behavior, migration, fixture,
configuration and required document in the complete parent ledger and child
FC1–FC5, including all subsequent repair cohorts. FC5 records the renewed
implementation-complete inventory and exact source/fixture identity; L4 and later
historical inventories are not permission or current-byte acceptance.
Uncertainty or a scope revision keeps that boundary closed. Fixtures may be
authored during implementation, as explicitly authorized by the operator;
we do not execute them early. This overrides the generic KBD acceptance template's
different fixture-authoring order and all weaker per-change testing instructions.

For the revised child, FC5 inventory and separate FC6 preparation precede
explicitly authorized FC7 review and FC8 local acceptance credited to parent V2. Fix an observed failure and rerun only its affected gate, preserving
failed and interrupted receipts. Missing prerequisites or unexecuted required
observations are BLOCKED. Exit 2 is never a pass. An exit 0 without the declared
positive and negative observations is not acceptance. No product gate runs in
GitHub Actions; `.github/workflows/deploy.yml` only builds/publishes artifacts.

## Exact selected function-first runner interface

Working directory for every invocation:
`/Volumes/T7/Projects/prometheus/universal-agent-runtime`.

```text
python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Volumes/T7/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/function-first-local.json
python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Volumes/T7/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/function-first-arm64.json
python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Volumes/T7/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/function-first-amd64.json
```

The fresh manifests and receipts are pending. The runner supports selecting exactly
one failed group for a repair rerun; that receipt does not replace the full
program inventory. Do not expose arbitrary shell execution through a manifest:
group names select a closed inventory of implemented operations.

### Manifest contract

| Field | Required value/meaning |
| --- | --- |
| `schema_version` | Integer 1; unknown versions rejected. |
| `phase`, `version` | `uar-outstanding-issues-v1-1-1`, `1.1.1`. |
| `stage` | `local` or `published`; no implicit selection. |
| `source_commit`, `source_tree`, `fixture_tree` | Exact source/fixture identities recorded when the complete candidate is frozen; local uncommitted candidate requires an explicit tree snapshot identity. Published requires the tag commit and unchanged fixture sources. |
| `image`, `platform` | Local explicitly labelled tag/image ID, or registry child `@sha256:` digest; `linux/arm64` or `linux/amd64`. Published mode forbids mutable tags. |
| `index_digest` | Required for published mode; proves child membership in the released index. |
| `groups` | Local exact six groups below; published exact streams/recovery; or one failed/blocked/interrupted group from that stage program with a hashed parent receipt. Empty pre-build groups cannot authorize repair selection. |
| `resource_root`, `evidence_root` | Absolute new private scratch and receipt directories, unique to this invocation, owned by the runner. |
| `infrastructure_file` | Absolute private file describing isolated actual DBs, volumes, endpoint credentials and disposable namespaces; public manifests store only its path and redacted identity. No live customer DB/namespace is permitted. |
| `implementation_inventory` | Absolute renewed FC5 whole-phase inventory and digest; it must declare `implementation_complete: true` and `implementation_boundary: "complete"` with exact source/fixture trees. Source mismatch refuses execution. Readiness is separate from operator authorization. |
| `image_preparation`, `prepared_image_id`, `image_preparation_receipt` | Closed prebuilt mode, actual inspected image ID and hashed preparation receipt containing schema version, image, platform, source/fixture trees and image ID. No build/pull fallback. |

No placeholder digest or invented source SHA is allowed in an executable
manifest. L3 creates schemas/templates, while concrete manifests are populated
with observed candidate/publication identities during V2/P2. Published mode may
start only the immutable image as UAR; it may not build UAR, mount a replacement
binary, patch source in the image or credit native-source tests as image evidence.
Fixture/client programs may run externally, with their exact revision recorded.

### Shared isolation and evidence fields

These fields apply to every behavior table below and every domain acceptance
fragment, without weakening their specific controls.

- Source identity: actual source commit/tree, fixture tree, runtime image/config
  digest, profile and pins; baseline `91a091d647fe2ce63651c07140172aba2c6c6302`
  is historical input, not the eventual accepted candidate.
- Real collaborators: shipped UAR server, actual SurrealKV/SurrealDB/PostgreSQL
  where applicable, authenticated protocol clients, independently running HTTP
  provider and observable MCP peer processes. Controlled wire responses qualify
  UAR protocol handling, not genuine external-model quality or deployment health.
- Isolation: create one private run root and container network; allocate ports,
  database namespaces/databases, queues, volumes and process identities under it.
  Child-process home/data/cache settings point to runner-owned scratch paths;
  do not change the user's global environment or use their credential stores.
  One lead owns cleanup and joins all owned children before storage readback.
  Preserve redacted evidence; remove only resources in the owned manifest.
- Prerequisites: FC5 whole-phase completion, authorized FC6 preparation and FC7
  review dispositions, exact FC8 operator instruction, pinned dependency/fixture
  sources available, Python/Docker and required protocol clients, real reachable
  isolated DBs, volume durability established before attesting the positive
  profile, no conflicting build writer, and controlled fixture authority.
- Evidence: append under `evidence/acceptance/<run-id>/<group>/`; retain exact
  command/argv, sanitized effective config, timestamps, exit status, required
  scenario IDs/counts, positive/negative observations, peer contact counts, event
  transcripts, persistence readback and redaction/receipt hashes. Record started,
  failed, blocked, interrupted and passed distinctly. Do not rewrite old receipts.
- Limitations: isolated fixtures do not establish know-me backing durability,
  live-agent contents, actual recovery, native amd64 hardware when emulated,
  downstream Kubo integration, or external Dot qualification.

## Group-to-production behavior map

Every domain fragment linked below is part of this contract. Its final local gate
means the exact invocation above with the named manifest/group, never an
independently authorized per-task test.

| Group / task IDs | Production entry point and boundary | Observable result and negative control | Backend / evidence |
| --- | --- | --- | --- |
| `binding` / R1,S1 | Authenticated POST `/api/uar/runs`, run stream/status, provider HTTP peer and durable accounting | Invalid budgets, unsupported/unattested store, missing accounting principal and client-construction refusal have safe category/reference with zero provider/tool effects. Qualified budgeted control succeeds, preserves timeout/retry inheritance and liabilities across restart. Inject secret-bearing constructor failures and observe no leakage. | Actual local durable + remote Surreal; secret-safe logs/events, contact counts, persisted status/accounting. |
| `streams` / R2,R3 | Public run admission and stream; native Anthropic `/v1/messages`, ordinary and graph/fallback execution | Empty/metadata/text/partial-tool/complete-tool EOF without terminal fails; cancellation and cleanup retain precedence; valid text/tool loops succeed. The function-first journey proves exact prompt/skill-dependent public and persisted output. Retain representative consequential EOF, malformed-data, overflow, UTF-8, post-terminal and valid unknown-event outcomes; exhaustive partition/field permutations remain ordinary regression coverage. | Actual provider wire peer + persistence; normalized sequence equivalence, status/history and no pending tool dispatch. |
| `mcp-outcomes` / R4 | Runtime model-driven tool dispatch through real MCP transport, admission and receipt retrieval | true/false/omitted isError, protocol errors, timeout and uncertain effects retain typed outcomes and payload semantics. Zero automatic replay on application failure/uncertainty; later distinct success leaves original receipt intact; native incidental isError has native semantics. | Actual MCP peer call/reconnect trace plus durable receipt cold readback. |
| `credentials` / T1,T2,T3,S3 | Verified credential HTTP CRUD/principal/preview/migration and actual run provider use | Issuer/tenant/local-installation isolation; missing shared identity, absent local ID, denied house fallback, service absence, store/decrypt errors, opaque supplied drivers and inherited/fallback clients cannot bypass policy. Exact grants and owned credentials succeed. Preview/copy/journal tests include drift, collisions, retries, rotated/deleted destinations, wrong owner and restart. | Memory contract supplement plus actual SurrealKV/shared Surreal/PostgreSQL. Each backend separately counted; unavailable or skipped backend blocks its requirement. |
| `recovery` / S2,S4 | Public owner-qualified KB create/upload/job/retrieve/run APIs and existing authenticated collaboration recovery APIs | Unattested upload rejects before 202; qualified upload durably commits bytes/job, restarts and retrieves under exact owner. Legacy rows unchanged; other owner denied. Team reclaim needs exact valid joined/exclusion proof; absent/forged/replayed proof denies without epoch/effect changes. | Actual DB/process restart, isolated supervisor proof fixture, KB inventory and tool/accounting uncertainty observations. No customer mutation. |
| `combined` / R5,T4,S4 | One installed runtime run crossing provider SSE, failed MCP result, event projection and durable state | A failed MCP result followed by incomplete Anthropic stream cannot settle successful or erase the failed tool receipt. A separate valid control succeeds. | Real runtime/provider/MCP/storage, invocation counts, final run and immutable receipts. |

Detailed domain contracts:

- [Runtime](../../../.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/spec/runtime-contracts.md).
- [Credential authority](../../../.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/spec/trust-contracts.md).
- [State and recovery](../../../.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/spec/state-contracts.md).

## R11 compiler profile restriction and renewed boundary

The active R11 repair cohort adds the `compiler-authoring-isolation` spec delta.
The `credentials` group invokes the compiler profile adapter against the local
installed candidate. The current published two-group program does not reobserve it. Two distinct admitted remote
principals and local shared-mode principals must receive the same safe 403
profile denial on both compiler REST aliases, every registered method/path,
spoofed headers and malformed request bodies, before storage or session effects.
Retained specs, reports, sessions and registered artifacts must remain unchanged
across denied requests and restart. A local single-owner control must still
read and operate on those resources. This qualifies the enforced profile limit,
not full owner-scoped authoring or #387 closure.

Historical R11 boundary: L9 superseded earlier source inventories for R11 production, fixture,
configuration and required documentation. No candidate build, source supplement,
installed-image group, or other verification may run until L9 records the
entire then-revised implementation complete with exact source and fixture identity.
The current child supersedes this historical threshold with FC5–FC8 and explicit
operator instructions; L9 does not authorize current commands.
The terminal local04 receipt and blocked Docker observations remain historical;
the next real program must establish current Docker availability and image
identity rather than crediting a timeout as a product result.

## Explicit metadata upgrade acceptance

The `binding` group additionally covers the confirmed second #385 failure:
eligible durable accounting with a 16,000-token envelope refuses the actual
catalog context-plus-output reservation before provider contact. Retain the
existing public per-call error and observe the safe ERROR stage/category,
accounting cause, reservation/limit values and reference in trusted logs.
Persist a bounded agent on an unattested store, restart and observe the new safe
startup warning; per-run refusal remains authoritative. Unsupported storage is
the actual in-memory source provider, not PostgreSQL, which supports accounting.
The adequate-envelope positive control and inherited timeout/retry control remain.

The `credentials` group also invokes the NEW local host command
`universal-agent-runtime credentials initialize-revisions --writers-quiesced`
inside the local candidate image against each supported disposable store.
The function-first published streams/recovery program does not repeat credential
metadata initialization; it supplies no per-child migration/upgrade acceptance.
Observe missing-acknowledgement refusal with unchanged rows; under documented
quiescence observe assigned missing revisions, preserved credential data/owners
and receipts, repeat idempotence and cold readback. Verify no listener, provider
or MCP contact. Acknowledgement is not evidence that customer writers stopped.
Ordinary startup and preview must leave missing revisions uninitialized.

## Source and image coverage are separate

The operator's Execute instruction on 2026-10-16 supersedes the broad source
validation plan. Run only real integration cases for this phase's known binding,
streaming, MCP outcome, credential and recovery scenarios. The six groups above
are the complete local acceptance scope; current published scope is exclusively
streams/recovery as amended above. Existing target fixtures may be reused for
those cases; do not run their unrelated cases, unit/mock-only checks, the historic
full Kubo program, routine validators, lint or a feature matrix. Internal SDK
seams still need real in-process integration with actual provider/storage
collaborators; a helper assertion cannot replace that observation. Concrete
case argv and receipts must be registered from the implemented fixtures before
the renewed FC5 boundary rather than guessed here.

L3 registers the following exact real source supplements in the local runner:

| Group | Cargo integration target | Exact case |
| --- | --- | --- |
| binding | bauar_full_harness_cursor | phase_runtime_binding_refusals_use_real_host_store_and_provider |
| mcp-outcomes | bauar_full_harness_cursor | phase_runtime_native_incidental_error_retains_success |
| credentials | credentials_api_integration_test | phase_native_sdk_supplied_credentials_use_real_provider_and_durable_store |
| credentials | credentials_api_integration_test | phase_memory_credential_migration_uses_real_http_and_atomic_store |

Each invocation is `cargo test --locked --no-default-features --features
server-full,postgres-backend,in-memory-backend --test <target> <case> -- --exact
--test-threads=1`, with a single lead-owned target directory, owned resource and
evidence roots, and `UAR_LOCAL_ONLY=1` for constructor refusal. Require exactly
one actual passing case. These are real host/auth/provider/store integrations,
supplemental to image observations, not unrelated whole-target runs. Memory has
no durable historical initialization: its public writes assign revisions. The
durable-backend adapter covers historical backfill and raw fixed-local sources.

Separately authorized preparation builds the candidate through the repository
`Dockerfile` using the existing release profile (`server-full,postgres-backend`).
The local functional command uses that unchanged prepared image for all six groups. No production runtime source
patch is allowed during this image run. Source tests can cover in-memory/internal
SDK boundaries unavailable as public image surfaces; the image group must still
exercise every corresponding shipped observable via its real public routes.

Published arm64 and amd64 invocations run streams/recovery on each separately
prepared immutable child, retaining supported Surreal/PostgreSQL recovery.
Credential migration, budget refusal, compiler restriction and MCP failure
handling are local-candidate evidence only.
They do not rebuild UAR or rerun source-only tests as substitute evidence. The
runner verifies platform identity and records native/emulated execution.
Companion DB/peer images need not use the runtime's architecture; do not force
unavailable companion platforms. No new fixture patch between candidate and
published acceptance is allowed without recording it and rerunning affected
candidate evidence against the frozen fixture revision.

No routine OpenSpec/schema validator is selected as product acceptance under
the operator's phase-only integration instruction. Required independent artifact
review requires the FC5/FC6 boundary and explicit FC7 authorization. No workflow/build-script edit is selected. If one becomes
necessary, resolve its mandatory policy-validator rule before editing; completion
of an unrelated task is not permission to run an early check.

## Release and issue handoff acceptance

| Field | Required declaration |
| --- | --- |
| Canonical identity | P1 publish, P2 published acceptance, P3 release/issue handoff. |
| Source identity | Frozen accepted commit, annotated immutable v1.1.1 tag, workflow commit/run, index and both child digests. |
| Production entry point | Existing deployment workflow publishes GHCR; local runner pulls immutable children; GitHub Release and issue comments link immutable receipts. |
| Real collaborators | Git repository, GHCR, build/publish workflow, local Docker runtime, actual DBs/protocol peers, GitHub Release/issues. |
| Boundary exercised | Source-to-artifact provenance, registry pull, both runtime architectures and external handoff. |
| Observable result | Both published streams/recovery programs pass; release assets contain exact digests, setup/upgrade instructions, compatibility and limits; issue dispositions match each criterion. |
| Negative control | Wrong child/source/platform, changed fixtures, missing required group or failed published behavior withholds success and never moves a published tag. |
| Isolation | Same private per-invocation resources; credentials remain external/private, and GitHub publication uses the authorized repository only. |
| Prerequisites | V1/V2 accepted, immutable source/tag, deployment workflow authorization from release request, both published children available; customer mutation requires separate authority. |
| Final local gate | Exact arm64.json then amd64.json invocations above, serially; source-only and local-tag receipts cannot substitute. |
| Evidence | `evidence/publication.json`, per-architecture acceptance receipts, release asset hashes/URL, issue comment URLs and readback identity; all pending. |
| Limitations | #382 external dependencies remain open; #385 product repair and actual customer recovery are separate. v1.1.0 remains immutable. |

## Corrective freeze and publication identity

Round1 findings reclose implementation until R7 and renewed L5. Team binding must exercise distinct-provider fallback allowed/denied controls; ordinary fallback must prove disabled ungranted entries remain inert. Secret absence must inspect raw output transiently and retain only safe booleans/redacted logs. Published manifests require a hashed publication_receipt linking the remote annotated tag commit, successful deploy.yml push run/attempt, actual architecture digest-export artifact IDs and both children to the registry index. Missing or expired digest exports block published acceptance. See docs/operations/uar-1.1.1-acceptance.md for exact fields.

# Retained supervisor qualification handoff

Date: 2026-10-16. Phase: `uar-outstanding-issues-v1-1-1`, Execute task `6.2`.
This is a records-only handoff. The waypoint read for this handoff was revision
3029, plan revision 61. It names retention of the reviewed qualification, fresh
same-artifact predecessor/replacement staging, then affected approved acceptance
after prerequisites pass. This handoff does not execute that sequence.

## Approval and authority

The operator retention receipt records the exact authority **“Approve this scoped
qualification record”** for `qualification-retained.json`, SHA-256
`0a3710de687bf1bb3f02ca19ab45438082c946f9dd8ad9a6e10d603e182e5cd5`.
Its scope is isolated Linux arm64 native boot exclusion and exact team reclaim
for UAR 1.1.1 rc04; it supplies no other platform or mechanism coverage and does
not authorize publication. The receipt is
`evidence/supervisor-operator-retention-20261016.json`, recorded at
`2026-10-16T22:30:40.321Z`.

`evidence/supervisor-native-evidence-review-20261016.json` records independent
retained-evidence review **PASS**, with no concrete blockers. Review alone did
not adopt the qualification or change services. The later operator receipt
resolves the earlier pending operator retention decision recorded in
`coordination/handoffs/direct-continuation-20261016.md` and
`evidence/direct-continuation-20261016.json`. It does not establish normal
controller selection/startup, fresh staging, or completed acceptance. The review
receipt names `native-proof-review.json`; the phase-retained review is the
`supervisor-native-evidence-review-20261016.json` record above.

Phase Exit development is stopped. Its tool adoption and decisions are removed
as prerequisites for this release. Preserve approved revision3 and the existing
tool evidence as history. Future lessons are already recorded separately at
`docs/research/uar-first-example-lessons-20261016.md` in the Phase Exit repository;
no further Phase Exit work is required here. Existing KBD/OpenSpec criteria and
the exact acceptance program remain authoritative.

## Retained artifact bindings

These are retained identifiers, not freshly recomputed or read from a guest:

| Binding | Retained value |
| --- | --- |
| Image | `uar-phase:1.1.1-rc20261016-04` |
| Image ID | `sha256:d221cda1aacc969b0e455f69a9894fdf171bc0f33f23d5922c3f9690385775fe` |
| Source tree | `sha256:3664d2351c10b19b0a8267ca8688daa4353c0ef133345c8e3426f3c2e4fedc4e` |
| Fixture tree | `sha256:cb4e13ce07768d82c550907c4eff7151eecd4bb63d462a338b2df3211c464f81` |
| Platform | `linux/arm64` |
| Exported/installed executable SHA-256 | `1aca3bb28ff1e345b9c76c6e6922b9664a3efe80b966bd9c184361ec4b058d95` |
| Controller ID | `uar111-native-controller` |
| Candidate qualification ID | `dot17-candidate-uar111-native` |
| Mechanism | `native-boot-exclusion-v1` |
| Reviewed controller digest | `7f6b629276b8a17dc0726522a8eaa897c4c07c59b01934471eafc8cf66c57cb5` |
| Reviewed candidate qualification digest | `903ee54ef29e089970ae6cc85e82ab6e2558a2606db6e91755c6a3c4b11301e6` |
| Reviewed evidence bundle digest | `9cd2dfd052e1d67e5724a1c89b48e9cef827dd78b54d31247c3129bbc82b46fd` |

The direct-continuation record retains source/fixture inventory06 unchanged.
The review observed local configuration selecting
`/etc/uar-supervisor/qualification-candidate.json` with the candidate digest
above. The approved retained record has its own digest; approval is not evidence
that the normal controller now selects it.

## What the native candidate proof establishes

`evidence/supervisor-native-candidate-20261016.json` records
`physical_qualification_passed=true` and `production_qualified=false`. Retained
build and physical-proof terminal observations each report exit code 0. The
independent review found matching receipt digests and consistent image,
source/fixture, executable, controller, and qualification bindings across the
retained records.

Before forced stop, API readback recorded queued attempt
`1d1dc34e-8bf0-4da7-b322-020c3fb4fa0e` and running attempt
`910ff4b3-8fc0-47b3-b233-b71727d0a680` at the exact predecessor fence. Retained
`limactl stop --force` exits 0 and records QEMU SIGKILL. The same machine ID
`bac25da7bc5d4d5d84b12fa478b4d465` changed boot ID from
`de0c84b1-c5ea-4d30-873f-429d7a732767` to
`37eab96d-6164-459b-8378-7e099e6c14ef`, matched by controller exclusion.

The exact reclaim receipt preserves catalog `uar-collaboration-catalog` and
service instance `2cc61c0d-b3f8-44e3-8ef8-59236cb04b06`, advancing predecessor
incarnation `429fc0ce-bd03-4501-84c9-a959437da5ef`, epoch 1, to replacement
incarnation `78b4e5cb-dc6a-4a01-983a-642fe5fed6f5`, epoch 2. Only the staged
queued attempt is listed as transferred; only the running attempt is listed as
uncertain. The queued readback retains its original admission fence; the reclaim
receipt proves authorization for transfer, not a rewritten admission fence or
later dispatch.

Committed totals remain zero tokens, cost microunits, and elapsed seconds before
and after reclaim. Reserved totals remain 2,000,000 tokens, 2,000,000 cost
microunits, and 6,000 elapsed seconds. The active attempt retains
`effectDisposition=uncertain` and `accountingState=reserved-unknown`; exclusion
reports `effects_reconciled=false` and `execution_resumed=false`.

Retained negative responses are absent/forged proof 409, ordinary challenge and
exclusion 403, wrong incarnation 409, replayed challenge and proof 409. Reviewed
snapshots remain equal around the recorded refusal/replay operations. Original
staging, missing-QEMU start, and ordinary-challenge failures remain preserved.
The ordinary-denial fixture initially reused the trusted subject; the retained
repair attests a distinct ungranted subject and the retry returns 403.

## Limits that remain material

The uncomfortable limit is that successful candidate bootstrap proof and scoped
operator retention do not certify the normal qualified-controller program or
final acceptance. Only same-image arm64 qualification was observed. The review
compared retained text digests for internal integrity; it did not authenticate
prior host/guest actions, recompute source/fixture fingerprints, reread guest
controller/executable bytes, or verify current process selection. No live guest,
runtime, network, model, service, credential, certificate, or key verification
was performed by that review or this handoff.

No standalone heartbeat time series is retained. Provider trace uses
`mode=complete` despite preparation installing a hold fixture; the pre-stop
running status is observed, but a 601-second hold or completion is not proven.
The trace is fixture-peer I/O, not paid-model or completed peer-tool evidence.
No separate wrong-epoch-only negative was observed; the malformed-fence negative
changes incarnation. The repaired subject is an attestation because credentials
were not inspected. Later queued dispatch/completion, exactly-once effects,
settled active accounting, other platforms, and publication remain unverified.

## Required next acceptance sequence

1. Retain/install and select the approved scoped qualification through the
   existing runbook, recording actual normal-controller selection/startup.
   Stage fresh same-artifact predecessor and replacement work against the bound
   rc04 artifact. Use fresh registration, fence, attempt, and challenge identities;
   do not reuse the consumed candidate challenge, proof, predecessor/replacement
   fences, or attempt IDs recorded above. Final acceptance needs its own scoped
   state and a guest-only reboot, with no production know-me mutations.
2. After all existing prerequisites and the full-phase implementation boundary
   hold, run the affected approved local program serially under the lead's sole
   build-target ownership. Preserve passing gates and repeat only affected failed
   gates. The complete image program remains the six groups `binding`, `streams`,
   `mcp-outcomes`, `credentials`, `recovery`, and `combined`.
3. Include the four exact real-source supplements registered in
   `openspec/changes/uar-outstanding-issues-v1-1-1/verification.md`:

   | Group | Integration target | Exact case |
   | --- | --- | --- |
   | `binding` | `bauar_full_harness_cursor` | `phase_runtime_binding_refusals_use_real_host_store_and_provider` |
   | `mcp-outcomes` | `bauar_full_harness_cursor` | `phase_runtime_native_incidental_error_retains_success` |
   | `credentials` | `credentials_api_integration_test` | `phase_native_sdk_supplied_credentials_use_real_provider_and_durable_store` |
   | `credentials` | `credentials_api_integration_test` | `phase_memory_credential_migration_uses_real_http_and_atomic_store` |

   These supplement actual image observations. They do not replace the image
   program or authorize unrelated source-target checks.
4. Publication follows all required local gates and the existing immutable
   source/tag and deployment requirements. Then pull the published arm64 child
   digest and run its entire six-group program locally; follow with the amd64
   child digest and its entire program, serially, recording exact
   source/fixture/image/platform identity and native/emulated execution. No
   replacement executable or source-only result substitutes. Complete release,
   setup/compatibility, and criterion-specific issue receipts before claiming
   release completion. The scoped approval above does not authorize publication.

No local04 full program was started in the direct-continuation evidence.
Normal-controller adoption, fresh staging, and final local/published acceptance
have no completion evidence in the records read for this handoff.

Dot architecture feedback is expected and has not been received. Do not invent
observations or wait for feedback to record this handoff. If concrete feedback
arrives, map it to existing criteria before any plan change; any authorized
implementation adjustment joins the same complete repair cohort and renewed
inventory before affected verification. The production source review cap remains
exhausted; this handoff performs no UAR source review and adds no requirements.

## Handoff accounting

- Changed file: only this new handoff,
  `coordination/handoffs/supervisor-retained-20261016.md`.
- Evidence used: waypoint; direct-continuation handoff and receipt; native
  candidate, evidence-review, and operator-retention receipts; existing phase
  plan, L3 runner handoff, and OpenSpec acceptance matrix.
- Actions performed: read records and write this handoff. No test, build, probe,
  verification command, external gate, guest contact, or UAR source review was
  run. No tier or phase acceptance is claimed.
- Unresolved decisions: no operator retention decision remains pending in these
  records. Normal-controller installation/selection and actual fresh acceptance
  remain required actions, with no result asserted here. Concrete Dot feedback
  remains absent.
- Unrequested additions: none. New guards: none. All behavioral limits and next
  actions above come from the existing recorded evidence and acceptance scope.
- Unverified behavior: normal qualified-controller startup, current guest/process
  bindings, fresh scoped reclaim, later transferred work execution, settled active
  effects/accounting, the final local six-group/four-supplement program, both
  published child programs, and release/issue completion.

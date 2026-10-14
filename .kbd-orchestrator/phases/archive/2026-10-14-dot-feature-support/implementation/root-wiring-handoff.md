# Root integration handoff — Dot Execute, 2026-10-12

Source authoring only. No product validation, compiler/build, lint, format, test or review gate has run. The full expanded phase boundary remains closed.

## Composition changes

- `src/server.rs`: instantiate ingestion pool as Arc, start durable DB discovery exactly once; use configured explicit CORS origins with empty default and reject wildcard/opaque entries. Preserve sidecar no-Origin boundary. Anthropic model routing requires explicit request or configured default, with no first-provider/catalog-model/default-literal fallback. Authenticated memory context injection and post-stream capture already retain the original UserContext.
- `src/uar/persistence/providers/surreal_host.rs`: initialize ingestion schema on trusted host/network migration path, using ingestion worker's new helper.
- `src/uar/runtime/manager.rs`: when model selection is empty, honor an explicit registry default, otherwise bypass availability routing so missing configuration reaches the central driver's diagnostic. Ignore empty skill-preferred strings.
- `src/uar/settings/manager.rs`: seed no default provider when global model is empty; preserve nonempty legacy syntax and existing persisted settings.
- `sdks/rust/src/runtime.rs`: direct memory get/update/delete now require the same verified UserContext as list/search. Worker-owned admin shims preserve explicit trusted-host operations and scoped ordinary user operations.
- Existing server CORS fixture adapted and negative cases authored; none executed.

## Team ownership

Astra ingestion worker owns knowledge API/domain, ingestion service/worker, new durable ingestion contract/adapters/migrations and fixtures. Shared PersistenceLayer/provider large-file ownership was explicitly released to root after production source authoring, while new ingestion modules remain worker-owned until final handoff.

Astra memory worker owns all memory implementation/native/MCP/admin/HTTP adapters, migration endpoint, docs and new real-store fixture. Root owns SDK callers.

Astra safe-defaults worker owns config/registry/driver defaults, shipped profiles, native merger scripts, docs and fixtures. Root owns server/manager/settings composition. Effective settings is a later separate production task.

## Required continuation

Obtain complete source handoffs before recording production task completion. No handoff is runtime acceptance. Continue remaining production via implementation-sequencing.json. Next shared persistence work is Dot09/10 durable accounting; preserve ingestion registrations. Dot12 controls/consent and future schedule work must reuse the owner-scoped service. Dot18/19 transports follow the existing actor/approval authorities.

The latest direct operator no-testing instruction supersedes pre-edit workflow-policy-validator timing for necessary native merge-script edits. The validator remains required and unrun at the final phase gate; no workflow test policy is relaxed.

The child is canonically closed. Registration receipts and corrected OpenSpec task IDs are in child handoff. Parent Execute is stored in-progress but activation cleared projection stageId; do not re-enter an existing stage or hand-edit projections. Exact next work was recorded via typed revise.


## Continued integration

Root added owner-scoped accounting inspection and trusted-host evidence reconciliation in src/uar/api/accounting.rs, registered the module and /api/uar/accounting route. Scope limits use generation CAS; provider evidence is explicit, not timeout inference. Root added accounting and A2A migration initialization and A2A typed persistence delegate registration in the agreed large-file window. A2A v1 handlers and with_run_api builder are now composed in server.rs.

Root converted thread/service remote reservation/usage/release calls to async host calls after the accountant removed mutation from the kernel facade; existing fixture caller signatures were adapted without execution. Invalid/missing remote usage cannot release reservations, and source diagnostics now say unresolved liability retained. Dot10 remains in progress while destination deployment house-budget semantics are resolved.

Root nested memory controls under /api/memory, registered MemoryPolicyTool, and captures an opaque owner-policy admission before queueing post-stream work. Failure/None never obtains a later replacement grant. All new source remains uncompiled/unverified.

## Source continuation after accounting/memory
Dot10 binding root service is implemented before publication; validation/digest/bind failure releases only the proven never-dispatched reservation. Dot13 controls/capture root integration is implemented. Both source tasks recorded complete, all acceptance remains deferred. Current root task20/2 authors effective-settings API, read-only provider/native/MCP snapshot accessors and live embedder endpoint introspection. New source not compiled. Team ownership is in execute-dispatch.json. Dot14 source fit rejected fixed-offset cron, root added exact already-locked jiff0.2.35 manifest/root lock edge without resolution; no versions.toml edits. Operator approved reboot-bound native recovery fallback, recorded canonical dot11-native-recovery-scope-20261012.

## Continuation 2026-10-12, native recovery scope approved

Dot14/2 source completion is recorded; canonical15/1 and19/1 are in progress. Root wired supervisor registration before executor startup, supervisor host-only API, schedules and followups into existing observer/native/API composition. Dot20/2 source completion is recorded. Current sequencing hash is602264e5ac2e0e8f53b0ba5159f0eaa57d1e42830e9fabb757a0e17f70bfdd53. No product checks have run.

Root now owns Dot16 preparation: new api/assistant_capabilities.rs (actual support predicates, registered native tools, explicit unsupported entry points), canonical receipt outcome read (no raw results/arguments), link from capabilities.rs, server wiring, docs/operations/assistant-capabilities.md and authored tests/fixtures/dot16_capability_inventory.py. Schedule/followup teammate provided exact interfaces and supported() predicates. Receipt provider flag was added by A2A teammate in its shared-file window. Dot16 task is not complete until remaining upstream production handoffs are integrated.

Root corrected governance/runtime_control.rs warning to say Required approvals remain enforced in Off mode. No change to unrelated risk policy. A2A teammate owns AGUI adapter/provider correlation and shared host approval resolver; source handoff is not yet final. Schedule teammate owns followup rules/inbox/quiet hours/docs/fixtures. Supervisor teammate is finishing controller documentation. Next fresh worker should own19/2 MCP elicitation, and after followup handoff a fresh worker should own17/3 complete runner authoring. Neither may execute verification.

Kubernetes qualified recovery now explicitly inhibits kubelet on the affected node and retains its mask until operator recovery; controller runs as a separate host service. This is opt-in, not generic Pod-delete exclusion. User-approved native reboot fallback never reboots automatically, and unknown effects remain held.

## Source boundary still closed — late integration
Dot11,14/2,15/1,16/1 now source-complete canonical tasks, with handoffs underimplementation/. Dot16 native policy.update corrected to unavailable (native tool is read-only); disabled memory native operations now error memory_disabled instead of success/empty. Receipt read required optional immutable full-principal provenance from resolved turn; existing JSON records and partition keys retained, legacy reads require exact live owner.

Server mounts AGUI build_router(config.security.clone()) at /api/uar/ag-ui with RunApiState and MCP interactions builder under/api/uar in same normal authentication layer. A2A teammate owns manager attach and cold A2A cancellation/exclusion integration; MCP teammate owns run-scoped transport interaction and durable disposition. Fresh Astra dot_phase_runner owns17/3 runner/docs and103-entry spec scenario matrix; no scenario executed. Runner inventory shape is being reconciled with source fingerprint. Do not invent or emit production-complete-inventory.json yet.

Currentcanonicalpendingprocess: none except any task transition explicitly noted in active tool session. Fullphase production remaining19/1,19/2,17/3. Root must integrate final packets and recordall23sourcecomplete before any check. No verificationhasrun in this continuation. Preservemain dirtytree; no pull/reset/commit/push performed.

## 2026-10-13 continuation — final runner production entry

Dot19/1 and19/2 handoffs have been recorded source-complete through typed KBD.
The aggregate production count is22/23; Dot17/3 remains implementation-in-progress.
The root authored scripts/dot/supervisor_journey.py and supervisor_probe.py,
plus docs/operations/dot-supervisor-qualification.md. These bind staged native
reboot and CRI exclusion to real private case inputs, pinned SSH observations,
actual controller journal records, exact descendant/boot identities and effect
traces. Controller authority fixtures observe wrong-certificate and
cross-deployment refusal without accepting network failure as authorization.
No current machine reboot, remote action, test, compiler, validator, build or
product review was executed. All drivers remain source-only.

Root also resolved the observed first-qualification startup cycle with an
explicit isolated-candidate controller flag; see the appended Dot11 handoff.
Production startup still requires a qualified record. No receipt promotion is
implemented by that flag. Remaining runner work is teammate-owned calendar,
core scenario mapping and staged recovery fault fixtures. Full-phase inventory
must not be created until the runner implementation is complete.

## Installed governance driver and specification alignment

Root authored scripts/dot/governance_journey.py and
 docs/operations/dot-governance-qualification.md. It composes real local Off/On,
actual non-Required native dispatch, preserved Ask/Deny, external Required
allow/deny/stale decisions, unselected tools and per-process warning observations.
Shared/authenticated profiles require refusal of Off. No fixture was executed.

An actual inconsistency was resolved against parent plan.md:297: the old Dot19
governance spec implied effective-policy denial bypass, while the approved plan
and implementation retain explicit Ask/Deny/current authority. The delta now
matches the approved plan. runtime_control.rs warning vocabulary was corrected
and its existing source-test expectation updated; admission semantics unchanged.
The required non-Required web_search scenario is conditional: no installed native
web_search exists, and all MCP descriptors remain Required. Do not claim that
specific positive dispatch was exercised by the native-system-info substitute.

The user asked for the infrastructure request to be explained in plain language.
The root explained dedicated disposable macOS/Windows native recovery hosts,
a clock-controlled Linux VM and isolated Kubernetes node testing, clarifying
these are later destructive acceptance needs, not prerequisites for finishing
implementation. No machines, access, provisioning, reboot, clock change or
node-stop authorization has yet been supplied in response. Continue source work.

## 2026-10-12 final target and supervisor composition source addendum

Operator goal is code complete with testing pending. Explicit final targets: this machine and LKE context know-me, existing Envoy gateway and TLS HTTPRoutes. Configuration-only kubectl reads found argocd/argocd-gateway, knowme-runtime-https/http, runtime.know-me.tools, Envoy data plane in envoy-gateway-system, and linode-block-storage-retain. No deployment, dry run, rollout/health probe, product gate or disruptive operation was run. Snapshot is configuration evidence only.

Root authored optional chart storageClass.create/existingSecret/image.digest/StatefulSet/per-Pod Services/routes and LKE override values. It binds existing HTTPS and HTTP listeners, uses prefix rewrite for runtime A/B and HTTPS redirect, supports long SSE request lifetimes, and retains the durable database volume. Private image digest, secrets and enabled-memory gate settings remain operator inputs. Root owns docs/operations/dot-validation-targets.md; no new infrastructure was purchased. Shared runner ownership remains with dot_phase_runner.

Root supervisor driver now requires explicit native/Kubernetes mechanism and four actual continuation roles (safe/uncertain checkpoint, safe/uncertain team), exact queued dispatch identities and independent effect-marker counts. Fixed final observation IDs cannot be emitted merely because physical exclusion returned HTTP200. Safe resumed runs must actually finish; safe transferred queued work invokes the real collaboration dispatch API with current team revision; uncertain work stays held. Probe reads marker counts from actual fsynced trace. Authority stage has fixed mechanism-specific ID. The HTTP helper returns actual negative/replay statuses and admitted continuation IDs. All source remains unexecuted.

24production denominator includes12/3 observed full-principal continuity correction. Source-complete inventory must not be created until12/3 and17/3 finish. Testing authorization remains false for the current goal. Compilation, Python/API compatibility, Helm rendering, migrations, source integrations, installed cloud operation, physical exclusion and publication all remain unverified.

## Local candidate input sequencing correction

Source inspection found the initial LKE OCI candidate depended on clean-only release provenance before the precommit product gates could run. Root authored an explicit --candidate-inventory opt-in in scripts/generate-release-input-provenance.mjs. It uses the existing complete/current/authorized inventory boundary and exact source fingerprint, permits draft root inputs while preserving clean pinned submodules and the original version/workspace/schema comparisons, and labels the result local_candidate with no release certification. Its source_tree is null; base Git tree and complete candidate source remain separate. The default released-source path still requires clean repositories. It rechecks source identity before output. No invocation occurred.

Runner owns fixed candidate OCI integration and source-content finalization metadata. This keeps actual tested source and results separate from finalized committed source; content or pin drift invalidates reuse. Published all-nine artifact conformance is still a separate planned boundary. No passing product gates are to be repeated solely because unchanged files were committed. The GitHub Actions policy validator for this workflow-invoked script remains deferred by the operator full-phase timing override and mandatory in the eventual serialized boundary; it was not run or weakened.


## Lead source-binding completion

Before recording the final inventory, the lead included native packaging, SDKs,
shipped root runtime YAML and Compose configurations, and .env.example in
SOURCE_PATHS. Their omission was an observed delivery-source identity gap.
This changes metadata scope only; no runtime or verification stage ran.
All authored behavior remains unverified.

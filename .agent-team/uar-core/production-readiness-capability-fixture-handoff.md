# Focused capability fixture author handoff

2026-10-31. Phase: production-readiness-476-capability-dispatch; Execute.
Worktree: `/Users/gqadonis/.claude/worktrees/production-readiness-476`.
Verification **CLOSED**. No tests/checks/lint/format/import/dry-run/compiler/build/
Docker/readiness/review gate was run or delegated. All cases **NOT_EXERCISED**.
Other source writers share the checkout; their edits were preserved. No commit,
external mutation, provider provisioning, app/service launch or real model call.

## Files authored by this owner

- `scripts/behaviors/features/capability-dispatch.feature`: selected CD01..07,
  CD09 estimated + existing finite-cost policy refusal, CD10; supported alias/
  mode matrix, text/image-only/attachment compatibility, invalid forms and bounds,
  actual capability destination refusal/success, disabled/cooldown/scope/exact/
  fallback behavior, warm history and safe diagnostics. CD08 and public strict-real
  counting are deferred per adopted revision2, without fake executable placeholders.
- `scripts/behaviors/steps/capability-dispatch.steps.mjs`: public HTTP operations,
  actual safe peer assertions, same-comparator hash swap/image-erasure/privacy
  controls, explicit owner-scoped disposable cleanup and exclusive safe receipts.
- `scripts/behaviors/support/capability-fixture.mjs`: explicit external endpoint/
  workspace/credential-free owned fixture manifest; current adopted digest binding;
  keyless exact owned registry/peer assertions; safe content hashes; snapshot
  counters; serial API/peer calls. No startup/admin mutations or credential steps.
- `scripts/behaviors/capability-dispatch.cucumber.mjs`: existing installed runner,
  focused feature/import only, serial, retry0, progress and JSON report.
- `scripts/post_lifecycle/controlled_peer.py`: bounded body-free ordered user/
  system/assistant/tool part summaries, decoded image hash/MIME/detail, text hashes,
  model hashes/label, safe scalar output/temperature, hashed tool/format controls;
  128-record ring with explicit truncation; 256-case config cap; fixed bounded
  response_json control reuses success/SSE path. HTTP chat arrivals count before
  parsing/body limit so fixture rejection cannot fake zero-send. Existing success/
  provider_failure/disconnect/tool/child/embedding behavior remains source-preserved.
- `scripts/behaviors/fixtures/capability-dispatch/generate-png.py`, `synthetic.png`,
  `synthetic.jpeg`, `oversized-dimensions.png`, `README.md`: owned synthetic binary
  assets + pure preparation source/provenance + exact external manifest contract.
  The 6200x6200 RGB PNG is structurally complete/compressed and represents
  115,320,000 decoded RGB bytes; no decoder was executed against it.
- Phase `scenario-to-assertion-map.md`: current contract revision2 digest
  `fdfa9dedf521eea6bb5f3c2383caf161215efbeb2893a749a43ee8db6ca6b3b8`, actual entries,
  expected truth sources, false-positive controls and explicit unsupported cells.

Source identity is the current shared worktree edits over the waypoint base
`629f5702`; this is not a final committed/artifact acceptance identity. Lead
freezes exact committed candidate/container/config/process/fixture identities
only at the whole-phase boundary. No standalone source-identity check was run.

## Source-supported interfaces and ownership

Actual ordinary aliases: `/v1/chat/completions`, `/api/chat/completion`.
Disposable agent creation follows existing GET `/api/agents/default-agent` +
POST `/api/agents`, agent provider policy primary/fallback captures and resource
none-policy. Deletes target only generated IDs. GET `/api/uar/providers` and
`/{id}` inspect safe `ProviderView`; GET `/health` establishes actual cooldown.
DELETE `/api/uar/sessions/{id}` removes only the fresh owner-scoped session.
GET `/api/uar/runs` maps current session to run; GET `/runs/{id}/events` supplies
public attempt/budget diagnostics. The API surface and peer controls were read
from source, not invented from memory.

Runtime source owner supplied existing artifact
`extensions.budgets.max_cost_per_session_usd` (`ThreadBudgets` in
`runtime/thread/policy_intersection.rs`) and current final admission refusal
`UnprovenImageBoundForFiniteAccounting`. The finite-cost case requires real text
success under the SAME finite agent before image zero-send refusal. External
provisioning must supply usable existing price/accounting facts; no specific
public error code/receipt is fabricated. Estimated image case requires a real
nonempty public budget/attempt receipt set, every fit_guarantee false.

The fixture runtime registry must contain only declared keyless generated
providers pointing at its controlled peers, including an outside candidate. This
trust-boundary guard excludes accidental live-model egress. Failure/cooldown
providers/peers must be distinct, not reused across cases. The fallback's own
output ceiling must be <64 and wireSettings nonempty/differing from the primary's
requested64 ceiling. Failover configuration is externally provisioned/frozen;
steps never mutate app configuration, provider settings or credentials.

## Open prerequisites and honest limits

1. Lead/external provisioning supplies explicit owned identity endpoint, selected
   Linux/amd64 image + owned Surreal volume/container/network/process identity,
   all12 fixture model roles and reachable loopback peer control/wire URLs. The
   Python peer still binds127.0.0.1; no speculative container launch/helper was
   authored. Network namespace/proxy must already be externally provisioned.
2. If provider administrative reads are denied by the external endpoint, report
   the existing read-authority/provisioning prerequisite. Do not add admin keys,
   JWTs, logins or authentication work to functional tests.
3. CD08 cold image identity is DEFERRED_NOT_EXERCISED per revision2 to#469/#465.
   Ordinary default RouterNode->AgentNode graph has no CheckpointNode; tool-end/
   approval checkpoint producers require actual governed image tool/approval
   fixture, durable protected envelope and safe current-authority/tamper/single-use
   negative controls plus owned cold lifecycle. No such producer was invented.
4. Public strict-real-image counting is UNSUPPORTED_NOT_EXERCISED. Synthetic exact
   wire contract proves its own JSON framing only. Existing context_history_integrity
   and budget component regression remain lead-owned supplementary gates.
5. Generated namespace providers cannot prove static catalog-overlay reentry.
   Disabled exact dispatch is authored; catalog-overlay cell stays UNVERIFIED.
   Existing external container logs/Rust Debug privacy also need supplementary
   owner/lead observation; HTTP assertions do not claim it.
6. Deterministic completions are transport/admission evidence only. No private
   visual-fact model understanding, provider-wide or release qualification claim.

## Commands actually issued and what they mean

All commands used explicit workdir above. Read-only source inspection used
`cat`, `sed`, `rg`, `head`, `tail`, and one `pwd`; no application/test module was
imported/executed. Python heredocs and shell `cat`/`mkdir` wrote allocated source,
assets and documentation only. Asset-generation commands actually run:

```
python3 scripts/behaviors/fixtures/capability-dispatch/generate-png.py
/usr/bin/sips --setProperty format jpeg scripts/behaviors/fixtures/capability-dispatch/synthetic.png --out scripts/behaviors/fixtures/capability-dispatch/synthetic.jpeg
```

Generator ran again only to replace the oversized negative fixture with a valid
compressed pixel stream and remove a dead authoring line. `sips --help` and
`--helpProperties` were read before conversion; actual conversion output named
only those two owned synthetic files. No install/download/tool probe or image
verification ran. These commands are authorized asset writing, not passing
acceptance evidence. No verification command/output exists to report.

## Handoff boundary

Authoring complete for the selected source-supported cells. Final fixtures are
externally provisioned prerequisites, not evidence of readiness. All cases are
NOT_EXERCISED; all syntax/runtime/decoder/HTTP outcomes remain unverified because
the operator lock is CLOSED. No unsolicited code/guards outside assigned files
were added; every fixture guard traces to wrong-runtime/live-egress, raw-image
leak, missing observation, cross-case cooldown contamination or uncertain cleanup
constraints in the task. Lead collects this with every writer's handoff and all
nine ledger items before independent artifact review and the explicit gate.

## Lead-requested source refinements before whole-phase inventory

Implemented in the same allocated helper/feature/steps/docs only; no verification
or process/network mutation ran. Identity fields now require full40hex committed
source, `sha256:`+64hex local Docker image configuration ID (explicitly distinct
from registry manifest digest), and64hex config/contract digests. The helper
recomputes actual adopted contract content using its declared recursively sorted,
compact UTF8 normalization, preserving arrays and excluding identity.contractDigest
and root digestNormalization; it compares that result against both stamps and
retains the recomputed hash. A stale cached digest cannot certify altered content.
This newly authored hashing path was not executed/imported.

The existing safeUrl boundary now requires literal127.0.0.1 or[::1] for client
identity, peer control and registered wire endpoints. External provisioning must
supply a shared owned container/peer network namespace and local client mapping;
public/private DNS or arbitrary remote HTTPS is refused. Shape/URL checks do not
prove real container ownership; lead still freezes exact independent source,
local image/container/labels/ports/config/process/store receipts. No Docker/config/
launch helper or egress exception was introduced.

Added CD02 source-grounded nonblank/blank convenience precedence cases on both
aliases. Existing ambiguous-message negative still contains supplied images, so
it tests actual discarded-image ambiguity rather than rejecting all dual fields.
Every affected case remains NOT_EXERCISED and the gate remains CLOSED.

## Final item7 operational-template and registered-output handoff

Authoring complete. Added source-backed nonsecret fixture
`runtime-overlay.template.yaml` and `manifest.template.json`, all twelve dedicated
controlled peers4601..4612 with matching actual ProviderConfig/ModelConfig fields,
failover threshold3/cooldown60, own embedded Surreal path and local fastembed assets.
Existing catalog pricing identity openai/gpt-4o-mini supplies accounting provenance
only, with every actual destination still a keyless loopback peer. Generic wire
output field is source-confirmed max_completion_tokens32 for the fallback. No
security/authentication field, custom team profile or temperature setting is
invented. Templates retain explicit namespace/source/image/config/contract
placeholders and are not standalone bootable secure configuration.

Source-read after provider/runtime handoffs confirms that the earlier fallback
provisioning obstacle is resolved: evaluate_destination returns optional registered
max_output_tokens; DestinationAdmission returns AdmittedDestination retaining that
limit; restrict_output min-clamps the typed request cap and any existing contract
limits.output_tokens. The ordinary primary/retry/fallback admission helper and graph
DestinationBudgetDriver both apply it before inner driver/accounting. This scalar
restriction neither installs nor substitutes a synthetic exact profile. CD06's
actual independent wire assertion remains required and NOT_EXERCISED.

Remaining external prerequisites are precise: already-provisioned secure baseline/
verified stable owner/tenant/workspace and provider-read authority; fresh owned
store/settings namespace; exact candidate committed source/local Docker image ID/
Linuxamd64 container/labels/ports/config/process/volume receipts; all owned peers
and scenario client reachable within the externally provided shared loopback
namespace or explicit owned client mapping; actual bundled local assets and
installed client dependency/source mount. SecurityConfig defaults cannot supply
these safely; no JWT/admin setup, secret movement or boundary weakening was added.
README documents each source and owner obligation. Source/format assertions alone
do not prove that an endpoint serves the selected artifact.

The lead reconciles all nine implementation items/hand-offs and presents the exact
inventory/commands; existing operator authorization permits the planned serialized
local bounded-phase acceptance at that complete boundary without routine fresh
approval. External effects/remote paid work/publication/deployment remain excluded.
Gate remains CLOSED throughout this authoring handoff. Only source inspection and
allocated documentation/template writing occurred; no tests/checks/imports/hash
execution/build/Docker/readiness/service activation or model call ran. Every case
remains NOT_EXERCISED; no source-present API is reported as behavioral acceptance.

## Review repair cohort R3/R4/R5 — implementation complete, unexercised

Read the source-only JS/Python reports under reviews. Repaired their concrete
failure mechanisms in owned helper/steps/controlled_peer.py and updated the map/
README. No unrelated production/config source was changed. Review contexts were
fresh native contexts in the same model family, weaker than cross-family review;
this is not a claim of independent-family or runtime acceptance.

R3: one Before-owned100second monotonic case controller/deadline now covers every
UAR/peer HTTP call and full body read. Tracked promises join after abort/closure
before After starts a fresh100second cleanup budget;180second Cucumber wrappers
remain larger than each internal I/O budget. Local fixture/receipt FS uses the
source-verified synchronous APIs with deadline checks, so no async FS promise can
outlive its callback (an OS stall may delay the process, not advance another case).
Installed Cucumber hooks/time race and Node declarations were read, not executed.
No global wrapper/runner/framework or forced server-cancellation claim was added.

R3 effect uncertainty: module-scoped poison latch on deadline/uncertain HTTP/
cleanup failure makes each subsequent Before refuse before initialization/peer/
record changes. Known-owned agent/session deletions may use cleanup's fresh
budget, but poisoned runs skip peer reset. Safe receipts retain all original IDs,
confirmed DELETE responses and unresolved agent/session identities. A client abort
or DELETE404 cannot settle a possible late server creation. A fresh run requires
external reconciliation; retry0 stays intact.

R4: CD09 requires actual final_provider_request uar.request-budget.v1, actual wire
destination and image observations, approximate quality/revision, serialized
request/output ceiling/inclusion fields, positive count/size,64hex wire digest,
reserved_output_tokens null and fit_guarantee false. Attempt manifests are checked
separately and cannot substitute. The same comparator rejects removing only actual
final receipts with attempts/image observations retained. All asserted fields were
read from current Liter/token-service source; no new field or real image fit claim.

R5: Python observer alone normalizes null tools to empty list; non-list/malformed
tool/content/image shapes yield fixed safe invalid_shape observations. Original
body, response/control modes and their existing validation are preserved, preventing
instrumentation from changing accepted text success/503 responses into400.

All this cohort remains NOT_EXERCISED. No test/import/hash/check/lint/format/build/
Docker/service/network probe ran. The lead's phase-local supplementary probes and
corrective source review join the renewed complete inventory before execution;
this handoff neither ran nor accepts them. External fixture prerequisites from the
prior handoff remain unverified. No requested review finding was dropped or hidden.

## Single corrective P2 repair — immutable pre-cleanup ownership

Read and retained corrective-js-review.md. Repaired the remaining evidence-loss
mechanism in the owned After hook plus README/map, without a third review cycle.
After joining case client I/O, the hook synchronously writes an exclusive0600
flushed `<uuid>.ownership.json` BEFORE cleanup's lifetime/requests begin. The record
contains exact original agent/session IDs, case/scenario/source/image/config/
contract/manifest/namespace/workspace anchors, initial isolation, planned terminal
receipt and cleanup-pending/reconciliation disposition. The normal terminal
receipt references its path. Cleanup timeout/failure/DELETE404 never rewrites it,
so original IDs survive even when the terminal receipt cannot be published.

Initial ownership-write failure poisons the fixture, attaches safe ownership data
if runner output works, starts no cleanup requests, and reports the persistence
failure. No claim can guarantee durable data on a failed filesystem. Synchronous
writes cannot asynchronously outlive the hook, but OS stalls, storage loss and
real cleanup/server completion remain unverified. Abort/join is still only client
ownership; the poison latch and exact-ID reconciliation obligations remain.
Installed writeFileSync WriteFileOptions supports flush; no API was invented.
No test/check/import/hash/build/Docker/probe/review ran during this repair. All
cases and ownership/lifecycle falsifiers remain NOT_EXERCISED; gate CLOSED until
the lead collects this handoff in the renewed implementation inventory.

The same narrow repair preserves outer lifetime accounting: pending case I/O at
hook entry poisons before join, and cleanup starts only if its whole100second
budget still fits within the remaining180second hook limit after ownership I/O.
Otherwise it leaves the written ownership record pending and starts no remote
cleanup. This is a stage-budget restriction, not an extension/exception to the
outer timeout, and does not claim to forcibly interrupt synchronous OS I/O.

## Source-observed global-scope/topology cohort — item7 handoff

Amended owned runtime-overlay/manifest templates, helper, feature/steps, Python
peer and docs/map only; preserved other writers. Outside is now the real global
FailoverConfig fallback (namespace-qualified model, keyless base_url4612/v1), not
merely a registry candidate. Manifest includes existing strategy/fallback_models
fields. Helper compares actual GET /api/uar/settings/llm-failover row effective
values/application_status for all five fields and records safe normalized global
settings/revisions. External existing admin-read authority is a prerequisite; no
settings mutations, key/header/login or invented API is added.

Actual global-chain controls precede CD06 positive and scope/exact negative cases.
Each creates a tracked disposable empty-fallback agent and **fresh tracked owned
session**, forces one primary failure, requires actual outside success/images,
and finally restores original agent/session/nonempty artifact policy. Each
failure_primary/failure_exact/failure_scope role appears once; the provider must
start with no prior errors and show exactly one healthy, below-threshold failure
before the actual case. The configured threshold is3 and llm.max_retries0 caps
source retry attempts. A substituted global outside therefore fails the positive
captured-fallback assertion and negative failed/zero-send assertion. No prior
control history or conversation policy is accepted as restricted-run evidence.
All created sessions/agents are tracked before calls, persisted in immutable
pre-cleanup ownership, visited by bounded cleanup and preserved in unresolved-ID
receipts on uncertainty. All prior assertions/lifetime/poison controls remain.

New explicit Python fixture --bind-host choice defaults127.0.0.1 (existing probes
unchanged), permits0.0.0.0 only for the selected owned container interface mode.
Runtime overlay server.host0.0.0.0 is private-container listener configuration;
host mappings must bind ONLY127.0.0.1. This allows the host proxy/client to reach
container-interface listeners while internal UAR/provider/global URLs stay127.
It does not claim NAT can reach a127-only container listener. No new forwarding
service/global harness, auth plumbing or remote-identity relaxation was added.

Remaining operational prerequisites: externally provisioned secure verified
identity plus providers/settings-read authority, fresh store/namespace and all12
owned peers, exact source/local image/container/private network/process/volume/
port publication receipts, local model assets and scenario dependency mount.
No checks/import/hash/test/compile/build/Docker/service/network probe was run in
this cohort. All new behavior remains NOT_EXERCISED; verification CLOSED until
lead reconciliation/renewed inventory. No claim of topology/readiness acceptance.

Global-control wire identity is source-traced, not guessed from route spelling:
legacy FallbackModel.model remains namespace-qualified, and Liter build_chat_request
initially copies that value. The pinned vendored client constructs its generic
OpenAiCompatibleProvider with that model_hint; hint_provider_name captures the
first segment, strip_model_prefix removes exactly that matching segment, and
prepare_request writes the resulting bare model into the wire body. Therefore
this selected global route namespace-outside/outside serializes as outside. The
control asserts the actual peer model hash against this derived alias and frozen
outside.wireModel, retaining both qualified routing identity and wire hash in
evidence. The captured registered capable fallback's existing alias assertion is
unchanged. No public identity/capability key or comparison was weakened. This
serializer trace is source inspection of vendor/git/liter-llm, not execution.

## D2 retry-layer audit and explicit fixture setting handoff

Source confirms ordinary UAR attempts are retries_enabled ? max(1,
min(host retry_max_attempts,llm.max_retries+1)) :1. Provider registry inherits
llm0, config::build_client_config passes0 to Liter, and manager supplies the
persisted per-turn resilience policy. Existing agent_config resilience overrides
cannot raise the ordinary UAR cap above the llm0+1 minimum. A503 is normally a
retryable pre-semantic opening failure, not a post-output retry exemption.

Amended only owned overlay/manifest/helper/steps/docs: explicit host
resilience.retries_enabled:false/retry_max_attempts:1 plus existing llm0; manifest
binds both layers. Helper reads the three existing nonsecret settings keys and
requires exact effective values/status, retaining safe revisions in evidence and
immutable ownership. No auth/settings write or invented schema. Global control
now requires exactly one actual primary request as well as one healthy,
below-threshold error, preserving the original one-failure intent. All12 peers,
global outside route/aliases/session isolation and other assertions remain.

No retry/settings/503 behavior or other code was exercised. No checks/import/hash/
compile/build/Docker/service/probes ran. D2 source audit/config authoring complete;
all corresponding runtime claims remain NOT_EXERCISED and gate CLOSED.

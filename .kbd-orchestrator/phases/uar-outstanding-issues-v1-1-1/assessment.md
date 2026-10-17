# Detailed assessment: outstanding UAR issues

Project: Universal Agent Runtime  
Phase: `uar-outstanding-issues-v1-1-1`  
Stage: Assess, 2026-10-15 (America/Chicago)  
Source baseline: `91a091d647fe2ce63651c07140172aba2c6c6302`, branch `codex/uar-1-1-0-kubo-conformance`. The checkout was clean before phase creation.  
Issue inventory: all five open repository issues returned by GitHub; immutable snapshot in [evidence/open-issues.json](evidence/open-issues.json).

## Executive assessment

The repository has two unresolved defect reports (#385 and #380), two release requests with delivery evidence but stale issue state (#379 and #383), and one cross-repository dependency tracker (#382). They do not require five independent product rewrites. The user selected UAR fixes plus v1.1.1, with external #382 dependencies recorded separately.

**The uncomfortable finding:** v1.1.0 passed its selected published-image acceptance program on arm64 and emulated amd64, yet an existing remote-Surreal deployment cannot complete chat. The release fixtures explicitly attested remote durability and did not establish the reported upgrade/configuration combination. Passing those fixtures does not contradict the observed outage or certify upgrade compatibility.

The strongest source-supported explanation for #385 is an explicit 16,000-token turn budget combined with an unattested remote store. The new runtime requires durable accounting for bounded execution and rejects that combination before constructing the run model. The catch-all error destroys the underlying cause. The repository agent was read; the live persisted agent was not. This is a strong causal explanation, **not a runtime reproduction or captured inner error**.

No runtime fix, deployment change, migration, test, compiler check, lint, build, or review gate was performed during this assessment. Read-only production evidence was collected earlier under explicit operator instructions. The only executed content scan was the operator-requested redaction scan of the diagnostic documents.

## 1. Complete issue inventory and disposition

| Issue | Current GitHub state | Assessment | Required disposition |
| --- | --- | --- | --- |
| [#385](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/385) chat/model binding and upgrade failures | OPEN | **PARTIAL / release blocker.** Generic diagnostics hide accounting/provider failures; reported configuration is incompatible with new bounded-execution durability requirements. KB namespace and team ownership changes need accurate recovery guidance. | Resolve diagnostic and supported-upgrade behavior, qualify the actual configuration class, explain verification/migration requirements, and attach immutable v1.1.1 evidence. Keep open until its exit criteria pass. |
| [#380](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/380) terminal truth, MCP outcomes, Anthropic framing, credential ownership | OPEN | **PARTIAL.** Large source repairs already exist, but manager completion defense, CR framing, unknown-event policy, installation/shared credential policy, migration inventory, and required acceptance coverage remain. | Finish bounded missing requirements and execute a new local production-path acceptance program after full implementation. Historic Dot evidence is contextual, not closure of current requirements. |
| [#379](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/379) nine Kubo conformance requirements | OPEN | **DONE for the accepted bounded v1.1.0 contract; issue reconciliation pending.** Original and follow-up requirements have published artifact receipts. | Link release and both architecture receipts, preserve limitations, distinguish the later #385 regression, then reconcile closure against the accepted contract. Do not claim downstream Kubo qualification was performed here. |
| [#383](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/383) v1.1.0 release and four Kubo gaps | OPEN | **DONE for the accepted release contract; stale checklist/no comment.** It duplicates delivered #379 work. | Supply the missing release/evidence handoff and reconcile its checklist/state. Do not rebuild or move v1.1.0. |
| [#382](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/382) commercial Dot integration roadmap | OPEN | **PARTIAL / externally owned tracker.** No selected cross-repository integration lane has been qualified by this phase. | Update UAR dependency status and explicit exclusions. Keep external owner issues and unknowns visible; do not close the tracker by assertion or revive cancelled qualification. |

No new open issue beyond these five was returned. This assessment does not equate all repository TODOs, old global KBD counters, or optional roadmap work with newly authorized defects.

## 2. Blocker #385: evidence, mechanism, and limits

### Observed deployment evidence

The five redacted documents are under [evidence/issue-385](evidence/issue-385/). They contain deployment/configuration, selected repository agent policy, the first 150 retained startup lines plus relevant later diagnostics, one correlated failed run, and image identity. Connection URLs, authentication-bearing fields, and request text were redacted before persistence. Secret values and the system prompt were not included.

- Context `know-me`, namespace `knowme`; pod `uar-6df66c6c5f-lv2l9`, start `2026-10-15T20:28:56Z`, zero restarts at collection.
- Running index: `sha256:064f5004fb3ba76506abce5cf81285b2a8aa3e54fa5a22e2c2bfc97b1211e43b` (v1.1.0). This supersedes the issue body's earlier statement that the cluster was currently on the old image.
- Operator-supplied last-working digest: `sha256:7dc13eb3dab415a2c9dbd6989fdbdcc1ed4745c24a2dcb10912e2330a198fcf5`. It was not run during assessment.
- Repository `charcoal-agent/uar/agents/knowme-site.json`: provider `openai`, model `qwen3.8-max`, no fallbacks, `presentation_render` allowlist, direct execution, `max_tokens_per_turn=16000`, no `extensions.uar.run_policy` value.
- Inspected deployment, ConfigMaps and credential entry names contain no `UAR_REMOTE_SURREAL_DURABILITY_ATTESTED` setting. Startup reports durable ingestion disabled. No container exec or process-environment inspection occurred.
- Startup constructs the global provider client with `timeout_secs=240`; preserve #384's per-provider timeout inheritance.
- Request `8286562f-96d7-41d8-a4d0-37538aa1e3a3`, run `d1ab3137-4e25-41a4-8dd0-5fc42fe6ecc6`: starts at 20:44:59Z, retrieves zero KB candidates, selects per-agent provider at 20:45:01Z, immediately logs model-binding failure.
- Separate startup warnings: `TEAM_EXECUTION_OWNER_CONFLICT` and durable ingestion unsupported. The former is a nonfatal team-catalog restriction and does not itself establish ordinary-chat causality.

### Binding and accounting

`ModelCallBudget::has_configured_limits` recognizes the explicit token ceiling. Manager budget preparation calls `with_durable_accounting` even with `cost_tracking=false`. Remote Surreal reports accounting support only with the trusted durability attestation; unsupported storage returns `durable_accounting_unsupported`. Manager then replaces it with `Run model binding failed` before ERROR logging. See [runtime-bindings.md](assessment/runtime-bindings.md) for the complete source chain and commit comparison.

The required diagnostic repair is to preserve safe failure categories and stages at the originating operation. Logging arbitrary error chains is unsafe because constructor-owned credentials may not yet be in the scrubber. An allowlisted reason and diagnostic reference can distinguish invalid budgets, unsupported accounting, missing verified principal, storage failure, inherited binding, team profile and provider construction without exposing secrets.

The eligibility guard must not be removed to restore apparent success. An actually durable remote profile needs documented operator attestation and supported restart semantics. An unattested or unsupported profile with configured budgets must fail actionably before provider/tool effects. Removing the user's token ceiling or silently making tracked work ephemeral changes the requested behavior.

There is a secondary source inconsistency: the new legacy ledger existence probe decodes full Surreal rows directly into JSON, whereas established history reading converts native Surreal values. It is not the established deployed cause. Do not turn this hypothesis into speculative production changes without causal evidence. Existing unreconciled ledger scopes must retain their liabilities and require explicit reconciliation.

### Knowledge visibility and durable uploads

Authenticated KB queries now use issuer/tenant/subject ownership and `principal_knowledge_*` tables. Schema initialization preserves old `knowledge_*` data without adopting it. The same raw subject does not prove ownership of legacy records. The observed empty KB is consistent with this transition, but live data preservation, exact identity, namespace/database, and the creator of the replacement KB were not independently inspected.

Uploads now durably commit source bytes and ingestion jobs before success. The old path used metadata plus an in-memory queue. The preexisting remote attestation requirement now governs that stronger upload promise. Explain the requirement and expose its actionable cause; do not acknowledge an ephemeral upload as durable.

Existing supported KB recovery is authenticated reupload while preserving legacy data. A privileged legacy KB preview/import API is **a candidate scope decision**, not an existing API or an automatic consequence of #385. If preservation of old IDs/chunks is required, Analyze must define an explicit owner mapping, source digest, atomic publication, conflict/idempotence rules and recovery of incomplete source material before implementation. See [storage-upgrade.md](assessment/storage-upgrade.md).

### Team ownership and model verification

Team ownership is a nonexpiring fenced claim. Both the conflict behavior and remote durability flag predate the reported working baseline; the new symptoms must not be attributed to the mere existence of those mechanisms. Clean shutdown already attempts joined release; quiesce/reclaim and qualified supervisor exclusion already exist. No evidence here establishes that the failed predecessor completed shutdown or has a valid exclusion receipt. Single replica, elapsed time, or Pod absence is not authority to discard a held claim.

Ordinary per-agent model capture does not consult model-verification receipts. The verification and target-admission files are unchanged across the reported baseline transition. Explicit target admission has a separate configuration-fingerprinted verification requirement. Reverification is not the diagnosed cure for accounting preparation; changed provider configuration or protected credential revision can independently require new proof.

### Required additional evidence

1. Read-only, sanitized live persisted-agent budget/provider fields; repository source is not live-state proof.
2. Actual binding failure category once diagnostic code exists, or an isolated equivalent production-path reproduction at the authorized verification boundary.
3. Surreal engine, backing volume and restart durability evidence before any deployment attestation change. Existing permission was get/describe/logs only; no rollout or data mutation is authorized by diagnostics collection.
4. Legacy/new KB inventory and verified ownership mapping if recovery needs more than reupload; ledger reconciliation state if durable accounting exposes legacy balances.
5. Current/prior team fence, shutdown and qualified exclusion evidence if team execution recovery is needed.

## 3. Issue #380 requirement-level assessment

| Requirement | Existing implementation | Remaining work / acceptance debt |
| --- | --- | --- |
| Terminal invariant on every provider attempt | Orchestrator rejects unterminated EOF; ordinary canonical-history behavior remains separate from graph history. | Manager ordinary loop still treats exhaustion as success absent another error; add explicit observation of successful orchestrator completion while retaining cancellation and cleanup precedence. |
| No tool dispatch from unterminated turn | Orchestrator's terminal check precedes admitted tool dispatch. | Prove empty/metadata/text/partial-tool/complete-tool EOF, ordinary full-harness failed projection, graph/fallback paths and valid tool loops. |
| Typed MCP application outcomes | Registry reads typed `is_error`; payload-preserving application failures flow through runtime and receipts. | Preserve working core behavior. Reconcile omitted `isError`, protocol errors, timeout, no replay/reconnect, later separate recovery, and native coincidental `isError` cases. Document compatibility behavior accurately. |
| Incremental native Anthropic SSE | Actual HTTP path uses an incremental byte decoder; invalid input/incomplete terminal errors exist; buffer is bounded to 1 MiB. | Pinned local decoder searches LF and does not implement required lone-CR framing. Define ignorable unknown-event behavior without swallowing malformed known events. Retain existing ping/unknown-field behavior. |
| Exact framing coverage | Bytewise and selected split scenarios exist. | Every two-way byte split, fixed partitions, split UTF-8/escape/CRLF, lone CR, multiline data, field spacing, unknown kinds, overflow, EOF and normalized event-sequence equivalence remain to be mapped or added. |
| Principal credential storage | Separate v2 store, issuer/tenant/subject keys, additive schemas, resolver threading and exact operator mapping exist. | Fixed issuerless/local-host authority strings are not installation-specific. Specify trusted stable installation authority and an explicit single-owner/shared policy. |
| Shared-mode fallback | Store/decryption failures stop resolution. Ordinary misses still fall through to system/environment credentials. | Shared missing ownership must fail closed; operator-owned shared system/environment fallback requires explicit permission. Apply the policy to all resolution seams, not only user credentials. |
| Credential migration | Atomic copy+journal, source preservation, exact approved mapping, conflict rejection and replay behavior exist. | Non-secret dry-run inventory/count/conflict report plus backup, quiescence, cutover verification, rollback and shared-mode enablement runbook. Never infer owner from first requester or matching subject. |
| Storage and end-to-end coverage | Historical scoped tests and real-backend targets exist. | Complete current memory/Surreal/PostgreSQL contract coverage, ignored PostgreSQL selection, full-harness/combined scenarios and actual upgrade paths; no skipped backend credited as passed. |

See [issue-380-and-release-reconciliation.md](assessment/issue-380-and-release-reconciliation.md) for source locations, existing adjacent fixtures, exact proposed commands and historical receipts. The assessment does not label every requirement absent merely because one test target lacks it.

The issue's older request for per-unit compiler/testing and CI checks conflicts with current AGENTS.md. Its eventual execution record must explicitly use the current full-phase boundary and local product gates. GitHub Actions remains deployment build/publish only.

## 4. Release evidence and specification alignment

v1.1.0 source/tag commit is `d815ef72ac7d2c57a9bd18b23fcc58ffb224f326`. The immutable index above contains amd64 manifest `sha256:94f78a7ed17197fc045baec514ed6ec30d3cf23433e8c26340ccfddb080fac90` and arm64 manifest `sha256:b3a3d8bdda9d919fa3489dded935c834256bf89984fa17b07a428acdb7a1c1ae`.

The [published Release](https://github.com/Prometheus-AGS/universal-agent-runtime/releases/tag/v1.1.0), prior phase `receipts/publication.json`, `receipts/published-acceptance.json`, `amd64-acceptance.md`, and two #379 comments establish the accepted release contract. The arm64 run and emulated-amd64 follow-up passed the selected full program. Native amd64 hardware, infrastructure rollout, startup capture gap and cross-family review limitations remain distinct unknowns. Historical successful fixtures do not cover every #380 requirement or the #385 upgrade profile.

| Contract | Alignment assessment | Next specification need |
| --- | --- | --- |
| `openspec/specs/kubo-runtime-conformance/spec.md` | Delivered bounded A2A/MCP/skills/defaults/approval/elicitation contract with published evidence. | Preserve it in v1.1.1 regression acceptance; do not reopen shipped features without evidence. |
| `provider-model-settings-certification/spec.md` | Deliberate provider selection and persisted authority must survive upgrade. | Add actionable run-admission/accounting configuration failures and timeout inheritance scenarios where the selected change belongs. |
| `provider-diagnostic-status/spec.md` | Secret-safe provider diagnostics already required; catalog status is not run binding status. | Specify safe runtime diagnostic categories without broadening credential exposure. |
| `knowledge-rag-product-certification/spec.md` | End-to-end upload/index/retrieve/chat is required; the observed remote upgrade fails before completing it. | Document durable eligibility and ownership migration/recovery semantics for the chosen supported profile. |
| Existing operational credential, streaming, accounting and ownership contracts | Substantial implementation exists; some required policy/operational gaps remain. | Create at least one OpenSpec delta per eventual change. Do not invent a new tenancy framework or new engine as an incidental repair. |

`versions.toml` remains authoritative: Liter 2.1.1 at its pinned commit, SurrealDB/types/server 3.3.0 and pinned image. No dependency change is selected by this assessment. #384 must remain intact.

## 5. Recommended priority and decisions for Analyze

This is a gap assessment, not an approved implementation plan. The phase's usable production behavior, persistence, contracts, operational settings, fixtures and documentation must all be planned before implementation-complete can be recorded.

| Priority | Bounded work area | Decision or input needed before implementation |
| --- | --- | --- |
| P0 | #385 diagnostics and budget/durability upgrade compatibility | Define safe typed failure taxonomy and public compatibility; confirm live artifact and supported storage profile. Preserve bounded execution rather than bypassing admission. |
| P0 | #385 operator recovery contract | Select supported reupload versus explicitly scoped legacy KB import; identify legitimate team recovery evidence and explain unresolved dead-predecessor cases. |
| P1 | #380 stream terminal truth and Anthropic framing | Specify manager completion invariant and local adapter/pinned-library change boundary. Any dependency pin change requires the repository's deliberate approval path. |
| P1 | #380 credential policy and migration | Select stable trusted installation identity/bootstrap, explicit shared-mode fallback rules and namespace upgrade mapping. Existing principal data must not be silently orphaned or globally adopted. |
| P1 | Missing real-path fixtures and regression inventory | Map every issue assertion to a concrete scenario/backend/receipt before opening the one full-phase gate. Reuse existing infrastructure fixtures where their contracts match. |
| P2 | #379/#383 issue reconciliation and #382 dependency accounting | Reconcile delivered evidence, record remaining external dependencies/exclusions, and close only criteria actually met. |
| Release | Immutable v1.1.1 | Preserve v1.1.0. Build locally after implementation, publish with deployment workflow, then run full acceptance locally on both published architecture digests and disclose native/emulated execution. |

Additional observed startup noise: the retained log contains bundled skill load failures for missing manifest `version` fields. These are actual observations, not established causes of chat failure. Inventory and triage them in Analyze; do not silently expand this phase into an unrestricted bundled-skill migration.

### Team and source ownership

The lead owns authoritative KBD state, complete issue inventory, integration order and the whole-phase verification boundary. Assessment contributors own only their three reports. The eventual implementation plan should assign `uar-runtime` to runtime/provider execution, `uar-trust-tools` to credential/trust contracts, and `uar-state` to storage/migration contracts with explicit files and serialized shared-file handoffs. One writer per file and build directory remains mandatory.

## 6. Verification, process and memory status

### What was actually done

- Read the canonical waypoint, project configuration, pinned versions, prior reflection, relevant specs, source/history and existing fixtures/receipts.
- Captured all five open GitHub issue bodies/comments using `gh issue list --state open --limit 100` with JSON fields; snapshot stored locally. No issue closed or relabelled during inventory.
- Reused explicitly authorized read-only cluster get/logs observations; never executed inside a container or changed resources, credentials, data, model receipts or the kill switch.
- Ran the requested `grep -nE 'Bearer|eyJ|sk-|password|secret_ref'` separately against the five diagnostic files: every result was exit 1 with empty output. Hashes and exact commands are in `evidence/issue-385/redaction-receipt.json`. This is a redaction receipt, not product acceptance.
- Created and activated the phase through the installed KBD lifecycle and typed runtime commands. Generated projections are not hand-edited.

### Build health and coverage

**Build health: UNKNOWN for new phase acceptance.** No build/check/test was run. Prior release artifacts retain their historical passing results and known limits. Current issue-level behavioral coverage is PARTIAL; no coverage percentage is asserted. The production implementation boundary is CLOSED: there is no accepted complete implementation inventory for this phase.

The assessment skill suggests model probes, sycophancy screening and adversarial validation. Those are deferred under the stronger repository rule prohibiting verification/review gates before full phase implementation. Three source investigations are assessment work, not an independent artifact-review pass. Product checks and the eventual isolated artifact review remain pending.

### Prior lessons applied

The assess-before memory hook wrote: `learning_recall library not found; no prior context retrieved`. No remote memory recall success is claimed. There were no retrieved `Knowledge gaps` entries to enumerate; missing recall itself is a tooling limitation. Local append-only memory and prior reflection were consulted instead.

- Prior reflection: “Whole-phase completion includes required fixtures and documentation.” Applied by keeping all product verification deferred, including per-fix testing and builds.
- `.prometheus/gotchas.md`, 2026-10-12: “An arbitrary healthy remote endpoint is not durability attestation.” Applied by refusing to prescribe the flag as an unqualified deployment fix.
- `.prometheus/gotchas.md`, 2026-10-13: “uses strict verified keys and disjoint durable namespaces, preserving legacy rows without automatic adoption.” Applied to KB and credential recovery scope.
- Prior release reflection: “Do not revive the cancelled Dot qualification program.” Applied by defining fresh issue-specific acceptance rather than restarting old global certification.

### Scope and limitations

The KBD skill's mandatory OpenSpec refresh used managed CLI 1.14.2 and recorded `authoredPathsChanged: []`; it did not migrate authored specification semantics or run product validation. Its unrelated generated integration churn was removed from this task's diff: 216 tracked integration files restored to the clean baseline and 12 newly generated files removed. The refresh backup, receipt and exact cleanup paths are recorded in `evidence/openspec-refresh-scope.json`; authored specs and other writers' reports were preserved. Canonical KBD commands also refresh metadata in older phase projections; this does not reopen their tasks or cancelled qualifications.

No runtime guards or product features were added. No production rollout, unsafe ownership adoption, hidden fallback, new model engine, or cross-repository implementation is authorized by this assessment. After explicit operator OK, the diagnostic draft was posted to [#385](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/385#issuecomment-6202319965). A read-back matched the exact draft body and SHA-256; `evidence/issue-385/post-receipt.json` records both. No issue was closed. Other issue closure and v1.1.1 success still require their actual criteria.

## Goal status and next handoff

| Goal | Status |
| --- | --- |
| Detailed assessment of every open repository issue | MET as source/evidence assessment; five of five issues classified with limitations. |
| Resolve #385 and remaining #380 defects | NOT MET; findings and candidate work are documented, production implementation pending. |
| Reconcile #379/#383 and preserve #382 external ownership | PARTIAL; evidence mapped, issue reconciliation remains pending. |
| Publish accepted v1.1.1 | NOT MET; no new release artifact or acceptance claim. |

Next: `/kbd-analyze uar-outstanding-issues-v1-1-1`, using this report and its three companion assessments to resolve the named choices and author a bounded specification/plan. Do not enter Execute or run verification from this assessment alone. The cancelled Dot program and immutable v1.1.0 remain preserved.

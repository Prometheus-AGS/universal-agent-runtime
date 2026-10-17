# Analyze: outstanding UAR issues for v1.1.1

Phase: `uar-outstanding-issues-v1-1-1`. Stage: Analyze. Date: 2026-10-15.
Baseline: `91a091d647fe2ce63651c07140172aba2c6c6302`.
Inputs: [assessment](assessment.md), its three source investigations, five redacted #385 documents and the five-issue snapshot. This is a design and reuse decision record for Spec/Plan; production implementation has not begun.

## Decision summary

**The uncomfortable finding:** accepted v1.1.0 image fixtures exercised an explicitly attested store, while the reported deployment has no such attestation in its inspected configuration. Better error text alone will not make that deployment's storage durable or restore ownership of quarantined knowledge. Product repair and operator recovery evidence are separate completion requirements.

Retain the pinned stack and existing host authority mechanisms. Select three adaptations, one protocol reference and one rejected replacement in [library-candidates.json](library-candidates.json). No new dependency, vendor edit or pin change is selected. The detailed team decisions are in [runtime](analysis/runtime.md), [trust](analysis/trust.md), and [state](analysis/state.md).

| Area | Selected direction | Evidence and consequence |
| --- | --- | --- |
| #385 binding failure | Preserve typed, allowlisted stage/reason plus correlation reference; keep public error compatibility and exclude arbitrary underlying strings. | The manager erases the cause before logging. Constructor errors can contain credentials outside the scrubber; raw-chain logging is not the safe fix. |
| #385 accounting/ingestion | Keep durable eligibility, explicit budgets and liability reconciliation. Describe supported attested remote and local durable profiles and actionable rejection. | The reported configuration is strongly consistent with accounting rejection, but live persisted agent and inner cause are unverified. |
| #385 knowledge recovery | Authenticated reupload into the verified principal namespace, retaining legacy data. | The issue asks whether migration is required; it does not require preserving old IDs/chunks. A privileged legacy importer is not needed for the supported reupload path. |
| #385 team ownership | Document existing quiesce/reclaim and qualified predecessor exclusion; preserve nonexpiring fencing. | Single replica, elapsed time or a missing Pod is insufficient takeover authority. Ordinary chat failure is not established to be a team-owner conflict. |
| #380 completion | Require explicit orchestrator completion observation before the ordinary manager declares success. | EOF alone currently reaches Done if another error is absent. Preserve cancellation, cleanup, interrupted history and tool admission ordering. |
| #380 Anthropic | Adapt native transport input around the existing pinned decoder; define future-event handling separately from malformed known events. | Local source has LF-only scanning; protocol requires CR/LF/CRLF. Do not rewrite working usage/tool accumulation. |
| #380 MCP | Retain typed application outcomes; fill exact fixture and documentation gaps. | Existing runtime already distinguishes application failure from transport uncertainty. No replay authority follows from reconnection. |
| #380 credentials | Add explicit host-owned credential policy and installation-bound local credential identity, scoped to the credential subsystem. | General principal keys also identify sessions and durable work. A global key rewrite would create unrelated migration obligations. |
| #379/#383 | Reconcile immutable v1.1.0 evidence, then preserve regression coverage in v1.1.1. | Both architecture receipts exist; amd64 was emulated. Do not move the published tag or claim downstream Kubo qualification. |
| #382 | Record UAR status and external dependencies; leave cross-repository qualification open. | No external implementation lane was selected; cancelled Dot qualification stays cancelled. |

## Research and reuse

The stack is specified by `versions.toml`: Liter 2.1.1 at `e446f5ef91003bbec5fce6b2f45fb94559dff335`, SurrealDB/types/server 3.3.0 and its immutable image remain fixed. Stack discovery is unnecessary.

Research ran from 21:13:32Z through 21:18:28Z, inside the 20-minute budget. Counts: GitHub 3, Context7 4, registry 1, primary web retrieval/location 4; every tier cap is 8. Exact queries and limitations are recorded in [analyze-research.json](evidence/analyze-research.json). Research stopped when the bounded decisions were supported; no maintenance or compatibility experiment was run.

1. **Adapt the existing decoder (`cand-001`).** The pinned local source retains UTF-8 and event state, bounded buffering and explicit EOF failure. A UAR-owned framing adapter can cover CR and split CRLF without editing vendor code. The adapter and driver still require production-path proof.
2. **Reject adding `eventsource-stream` (`cand-002`) for this phase.** GitHub identifies a Rust stream adapter; the registry reports 0.2.3, released 2022-02-17, MIT OR Apache-2.0. Context7 did not return an exact Rust match. It is a plausible alternative, but replacing working framing and adding a dependency is broader than the observed gap. Release age/downloads do not establish a correctness defect. [Registry](https://crates.io/api/v1/crates/eventsource-stream), [repository](https://github.com/jpopesculian/eventsource-stream).
3. **Reference protocol and parser behavior (`cand-003`).** Context7's JavaScript parser reference supports incremental input and bounded buffers, but is not an adoptable Rust implementation. WHATWG specifies the three line endings and incomplete-event handling. Anthropic explicitly permits future event types. These primary sources settle behavior that SDK examples alone did not establish. [WHATWG framing](https://html.spec.whatwg.org/multipage/server-sent-events.html#parsing-an-event-stream), [Anthropic streaming](https://platform.claude.com/docs/en/build-with-claude/streaming), [parser reference](https://github.com/rexxars/eventsource-parser).
4. **Adapt existing credential and persistence contracts (`cand-004`, `cand-005`).** Principal credential storage, atomic migration journals, durable upload jobs and fenced team ownership already exist. External identity frameworks or replacement storage engines cannot infer legacy owners, prove backing-store durability or recover discarded secrets. Extend the present contracts where the reports identify a missing policy or diagnostic.

Current SDK documentation is a protocol reference, not proof of compatibility with the pinned Rust library. Primary pages were read after the earlier tiers left normative behavior unresolved. No upstream bug or version-change claim is inferred from search results; decoder observations concern the checked-in pinned source.

## Binding diagnostics and compatibility

Define bounded failure categories at each origin: budget-policy parsing, accounting eligibility/setup, verified-principal requirement, inherited binding, team profile binding, and provider construction. Diagnostic stage/reason values and correlation IDs must be safe without string scrubbing. Never attach provider keys, URL credentials, prompts, raw connection errors or arbitrary cause chains to public events or logs.

Preserve `orchestrator_start_failed` and existing team codes. Put the safe stage, category and diagnostic reference into the existing message string and structured logs; retain the existing AG-UI RUN_ERROR code/message shape. No new event fields or diagnostic service are selected. The inspected projection is `src/uar/api/agui.rs:285,294`. Categories must survive the manager catch-all and remain useful at INFO-configured production logging. Unknown internal failures retain a generic category with a diagnostic reference. Do not classify by matching potentially secret-bearing error strings.

For the reported deployment, the source-supported path is explicit token budget → durable accounting preparation → unsupported remote accounting → erased cause. Qualify that exact configuration class later. Keep #384 timeout/retry inheritance. Do not remove the token ceiling, silently downgrade accounting, or prescribe model re-verification as the cure. Explicit target admission may independently need fresh configuration-bound verification receipts; ordinary per-agent model capture does not consult them.

The legacy Surreal ledger deserialization inconsistency remains an unconfirmed secondary hypothesis. Carry it into failure triage; do not add a speculative production fix before a named reproduction or source proof establishes it.

## Supported upgrade and recovery

The release runbook must distinguish an unattested remote endpoint from a deployment whose engine, persistent storage and restart behavior justify attestation. Explain which accounting, upload, checkpoint and owner capabilities need it. Rejection must happen before provider/tool effects or successful upload acknowledgment. No deployment mutation is part of this Analyze stage.

Knowledge recovery uses the current authenticated upload/bind/retrieve path with original source files. Legacy records remain preserved and inaccessible to arbitrary raw-subject matches. Record that old KB IDs/chunks are not automatically transferred. If source files are missing or preservation of old IDs becomes a requirement, that is an explicit additional scope decision for an owner-verified import contract; it cannot be hidden inside this fix or called recovered.

Team recovery uses existing trusted-host authority and qualified exclusion evidence. The runbook must distinguish clean joined shutdown, a reachable owner that can quiesce, and an abruptly lost owner with no valid exclusion receipt. The last case remains blocked for takeover until legitimate authority exists; do not add a TTL, delete claims, or infer authority from replica count.

## Credential policy and migration

Specify single-owner compatibility and shared operation as explicit operator policy. Omitted mode retains the current single-owner compatibility behavior; operators must explicitly enable shared mode. Remote networking does not itself imply shared tenancy, and an operator may explicitly select single-owner operation remotely. These defaults must be clear in upgrade notes, not silently described as shared isolation. Shared requests missing required verified ownership fail before credential fallback. System/environment fallback in shared mode requires explicit operator permission and must govern all lookup and binding paths, including missing-service, inherited-client and fallback-provider seams. Session or request metadata cannot grant that permission. Store/decrypt errors remain errors.

Bind issuerless and local-host credential ownership to a stable trusted installation identity. Keep the generic principal storage identity unchanged this phase because it also scopes sessions, schedules, followups and knowledge. The installation authority belongs to credential derivation, bootstrap and migration. Select an operator-provisioned canonical UUID, stable across the installation's replicas/restarts and backed up with its credential configuration. Require it when binding local/issuerless credential owners; missing ID returns an actionable credential-identity error. Externally issued principal credentials keep their existing identity and do not require this local-ID migration. Do not fail unrelated external-principal startup or introduce a database-global installation singleton. A new identity per process, request or Pod would orphan credentials and is prohibited. Explicit external issuer identities remain governed by their existing verified trust configuration.

Extend the current exact-mapping migration with a non-secret dry-run inventory: counts, source/destination identifiers, missing mappings, existing destinations and conflicts. Cover both legacy subject-keyed credentials and existing v2 credentials in old fixed local namespaces. Preserve encrypted sources and journal semantics. No first-request ownership adoption, raw-subject fallback, overwritten-history reconstruction or implicit authority remapping.

The operational sequence is inventory and backup, quiesce writers, approve exact mapping, migrate atomically, verify counts and cold readback, then enable the selected shared policy. A rollback to a subject-only binary cannot safely retain shared access. Specification must bind preview and execution to the same source identity and detect changes/conflicts instead of trusting a stale count.

## Bundled-skill warning triage and source authority

The startup missing-version warning remains an observed symptom with an unidentified consumer. Current builtin loading treats absent SKILL.md version as optional and defaults it; absent pack provenance is also nonfatal. The runtime report names those source paths. The selected work is troubleshooting documentation requiring the resolved pack root and sanitized failing path/error, not mass edits to the pinned skill submodule or weakened compiler validation. No claim is made that this warning is repaired or causes chat failure. If evidence identifies a UAR defect, revise the phase scope before its implementation boundary.

The state role brief mentions SurrealDB 3.2.4; `versions.toml` governs and resolves this conflict to 3.3.0 for all later source, fixture and deployment work. Do not update the pin to match a stale role brief. Compass retrieval was unavailable/stale for investigators; direct source evidence was used, and no graph accuracy claim is made.

## Required acceptance inventory for Spec/Plan

These are future requirements, not commands executed during Analyze. The executable plan must map each row to real production entrypoints, collaborators, owner, backend and receipt. Reuse existing targets where they cover the contract, and author only missing fixtures.

| Gate | Required production observations |
| --- | --- |
| #385 configuration matrix | Explicit token budget with unattested remote store rejects actionably before effects; qualified durable profile succeeds through accounting and restart; invalid/provider binding categories remain secret-safe; #384 timeout inheritance remains visible. |
| Upgrade recovery | Legacy KB remains untouched; authenticated reupload indexes/retrieves after restart with correct principal isolation; missing source material is disclosed; team conflict/recovery honors valid fencing proof; no blanket re-verification claim. |
| Terminal truth | Empty, metadata-only, text-only, partial-tool and complete-tool EOF fail without protocol terminal; valid tool loops succeed; ordinary/full-harness, graph, cancellation and fallback preserve correct persisted status/history. |
| Native Anthropic framing | Every two-way split plus bytewise/fixed partitions; split UTF-8 and escapes; CR/LF/CRLF including boundary splits; multiline data/comments/field spacing; unknown-event policy; malformed known data; overflow; EOF; equivalent normalized event sequences. |
| MCP outcomes | Explicit true/false/omitted isError; protocol error, timeout and uncertainty; zero automatic application-failure replay; later independent recovery cannot rewrite an earlier failed receipt; native coincidental isError field does not acquire MCP semantics. |
| Credential isolation/migration | Cross-issuer/tenant/installation isolation; missing shared identity rejects; permitted/denied system fallback on every resolution seam; no fallback after decrypt/storage error; dry-run conflict/change detection; atomicity/idempotence and cold restart across memory contract, real Surreal and PostgreSQL. |
| Combined real path | Actual HTTP provider stream and observable MCP peer through runtime admission, persisted state and AG-UI; a failed tool outcome and incomplete stream cannot become successful completion. |
| v1.1.0 preservation | Full Kubo conformance program against the new candidate and both published architecture digests, including approvals, MCP isolation/elicitation, skill upload, auth/defaults, discovery/CORS/settings. |
| Release | Build/exercise the production candidate locally, publish immutable v1.1.1 through deployment-only workflow, pull by digest, run local full acceptance for arm64 and amd64, record native versus emulated execution and all digests, then publish receipts and issue dispositions. |

Final program order is serialized, with one build-target writer. Product checks run locally. After a failure, repair the failing behavior and rerun only the affected gate. Historical receipts do not certify the new source. Published-image failure withholds the release success claim; an immutable tag is never silently moved.

## Evidence still needed and exclusions

- Live persisted agent/provider/budget identity, actual inner failure category, backing-store durability and any legacy ledger state remain unverified. These are execution/recovery evidence obligations, not grounds to invent an answer during Analyze.
- Existing authorization covered get/describe/logs for specified cluster surfaces. It did not authorize live-data migration, configuration changes, rollout or touching `site-chat-kill-switch`.
- Do not promise preservation of old KB IDs/chunks, model/provider expansion, dependency upgrades, UI work, a new tenancy framework, external Dot delivery or native-amd64 hardware qualification without corresponding scope and evidence.
- All five open issues have a disposition; #382 is intentionally an external dependency tracker. An issue remaining open for an external owner is not falsely reported as a UAR repair failure or completion.

## Process, memory and verification boundary

The Analyze skill and repository team contract were applied. Runtime, trust and state teammates own only their analysis documents. The lead owns the aggregate artifacts and canonical lifecycle. No teammate ran a review or product gate. Source inspection is design work, not independent approval.

The before hook again reports `learning_recall library not found; no prior context retrieved`. There are no retrieved Knowledge gaps to enumerate or learning exercises to initiate. Local lessons applied: a healthy remote endpoint is not durability attestation; private data cannot adopt raw-subject rows; whole-phase completion includes fixtures/docs; cancelled Dot qualification stays cancelled. These lessons appear in the assessment and append-only `.prometheus` history.

**Boundary CLOSED.** No production code, fixture or release implementation was performed. No tests, checks, builds, formatters, compilers, schema validators, model probes, packaging or isolated review gates ran. The skill's adversarial vet remains deferred under the stronger full-phase rule. Administrative stage transitions and source/document retrieval do not establish product correctness. Candidate compatibility, implementation feasibility and release acceptance remain unverified until that boundary.

Next lifecycle stage: `/kbd-spec uar-outstanding-issues-v1-1-1`. It must produce bounded OpenSpec deltas for the selected contracts, followed by the executable complete-phase plan and explicit per-file ownership. Analyze completion is not implementation completion or authorization to start verification.

# Spec: runtime-harness-gap-closure

Stage: Spec. Date: 2026-09-16. Backend: OpenSpec, spec-driven. Source phase goals and analysis remain authoritative inputs. feedback-source.txt preserves the operator attachment byte-for-byte; its verified digest is recorded in the manifest. This artifact specifies future work; no runtime fix or passing runtime test is claimed.

## Change set and ownership

| Order | Change | Feedback | Primary behavior |
|---|---|---|---|
| 1 | [harness-protected-context](../../../openspec/changes/harness-protected-context/proposal.md) | F3; explicit protected-data budget request | Canonical receipts, lossless history, pure budget function, prose-only compression |
| 2 | [harness-model-profiles-routing](../../../openspec/changes/harness-model-profiles-routing/proposal.md) | F1/F2/F3; explicit per-model template request | Exact destination templates/settings, constrained routing, final request validation |
| 3 | [harness-task-graph-lifecycle](../../../openspec/changes/harness-task-graph-lifecycle/proposal.md) | F4/F6 | Durable cross-protocol identity, graph/child lifecycle, shared limits and cleanup |
| 4 | [harness-skill-activation-quality](../../../openspec/changes/harness-skill-activation-quality/proposal.md) | F5 | Scoped explicit activation, immutable bodies, measured task success |
| 5 | [harness-knowledge-evidence](../../../openspec/changes/harness-knowledge-evidence/proposal.md) | F7 | Access at use, claim/evidence verdicts and provenance |
| 6 | [harness-governed-evaluation](../../../openspec/changes/harness-governed-evaluation/proposal.md) | F6/F8 and all feedback certification | Governed tool-inclusive local evaluations, verification debt and profile evidence |

Paths in this table resolve from the phase directory; the manifest records repository-relative paths for tools. Each change has proposal.md, design.md, tasks.md and at least one capability delta. All implementation checkboxes remain unchecked. OpenSpec artifact readiness is not KBD execution readiness.

Implementation dependency order is 1 → 2 → 3 → 4 → 5 → 6. This conservative total order serializes shared manager/orchestrator/prompt/persistence edits and resolves ownership conflicts. Plan may refine independent fixture work while preserving producer-before-consumer edges and one writer per shared file/build target. Change 1 owns early context/request fixtures; each subsequent change adds its own fixtures. Change 6 integrates these and does not delay regression discovery until final certification.

## Scenario traceability

| Requirement source | Delta and acceptance scenarios |
|---|---|
| F1 routing | model-path-resiliency: explicit selection, healthy/admissible ordered fallbacks, frozen routing quality/cost/latency baseline, rate limits and semantic commit |
| F2 templates/settings | prompt-assembly: exact profile and override, ambiguous family, missing slots, endpoint serialization, every entry path, continuity and cache compatibility |
| F3 P0 history | conversation-history-integrity: canonical pairs/parallel groups, pending versus terminal results, empty assistant, repeated user turns, bounded acquisition and checkpoint completeness |
| F3 P1 model context | protected-context-budget: arithmetic, actual output cap, full final count, media uncertainty, protected overflow, eligible prose only, summary failure and revocation |
| F4 task lifecycle | agent-thread-kernel: cross-protocol identity/cancel/artifacts, cross-tenant rejection, creation crash boundaries and pending approval restart |
| F5 skills | skill-activation-runtime: scope precedence, explicit governed selection, 99% Recall@10 shadow gate, body/version reattachment and frozen outcome metrics |
| F6 graph depth/observability | multi-agent-orchestration: causal correlation, terminal/replay identity, approval deny, shared exhaustion, child cancellation and completed-effect resume |
| F6/F8 agentic evaluation | eval-harness: production governed path, tool-inclusive success, seeded failures, strict local baseline gate and retained traces |
| F7 knowledge | rag-provenance and knowledge-rag-product-certification: authorized evidence at use, claims beyond lexical match, conflicts/staleness, revocation/injection and existing knowledge journey |
| F8 debt/profiles | runtime-profile-certification: exact historical failures, current coverage denominator/exclusions and 60% gate, supported profiles and separately scoped local milestone |

## Shared contracts

- Protected content is not compressible merely because it has a text role. Calls, raw arguments/results, structured/code/log/file data, multimodal references, evidence, required instructions/current input, durable decisions and pending work remain canonical. Only host-marked eligible prose can enter summarization. Preservation does not override authorization or deletion.
- Budget input allowance is min(independent input limit, host input limit, total context minus actual output reserve, additional non-overlapping reasoning reserve and explicit uncertainty). Final serialized requests, including all reattached fragments, must fit an established bound. Unbounded approximations cannot certify fit.
- Exact destination profiles separate template layout, endpoint serialization and provider settings. Every initial/iteration/retry/failover/graph/resume path starts from canonical data and current authorization. The existing semantic-commit retry boundary remains binding.
- All writes, receipt acquisition, summaries, task scheduling, tool execution and approval enforcement remain host-owned. Existing dependency pins are reused; no new template engine, agent framework or external verification algorithm is selected here.
- The current UI and protocol compatibility remain in scope as preservation contracts. No UI implementation is requested by these specs. Any necessary later UI code edit must first follow the repository's UI routing rules.

## Required Plan inputs and blockers

These are mandatory prerequisites, not permission requests or optional future improvements:

1. Complete five targeted external-candidate evaluations: durable A2A recovery and graph observability (change 3), skill-selection evaluation (4), evidence verification (5), and tool-inclusive agentic evaluation (6). Record sourced adopt/adapt/build decisions, license/version compatibility and concrete mechanisms before implementation commitments. The analysis WARNING remains open until these artifacts exist.
2. Inventory exact configured provider/endpoint/model/revisions, supported settings and roles, input/output/reasoning semantics, continuity and count bounds with current official documentation and driver-boundary fixtures. Unsupported/unknown destinations cannot be certified by generic family assumptions.
3. Freeze acquisition/storage policy, legacy completeness handling and every context entry point; freeze routing/compaction/skill/knowledge/eval datasets, baselines and numerical acceptance criteria before implementation. Preserve the existing skill Recall@10 >=99% and frontend coverage >=60% gates.
4. Identify the two historical failing eval cases and exact coverage metric/denominator/exclusions. Record claims without a reproducible identity as unresolved debt. An interrupted earlier compile (exit 143) leaves build health UNKNOWN.
5. Register a separately scoped local Tier 3 supported-profile certification milestone, link it as a prerequisite of phase completion, and inventory environments/live credentials required for Linux/macOS stable and Windows experimental support across default/minimal, server-full, desktop-full and embedded/mobile. Missing required evidence blocks completion.
6. Inventory any remaining GitHub Actions product tests, including invoked scripts. Only observed violations are edited; the mandatory policy validator runs before/after those edits. Product verification stays local.

If research changes these behavior contracts or the dependency graph, revise the affected artifacts and revalidate/review before Plan is accepted. Task lists describe bounded work units for planning; Plan must map each to exact files, fixtures and tier-appropriate commands before Execute. The current stage does not waive those prerequisites.

## Verification and review

Tier 0 covers artifact structure, whitespace, capability/requirement consistency and strict OpenSpec validation for these six changes only. No Rust build, runtime unit/integration test, frontend coverage or Tier 3 certification is run in Spec. The prior readiness reflection remains pending.

Review covers the complete sibling set, including proposals, designs, tasks and every delta plus this index/manifest. The installed review packet collector only recognizes native-KBD spec files, so its OpenSpec artifact collection must be adapted with an explicit complete file inventory; do not pretend a native-only packet reviewed this set. Completed review results will be recorded under review/spec/. The current Tier 0 receipt is evidence/spec-verification.json; final review status is recorded separately so the packet never claims its own pending review is complete.

ZeeSpec: inactive; no subject workspace exists. Coverage is unknown-acceptable for the Spec gate, not evidence of runtime coverage.

## Uncomfortable scenario

Every individual module can pass while a graph retry still bypasses destination preparation or loses a large tool receipt. The acceptance boundary is the final production request plus canonical receipt identity and cross-protocol trace. If protected data cannot fit any authorized destination, the correct outcome is explicit failure, not a successful but lossy run.

# Shared skill-service fixture repair

Phase: `production-readiness-476-capability-dispatch`, Execute.
Verification: **CLOSED_OPERATOR_LOCK**. All activity in this task was source
inspection or authorized fixture/document writing. No test, review, compiler,
formatter, lint, check, build, API/container probe, import probe, preflight,
`git diff --check`, commit or hook ran. Every change remains **UNVERIFIED**.

Owned source: `tests/context_history_integrity.rs`,
`tests/support/harness_context.rs`, and this handoff. This continuation changes
only `tests/context_history_integrity.rs` plus this new handoff. The prior helper
and all teammates' source/configuration/service work are preserved. The lead owns
the exact final source identity and renewed implementation inventory.

## Source cause

This resolves the previously open **source diagnosis** of the checkpoint
fixture's failure; it does not claim a passing execution. It supplements the
earlier `production-readiness-pending-checkpoint-fixture-handoff.md`, whose open
graph diagnosis records what was known before this additional trace.

Both fixtures construct `RunManager` with actual durable persistence. Its
constructor installs a `SettingsManager`. With no persisted
`skills.matching.engine`, `SettingsManager::skill_matching_engine_for_admission`
(`src/uar/settings/manager.rs:523`) returns `SkillMatchingEngine::default()`,
which is `Service` (`src/uar/runtime/skills/service.rs:50`). The original fixtures
supplied a standalone `SkillRegistry` but no `SkillService`.

`RunSkillBindings::capture` then has no matching snapshot. Before either graph
capture or provider dispatch, manager.rs selects the declared Service owner;
its required snapshot lookup at the `needs_llm` calculation returns
`skill_service_resource_required`. The public failure is
`skill_matching_unavailable`. A default TF-IDF classifier does not replace the
selected owner. The unavailable vector backend is not the cause of this missing
resource and was not replaced with fabricated embedding output.

## Implementation

Both existing cases now create one actual `Arc<SkillService>` with
`SkillService::new(Some(persistence.clone()), None)`, configure it through the
existing `set_matching_config` API with explicit `engine: Service` and
`algorithm: Keyword`, pass `Arc::clone(skill_service.registry())` to RunManager,
and attach the same service Arc using `with_skill_service`.

The Keyword implementation (`service.rs`, `match_in_registry_checked`) ranks
keyword candidates directly and does not acquire an auxiliary LLM or vector
matcher. The fixtures keep their existing unavailable vector resources for the
unused constructor slot; there is no setting that disables or skips matching.
The empty shared skill registry is intentional: these cases exercise history
and checkpoint admission, not a synthetic skill implementation.

The existing cases are:

- `run_manager_persists_pending_tool_calls_and_blocks_resume_dispatch`
- `checkpoint_resume_rejects_raw_metadata_and_legacy_admission_without_mutating_history`

Each now asserts the actual retained run context has requested and effective
method `skill_service.keyword`, with null requested/effective auxiliary models.
The pending case still requires RunStart, exact `PROVIDER_INTERRUPTED`, one
provider call, empty pending assistant content, MissingResult, complete durable
owner-qualified history equality, second Error with `invalid_history` naming
the pending call, and no second provider dispatch. The graph case retains its
unchanged three-second capture bound, diagnostic terminal observation, exactly
one node execution and zero provider calls, raw/revoked metadata denial, direct
legacy admission denial, execution/provider equality and warm-history equality.

The prior pending fixture capability registry and exact artifact destination
remain. Its existing Complete-then-Error synthetic provider sequence remains.
The runtime teammate separately owns native partial-call observation while
preserving provisional completion and caller-tool privacy; no production source
was changed by this fixture writer. No exact budget proof or token count was
invented, no persistence/verified identity was removed, and no new harness was
introduced.

## Remaining decisions and unverified claims

No fixture configuration decision remains open from this source cause. The lead
must reconcile the runtime handoff and whole-phase inventory before requesting
the operator's exact gate commands. Compilation, warnings, actual keyword
context, provider interruption persistence, graph capture timing, all denial
assertions and lifecycle cleanup remain UNVERIFIED. No earlier result certifies
these edits.

No extra feature or defensive production guard was added. Added fixture-stage
assertions trace to the observed unrelated Error acceptance and original graph
capture timeout; they do not replace any original product assertion.

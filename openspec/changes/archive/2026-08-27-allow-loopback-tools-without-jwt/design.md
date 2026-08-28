## Context

See [proposal.md](proposal.md) for the motivation and the capability deltas for the required behavior.

The current server constructs and attaches a `GovernanceEngine` before it initializes the shared `SettingsManager`. The in-run tool gate then evaluates effective run-policy denial, Cedar authorization, and heuristic approval for every tool call. An empty Cedar policy directory can therefore deny tools even though the HTTP request was correctly admitted as anonymous.

The settings system persists typed values and exposes live namespace updates through the frontend entity graph. `governance.default_mode`, `governance.allowed_actions`, and `governance.policy_reload_enabled` already exist. The Governance panel is hand-authored, despite the proposal's initial estimate that the new boolean would be rendered only by the generic schema panel. This design supersedes that frontend impact estimate: the panel, settings API, and entity projection require narrow changes because eligibility is runtime-derived, an Off state changes the meaning of every subordinate control, and settings are applied only after the operator selects Save. No new frontend dependency is required.

Listener and JWT settings describe boot-time security posture. Saving `server.host` or `security.jwt_required` does not rebind the listener or replace the active authentication middleware in the current process. Governance eligibility must therefore come from the effective boot configuration, not from unsaved or restart-pending values in the settings UI.

## Goals / Non-Goals

**Goals:**

- Provide one runtime authority for the exact local-eligibility predicate, the effective On/Off state, validation, and warning cardinality.
- Make a successful persisted setting update visible to subsequent tool calls without restarting the process.
- Keep governance fail-closed when the active listener/authentication posture is ineligible, including after a previously local database is reused elsewhere.
- Present the control in the existing Governance information architecture with accessible active, inactive, pending-save, and mandatory states.
- Preserve configured Cedar options while governance is Off so turning it back On restores the prior policy configuration.

**Non-Goals:**

- Changing tool registration, tool selection, argument validation, provider routing, transport behavior, or execution error handling.
- Treating resolved DNS addresses, IPv6 loopback, wildcard binds, private-network addresses, or alternate localhost spellings as eligible.
- Applying saved listener or JWT changes live; those settings retain their existing restart semantics.
- Adding a repeating toast, per-run warning event, or dismissible warning preference.
- Redesigning the settings navigation, Governance panel visual language, or unrelated authorization settings.

## Decisions

### 1. A shared runtime control owns posture and effective state

Add a small governance runtime control in the governance capability. Bootstrap and `SettingsManager` receive its mutation handle; the settings API and `RunManager` receive read-only status/gate handles. It contains one lock-protected snapshot with:

- a boot-instance identifier and monotonically increasing revision;
- an `Initializing`, On, or Off phase and coherent effective-enabled value;
- the configured host literal, installed authentication mode, and successfully bound listener addresses;
- the persisted operator preference once it has been resolved; and
- a process-scoped `Once` used to emit the inactive warning.

Eligibility requires all of the following: the configured host string is exactly `localhost` or `127.0.0.1`; the authentication middleware actually installed for the process does not require JWT; and every successfully bound UAR listener address is loopback. The literal requirement preserves the specification, while the bound-address check prevents an injected or unexpectedly resolved listener from making a remotely reachable process eligible. Unresolved, wildcard, non-loopback, or otherwise unverified posture is ineligible. Once finalized, the control computes `effective_enabled = !may_disable || persisted_enabled`, so an ineligible process cannot enter governance-disabled local mode even if storage contains a stale `false` value.

The control starts in `Initializing`, which gates as On and projects effective state as Unknown. Boot follows one sealed sequence:

1. Load `governance.enabled` as optional without seeding or applying it.
2. Install the authentication middleware and record the mode actually installed.
3. Bind every declared tool-capable ingress, including the primary HTTP listener, any companion listener, and enabled A2A gRPC ingress. Each ingress must register its bound address and receive an unforgeable registration proof from the control. A primary listener is required.
4. Seal and verify the inventory. Sealing rejects later registration and fails startup unless every declared ingress supplied a proof. It prepares an admission token for each registered ingress, but those tokens remain inactive until successful governance finalization.
5. Derive eligibility from the allowed configured literal, installed authentication mode, and sealed bound-address inventory.
6. Under the governance mutation lock, durably seed a missing row, normalize an ineligible `false`, or retain the loaded value; then finalize the coherent snapshot and warning exactly once.
7. Activate the admission tokens and construct/expose `RunManager`. An ingress cannot enter its serve loop or admit a run without an active token. Construction/registration after seal fails rather than creating an unaccounted network path.

Missing, late, failed, or unverified ingress proof makes the posture ineligible. Unavailable or failed persistence finalizes On with `mutation_available = false`; it never substitutes the local Off default because an unreadable store might contain an operator-selected On value. Off is possible only after reading a durable `false` or successfully inserting the eligible missing-row default of `false`. An eligible Off finalization emits the inactive warning before request admission. There is no interval in which a persisted On installation transiently gates Off.

Each projection is one coherent snapshot read under the same lock. The boot-instance identifier changes on restart; revisions are compared only within that instance. This makes a new process authoritative even when a browser retains a larger revision from the previous process. The configured-literal and bound-address predicates are intentionally different: configured `::1` is not eligible, but bound `::1` is valid loopback proof when produced by an allowed configured `localhost` or `127.0.0.1` listener. Predicate tests cover both cases.

Alternatives considered:

- Reading `governance.enabled` from persistence at every tool call was rejected because it couples the hot path to storage availability and makes warning cardinality and fail-closed behavior harder to centralize.
- Attaching or detaching `GovernanceEngine` was rejected because `None` already means permissive behavior in parts of the tool gate and does not bypass effective run-policy denial. A first-class effective state makes the complete bypass explicit.
- Deriving eligibility from mutable settings rows was rejected because those rows can describe a future restart rather than the active listener and middleware.
- Treating the configured host literal alone as proof of reachability was rejected because the server accepts an injected listener and may create a companion listener; the successfully bound addresses are the final network-boundary evidence.

### 2. `SettingsManager` validates and synchronizes the special setting

Add `governance.enabled` to the Governance schema. Its first-boot seed is posture-dependent: `false` only for the exact eligible posture and `true` otherwise. Existing API-owned values remain durable on restart when the posture remains eligible.

All writes continue through `SettingsManager::set_value`. A single async Governance-namespace mutation mutex serializes every `governance.*` single-key set/reset, complete namespace batch, seed, and normalization. A batch holds the mutex from its first prerequisite policy write through master validation, durable writes, cache replacement, runtime publication, token creation, and notification scheduling; participating single-key writers cannot interleave. For `governance.enabled`, the manager asks the shared runtime control to validate the requested boolean before persistence. A requested `false` in an ineligible process returns a validation error without changing storage, cache, or runtime state. The serialized linearization sequence is: validate → durable write → infallible cache replacement → coherent runtime snapshot/revision publish → change notification scheduling → response. A save is successful only after runtime publication. This order prevents a failed durable save from changing live enforcement, prevents interleaved writers from defeating policy-before-On dependencies or publishing out of storage order, and ensures frontend change notifications cannot precede the effective transition. There is no fallible step between cache replacement and snapshot publication; an unexpected panic terminates the process rather than returning a contradictory success.

During the sealed boot sequence, an ineligible process that encounters a persisted `false` normalizes it to `true` before exposing settings or accepting runs. The normalization is idempotent. Its structured event is `governance.persisted_state_normalized`, distinct from the inactive warning and unable to consume its `Once`. If the normalization write fails, the runtime finalizes On, marks governance mutation unavailable in the status projection, and does not expose a writable Governance form; it never activates the stale Off value.

Reset uses the same posture-derived default and the same validation path. Bulk namespace updates retain their existing per-key result model rather than claiming an unsupported cross-provider transaction, but return a result for every submitted key, including `updated`, `validation_rejected`, `dependency_failed`, and `skipped`. The frontend snapshots the submitted draft version, disables all submitted controls while saving, and reconciles each result: clear a successfully saved draft only if it still equals the submitted version; preserve edits made after submission; preserve transient, dependency-failed, skipped, and unrelated failed drafts; and clear a rejected `governance.enabled` draft only for authoritative posture validation after refreshing status so it cannot masquerade as effective Off.

The backend processes policy keys before an Off-to-On master transition; if any submitted policy key fails, it does not attempt that On transition and returns `dependency_failed` for the master. An On-to-Off transition may still proceed after an unrelated policy failure because disabling enforcement does not activate that policy. Responses and announcements distinguish complete success from partial success and name every key that was applied or retained as a draft.

Change notification delivery is non-transactional after commitment: delivery failure cannot turn an applied mutation into an API error. The response still returns success with the applied status token and logs a distinct delivery failure. Remote panels recover by refetching on live-channel reconnect, window focus, namespace Refresh, and a low-frequency status revalidation while the Governance panel remains mounted.

Alternatives considered:

- Validating only in the HTTP handler was rejected because admin and embedded callers can mutate settings through other host-owned paths.
- Keeping a persisted `false` while reporting an effective `true` was rejected because the Governance switch would misrepresent the active state and could unexpectedly reactivate Off after a later posture change.

### 3. The tool gate bypass is the first decision when governance is Off

`RunManager` captures the shared runtime gate handle in the approval-gate closure. For each tool call it reads the effective state from one coherent snapshot before evaluating the effective run-policy tool setting, Cedar, the heuristic risk classifier, or approval waiting. When Off, it immediately returns a distinct `GovernanceBypassed` result to the executor. Registration, selection, schema/argument validation, provider/transport work, and execution remain downstream and unchanged.

When On, the existing decision order and event semantics remain intact: effective policy denial rejects; Cedar may allow, deny, or require approval; and risk/Ask paths may wait for operator approval. A toggle affects the next gate evaluation, including a later tool call in an already-running session, but does not revoke or replay a tool call whose gate decision has already completed.

Extend the internal approval result with a distinct governance-bypass outcome instead of returning ordinary `Approved`. The executor maps that outcome to proceed, emits none of `ToolCallApprovalRequired`, approval-granted, or governance-denied, and records only an internal `decision_source = "governance_disabled"` trace. A focused test observes the normalized event sink so bypass cannot masquerade as operator or policy approval now or after later telemetry changes. The one operator warning remains the only inactive-governance warning event.

Alternatives considered:

- Making Cedar use a synthetic permit-all policy was rejected because effective run-policy denial and heuristic approval would still run.
- Filtering only `web_search` or a tool allowlist was rejected because the requested local posture is governance-free for every otherwise available tool.

### 4. One structured process warning comes from the runtime control

The first finalized transition to effective Off calls the control's process-local `Once` and emits exactly one `tracing::warn!` event named `governance.inactive_local_mode`. Its stable fields are `boot_instance_id`, `configured_host`, `bound_addresses`, `jwt_required = false`, `effective_enabled = false`, and `bypassed_gates = "cedar,run_policy,risk_approval"`. Later settings reads, requests, runs, tool calls, or On-to-Off transitions in the same process cannot emit it again. Initialization and normalization never consume this `Once`. Persistence failure always finalizes On; the inactive warning occurs only after a durable `false` read or successful eligible default insertion.

The warning is not persisted. Restarting creates a new control and therefore a new one-warning scope, matching the specification.

Alternatives considered:

- Warning in the tool loop was rejected because it naturally repeats and fails to warn until a tool is called.
- Persisting a “warning seen” flag was rejected because the warning scope is the process, not the user or installation.

### 5. Keep the control at the top of the existing Governance panel

Retain Governance under the existing “Governance & Agents” navigation category. Do not move the switch to Security or Native Tools: JWT/listener values determine whether Off is available, but the switch changes the Cedar, run-policy, and approval system as a whole.

The hand-authored Governance panel becomes a dedicated master-detail form:

1. A top card labeled **Enforce tool governance** contains the incumbent `Toggle` control and a compact badge for the authoritative effective state: On, Off, or Required. “God mode” is internal shorthand only and never appears in interface copy.
2. Effective Off shows a separate persistent, non-dismissible warning region: “All available tools can run without Cedar policies, run-policy restrictions, or approval prompts.” This visual state is not the process warning and does not create a notification event.
3. A neutral pending note describes an unsaved draft and its consequence: “After Save, all available tools can run without Cedar policies, run-policy restrictions, or approval prompts” for draft Off, or “After Save, policy checks and approval prompts resume” for draft On.
4. A mandatory state uses the complete reason-code set from the runtime projection. The closed codes are `initialization_incomplete`, `configured_host_not_allowed`, `authentication_unverified`, `jwt_required`, `ingress_inventory_unsealed`, `ingress_proof_missing`, `bound_ingress_not_loopback`, and `persistence_unavailable`; multiple codes may be present and are rendered together rather than hidden by precedence. Copy names each active reason, for example “Required because JWT is enabled and a bound ingress is not loopback.” Supporting help states which configuration must change and that UAR must restart before Off can become available. The invariant is `may_disable == false` implies `effective_enabled == true` after finalization.
5. Existing authorization mode, allowed actions, and hot-reload controls remain visible under the semantic legend **Policy behavior when governance is on**. Their availability follows the draft master value: they are enabled for draft On so the master and policy values can be saved together, and disabled for draft Off. Toggling Off never clears policy drafts; Save persists the master state and preserved policy values, while Reload discards all drafts.
6. Durable state, warning, pending information, and errors remain visually and semantically distinct: the compact badge reports effective state; only effective Off uses the warning region; drafts use a neutral note; and save failures use the existing assertive error alert. Only asynchronous saving and confirmation messages use a polite live region.

The existing `Toggle` primitive receives the narrow support needed for `id`, label association, `aria-describedby`, and a focusable `aria-disabled` state whose pointer and keyboard changes are guarded. Every existing Governance select, switch, and allowed-actions group also receives stable programmatic labeling. The subordinate controls use a real `<fieldset disabled aria-describedby>` and `<legend>`; the visible explanation sits outside individual disabled controls. Status and warning copy use explicit text plus iconography, never color alone.

The dedicated panel reads saved settings, per-key drafts, saving state, errors, and the authoritative runtime projection separately. It owns one atomic polite announcement rather than combining the generic `SavedBanner` and `PanelHeader.statusText`. The state contract is:

| State | Master presentation | Policy controls | Announcement / recovery |
|---|---|---|---|
| Loading or projection unavailable | No boolean switch claim; badge is Unknown and editing is disabled | Disabled | “Runtime governance status is unavailable; enforcement cannot be verified.” Keep Refresh focusable. A cached value, if shown, is labeled last-known. |
| Mutation unavailable | Authoritative On/Off/Required badge; all setting controls disabled | Disabled | Persistent recovery copy names the storage/status failure and keeps Refresh focusable. Effective enforcement remains projection-driven. |
| Required | Checked, focusable, `aria-disabled` toggle; Required badge and all applicable reasons | Enabled | No live announcement on initial render. A newly accepted remote status token is announced once while mounted. |
| Saved On | Checked toggle; On badge | Enabled | No live announcement for the durable state. |
| Saved Off | Unchecked toggle; Off badge and persistent warning | Disabled | No repeating live announcement. |
| Draft On | Checked toggle; effective badge remains Off; neutral pending note | Enabled | “After Save, policy checks and approval prompts resume.” |
| Draft Off | Unchecked toggle; effective badge remains On; neutral pending note | Disabled | “After Save, all available tools can run without Cedar policies, run-policy restrictions, or approval prompts.” |
| Saving On / Off | Toggle, Save, and every submitted control disabled; effective badge remains authoritative | Submitted fieldset is disabled regardless of draft value | Polite “Turning tool governance on…” or “Turning tool governance off…”. |
| Save confirmed | Draft clears only after the returned/refetched projection confirms or supersedes the applied token | Follows confirmed state | One polite complete-success, partial-success, or “saved, then changed elsewhere” announcement. |
| Save rejected or partial | Effective badge and warning remain projection-driven; reconcile every returned key independently | Follows the reconciled master value | Clear unchanged `updated` drafts; preserve failed, skipped, dependency-failed, and post-submit edits; clear the master only for authoritative posture rejection after status refresh; announce applied keys and remaining drafts. Never show complete success for a partial result. |

Base-state precedence is Unknown → Mutation unavailable → Required → Draft → Saved. Saving is an orthogonal overlay: it disables the Save action and every submitted control regardless of any simultaneous Required or Draft base state, while the authoritative badge and reasons may continue to update. This prevents a Required update during save from re-enabling submitted controls.

Layout stays in the existing single-column responsive flow; the master row uses `minmax(0, 1fr)` plus an auto-sized control track, long status/reason copy can wrap, and actions wrap without horizontal scrolling. At 320 CSS px and at a desktop viewport under 200% browser zoom, the page, card, and fieldset must have no horizontal scrolling, clipping, or overlap; wrapped actions retain visible focus and operable targets. Keyboard acceptance verifies the master switch's accessible name/state/description, Space and Enter suppression while Required, logical Tab order, visible focus, and descendant disabling within the policy fieldset. Text in the warning treatment must measure at least 4.5:1 contrast and meaningful non-text boundaries 3:1 in both themes. No new modal, drawer, or confirmation step is introduced because the explicit Save action is already the commit boundary.

The Governance navigation item remains in place but its subtitle becomes **Tool policies, approvals, and enforcement** so the master control is findable without duplicating it under Security or Native Tools.

This treatment follows the task-specific distillation from Impeccable critique/audit guidance, Anthropic `frontend-design`, UI/UX Pro Max, and the Vercel Web Interface Guidelines: preserve the incumbent design system, place the master control before its dependents, keep labels and consequences adjacent, use semantic controls and visible focus, expose asynchronous status through polite live regions, keep corrective errors actionable, and avoid transient toast-only disclosure for a durable security state.

Alternatives considered:

- Placing the switch in Security was rejected because it would separate the master control from the policy settings it governs.
- Hiding all policy controls while Off was rejected because it obscures preserved configuration and makes re-enablement less predictable.
- Leaving dependent controls editable while Off was rejected because edits would appear effective when they are not.
- Adding a confirmation modal was rejected because the local-only eligibility boundary and explicit Save action already prevent accidental immediate activation; the inline warning communicates the consequence without interrupting every toggle.

### 6. The UI consumes an authoritative governance-status projection

Expose a read-only governance status projection from the settings API containing `boot_instance_id`, `revision`, `phase`, `may_disable`, `effective_enabled`, `configured_host`, `bound_addresses`, `active_jwt_required`, `mutation_available`, and stable `ineligibility_reason_codes`. It is one coherent snapshot from the shared runtime control, not a collection of separately read atomics or submitted settings values. Reason codes, rather than server-authored prose, drive localized UI copy. The frontend treats unknown codes and impossible combinations as Unknown instead of rendering a misleading boolean state.

When the Governance namespace loads, the existing settings store also fetches this projection and ingests it as one normalized entity. A focused settings-domain hook subscribes to that entity; the component does not fetch, inspect global stores, or import transports. Every local or remote `governance.enabled` change notification triggers a projection refresh after the server has applied the shared control.

Within one boot instance, the store accepts only a higher revision, except that the mutation response acknowledging the current revision may complete its matching save. Each status or mutation request receives a monotonically increasing client request sequence, and the store records `lastAcceptedRequestSequence`. A response with a lower sequence cannot replace accepted same-instance state. When the newest acceptable response carries a different boot-instance identifier, the store adopts it regardless of numeric revision, advances `lastAcceptedRequestSequence`, retires/cancels requests from the old instance, and immediately starts one confirmation request against the adopted instance. Responses from a retired instance cannot directly mutate state.

Each mutation returns the applied boot-instance/revision token. A save completes when that token is observed, a newer same-instance token supersedes it, or the post-restart confirmation produces a terminal outcome. Mutation and confirmation requests use a 10-second deadline; timeout cancels the request and terminates as outcome Unknown. The bounded terminal outcomes are confirmed, partial/changed elsewhere, rejected, or outcome Unknown; none leaves the UI indefinitely Saving. If a newer projection differs, the UI reconciles immediately and announces “Settings saved, then changed elsewhere. Tool governance is now On/Off.” If the old process committed but the new process cannot confirm the durable result, the UI reports “The previous runtime stopped before the save outcome could be verified” and presents Unknown/Refresh rather than success or rejection. Accepted remote revisions are deduplicated by effective state: policy-only revisions remain silent, while each On, Off, or Required transition accepted while the panel is mounted is announced once. Initial durable rendering stays silent.

These rules preserve the existing component → hook → store → service layering, handle restart, second-window, or host-owned mutations, and prevent unsaved host/JWT drafts from unlocking the switch.

The endpoint is observational only. The persisted `governance.enabled` setting remains the sole mutation surface, and the backend still validates every Off write. Missing status is Unknown, not On: the UI makes no boolean enforcement claim, disables editing, retains a focusable Refresh action, and states that enforcement cannot be verified.

Alternatives considered:

- Computing eligibility in the component from the Server and Security namespaces was rejected because those values can be dirty or restart-pending and would duplicate a security predicate in TypeScript.
- Adding eligibility as another persisted setting was rejected because eligibility is derived runtime state, not operator configuration.
- Extending every generic setting record with governance-specific metadata was rejected because it would leak one capability's concerns into the generic settings contract.

## Risks / Trade-offs

- **[Risk] A user saves a new host or JWT setting and expects governance eligibility to change immediately.** → The panel uses active runtime status, not drafts, and the locked-state copy names the active listener/JWT condition. Existing restart semantics remain unchanged.
- **[Risk] A persisted local `false` value is reused when the runtime becomes remotely reachable.** → Bootstrap normalizes it to `true`, and the runtime control independently forces effective On for every ineligible posture.
- **[Risk] A tool call races with a settings save.** → The control changes only after durable persistence succeeds and before the updated setting is published; each gate observes either the complete old or complete new snapshot, never partial state.
- **[Risk] Governance is disabled while a run has already requested approval.** → The change applies to subsequent gate evaluations only. Existing pending approvals are not auto-approved or cancelled; documenting this boundary avoids silently mutating in-flight security decisions.
- **[Risk] A locked switch can hide why it is unavailable from keyboard or assistive-technology users.** → Keep it focusable with guarded `aria-disabled` behavior, associate the reason via `aria-describedby`, and keep the explanation visible outside the control.
- **[Risk] A status refresh races with another settings mutation or server restart.** → Read coherent snapshots; identify the boot instance; order server publication after the runtime transition; and apply the instance/revision/request-order rules above.
- **[Risk] Multiple policy drafts partially save.** → Snapshot the submitted versions, disable submitted controls, reconcile per key, and explicitly report any policy values saved before a rejected master transition.
- **[Trade-off] Policy controls remain visible while inactive.** → This consumes more vertical space than hiding them, but preserves discoverability and communicates that configuration is retained.
- **[Trade-off] The status projection adds a narrow read endpoint and entity.** → It prevents the security-sensitive UI from inferring active posture from stale configuration and keeps backend enforcement authoritative.

## Migration Plan

1. Before implementation planning, synchronize the earlier OpenSpec artifacts with this inspected design: replace the proposal's generic-UI impact estimate; specify the sealed boot-effective eligibility and restart behavior; and add UI scenarios for authoritative, pending, required, mutation-unavailable, Unknown, rejected-save, and dependent-control states. This is an artifact correction, not a behavior expansion.
2. Add the runtime control and focused predicate, initialization/finalization, coherent snapshot, bound-listener, state, and warning tests without changing the tool gate. Use deterministic barriers to prove a gate read before publication uses the old state, a read after publication uses the new state, and completed or pending decisions retain the documented behavior.
3. Add posture-dependent schema seeding, bootstrap normalization, write validation, and persistence/restart tests.
4. Reorder server composition so the shared control and initialized settings manager are available to `RunManager`; retain the existing governance engine for the On path.
5. Put the control check first in the tool approval gate and add focused tests for Off bypass, On denial, live toggling, and unchanged capability failures.
6. Add the read-only status projection and the Governance panel master-detail states. Add the per-key draft-reconciliation helper rather than clearing unrelated drafts. Verify programmatic names/descriptions, the complete save-state contract, remote announcements, locked and Unknown behavior, the instance/revision races, keyboard/focus operation, the explicit 320 CSS px and 200% zoom pass conditions, warning contrast, and light/dark themes using the required UI quality gates.
7. Run the repository's tiered local verification at the plan-defined tier. No non-deployment test is added to GitHub Actions.

Deployment requires no database schema migration because settings types and values are already schema-managed. First startup inserts the new row only after the ingress inventory is sealed and eligibility is known. Existing eligible installations receive the local Off default only when a successful read proves the row is missing; ineligible installations seed On or perform the idempotent persisted-value normalization described above. A settings read, seed, or normalization failure leaves runtime enforcement On and makes governance mutation visibly unavailable.

Forward deployment installs the status/control backend before the UI and verifies the old/new backend–UI compatibility matrix: old UI with new backend, new UI with a missing status endpoint, and new UI with the new backend. The release deliverables include a verified rollback backend/commit that unconditionally enforces On while retaining a truthful status endpoint with `mutation_available = false`; it is built and exercised before forward deployment. Rollback deploys that artifact first, then removes the UI and endpoint. Each supported downgrade target's unknown-row behavior and reversible row-removal procedure are pre-verified. If removal is required, export and retain the row/value before deleting that single row so cancellation can restore it. Rollback never interprets the row as permission to bypass governance.

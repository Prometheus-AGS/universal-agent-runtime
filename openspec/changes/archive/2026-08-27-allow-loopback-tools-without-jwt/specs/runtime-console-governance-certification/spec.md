## ADDED Requirements

### Requirement: Eligible local governance is persisted and defaults Off
The runtime SHALL expose a persisted boolean `governance.enabled` setting. When no persisted value exists, it SHALL default to `false` for a governance-optional local posture and to `true` for every other posture. An eligible local operator SHALL be able to turn the setting On or Off, and the saved value SHALL survive restart. Outside the eligible local posture, the runtime SHALL reject an attempt to save `false` and SHALL keep governance enabled.

#### Scenario: New eligible local runtime defaults Off
- **WHEN** an eligible local runtime starts without a persisted `governance.enabled` value
- **THEN** the setting is seeded as `false` and the settings surface reports governance as Off

#### Scenario: New non-local runtime defaults On
- **WHEN** a runtime that is not eligible for governance-optional local mode starts without a persisted `governance.enabled` value
- **THEN** the setting is seeded as `true` and governance is enforced

#### Scenario: Local operator enables governance
- **WHEN** an eligible local operator changes `governance.enabled` from `false` to `true`
- **THEN** subsequent tool-governance decisions use the complete governance and approval path without requiring a listener change or restart

#### Scenario: Local operator disables governance
- **WHEN** an eligible local operator changes `governance.enabled` from `true` to `false`
- **THEN** subsequent tool-governance decisions use governance-inactive local mode without requiring a listener change or restart

#### Scenario: Local setting survives restart
- **WHEN** an eligible local operator saves `governance.enabled` and restarts UAR with the same eligible posture
- **THEN** the runtime preserves and applies the saved On or Off value instead of reseeding the default

#### Scenario: Ineligible operator attempts to disable governance
- **WHEN** JWT is required or the configured host is not exactly `localhost` or `127.0.0.1` and an operator attempts to save `governance.enabled` as `false`
- **THEN** the settings update is rejected with a validation error and governance remains enabled

### Requirement: Governance-disabled local runs bypass governance decisions
While an eligible local runtime has `governance.enabled` set to `false`, it SHALL execute each available configured tool without applying Cedar authorization, effective run-policy tool denial, or risk-based human approval. Tool registration and selection, argument validation, transport behavior, provider behavior, and ordinary execution failures SHALL remain unchanged. When governance is enabled, the complete existing governance and approval behavior SHALL apply.

#### Scenario: Available web search executes while governance is Off
- **WHEN** an eligible local anonymous run calls an available configured `web_search` tool while `governance.enabled` is `false`
- **THEN** the tool executes without a governance-denied or approval-required outcome

#### Scenario: Governance rules do not override governance-inactive local mode
- **WHEN** an eligible local tool call would otherwise be denied by Cedar, denied by the effective run policy, or routed to risk approval and `governance.enabled` is `false`
- **THEN** those governance outcomes are not evaluated or emitted and the available configured tool proceeds to execution

#### Scenario: Disabled or unselected tool remains unavailable
- **WHEN** a tool is not registered, not selected, or otherwise unavailable while local governance is Off
- **THEN** the runtime does not invent or expose the tool and reports the ordinary capability or execution failure

#### Scenario: Enabled governance preserves denial semantics
- **WHEN** `governance.enabled` is `true` and Cedar denies a tool execution
- **THEN** execution stops, an auditable denial is emitted, and human approval cannot override it

### Requirement: Inactive governance posts one warning per process
The runtime SHALL emit an operator-visible structured warning the first time governance is observed inactive in a process. It SHALL emit no additional inactive-governance warning in that process for later settings reads, requests, runs, tool calls, or repeated Off transitions. A new process SHALL be eligible to emit its own single warning if governance is inactive.

#### Scenario: Process starts with governance Off
- **WHEN** an eligible local process starts with `governance.enabled` set to `false`
- **THEN** it emits exactly one structured warning that governance is inactive and local tool execution is unrestricted

#### Scenario: Governance is disabled after startup
- **WHEN** a process starts with governance On and an eligible local operator later changes `governance.enabled` to `false`
- **THEN** the process emits its one inactive-governance warning when the Off state is first applied

#### Scenario: Repeated local activity does not repeat the warning
- **WHEN** a process has already emitted its inactive-governance warning and then serves additional requests, runs, tool calls, settings reads, or On-to-Off transitions
- **THEN** no additional inactive-governance warning is emitted by that process

#### Scenario: Restart creates a new warning scope
- **WHEN** UAR restarts into an eligible posture with governance still Off
- **THEN** the new process emits exactly one inactive-governance warning

### Requirement: Governance status is one coherent runtime authority
The runtime SHALL expose one read-only Governance status projection containing a boot-instance identifier, a monotonically increasing revision scoped to that boot instance, phase, effective enabled state, mutation availability, boot-effective configured host, sealed bound addresses, installed JWT mode, and a closed list of simultaneous reason codes. A single projection SHALL be read from one coherent snapshot. The effective state SHALL be Unknown while Initializing, Required when the boot posture is ineligible, On when eligible and enabled, and Off only when eligible and durably disabled.

#### Scenario: Initializing status is Unknown and gates On
- **WHEN** the process has not finalized ingress inventory and durable preference resolution
- **THEN** status reports phase Initializing and effective state Unknown, mutation is unavailable, and every tool call uses the complete governance path

#### Scenario: Ineligible posture reports Required with all reasons
- **WHEN** more than one mandatory condition applies, such as installed JWT-required authentication and a non-loopback bound ingress
- **THEN** status reports effective state Required and includes every applicable closed reason code without collapsing them to one message

#### Scenario: Eligible durable preference reports authoritative On or Off
- **WHEN** the process finalized an eligible posture from a durable `governance.enabled` value
- **THEN** one coherent projection reports that durable effective state, its boot instance, and its revision

#### Scenario: A restart replaces the prior revision domain
- **WHEN** a new process starts with a new boot-instance identifier while a client retains a larger revision from the prior process
- **THEN** the client treats the new boot instance as authoritative and does not reject its status because of the prior process revision

#### Scenario: Impossible status combinations are rejected
- **WHEN** a producer or client encounters a projection such as Off with mutation unavailable, Off with a mandatory reason, or Required with `may_disable` true
- **THEN** it rejects the malformed projection and presents effective state Unknown rather than inventing a writable Off state

### Requirement: Preference resolution and mutation fail closed
The runtime SHALL gate governance On when persistence is unreadable or unavailable or when reading, seeding, or normalizing `governance.enabled` fails. It SHALL report Governance mutation unavailable and SHALL NOT emit the inactive-governance warning. Effective Off SHALL require either a durable `false` read or successful insertion of the eligible missing-row default. An ineligible persisted `false` SHALL be normalized to `true` before request admission; normalization failure SHALL leave the runtime On and non-writable.

#### Scenario: Persistence is unavailable during boot
- **WHEN** the settings store cannot be opened or read while governance posture is initialized
- **THEN** governance finalizes On, status reports mutation unavailable with its reason, no inactive warning is emitted, and ingress admission may activate only with that fail-closed state

#### Scenario: Eligible default seed fails
- **WHEN** an eligible process has no row and inserting the posture-derived `false` default fails
- **THEN** governance finalizes On and mutation unavailable instead of applying a non-durable Off default

#### Scenario: Ineligible stale false is normalized
- **WHEN** an ineligible process reads a persisted `governance.enabled=false` and successfully writes `true`
- **THEN** governance finalizes Required, persistence reports `true`, and the runtime emits `governance.persisted_state_normalized` without consuming the inactive-warning scope

#### Scenario: Ineligible normalization fails
- **WHEN** an ineligible process reads a persisted `false` but cannot durably normalize it
- **THEN** governance finalizes Required with mutation unavailable, does not expose a writable form, and does not apply or warn about inactive governance

### Requirement: Governance mutations are serialized and authoritatively confirmed
Every `governance.*` single-key set or reset, namespace seed or normalization, and complete Governance namespace batch SHALL execute under one async namespace mutation mutex. A successful mutation SHALL linearize validation, durable write, infallible cache replacement, coherent runtime snapshot and revision publication, change-notification scheduling, and response in that order. The response SHALL carry the applied boot instance and revision. Notification delivery failure after commitment SHALL NOT turn the applied mutation into an API error.

#### Scenario: Concurrent writers preserve storage and runtime order
- **WHEN** a single-key policy writer races a batch that changes policy fields and turns governance On
- **THEN** the Governance mutex prevents interleaving, and durable storage, cache, runtime status, and response tokens reflect one serial order

#### Scenario: Off-to-On waits for submitted policy prerequisites
- **WHEN** a batch submits policy fields plus `governance.enabled=true` and a submitted policy field fails
- **THEN** the runtime does not attempt the master transition and returns `dependency_failed` for `governance.enabled`

#### Scenario: On-to-Off can proceed despite unrelated policy failure
- **WHEN** a batch submits `governance.enabled=false` plus an unrelated policy value that fails validation
- **THEN** the safe master transition may commit and the response reports the master as `updated` and the failed policy key separately

#### Scenario: Partial batch names every submitted result
- **WHEN** a Governance namespace batch is only partly applicable
- **THEN** the response includes exactly one `updated`, `validation_rejected`, `dependency_failed`, or `skipped` result for every submitted key and identifies which drafts remain unapplied

#### Scenario: Authoritative master rejection leaves state truthful
- **WHEN** an ineligible process rejects `governance.enabled=false`
- **THEN** storage, cache, and runtime stay enabled, the response is `validation_rejected`, and a refreshed status reports Required rather than transient Off

#### Scenario: Post-commit notification delivery fails
- **WHEN** durable write and runtime publication succeed but realtime notification delivery fails
- **THEN** the API returns success with the applied status token, logs a distinct delivery failure, and clients recover through status revalidation

### Requirement: Governance settings distinguish durable, draft, and saving state
The existing Governance panel SHALL place a master control labeled **Enforce tool governance** above dependent policy controls and SHALL render its badge solely from authoritative status. Draft changes SHALL NOT alter the badge or inactive warning before authoritative confirmation. The panel SHALL represent Unknown, mutation unavailable, Required, On, Off, Saving, confirmed, partial, changed-elsewhere, rejected, and timeout outcomes without an indefinite Saving state.

#### Scenario: Initial durable state is silent
- **WHEN** the Governance panel first renders an authoritative On, Off, or Required state
- **THEN** it shows the matching badge and description without announcing a transition that did not occur during the user's session

#### Scenario: Draft Off does not masquerade as durable Off
- **WHEN** authoritative status is On and the operator changes the unsaved master draft to Off
- **THEN** the badge remains On, the inactive warning remains absent, and a neutral draft note says the change takes effect after Save

#### Scenario: Draft On preserves durable Off warning
- **WHEN** authoritative status is Off and the operator changes the unsaved master draft to On
- **THEN** the badge remains Off and the persistent inactive-governance warning remains visible until authoritative confirmation

#### Scenario: Saving waits for authoritative confirmation
- **WHEN** the operator saves a Governance draft
- **THEN** submitted controls enter Saving, success is not announced from the HTTP mutation response alone, and the operation terminates as confirmed, partial or changed, rejected, timeout, or Unknown after status reconciliation

#### Scenario: Partial save preserves unapplied and newer drafts
- **WHEN** some submitted keys apply while others fail or the operator edits a submitted field after the request began
- **THEN** only unchanged successfully applied draft versions clear, post-submit and failed drafts remain, and the result message names applied and retained keys

#### Scenario: Authoritative rejection clears only the rejected master draft
- **WHEN** a saved Off master draft is rejected because active posture is mandatory
- **THEN** the panel refreshes status, clears that rejected master draft, reports Required with reasons, and preserves every unrelated policy draft

#### Scenario: Remote revision arrives during a save
- **WHEN** another writer publishes a newer accepted revision before the pending mutation is confirmed
- **THEN** the panel terminates Saving as changed elsewhere, accepts the newer authoritative status, and preserves local drafts that cannot be matched to the confirmed mutation token

#### Scenario: Confirmation deadline expires
- **WHEN** mutation or authoritative confirmation does not complete within ten seconds
- **THEN** the request is cancelled where possible, status becomes Unknown, Saving ends, local drafts remain, and Refresh is offered without claiming failure or success of an unobserved commit

### Requirement: Governance master and dependent policies are accessible
The Governance master SHALL be a programmatically labeled semantic switch with its current state and description exposed to assistive technology. Required or mutation-unavailable states SHALL remain focusable with `aria-disabled`, SHALL guard Space and Enter from changing the value, and SHALL associate every mandatory reason with the control. Dependent policy controls SHALL be grouped in a semantic fieldset with a legend and SHALL become semantically disabled while durable governance is Off or while submitted values are saving, without deleting their drafts.

#### Scenario: Assistive technology receives name state and reasons
- **WHEN** a screen-reader user focuses the governance master in Required state with multiple reasons
- **THEN** the accessible name is “Enforce tool governance”, state is On and unavailable, and every mandatory reason is included in its accessible description

#### Scenario: Locked switch remains discoverable but cannot toggle
- **WHEN** the master is Required or mutation unavailable and receives keyboard Space or Enter
- **THEN** focus remains visible, the value does not change, and no mutation request is sent

#### Scenario: Dependent policy fieldset follows durable state
- **WHEN** authoritative governance becomes Off
- **THEN** the dependent fieldset is disabled with an explanatory legend or description, its values and unsaved drafts are preserved, and the master remains first in logical Tab order

#### Scenario: Atomic announcements separate transitions from errors
- **WHEN** a local confirmation or accepted remote revision changes authoritative state
- **THEN** one atomic polite announcement names the new durable state; partial or rejected saves use an assertive error message and never announce success

### Requirement: Inactive warning has stable operational and visual semantics
The runtime SHALL emit the structured event `governance.inactive_local_mode` exactly once per process after the first finalized effective Off state and before request admission. The event SHALL contain boot instance, configured host, bound addresses, `jwt_required=false`, `effective_enabled=false`, and bypassed-gate fields. Independently, the Governance panel SHALL display a persistent non-dismissible warning whenever authoritative state is Off and SHALL remove it only after authoritative On or Required confirmation.

#### Scenario: Runtime warning contains stable fields
- **WHEN** a process first finalizes governance Off
- **THEN** exactly one `governance.inactive_local_mode` event is emitted with all required fields before an ingress can admit a request

#### Scenario: Reads and repeated toggles do not repeat the runtime warning
- **WHEN** the process has emitted its inactive warning and later serves status reads, runs, tool calls, or additional On-to-Off transitions
- **THEN** the event is not emitted again in that process

#### Scenario: Visual warning follows only authoritative Off
- **WHEN** authoritative status is Off
- **THEN** the panel persistently warns that all available tools can run without Cedar policies, run-policy restrictions, or approval prompts, while a draft or unconfirmed response alone cannot show or hide that warning

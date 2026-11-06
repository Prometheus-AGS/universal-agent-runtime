# A2UI interaction recovery

A logical user click has its own `interaction_id`. Updated clients persist the
original intent, payload and surface generation before POST. Transport retries,
new wire-run IDs, reconnects and lost acknowledgments retain that identity. A
second intentional click allocates another ID. Timestamps do not identify clicks.

The trusted host binds a receipt to the verified credential principal, agent,
thread, surface generation, exact Button/component/action and semantic payload
digest. It reserves a host-generated continuation run ID durably before dispatch.
Only the creator may issue that execution command. Other contenders read the
original receipt; a changed payload conflicts. Receipts are immutable admission
tombstones: they are not automatically expired or deleted during run-cache
retention. An unavailable backing store refuses protected admission. SurrealDB
provides this capability; the default PostgreSQL implementation explicitly reports
unsupported. A legacy request without an interaction ID retains fresh-request
semantics and reports `retry_protected:false`; retrying it may execute again.

REST clients use these owner-qualified paths:

- `GET /api/uar/runs/{source}/a2ui/surface-authority` returns `status`,
  `authoritative`, `max_prior_runs`, `max_surfaces`, `recovery_hint` and `surfaces`.
  Surface entries contain `surface_id`, `surface_generation` and `source_run_id`.
- `POST /api/uar/runs/{source}/a2ui/actions` accepts existing camel-case action
  fields plus `interaction_id` and `surface_generation` (camel-case aliases are
  accepted). A protected request requires both a nonempty ID and generation.
- `GET /api/uar/runs/{source}/a2ui/actions/{interaction_id}` reconciles without
  dispatch. A found receipt returns202 with `run_id`, `continuation_run_id`,
  `status:"continued"`, `interaction_id`, `retry_protected:true` and `outcome`.
  The outcome is `running`, `completed`, `failed`, `cancelled` or `uncertain`.
  The outcome describes the native run, not an external system's independent
  settlement. A404 is not permission to resend an unconfirmed effect.

An existing REST receipt is reconciled using the middleware-verified owner and
its original source run's durable agent/conversation identity. Source-run
residency and a reconstructed live presentation are not prerequisites for GET
or an identical protected POST retry. A retry returns the original continuation
ID, including after reopening the durable store; changed semantic input or a
different source run conflicts. Another source agent/conversation cannot select
that receipt. Missing or foreign source evidence refuses recovery. This path
reads existing admission only. If the receipt does not exist, POST still requires
the original resident presentation capture and all current surface, policy,
resource, approval and budget fences before reserving or dispatching new work.
Durable source history never grants new execution authority.

AG-UI requests put the ID/generation alongside `userAction` in
`forwardedProps.a2uiAction`. A retry with another wire-run ID attaches to the
original binding. The read-only lookup is
`GET /api/uar/ag-ui/agents/{agent}/threads/{thread}/actions/{interaction_id}`.
It requires the same verified owner/workspace and reports the original
`wire_run_id`, continuation ID and outcome. Neither lookup issues execution.

Both lookup protocols report `running` only for an owner-qualified resident
nonterminal continuation matching the receipt's exact agent and conversation.
Persisted `Pending`, `Running` or `Paused` without a resident producer is
`uncertain`, even when the durable record still carries its old status. Matching
terminal `Done`, `Error` and `Cancelled` records remain `completed`, `failed` and
`cancelled` after restart. Missing or mismatched continuation evidence stays
`uncertain`. None of these outcomes restores an approval lane, settles an
external effect, changes its budget, or authorizes redispatch.

AG-UI emits `CUSTOM` events named `uar.a2ui.surface_authority` (the authority
object above) and `uar.a2ui.retry_protection` (the request's `retry_protected`,
`interaction_id` and `interaction_id_required`). An `ACTIVITY_SNAPSHOT` surface
also carries `content.surface_generation` with its cumulative A2UI operations.

## Bounded surface authority

A surface is executable only if its latest exact component is a Button declaring
that action, its generation matches, it is not deleted, and its source history
is complete. Surface data and user text confer no tool authority. Existing host
resource capture, artifact revision, policy, owner and graph-agent fences remain.

AG-UI examines at most64 earlier runs and restates at most48 surfaces. More prior
runs report `expired`; more live surfaces report `truncated`; unavailable or
incomplete history reports `unavailable`. These are not authoritative empty UIs.
No fresh action is accepted under incomplete authority. Recovery is to open a new
conversation and request a fresh surface. First reconcile an existing uncertain
interaction; a new conversation is not permission to repeat its effect.

The rendered action outcome and surface authority are independent feedback.
An already-known pending, responding, uncertain or interrupted action remains
visible alongside unavailable, missing or stale authority. In particular,
`Action outcome is unconfirmed. Do not submit again.` remains an alert even
when history recovery or a fresh surface is suggested. Authority loss does not
change the original receipt or establish whether an external effect happened.
Controls remain disabled under the existing authority and submitted-intent
guards. `Recover action status` reads the original receipt and surface history;
it does not POST another action or allocate a replacement interaction.

Explicit REST publication and action declaration/claim serialize within the
selected single owned host and verified conversation. Weak lock entries are
removed after use; at1024 active keys a new key receives a bounded busy refusal.
This closes the observed publication-versus-claim race in the mounted REST paths.
It is not cross-process publication coordination. Deploying multiple writers to
the same A2UI conversation is outside this authority contract.

REST surface authority reconstructs durable publications for the source run,
including updates and deletions. Exact validated surface StatePatch events and
the host's matching artifact projection fold once; standalone trusted patches
remain available. Arbitrary nested strings or unrelated patch paths confer no
action authority, and the trigger path still emits exactly its original single
UI StatePatch. There is no invented total ordering across
native runs. A bounded owner/conversation enumeration includes legacy records
without a conversation index. At65 candidates the snapshot is expired. If another
run in that conversation publishes to the same live surface ID, the source
surface is conservatively expired; an active competing run or unreadable history
makes it unavailable. A continuation with no competing surface publication does
not alone expire the surface. This contract deliberately refuses some historical
surfaces instead of claiming to know a later global surface state. Legacy index
absence cannot silently hide a competing publication.

The uncomfortable limit is an interrupted creator: a receipt may exist while its
reserved execution cannot be found. That is `uncertain`. Repeated GETs and POSTs
with the same intent never issue that execution again. A settled run can be
reconciled after restart; an uncertain external effect stays uncertain until the
external owner supplies authoritative disposition. UAR does not promise global
exactly-once effects across arbitrary external systems.

## R031 action-feedback regression source (not executed)

The existing `frontend/src/features/chat/components/a2ui-artifact-block.test.tsx`
now authors the canonical persisted-chunk/UAR-renderer path for a lost POST
acknowledgment followed by unavailable authority and read-only recovery. It
asserts that the uncertainty alert survives, the original interaction is retained,
and recovery introduces no second POST or continuation attachment. A seeded
graph-state matrix separately covers pending/responding/uncertain/interrupted
feedback with unavailable, stale and missing authority, its status/alert roles,
disabled action controls and no POST on attempted resubmission or recovery.
Existing successful submission, changed-generation refusal and missing-publication
controls remain. Seeded states and mocked transport are bounded component
fixtures; they do not prove live effect settlement, screen-reader announcements,
visual layout or browser behavior. Every assertion is unexecuted while gates are
closed. The lead retains whole-phase inventory and later source-review/execution
decisions; this source repair clears no issue or acceptance gate.

## Final evidence still required

All code for issues462–475 precedes one operator-approved combined validation.
No source change here is current acceptance. The final existing A2UI API/browser
fixtures must cover owned effect counts under concurrent/lost-response/new-wire
retry, changed payload, distinct intent, restart/unknown receipt, foreign
owner/thread, stale generation/update/delete, >64 runs, >48 surfaces and missing
history. Retain existing shipped-renderer and no-duplicate-run assertions. Pin
source, API artifact, UI bundle and store configuration; keep functional requests
credential-free through the externally provisioned identity endpoint.

### Authored R020 regression source (not executed)

`tests/a2ui_logical_receipts.rs` retains the existing durable claim/effect-count
fixture and adds mounted REST/AG-UI receipt cases. The REST case closes/reopens
its real SurrealKV store, reconstructs a manager with no resident source or
continuation, then checks settled GET, identical POST, changed input, foreign
owner, other source/agent/conversation, missing live authority for a new intent,
unchanged original receipt and zero provider dispatch. Outcome cases cover all
six durable statuses, absent/mismatched continuation evidence, and actual held
resident producers through both mounted receipt routes. These are authored
regressions, not production restart acceptance or test-pass evidence.

The mounted cases reuse the existing `external_identity_host` adapter. Future
execution requires externally provisioned dedicated `a2ui-recovery` and
`a2ui-recovery-foreign` suites in the existing nonsecret profile manifest, with
`identity-capture` nodes admitting subjects `a2ui-owner` and `a2ui-foreign`.
Fixture requests do not read, mint or send authentication credentials. Profile
availability, compilation and every runtime assertion remain unverified while
the operator gate is closed. The lead owns the renewed full-phase inventory and
any request for exact execution commands; this document opens no gate.

### Rendered publication identity (R025 source repair; not executed)

Native A2UI publications carry `metadata.surface_generations`, a map from exact
surface ID to the host token for that publication. The map travels with the
artifact content through live chat and the persisted
`application/vnd.uar.a2ui+json` wrapper. Restoring a wrapper preserves its own
metadata; it does not fetch a newer token to label older content. Publications
that update an existing surface advance its generation even when the Button ID
and action name stay identical. Dedicated AG-UI snapshots retain their generation
alongside cumulative operations.

The canonical chat renderer passes that captured token into the typed domain
intent. Before its first await, the domain snapshots the payload and allocates
or retains the interaction ID; both the receipt and its original input persist
that exact generation/interaction before authority I/O. Admission requires one
complete, authoritative surface row matching the native source run, surface ID
and captured generation. Missing, conflicting, foreign, deleted or stale
publication identity sends no action POST. Current authority may disable an old
control; it cannot rewrite its intent as a newer publication. A legacy artifact
without a token can still render but stays read-only. Request a fresh surface
before acting on missing or stale content. Recover action status reads authority
and any original receipt; it neither refreshes rendered content nor replays the
click. An uncertain existing effect must still be reconciled first.

Authored domain regressions cover stale G1 against G2 with unchanged identities,
matching/missing/duplicate/foreign/deleted authority, mutation of caller input
while persistence is paused, and exact uncertain-recovery identity with one POST.
The component fixture restores the actual reserved MIME wrapper, uses the
canonical UAR renderer, and drives its Button through domain admission. The
existing browser fixture retains positive and lost-response paths and compares
their POST generation with the genuine native artifact's metadata. Its new stale
scenario publishes a real data update through the mounted same-source messages
route, requires changed authority without changing Button/action IDs, clicks the
still-displayed old control, and asserts refusal with zero action POST. It does
not replace or intercept authority with a fabricated response.

These are authored assertions only. Compilation, browser behavior, real
publication ordering and integration with host token generation remain
unverified while gates are closed. The lead owns full-phase inventory,
independent source assessment, later exact-command execution and release
acceptance; source authorship clears no issue or delivery gate.

### Original deployed action identity (R028 source repair; not executed)

A deployment selector in an AG-UI path and the native agent artifact admitted
behind it are distinct identities. The immutable logical receipt retains its
original wire lookup key. Its binding snapshot is reserved before native
admission, so that snapshot may legitimately lack `native_agent_id`. Before
producer dispatch, the host durably captures the resolved artifact ID in the
original AG-UI binding. Receipt reconciliation joins that exact original durable
row; it never resolves the deployment's current catalog entry to fill the gap.

The join authenticates the verified owner, workspace, wire agent/thread/run,
logical interaction, original input digest, native continuation and native
conversation. An already captured snapshot identity cannot change. Only the
expected native capture and terminal projection advancement may differ from the
snapshot. The original source must also have a settled prior binding in that
same wire conversation, with owner-qualified native source evidence matching
its own admitted artifact and session. Neither a receipt snapshot alone nor
another run with a matching terminal status supplies this proof. Source surface
reconstruction applies the same native owner/artifact/session fences before a
new action can be declared.

With that evidence, resident and historical native `Done`, `Error` and
`Cancelled` reconcile as `completed`, `failed` and `cancelled`. A durable
nonterminal record without a resident producer stays `uncertain`. Missing,
foreign or conflicting binding, source, native identity or conversation evidence
stays refused or uncertain. An identical new-wire retry attaches only to the
original authenticated binding; a changed semantic input conflicts. It never
creates a replacement execution. Removing or replacing a mutable deployment
entry does not rewrite the original receipt or its admitted native identity.

The added `tests/a2ui_logical_receipts.rs` source installs an actual collaboration
package/deployment binding and submits actions through the mounted AG-UI POST.
Its supplied provider observes that the original receipt still has a pre-native
snapshot while the durable binding already captures the actual native artifact.
The actual manager continuation produces Done/Error or is cancelled through the
host. Resident GET, identical new-wire retry, stale generation, changed payload,
foreign owner, immutable receipt and exact dispatch counts are asserted. The
store is closed/reopened; a new manager with an empty collaboration catalog must
return the same original receipt outcome and retry without another provider
call. Separate seeded cases cover absent/conflicting durable proof, foreign or
mismatched native/source evidence, and nonresident Pending/Running/Paused.

The source surface publication/history/binding and model-verification receipt
are deliberately seeded. The supplied provider and explicit host cancellation
are controlled collaborators. This fixture does not claim a renderer-produced
surface, live provider/model qualification, crash injection, an independently
installed deployment, or external-effect settlement. Existing durable claim,
REST payload/owner/source checks and held-producer assertions remain. Every new
and retained assertion is unexecuted; no compile, runtime, acceptance or issue
clearance follows from source authorship. The lead retains the full-phase
inventory and gate decision.


## Native artifact identity through chat projection

The native `uar.agui/1` artifact display and input-request custom events retain the producer's camelCase artifact identity and type in the durable journal. The adapter supplies those fields to the existing chat consumer without changing content, title, language or publication metadata. Native artifact type remains available in raw runtime projection; rich persisted display/input chunks retain the original artifact identity and original publication generation/source metadata. The input renderer passes the original backend artifact ID rather than its local card ID. An input without original identity remains read-only and reports “This input is unavailable. Request a fresh input form.” Existing legacy artifact events retain their working path. This field/identity bridge does not grant surface authority, reconstruct a missing publication generation, replay an action or submit an input response. The uncomfortable limit: authored local integration fixtures observe the policy-component boundary; actual form submission, A2UI surface validation, visual behavior and installed/runtime operation remain unverified until their separately authorized real-path gates.

R031 evidence remains source-only: the authored native/legacy/PGlite/renderer-policy-boundary cases have not run. This section does not certify actual form submission or installed behavior.


## Canonical A2UI recovery from the local journal

When a native display arrives after an interrupted assistant message was cached, local reload projects that new journal publication through the same declared A2UI interpretation used by live chat. The reserved durable display wrapper carries original artifact identity/type, content, language, nested metadata/profile, source run and host publication generation map; portable reconstruction retains the artifact ID and protocol version. Local recovery neither fetches server history nor submits an action. Mounting a valid recovered surface still performs the existing read-only authority lookup; an unavailable server leaves controls disabled. Missing historical generations remain missing and read-only, while absent/unsupported profiles keep the existing display refusal. Generic artifacts and previously cached rich evidence retain their existing paths and identities. Raw native journal evidence is not rewritten. These are authored source changes and regression scenarios; no runtime or renderer qualification has executed under the closed gate.

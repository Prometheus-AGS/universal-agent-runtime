# A2UI team surface contract

Status: **Official Draft Specification 0.1.0-draft.1**; proposed team binding atop UAR's existing `uar.a2ui/1` profile. Runtime conformance remains unproved.

## Version and catalog boundary

Team surfaces MUST preserve UAR's production A2UI v0.9.1 profile, supported compatibility label `v0.9`, and approved catalog `urn:uar:a2ui:catalog:1`. The existing approved components are Text, Button, TextField, CheckBox, ChoicePicker, Row, Column, Card and Divider. This draft does not widen that list or silently adopt A2UI 1.0. No `actionResponse` or `surfaceProperties` semantics are inferred for these surfaces.

The upstream [v0.9 specification family](https://a2ui.org/specification/v0.9-a2ui/) describes the protocol; UAR's [profile](../../../../protocols/a2ui-profile.md) and pinned source `src/uar/a2ui/protocol.rs` define the accepted restricted payload. Team negotiation includes profile and exact approved catalog revision/digest. A familiar catalog URI alone does not authorize arbitrary downloaded code or a substituted catalog. An unsupported required catalog produces a visible non-executable failure.

## Ownership and concurrent surfaces

A surface has a server-assigned globally unambiguous ID and durable ownership record: authenticated owner/workspace, team instance, task, member, run segment, catalog revision and monotonically increasing surface revision. That record is outside portable agent definitions. Two members or teams can render concurrently without reusing a surface ID. Names such as `main` are local authoring aliases resolved within this binding, never cross-team identifiers.

The transport carries a `uar.team.v1.surface.message` CUSTOM event. Its envelope establishes team/task/member correlation; payload contains `profile`, `catalogId`, `surfaceId`, `surfaceRevision` and `message`. The inner message remains an accepted `createSurface`, `updateComponents`, `updateDataModel` or `deleteSurface` object. It gains no invented team-routing properties. Catalog and inner/outer surface identities MUST agree. Duplicate event replay does not increment the surface revision again.

Create establishes ownership before rendering. Subsequent updates require the same binding and increasing revision. A delete tombstones that surface; late updates and actions cannot resurrect it. Ending a run does not silently transfer a surface to a new member. A read-only artifact may remain visible under its original task access; an interactive surface must be explicitly rebound or recreated with current authorization before accepting actions after recovery.

## Action routing

The host converts a user gesture into the proposed [surface-action.schema.json](../schemas/surface-action.schema.json): schema version, command ID, surface ID, expected surface revision, action ID and typed payload. This is a **UAR host command**, not an upstream A2UI client message or a v1.0 action response. An `actionId` names an action registered by the approved component definition; it is not authority to execute a tool.

The server MUST, in order:

1. Authenticate caller and resolve the surface record within that owner's workspace.
2. Check current view/action authorization, ownership, lifecycle and expected surface revision.
3. Resolve the registered action and validate its payload against the catalog/action contract.
4. Derive the intended team, task and member from server state. Optional client routing hints must match and may not override it.
5. Atomically record the command receipt and intended message/task transition, then dispatch through ordinary admission.
6. Return accepted/rejected/conflict and update the visible surface/task state. Acceptance is not effect completion.

A repeated command ID with the same semantic request returns the original receipt. Reuse with a changed payload is a conflict. A stale revision yields a visible conflict and refresh path; the server does not reinterpret a click against a new action. A pending turn does not require disabling unrelated teams' controls. Each surface exposes its own progress and result.

An action requesting issue publication, model override or approval is subject to the same server-side authority as every other entrypoint. An agent-rendered Approve button cannot mint an executable grant. Actual tool approval uses the established trusted host path, exact effect identity and current authority. A surface supplied by another task cannot approve it by guessing the ID.

## Recovery, privacy and accessibility

Reconnect replays accepted declarative updates or supplies the current authorized surface snapshot and revision. Client caches are projections and cannot recover ownership independently. A restarted runtime invalidates old live-run authority; unresolved actions remain identifiable by command ID and receipt. In-flight external effects become reconciliation work rather than automatic retries.

Only approved declarative data crosses the model-to-renderer boundary. React modules, HTML, JavaScript, CSS, event handlers, arbitrary URLs and unapproved component properties remain rejected. Private content redaction applies equally to live surfaces, snapshots, logs and exports. A named team member's output does not gain another member's memory rights.

The Boss must expose meaningful labels, keyboard focus, status announcements and actionable errors for each surface; progress/result signals cannot rely only on color or a disabled button. Surface identity and task/member association remain visible when multiple panels are open. Localized host feedback does not alter the canonical command identity.

The uncomfortable case is a user clicking a still-visible approval control after restart or task reassignment. Keeping the button on screen is not permission to honor it. The host must show the stale result and fresh trusted action path instead of dispatching under obsolete authority.

See [concurrent surface/action trace](../traces/a2ui-concurrent-actions.jsonl). The trace's UAR command and internal result rows are explicitly distinguished from A2UI wire messages.

# Analysis — A2UI surface architecture (revision 2)

KBD Analyze, 2026-10-28. Read-only. Nothing was built, tested or run.
Revision 1 and its independent review are in `analysis-rev1.md`.

Evidence files in this directory: `assessment.md` (with §6 PEM addendum),
`upstream-research.md`, `uar-surface-consumers.md`, `pem-4.2.0.md`.

Operator constraints for this revision (2026-10-28):

- PEM 4.2.0 `@prometheus-ags/a2ui-react` is the client runtime.
- UI/UX testing covers both React and Flutter rendering.
- We own PEM. Changes to PEM become GitHub issues for the session working in
  that repository.

**Recommendation: combination R** (activities + `web_core` mirror + split data
+ thread push), defined below. **Readiness: BLOCKED on operator decisions
D1-D17.**

## 1. What changed from revision 1

| Finding | Disposition in revision 2 |
|---|---|
| C1 scoring was circular | Shared premises are applied to every option (§3). Each axis is scored on its own (§4-§7). The whole-system comparison uses the same premises (§9). |
| C2 morphing erases input | Morphs and updates are incremental. Live clients never receive a `replace:true` restatement for a surface they hold. Reconnect uses surface versions declared by the client (§8.3-8.4; D3, D4). Unsent input survives reconnect only through client drafts (PEM P5). |
| C3 server structure disagrees with the client | The server keeps a non-authoritative mirror that runs the same official `web_core` 0.12.0 the client runs, so it accepts and rejects exactly what the web client does (§5, option 2d). Flutter's GenUI remains a second implementation, which is named and tested (§10, T4). |
| W1 push channel | Axis 4 (§7), D6. |
| W2 `Last-Event-ID` | Kept as an optimization only. Correctness never depends on it (D5). |
| W3 San Saba carriage | Mapped in §11, D13. O9 is **blocked** by G12. |
| W4 forge rationale | Corrected: forge assembly is unexercised in San Saba (SSW-A2UI-011:66-71). |
| W5 catalogs need implementations | The catalog registry tracks one implementation per renderer (§3, P-premise 2). |
| W6 compaction | The mirror compacts. Live clients are not affected (§8.5, D7). |
| W7 follow-up runs | The server injects a bounded surface summary from the mirror into the run context (§8.6). |
| W8 browser-persisted agent writes | Agent entity writes happen server-side in the trusted host. AG-UI `STATE_*` is never the system of record (§6, D10). |
| W9 concurrency, personas, tabs | Serial processing per surface, idempotency keys, entity revisions in handlers, and a surface scope model (§8.7-8.8, D8). |
| W10 data-model capture | `sendDataModel` through PEM `getClientDataModel()` (`runtime.ts:335`) (§8.6). |
| W11 missing options and costs | Option 2d added; A2A evaluated (§4); every option's dependencies are counted (§9, §13). |
| W12 missing decisions | D1-D17 (§14). Q4 merged into D1. Cedar is D14. |

## 2. Objectives

O1-O9 are unchanged from `assessment.md` §1:

1. live;
2. recoverable;
3. interactive across turns;
4. extensible (one catalog);
5. safe;
6. full applications;
7. personalized morphing;
8. standards;
9. San Saba fit.

Added for this revision:

- **O10. Cross-renderer testability.** The same surfaces and actions behave the
  same in the React path (PEM 4.2.0) and the Flutter path (GenUI 0.10.2), and
  this is shown by shared fixtures.

## 3. Shared premises (applied to every option)

These are not credited to any one option.

1. **Scope.** Surface records belong to a thread or application session, not a
   run (gap G2).
2. **One catalog registry.** a2ui.org catalog documents
   (`definitions`), with a recorded implementation per target renderer (React
   via PEM application-supplied implementations, commit `34a21156`; Flutter
   via the GenUI catalog). A component is admitted only for surfaces whose
   target renderers all implement it. A catalog change is a new `catalogId`
   version; existing surfaces keep their catalog until re-created.
3. **Admission caps** on bytes, operations, depth, components and surfaces per
   surface and per thread, aligned with the client's limits
   (`a2ui-rendering-limits.ts:2-5`: 256 KiB, 128 messages, 500 components, 16
   surfaces).
4. **Server-side authority.** Tenant and user come from verified identity
   supplied by external infrastructure (project rule: authentication outside
   UAR tests; G12 for San Saba).

## 4. Axis 1 — carriage

| Option | What travels | Verdict |
|---|---|---|
| 1a AG-UI activities | `ACTIVITY_SNAPSHOT` / `ACTIVITY_DELTA`, `activityType:"a2ui-surface"`, `content.a2ui_operations`; actions in `forwardedProps.a2uiAction.userAction` | **Recommended.** Upstream de-facto mapping (`upstream-research.md` §4); PEM 4.2.0 consumes it (`runtime.ts:272-316`). |
| 1b `STATE_*` (UAR today) | surfaces inside shared state | Rejected: state is application data that returns to the agent, and A2UI is renderer-owned (§3, implication 8). |
| 1c `CUSTOM` artifacts (UAR chat, DGA) | whole message batch per artifact | Rejected as the primary path: `CUSTOM` must not carry semantics owned by standard events. Kept as a migration adapter (D13). |
| 1d A2A | A2UI's only formal transport extension | Not for the browser path: the San Saba client channel is AG-UI (SSW-ARCH-001 §1.2). A2A is used **behind** UAR to dispatch actions to agents or tasks (§8.6; D13). |

**Constraint found in PEM 4.2.0.** `processAgUiEvent` handles only
`ACTIVITY_SNAPSHOT`, and a `replace:true` snapshot deletes and recreates its
surfaces (`runtime.ts:289-315`). Incremental updates therefore need either:

- `ACTIVITY_DELTA` appends (`add /a2ui_operations/-`), which needs **PEM P1**;
  or
- a new activity `messageId` per batch, carrying only new operations. This
  works with PEM today (inferred, not run), but it multiplies activity
  messages in `MESSAGES_SNAPSHOT`.

That choice is D3.

## 5. Axis 2 — where reduced A2UI state lives

| Option | Description | Authorize "offered" (O6) | Compact (O2, op 129) | Agrees with client (C3) | Reducers to keep equal |
|---|---|---|---|---|---|
| 2a Client only | Server stores the op log | no: cannot know which updates the client rejected | no: needs a reducer | n/a | 1 |
| 2b Rust structure map | Revision 1's F | partial | no | **no**: skips catalog and topology validation | 2 (partial) |
| 2c Rust authoritative reducer | Options A/B, change 1 | yes | yes | only by an exact port (failed four reviews) | 2 |
| **2d Official `web_core` mirror, non-authoritative** | The server runs the client's own `web_core` 0.12.0 via a headless PEM runtime (P3). It processes every admitted batch atomically, as the client does. | yes, for agent-sent state | yes: PEM's reconstruction (`runtime.ts:445-486`) to a v1.0 inline `createSurface` | yes, for web clients by construction; Flutter tested (T4) | 1 (web) + GenUI (Flutter) |

**Recommended: 2d.**

**Non-authoritative** means the mirror never overrides a client's data model.
It reflects agent-sent state plus values reported with actions. It is used
for:

- admission;
- the "offered" check;
- compaction;
- follow-up context.

**Unverified.** `web_core` depends on `lit` and `@preact/signals-core`
(`package.json`). That its processing modules run under Node without a DOM
has not been shown. If they do not, 2d falls back to 2a plus a Rust
structure check, with revision 1's C3 risk named.

**Where the mirror runs** is D2:

- a Node process beside UAR;
- an embedded JS engine in UAR;
- the application's trusted BFF.

## 6. Axis 3 — where application data lives

| Option | Description | Verdict |
|---|---|---|
| 3a A2UI data model | The agent sends `updateDataModel` | **For chat and surface-local values.** |
| 3b Domain entities | Records live in the application's store and sync to clients through PEM. The A2UI data model receives bound values on the client. | **For applications** (San Saba Shared Entity-Graph Pattern, SSW-ARCH-001 §1.3). Needs a declarative binding projection in PEM (**P4**). Today it is unexercised (SSW-A2UI-011). |
| 3c AG-UI `STATE_*` as the data channel | The agent writes state and the browser applies it to the graph | Rejected as a write path: double application across tabs and replays, browser-persisted agent writes, and bypassed capability inversion (W8). Allowed only as agent-visible read state. |

**Agent writes** to entities are server-side tool, MCP or A2A calls in the
trusted host. Entity changes reach clients through the application's sync
channel (realtime fabric, Electric, or SSE invalidation as in SSR BIBLE.md
380-445), not through A2UI operations. That keeps high-rate data out of the
operation stream.

**Choosing the projection** is D9. The two options are:

- **client-side projection (P4):** data never enters the op stream;
- **server-side `updateDataModel` from entity changes:** simpler, but it grows
  the stream, which compaction absorbs.

## 7. Axis 4 — push outside a run

| Option | Description | Verdict |
|---|---|---|
| 4a Run streams only (today) | Updates reach clients only inside a run | Fails O6 multi-tab and O7: a condition-driven morph has no open stream (W1). |
| **4b AG-UI `connect()` over a thread stream** | UAR serves one stream per thread for each viewer, carrying activities and run lifecycle. The client agent implements `connect()` against it. | **Recommended.** Standard hook, but upstream's JS default throws `ConnectNotImplementedError`. Fit with real clients is unverified. |
| 4c Application sync channel only | Morphs signalled through entity sync; the client then starts a run | Possible for San Saba, but a round trip per morph, and it leaves chat-only clients without push. |

## 8. Recommended combination R — mechanics

### 8.1 The surface record (server, durable)

Each record holds:

- `surfaceId`;
- `scope`: thread, user or session (D8);
- tenant and owning user;
- `catalogId`;
- a monotonic `version`;
- the compacted base (a v1.0 inline `createSurface`), plus operations since
  that base;
- the activity `messageId`.

Records are stored with run history (`run_history` providers exist), not in
the in-memory backbone.

### 8.2 Ingress (tool output, REST, controllers)

1. `parse_message` and the admission caps run first.
2. The whole batch is applied to the mirror atomically. The mirror refuses
   what the client would refuse, and a refused batch emits nothing.
3. If accepted, `version` increases and the operations are appended.
4. One AG-UI activity is emitted per affected surface: incremental (D3), to
   every viewer in scope.

### 8.3 Live clients

Each client view keeps **one PEM runtime** for its thread or session, not a
processor per artifact (G16). Updates and morphs arrive as incremental
operations, so the client's data model, including typed input, is preserved.
Morph removes or adds components. It does not restate the surface.

### 8.4 Reconnect and second devices

The client sends the versions of the surfaces it holds, as a UAR extension in
`forwardedProps` or `connect()` parameters (D4). For each surface:

| Client version | Server sends |
|---|---|
| retained | operations since that version |
| unknown, or older than the compacted base | the compacted snapshot (`replace:true`) |

A fresh device has no input to lose. A client that receives a full snapshot
loses unsent input unless it restores a local draft (P5). This meets AG-UI's
rule to send a snapshot when the baseline cannot be guaranteed, and it
does not depend on the 512-event ring.

### 8.5 Compaction

When a surface exceeds an operation threshold (D7, below the client's 128),
the mirror rebuilds a v1.0 inline `createSurface` from its state (PEM's
reconstruction technique, P3). That becomes the new base.

Live clients are not sent the compaction; it changes only what reconnecting
and new clients receive. This removes the operation-129 failure for long-lived
surfaces.

### 8.6 Actions

Actions arrive as `forwardedProps.a2uiAction.userAction`, plus:

- the client's rendered surface `version`;
- the data model, via `sendDataModel` and PEM `getClientDataModel()`
  (`runtime.ts:335`), when the surface opts in (this keeps today's
  `a2uiClientDataModel` behavior; W10).

PEM's client-side policy is a first filter only. UAR then checks, serially
per surface:

1. **Offered.** The component and action exist in the mirror. For surfaces
   with `sendDataModel`, dynamic child lists resolve against the reported data
   model. This is a UI-integrity check, not authority.
2. **Current.** Against `version`. A changed component or action returns 409
   with the operations since that version.
3. **Idempotent.** By action id.
4. **Permitted.** The handler's own tenant and user check, plus entity
   revision compare-and-swap for writes (SR9).

Dispatch goes to a deterministic **surface controller** or to the agent. A2A
tasks are one controller kind (§11). An agent follow-up runs on the same
thread, and the server injects a bounded summary of the surface from the
mirror into the run context (W7). Activity messages stripped from client
input therefore do not matter.

### 8.7 Scope and personas

| Scope | Visible to |
|---|---|
| Thread | every participant |
| User | one user |
| Session | one tab or session |

Per-persona variants of a shared record, such as a deal checklist, are
user-scoped surfaces assembled from one template over shared entities (SR7).
Snapshots and `connect()` streams are filtered per viewer to the surfaces in
their scope.

### 8.8 Two tabs

- **Updates.** Both tabs receive incremental updates. Each keeps its own
  typed input, which lives in its own runtime.
- **Actions.** Actions from both tabs are processed serially. A second action
  against a stale version is checked by rule 2; a duplicate is absorbed by
  rule 3.
- **Data writes.** Writes compare and swap on entity revisions.

### 8.9 Morphing

An assembler runs again on a condition change: a controller, forge
`surfaces/assemble` (D2 and D13), or the agent. The condition can be:

- an entity change;
- a notification;
- a schedule;
- an agent decision.

The assembler emits incremental component changes through §8.2 for the
affected user-scoped surfaces. Stable component ids and bound data keep
values.

## 9. Whole-system comparison (same premises §3)

| | R (1a+2d+3a/3b+4b) | C+ (1a+2a+3a/3b+4b) | B/A (1b+2c) | D-authoritative (shared reducer that overrides clients) |
|---|---|---|---|---|
| O2 recover incl. long-lived | yes (compaction) | **no** past client caps | yes | yes |
| O5 safe | yes: official validation plus caps | yes | **no** (revision 4 depth and size) | partial: JS engine attack surface |
| O6 authorize "offered" | yes | **no** (C3) | yes | yes |
| O7 morph keeps input | yes (incremental) | yes (incremental) | no (whole sends) | **no**: server overrides typed values |
| O8 standards | yes, plus two named extensions (D4 versions, D6 thread stream) | yes, plus the same extensions | no | partial |
| O10 cross-renderer | one web reducer; Flutter tested | same | dual reducer | one web reducer |
| New dependencies | JS runtime for the mirror (D2), PEM 4.2.0, entity sync for 3b | PEM 4.2.0, entity sync | none | JS engine |
| Main residual risk | `web_core` headless unverified; GenUI divergence | operation 129; authorization gap | the change-1 loop | input loss |

R is preferred over C+ because it is the only one that meets O2 (long-lived
surfaces) and O6 (authorization) together. C+ is the fallback if the mirror is
infeasible.

## 10. UI/UX test plan (authored in Plan, run only at the phase gate)

All tests are BDD behaviors (project rule). No credentials are carried
(project rule, 2026-10-26).

| Id | What | React path | Flutter path |
|---|---|---|---|
| T1 | Shared fixture corpus: message sequences and expected outcomes (P6) | consumed | consumed |
| T2 | Render: create, incremental update, morph, delete | UAR frontend on PEM 4.2.0, one runtime per thread view | `entity_graph_flutter` + GenUI 0.10.2 client against the same UAR (P7) |
| T3 | Typed input survives an incremental morph | browser behavior | simulator integration |
| T4 | Accept and reject parity with the mirror for every T1 fixture, plus normalized structure parity (P6 structure dump) | yes | yes; this is where GenUI divergence shows |
| T5 | Reconnect with declared versions; a fresh device gets the compacted snapshot; more than 128 operations | yes | yes |
| T6 | Actions: allowed, denied by client policy, denied by server, stale (409), duplicate, two tabs | yes | yes |
| T7 | Follow-up run sees the surface; condition-driven morph over `connect()` outside a run | yes | yes |
| T8 | Accessibility: WCAG AA, keyboard (SR11) | browser audit | Flutter semantics audit |

Before any UI code is written, the project's UI routing (Impeccable, then
`frontend-design`, then UI/UX Pro Max, with dual critique and adversarial
review) applies in Plan and Execute.

## 11. San Saba fit (O9: blocked)

- **G12 is a prerequisite.** UAR needs tenant claims, RS256-compatible
  identity, and tenant-partitioned A2A tasks (D13).
- **DGA artifacts** (`{__type:'a2ui', schema, data}` as `CUSTOM
  A2UI_ARTIFACT`, DGA/README.md:43-48) are not a2ui.org A2UI. Either migrate
  DGA to A2UI activities, or keep a client adapter during migration (D13).
- **"Button → Action → A2A task"** maps to an A2A controller behind §8.6.
- **Forge layout regions** (`components[]` with `layout.region`,
  SSW-ARCH-001 §3.3(A)) need a mapping. Either forge emits a v1.0
  `createSurface` with a region layout component, or a region component is
  added to the catalog (D13).
- **San Saba's design currently applies agent `STATE_DELTA` to the graph in
  the browser** (§3.2). Under R this moves server-side (D10). That is a change
  to San Saba's design, not only UAR's.

## 12. Changes by owner (for Plan)

**UAR**

- Surface records (§8.1).
- The mirror host (D2).
- Activity carriage on the dedicated AG-UI route and the run stream (G4, G5).
- Action ingress (§8.6).
- Surface controllers.
- The thread stream for `connect()` (D6).
- The catalog registry (G6).
- Admission caps (G8).
- Retiring `/a2ui/surfaces` from `STATE_*`.
- Frontend:
  - PEM 3.0.3 → 4.2.0 (G14);
  - resolving the package-name collision (G15);
  - one runtime per view (G16);
  - the console reading records instead of the patch log (G13).
- Specs to revise: `ag-ui-chat-conformance`, `agui-runs-stream-fidelity`,
  `runtime-console`, `a2ui-uar-renderer`, `docs/protocols/ag-ui-profile.md:33`.
- Supersede change 1 (D17).

**PEM** (GitHub issues for the PEM session)

| Id | Change |
|---|---|
| P1 | `ACTIVITY_DELTA` support in `processAgUiEvent`: append-only `a2ui_operations` without deleting surfaces |
| P2 | Version-aware activities: carry a surface version, ignore stale snapshots and deltas, and expose held versions for reconnect |
| P3 | A headless runtime entry (no React/DOM) with the same catalog validation, plus a public `exportSurfaceMessages(surfaceId)` compaction API that generalizes the preflight reconstruction to a v1.0 inline `createSurface` |
| P4 | Declarative entity-to-data-model binding projection, React and Flutter, with write-back through the existing action policy |
| P5 | Per-surface local draft persistence for unsent input, restored after a full snapshot |
| P6 | A shared cross-renderer conformance corpus (JSON fixtures, expected accept/reject, normalized structure dump) used by React, Flutter and the mirror |
| P7 | Flutter A2UI moved from the showcase into a published package with the same activity transport, policy and P1/P2 semantics |
| P8 | Housekeeping: `release/npm-registry-status.json` is stale (npm `latest` is 4.2.0) |

**San Saba, DGA and forge:** D13 items.

## 13. Costs and the uncomfortable parts

- **The mirror is server-side reduction again.** It is acceptable only
  because it runs the client's own code at the same pin. That couples every
  `web_core` upgrade across UAR, PEM and clients. If `web_core` cannot run
  headless, the recommendation weakens to C+ with an authorization gap.
- **Flutter is a second reducer.** GenUI 0.10.2 is a separate implementation.
  T4 can detect divergence; nothing prevents it.
- **Two UAR extensions to AG-UI:** client-declared surface versions (D4) and a
  thread stream for `connect()` (D6). Standard clients ignore them, but without
  them O2 and O7 degrade.
- **Large scope.**
  - Most of UAR's current A2UI transport, console replay and four specs change.
  - PEM gains seven feature issues.
  - San Saba's agent-write path changes.
- **Not fixed by R.** San Saba still cannot use UAR until G12 is fixed.
- **Evidence.** This analysis rests on source and documents. No mechanism here
  has been run, including PEM's incremental activity path.

## 14. Operator decisions before Plan

| Id | Decision | Recommended |
|---|---|---|
| D1 | Adopt combination R (includes AG-UI activity carriage, formerly Q4) | Yes |
| D2 | Mirror host: Node process beside UAR, embedded JS engine, or the application BFF | Node process beside UAR, after a feasibility check that `web_core` runs headless |
| D3 | Incremental carriage: `ACTIVITY_DELTA` (needs P1) or a new snapshot `messageId` per batch | `ACTIVITY_DELTA` with P1; new `messageId` only until P1 lands |
| D4 | Reconnect: client-declared surface versions (extension), or pure snapshot restatement | Declared versions |
| D5 | Keep the AG-UI route's `Last-Event-ID` cursor | Keep as an optimization; never required for correctness |
| D6 | Push: implement AG-UI `connect()` over a UAR thread stream | Yes |
| D7 | Compaction threshold and caps | Compact at 64 operations; caps equal to client limits |
| D8 | Surface scope model: thread, user, session | Yes, as in §8.7 |
| D9 | Entity values into surfaces: client projection (P4) or server `updateDataModel` | Client projection |
| D10 | Who persists agent entity writes | The trusted host only (tools, MCP or A2A); never the browser |
| D11 | A2UI versions admitted | v1.0-RC and v0.9.1; retire v0.9 (UAR's own tool example uses v0.9) |
| D12 | AG-UI version target: 1.0 spec, or `@ag-ui/core` 0.0.59 as PEM pins | Verify in Plan; align PEM and UAR on one |
| D13 | San Saba migration: G12 first; DGA to A2UI activities; A2A as a controller; forge regions mapping; agent writes server-side | G12 first, as a separate change; the rest in Plan |
| D14 | Cedar on every tool call (ssr-frontend spec) versus no Cedar (workspace docs) | Confirm the workspace docs |
| D15 | UAR renderer: replace `a2ui-uar` with PEM's runtime plus UAR catalog implementations, or rebuild `a2ui-uar` on it; rename UAR's local `@prometheus-ags/a2ui-react` | Rebuild on PEM; rename the local package |
| D16 | File PEM issues P1-P8 now (the operator has authorized issues) | File after this review |
| D17 | Supersede change 1 and the `agui-a2ui-projection-and-custom-components` plan | Yes, keeping their findings as evidence |

## 15. Unverified

- `web_core` 0.12.0 running headless under Node.
- PEM's incremental activity path (new `messageId` with only updates)
  preserving the data model.
- Real AG-UI clients implementing `connect()`.
- How the PEM graph projects into a surface's data model (not exercised
  anywhere yet).
- GenUI 0.10.2 parity with `web_core` on the same fixtures.
- San Saba's G12 blockers against current UAR source (recorded 2026-07-29).
- Upstream restore-after-reload behavior.

---

## Revision 2 independent review — 2026-10-28

A fresh-context `artifact-critic` reviewed this file alone. Verdict: **defects
found — 4 critical, 10 warnings, 5 suggestions.** It confirmed 12 citations as
correct.

The lead re-verified the following against source:

- PEM's delete-and-reapply on any snapshot without `replace:false`
  (`runtime.ts:289-315`).
- The action id exists only for v1 `wantResponse` actions, built from a counter
  local to one runtime (`runtime.ts:362-367`).
- The v0.9 client action schema has no id field (`web_core`
  `src/v0_9/schema/client-to-server.js:19-33`).
- No `v4.2.0` tag exists. Commit `34a21156` follows `580997b5`
  ("chore(release): prepare 4.2.0").

### Critical

- **C1. Redelivery rolls a surface back.** In PEM, every activity `messageId`
  is a delete handle, and AG-UI's `replace` defaults to true. A creation
  activity delivered twice deletes the surface and rebuilds it from the
  creation operations alone. Typed input and later updates are lost. Each later
  update then fails on the deleted surface (`message-processor.js:570`).

  R itself creates the conditions that cause redelivery: a run stream next to
  a `connect()` stream, reconnect replay, and `MESSAGES_SNAPSHOT`. The plan
  also leaves undefined how activities are deduplicated and what form they
  take in `MESSAGES_SNAPSHOT`.
- **C2. Compaction does not fix live clients.** UAR's
  `a2ui-stream-accumulator.ts:125-140` counts operations, bytes and components
  over a surface's whole life. Its lifecycle count resets only on `clear()`.
  A live long-lived surface therefore still fails at operation 129, and §12
  does not address the accumulator. The O2 row that separates R from C+ is
  unsupported.
- **C3. One user's values leak to others.** On a thread-scoped surface, values
  user A reports with an action enter the mirror. Compaction and follow-up
  context then deliver A's values to user B.
- **C4. Idempotency has no id to use.** A v0.9.1 action has no id, and PEM's
  per-runtime counter repeats across tabs and reloads. A legitimate second
  action is dropped as a duplicate.

### Warnings (summary)

- **W1. Compacted snapshots don't fit live clients.**
  - Sent to a client that still holds the surface, a compacted snapshot fails
    with "already exists" when its `messageId` is not tracked for that surface.
  - §8.1's single `messageId` per record contradicts the D3 interim plan.
  - PEM's reconstruction drops `catalogId`, `rootId` and v1 response metadata.
- **W2. The two server-side JS options are scored unequally.** The "offered"
  check is not authority and uses client-reported data, so R's O6 advantage is
  overstated.
- **W3. `web_core` 0.12.0 cannot remove a component.** Morphs leave orphans,
  which still count toward caps and are rebuilt in compaction. The rule that an
  assembler never writes paths bound to inputs is not stated.
- **W4. Frontend gaps are missing from §12.** `agui-schema.ts:8-15` drops
  `ACTIVITY_*` events, and PGlite run-event persistence is a second replay path
  with the C1 problem.
- **W5. No recovery when the client rejects a batch the mirror accepted.**
  There is no client error report and no resync.
- **W6. P7 is several issues.** It needs:
  - a Dart AG-UI client with `connect()`;
  - Flutter versions and drafts;
  - Flutter implementations of UAR's catalog components.
- **W7. The P items are coupled to open decisions** (P1 to D12, P2 to D4,
  P3 to the headless check, P6 to P7). PEM changes are also missing:
  - unique action ids;
  - replace and replay rules;
  - client error and resync reporting;
  - applying the operations returned with a 409;
  - sending the held version with actions.
- **W8. Premise 2 is unverified.** It relies on `34a21156` being in the
  published 4.2.0.
- **W9. Decisions are missing:**
  - action id source;
  - the replace and `messageId` policy;
  - the source of `MESSAGES_SNAPSHOT`;
  - how a surface declares its target renderers;
  - GenUI divergence handling;
  - the mirror's lifecycle across restarts and replicas;
  - session identity across reload;
  - whether client caps stay.

  D3 also depends on the undecided D12.
- **W10. The test plan has gaps.**
  - Per-viewer and scope tests need a second owner, and the plan doesn't say
    how to provision one without credentials.
  - Missing tests: double delivery, `MESSAGES_SNAPSHOT` replay, a compacted
    snapshot sent to a client still holding the surface, a mirror restart,
    client-rejected batches, and the C3 leak.
  - T8 audits are not BDD behaviors.

### Suggestions

- Any snapshot without `replace:false` deletes, and `replace:false` on a known
  `messageId` is ignored.
- The existing reconstruction produces three v0.9.1 messages, not a v1.0
  inline create.
- The bridge doc lists `web_core` 0.10.5 while the package pins 0.12.0.
- `pem-4.2.0.md` misnames HEAD.
- Durable run history defaults to off (`persistence/mod.rs:133`).

### Lead assessment

These findings are about delivery mechanics, not the architecture's shape.
Most fall in code we own:

- **PEM's activity semantics:** idempotent, versioned application;
  deduplication; unique action ids; client error and resync reporting;
  lossless export; component pruning.
- **UAR's frontend:** the accumulator, `ACTIVITY_*` parsing, and the PGlite
  replay path.
- **One rule in UAR's mirror:** it holds agent-sent state only, and values
  reported with actions are used once and never stored (fixes C3).

That differs from revision 1, where the defect was the architecture's central
claim. It is still the second consecutive review with critical findings, so
R is not ready to decide as written.

Per the operator's instruction of 2026-10-28, the PEM changes are filed as
GitHub issues (recorded below). The UAR-side corrections and missing decisions
are input for revision 3.

**Readiness: BLOCKED.**

### PEM issues filed — 2026-10-28

#70 redelivery rollback (bug), #71 versioned incremental activities, #72 unique
action ids, #73 headless runtime and lossless export, #74 release and doc
drift; comments on #65 (Flutter transport, parity, entity binding, drafts) and
#69 (cross-renderer fixtures). Links in `pem-issue-drafts/README.md`.

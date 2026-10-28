# Analysis revision 1 (superseded 2026-10-28 by analysis.md revision 2) — A2UI surface architecture

KBD Analyze, 2026-10-28. Read-only. Nothing was built, tested or run.
Evidence: `assessment.md`, `upstream-research.md` (A2UI `ccf5dc20`, AG-UI
`dab9c95f`, CopilotKit `4693af4d`), `uar-surface-consumers.md`, and the San
Saba workspace documents cited in `assessment.md`.

**Recommendation: option F, "activity surfaces with entity-bound data".**
**Readiness: BLOCKED on the operator's architecture decision (Q1-Q6).**

## Objectives

O1-O5 were accepted on 2026-10-28. O6-O9 follow the operator's scope decision
of the same day (`goals.md`).

| Id | Objective |
|---|---|
| O1 | Live: surfaces appear and change as the agent works |
| O2 | Recoverable: reload, reconnect, a second device or the console shows the current surface |
| O3 | Interactive across turns: an action's follow-up can update the same surface |
| O4 | Extensible: one catalog definition read by server admission and every renderer |
| O5 | Safe: untrusted output cannot crash the server, inflate storage or inject code |
| O6 | Full applications: deterministic action handling, action authorization against what was shown, paging, high update rates, navigation, several tabs |
| O7 | Personalized morphing: a surface changes shape for one user as real-time conditions change, without losing that user's input |
| O8 | Standards: follows A2UI and AG-UI as upstream defines them (project rule: open standards first) |
| O9 | San Saba fit: can implement a specification like `sansaba-workspace` |

## Options

- **A. Server reducer with diff patches.** The path of change 1. The server
  ports `web_core`'s reducer, computes minimal RFC 6902 diffs, and sends them as
  AG-UI `STATE_DELTA`.
- **B. Server-authoritative reduced state, sent whole.** The server's reducer
  defines the semantics and sends whole surfaces.
- **C. Operation log; clients reduce.** The server stores and orders each
  surface's A2UI operations and sends them. Every renderer reduces them.
- **D. One shared reducer.** Rust compiled to WASM in the browser, or `web_core`
  embedded on the server.
- **E. A narrowed UAR A2UI profile.** Admission caps and a restricted subset.
  Combines with the others.
- **F. Activity surfaces with entity-bound data.** Combines upstream's carriage
  (C), structure-only server state (part of B), E's caps, and San Saba's
  entity-graph pattern. Detailed below.

## Scoring

Marks: **yes**, **partial**, **no**. Each cell gives the deciding reason.

| | A | B | C | D | F |
|---|---|---|---|---|---|
| O1 Live | yes | yes | yes | yes | yes |
| O2 Recoverable | partial: diffs need an exact baseline | yes: send state | yes: send the op list | yes | yes: AG-UI snapshot restatement |
| O3 Across turns | needs thread scope | needs thread scope | needs thread scope | needs thread scope | yes: thread scope is part of F |
| O4 One catalog | independent | independent | independent | independent | yes: the catalog registry is part of F |
| O5 Safe | **no**: faithful port enables depth and size attacks (rev 4 review) | partial: a server reducer still parses untrusted data paths | yes, with caps: the server stores, doesn't reduce data | partial: an embedded JS engine is a new attack surface | yes, with caps |
| O6 Authorize actions against what was shown | yes | yes | **no**: the server holds no structure | yes | yes: structure-only server state |
| O6 High update rate | partial | **no**: whole sends | partial: the op list grows | partial | yes: high-rate data lives in entities, not the A2UI data model |
| O7 Morph without losing input | partial | partial: a whole send resets data | partial | partial | yes: structure replaces, entity data and stable ids remain |
| O8 Standards | **no**: A2UI in `STATE_*` is not the upstream pattern | **no**: same | yes: matches `@ag-ui/a2ui-middleware` | partial | yes: activity carriage; entity binding is an extension (see costs) |
| O9 San Saba | **no**: San Saba retired binding surfaces to a run stream | partial | partial: no entity binding | partial | yes: matches SSW-ARCH-001 §1.3, §3.2-3.3 |
| Two reducers that must agree | **yes (the defect)** | yes, unless clients render server state | no | no | no |
| Cost in UAR | high, unbounded review loop | high | medium | high plus a new dependency | medium to high (see costs) |

A is rejected. Four review rounds failed on its central requirement: two
reducers in two languages must agree exactly on untrusted input.

## Option F in detail

1. **A2UI travels as AG-UI activities.** Each surface is an `ACTIVITY_SNAPSHOT`
   with `activityType: "a2ui-surface"` and content `{a2ui_operations:[…]}`, sent
   with `replace:true`. This is the upstream de-facto mapping
   (`upstream-research.md` §4). Appends can use `ACTIVITY_DELTA`
   `add /a2ui_operations/-`. An append needs no reduction, so it brings back
   none of change 1's problems. A2UI leaves `STATE_*` and `CUSTOM`. Upstream
   says `CUSTOM` must not carry semantics that standard events own, and that
   state is application data (§3).
2. **Clients reduce, using the renderer they already ship.** That is
   `web_core`'s `MessageProcessor` on the web, and the official Flutter, Lit,
   Angular and React renderers elsewhere (§6). The server never reduces a data
   model.
3. **The server keeps a durable, structure-only record per surface.** It is
   scoped to the thread or application session, not the run, and holds:
   - the current operation list (chat surfaces) or assembled declaration
     (application surfaces);
   - the component map, reduced by id, which is a plain map insert;
   - the declared actions, a version, and the owning tenant and user.

   Admission caps bound operations, bytes and depth per surface and per thread
   (E). This record replaces the run-scoped in-memory backbone and the 512-event
   ring as the source for recovery.
4. **Durable data lives in domain entities, not the A2UI data model.** This is
   San Saba's Shared Entity-Graph Pattern (SSW-ARCH-001 §1.3, §3.2):
   - application records live in the entity graph, held server-side in
     Postgres and synced by the realtime fabric or Electric/PGlite;
   - agents write entities through tools or A2A, or through AG-UI `STATE_*`,
     which upstream defines as application state;
   - the A2UI data model holds only surface-local values (filter text, open
     tabs). The host projects bound entities into it on the client, so
     standard renderers still see an ordinary data model.
5. **Actions arrive as AG-UI run input**, as
   `forwardedProps.a2uiAction.userAction`, the upstream shape (§4). UAR then:
   - checks the action against the stored structure: offered, current version,
     permitted for this tenant and user;
   - routes it to a **deterministic surface controller** or to the agent;
   - runs the follow-up on the same thread, so it sees the surface (O3).
6. **Reconnect restates snapshots.** AG-UI 1.0 SSE cannot resume and requires
   replays to be restated as snapshots (§3). On connect, UAR sends
   `MESSAGES_SNAPSHOT` carrying the thread's complete activity set, plus
   `STATE_SNAPSHOT` for application state. No event-ring position is needed.
7. **Morphing is re-assembly.** An assembler (a deterministic controller,
   flint-forge `surfaces/assemble`, or the agent) runs again with the user's
   context and the changed conditions, then sends a new `replace:true` activity
   for that user's sessions. Data stays in entities, and stable component ids
   keep local input.
8. **One catalog registry.** Catalogs are a2ui.org catalog documents
   (`definitions`), and both server admission and client renderers read them.
   flint-forge already serves one with 55 definitions (SSW-A2UI-011). UAR's
   closed 9-component enum and the separate client catalog would be replaced by
   catalogs read from that registry.

### San Saba requirement coverage under F

Requirement ids are from `assessment.md` §3.3.

| Req | How F meets it | Still needed outside F |
|---|---|---|
| SR1 durable surfaces | rule 3: thread- or session-scoped record | — |
| SR2 entity data and invalidation | rule 4: entity graph, entity-change sync | PEM and realtime fabric integration |
| SR3 assembly and catalog | rules 7 and 8 | one assembler interface (Q2) |
| SR4 chat artifacts with HITL | rules 1, 2 and 5 | — |
| SR5 identity and tenant | rule 5 checks the tenant and user | G12 blockers (Q5) |
| SR6 capabilities per Mini App | rule 5 checks capabilities from the manifest | manifest read by UAR (Plan) |
| SR7 per-persona views and notifications | rule 7: re-assembly per user | notification bus stays in the app |
| SR8 job status and results | jobs are entities with status; surfaces bind to them | job entities in the app |
| SR9 drafts and compare-and-swap | rule 4: PEM edit buffer; rule 5 version check on actions | — |
| SR10 dense screens | catalog `DataTable`/`DataGrid`/`Pagination`; data in entities | — |
| SR11 accessibility | renderer responsibility | renderer audits |

## Costs and the uncomfortable parts

- **Most of the existing direction is discarded.**
  - UAR's `/a2ui/surfaces` shared-state projection, change 1's four revisions,
    the `actions:{}` initial-state fix and the console's patch replay all lose
    their purpose.
  - Specs to revise: `ag-ui-chat-conformance` (surfaces in `STATE_SNAPSHOT`),
    `agui-runs-stream-fidelity`, `runtime-console` and `a2ui-uar-renderer` (9
    components).
  - `docs/protocols/ag-ui-profile.md:33` excludes "draft upstream activity"
    events. AG-UI 1.0 now standardizes them, so the profile must change.
- **AG-UI has no resumable stream.** Long-lived push for dashboards and morphing
  needs AG-UI's persistent `connect()` path. Upstream's JS default throws
  `ConnectNotImplementedError` (§3). UAR would implement it and clients would
  call it. Fit with real clients is unverified.
- **Operation lists grow.** Cumulative snapshots resend everything. F stays
  efficient only if high-rate data lives in entities. An application that
  streams metrics through `updateDataModel` performs badly under F.
- **Entity binding is a Prometheus extension, not A2UI.** It ties full
  applications to the entity graph and realtime fabric, which conflicts in part
  with "open standards first". The mitigation is that bindings resolve on the
  client into a standard data model. Pure A2UI clients still render, but without
  live entity data.
- **The client's data model diverges once the user types.** The server copy is
  never authoritative for it. Unsent input is either sent with the action
  (upstream `context`) or kept per surface on the client.
- **San Saba cannot use UAR today regardless of this choice.** SSW-ARCH-001
  §0.1 records three blockers:
  - UAR's user claims omit `tenant_id`;
  - its shared-secret authentication does not fit the workspace's RS256
    identity;
  - its A2A task store is not partitioned by tenant.

  F does not fix these. They are prerequisites for O9.
- **The evidence is source and documents only.** Nothing here was run. The
  upstream restore-after-reload path was not found (unverified, §4).

## Recommendation

Adopt F. It is the only option that meets O6 (action authorization) and O8
(standards) together. It also removes the two-reducer problem, because the
server reduces structure only, a map insert, and never the data model.

**Fallback:** if full applications were deferred, C alone (activity carriage,
op-list storage, clients reduce) would serve chat. The operator has ruled that
out, because chat and applications are both in scope.

## Operator decisions before Plan

- **Q1.** Adopt F as the A2UI surface architecture? Recommended: yes.
- **Q2.** Where does assembly live: UAR surface controllers, flint-forge
  `surfaces/assemble`, or both behind one interface? Recommended: one interface,
  with forge as the first implementation, because San Saba already uses it.
- **Q3.** Where does application data live: the entity graph plus realtime
  fabric, or AG-UI `STATE_*` alone? Recommended: the entity graph for
  applications, and AG-UI state as the agent's write channel into it.
- **Q4.** Revise UAR's AG-UI profile to adopt AG-UI 1.0 activity events and
  `forwardedProps.a2uiAction`? Recommended: yes. This is required by F.
- **Q5.** Fold San Saba's three UAR blockers (tenant claims, identity
  compatibility, tenant-partitioned A2A tasks) into this program, or track them
  separately? Recommended: separately but first, because O9 depends on them.
- **Q6.** Supersede change 1 (`a2ui-surface-state-patches`) and its dependants?
  Recommended: yes, keeping its findings as evidence.

---

## Independent review — 2026-10-28

A fresh-context `artifact-critic` reviewed `assessment.md` and `analysis.md`
alone. Verdict: **defects found — 3 critical, 12 warnings, 3 suggestions.**
It confirmed 30 citations as correct, covering UAR source, the San Saba
documents and `web_core`.

The lead re-verified the three claims that weigh most:

- the chat renderer builds a fresh `MessageProcessor` per artifact
  (`frontend/src/features/chat/components/a2ui-artifact-block.tsx:271-273`);
- client limits are `MAX_A2UI_MESSAGES = 128` and 256 KiB of source
  (`frontend/src/features/a2ui/a2ui-rendering-limits.ts:2-3`, path located by
  search);
- the dedicated AG-UI route resumes from a UAR-format `Last-Event-ID` cursor
  (`src/uar/api/agui.rs:662-670`).

### Critical

- **C1. The scoring is circular.** C is defined as storing the op list, which
  contains the full structure, yet scored "no" on authorization. F is credited
  with thread scope and the catalog by definition, while A-D are marked as
  needing them. With the same additions, C matches F except for entity
  binding. The real choice, "C plus entity binding plus controllers", is never
  put to the operator.
- **C2. Morphing erases input.** Rule 4 keeps user values in the A2UI data
  model, and component ids don't protect data-model paths. A cumulative
  `replace:true` snapshot starts with `createSurface`, which `web_core` rejects
  for a surface that already exists (`message-processor.js:465-467`).
  Sequence in UAR's renderer:
  1. the user types;
  2. a morph arrives;
  3. a fresh processor replays the initial `updateDataModel`;
  4. the typed input is gone.

  CopilotKit avoids this only with undocumented workarounds (drop
  `createSurface`, dedup by hash), and those workarounds have their own errors.
  O2 for surface-local values fails the same way.
- **C3. Server structure can differ from what the client shows.** `web_core`
  validates each `updateComponents` all-or-nothing against the catalog and the
  topology (`message-processor.js:572-581, 684-724`). The server's map insert
  does not. So an action on a component the client rejected passes the
  "offered" check. Template child lists also depend on the data model, which F
  keeps off the server. This is a second reducer: smaller than A's, but it
  still has to agree with the client.

### Warnings (summary)

- **W1.** O6 (high rate, several tabs) and O7 need a persistent push channel.
  `connect()` is unimplemented upstream, and UAR streams end at the terminal
  event, so a morph triggered outside a run reaches no open stream.
- **W2.** Rule 6 drops event-ring cursors, but UAR's AG-UI route already
  resumes with them (`agui.rs:662-670`). The inventory left this route out.
- **W3.** San Saba's chat artifact is not a2ui.org A2UI (DGA
  `{__type:'a2ui', schema, data}` as `CUSTOM A2UI_ARTIFACT`). Its application
  actions go through A2A. It validated its design against `STATE_*`. Forge
  layout regions have no mapping in F. O9 should be "blocked", not "yes".
- **W4.** The Q2 rationale "San Saba already uses forge assembly" is false:
  the C-29 slice never calls it (SSW-A2UI-011:66-71).
- **W5.** A catalog document gives renderers no code. Forge's 55 definitions
  need an implementation in every renderer. SR10 and the hot-reload of
  `catalogId` are not addressed.
- **W6.** Op lists cannot be compacted without a reducer. UAR's renderer stops
  at operation 129. Follow-up runs SHOULD start with a snapshot, which resends
  the whole list each time.
- **W7.** Activity messages are stripped from run input, and UAR builds the
  prompt from client context. Nothing injects the stored record into the
  follow-up, so O3 is not delivered.
- **W8.** Q3 uses AG-UI state as the agent's write channel. That makes the
  browser persist agent writes, which risks double application across tabs or
  replays and bypasses capability inversion.
- **W9.** There is no atomic compare-and-swap on the version check. One owner
  per record can't express per-persona variants of a shared surface.
  `MESSAGES_SNAPSHOT` would need filtering per viewer.
- **W10.** Adopting upstream `userAction` drops UAR's existing
  `a2uiClientDataModel` capture.
- **W11.** Missing options and costs:
  - a non-authoritative server `MessageProcessor` mirror, which upstream allows
    and which would fix C3;
  - A2A carriage;
  - F's own new dependencies (PEM, realtime fabric, Electric, forge).
- **W12.** Decisions missing from Q1-Q6:
  - the push channel;
  - compaction and caps;
  - shared versus per-user surfaces;
  - migrating San Saba's carriage;
  - keeping `Last-Event-ID`;
  - the A2UI version target;
  - who persists agent entity writes;
  - the Cedar conflict.

  Q4 is implied by Q1.

### Suggestions

- Name which `routes.rs` each gap cites.
- SH-5's one-minute limit applies only to the inbox item and the email.
- G8 should note the client's existing caps.

### Lead assessment of the review

The review holds, and it changes the recommendation rather than refining it:

- Carriage as AG-UI activities still stands, on upstream evidence.
- The claim that F removes the reducer problem does not. Something must
  reduce operations incrementally to:
  - apply morphs without erasing input;
  - compact long-lived surfaces;
  - know what the client actually shows.
- The decision decomposes into four independent axes:
  1. carriage;
  2. where reduced A2UI state lives;
  3. where application data lives;
  4. the push channel.
- The strongest unexplored option is a **non-authoritative server mirror that
  runs `web_core` itself**, a form of D. It gives one reducer, the same
  validation as the client, and compaction through a v1.0 inline
  `createSurface`. It pairs with clients that keep one live processor per
  surface and apply deltas instead of replacing.

**Readiness: BLOCKED.** The analysis needs a revision before an operator
decision is well-posed.

# The chat-plus-canvas three-pane shell

A pattern definition for agentic workspace products, and how three of ours apply it:
the Universal Agent Runtime (UAR) operator shell, PatientGPT, and the San Saba Workspace.

Date: 2026-10-05. Status: working standard for these three products.

---

## 1. Name

There is no single official industry name. Two vocabularies describe it, and together they are precise:

- **The layout.** Setproduct calls it the "two-pane standard": a left rail with history, a centre message
  column capped at about 720–768 px, and an optional right panel for artifacts. Claude.ai, ChatGPT and
  Perplexity all converge on it ([Setproduct](https://www.setproduct.com/blog/ai-chat-interface-ui-design)).
  Apple's term for the same structure is a **split view with a sidebar and an inspector column** on the
  trailing edge ([Apple, UISplitViewController](https://developer.apple.com/documentation/uikit/uisplitviewcontroller.md)).
- **The right-hand surface.** **Artifacts** or the **split-pane** pattern: AI output renders in a panel
  beside the chat instead of in the message stream
  ([AI UX Playground](https://aiuxplayground.com/pattern/chat-artifact/)). AWS Cloudscape calls the
  expanded form a **canvas preview**, shown "in an adjacent area"
  ([Cloudscape](https://cloudscape.design/patterns/genai/artifact-previews/)). Product names are Claude
  **Artifacts** and ChatGPT **Canvas**.

**Our name: the chat-plus-canvas three-pane shell.** Its regions are **Navigator**, **Thread** and
**Canvas**. In code, the canvas component may be called `Inspector` (Apple's term), which avoids clashing
with ChatGPT's product name and the HTML `<canvas>` element.

## 2. Regions

```
┌──────────────┬───────────────────────────────┬────────────────────────┬──┐
│  NAVIGATOR   │            THREAD             │         CANVAS         │R │
│              │                               │                        │A │
│ new · search │ breadcrumb: container / chat  │ tabs: what the work     │I │
│ containers   │ messages, execution record,   │ produced and the tools  │L │
│ recents      │ approval cards, inline A2UI   │ that act on it          │  │
│ settings     │ composer: target, model, mode │ (files, runs, browser,  │  │
│              │                               │  mini apps, A2UI views) │  │
└──────────────┴───────────────────────────────┴────────────────────────┴──┘
   ~240–272px        flexible, text ≤ 760px          360–760px, resizable   44px
```

| Region | Job | Never |
|---|---|---|
| **Navigator** (left) | Where you are and what you can open: new chat, search, the **containers** (projects, patients, deals), recents, settings | Holds the work itself |
| **Thread** (centre) | The conversation and the agent's work: messages, execution record, approval cards, small inline A2UI cards, composer | Hosts a full application |
| **Canvas** (right) | What the work produces and the tools that act on it: files, previews, run details, browser, **mini apps**, larger A2UI surfaces | Opens without a reason; steals the thread on wide screens |
| **Rail** (right edge, optional) | One icon per pinned canvas tool, so a tool can be opened without a menu | Becomes a second navigator |

## 3. Rules every instance follows

1. **The container is the unit of context.** Each product has one container type that owns chats, files,
   data and permissions: a UAR project, a PatientGPT patient and case, a San Saba deal. The navigator lists
   containers; the thread and the canvas always show which one is in scope.
2. **The thread stays the focus.** The canvas opens on demand: when the agent produces something, when the
   user opens a tool, or when the user pins it. On narrow screens it becomes a slide-over, then a bottom sheet.
3. **Applications live in the canvas, not the thread.** Anything with its own navigation (an embedded app, a
   browser, a document editor) is a canvas tab. The thread may show a small inline card that opens it.
4. **The canvas can grow.** It has four states: closed, docked (360–760 px, resizable), **expanded** (takes
   the main column; the thread collapses to a narrow strip that stays visible), and **popped out** into its
   own window on desktop. That is how a full application fits without leaving the pattern.
5. **The agent and the canvas share context in both directions.** Opening a container or a record in the
   canvas tells the thread what is selected; an agent action can open a canvas tool already pointed at the
   right record.
6. **Settings reuse the shell.** The navigator becomes the settings menu, and the canvas hosts flows such as
   adding a provider.
7. **Human gates are drawn by the app.** Approvals and sign-offs appear as app-drawn cards in the thread
   (and in the canvas when the tool needs them), never as model-generated buttons.

### Evidence for "tools on the right edge"

- **Claude.** Artifacts open in a panel to the right of the chat
  ([Fast.io](https://fast.io/resources/claude-artifacts-guide/), [AI UX Playground](https://www.aiuxplayground.com/teardowns/claude/artifacts/)).
- **Claude Code desktop.** Browser, diff, terminal, file, plan, tasks and subagent views are panes beside the
  chat. Users can drag them anywhere, close them with ⌘\ and pop them out into windows
  ([Claude Code desktop docs](https://code.claude.com/docs/en/desktop)). Adjacent is the norm; the right
  edge is the default, not a fixed rule.
- **Codex app.** A project sidebar, the thread, and a review pane for diffs and previews; files open "in a
  sidebar" with rich previews, and the in-app browser sits alongside
  ([OpenAI Codex app](https://developers.openai.com/codex/app), [summary notes](https://notes.dsebastien.net/30+Areas/33+Permanent+notes/33.02+Content/Codex+App)).
  OpenAI's public docs do not state which side.
- **Cherry Studio.** Mini apps are web apps opened from a Launchpad and pinned to the (left) sidebar with
  "Add to sidebar"; the app then fills the main area
  ([Cherry Studio Launchpad](https://docs.cherryai.com.cn/cherry-studio/preview/launchpad)). That is the
  model San Saba follows today. The difference: Cherry Studio's mini apps are separate destinations, while
  San Saba's mini apps must work **beside** a deal's conversation, which is the Claude and Codex model.

**Conclusion for San Saba: confirmed.** Mini apps belong in the canvas on the right, opened from a right-edge
rail, with an expanded state for the core app. This matches Claude and the Codex app, and the default
arrangement of Claude Code desktop.

## 4. Three instances

| | UAR operator shell | PatientGPT | San Saba Workspace |
|---|---|---|---|
| Users | Operators and builders of agents | A surgeon and clinic staff (prior authorization) | Land and acquisitions staff buying mineral and lease interests |
| **Container** | Project (proposed) | Patient → case | **Deal** (proposed in brief SSR-006; not built yet) |
| Navigator | New chat, search, Projects, Knowledge, Agents, A2UI registries, Runs, recents, settings | Patients with their cases; each case row shows its state track | **Deals** grouped by stage, plus search; core app and modules move to the rail |
| Thread | Chat with orchestrator routing or a pinned agent; execution record; approval cards | Case chat; scripted AG-UI turns; allow-listed A2UI cards; surgeon-only controls | **Deal chat**: the deal's agent, quick actions from the active module, human-review cards |
| Canvas | Run · Files · Browser · Plan; add-provider and other flows | Files and evidence: chart documents, generated records | **Mini apps**: Mineral Rights Manager (core, embedded), Reports, Letter Agreement, later Sourcing and Curative Tracker; deal documents; A2UI surfaces |
| Rail | Not needed (four tabs) | Not needed | **Yes**: one icon per available mini app plus Documents |
| Gates | Tool approvals, Cedar policy | Affirmation, signing, submission (surgeon-only) | Valuation, send / no-send, document release (named human); nothing transmits or moves money |
| Responsive | Canvas docks ≥1181 px, slide-over below, full-width sheet on phones | Docks ≥1100 px, right overlay 768–1099 px, bottom sheet on phones | Same breakpoints as UAR; expanded canvas for the core app |

## 5. The San Saba Workspace in this pattern

### 5.1 Where the code is today

Read from `sansaba-workspace` on 2026-10-05 (`web/src/app/workspace-shell.tsx` and neighbours):

- Three planes already exist, but in a different arrangement: **navigation left** (230 px; "Core App" and
  "Agentic Modules"), **mini app in the centre** (`content-plane.tsx`, one active at a time), and **chat on
  the right** (`conversation-plane.tsx`, always shown at ≥xl, toggleable at ≥lg, one plane at a time below
  lg with a bottom pane switcher).
- Mini apps are described by `manifests/mini-app.schema.json` (`apiVersion: prometheusags.ai/v1alpha1`).
  The `surface.kind` is `embedded-app`, `a2ui-surface` or `native`. Embedded apps mount through
  `RemoteMiniAppHost` as a sandboxed cross-origin iframe and talk over the `sansaba.mini-app-bridge`
  postMessage channel (`entity.intent` / `entity.apply`, `navigation.intent` / `navigation.apply`,
  `auth.state`). A2UI surfaces are assembled by Forge into `region: "main"`.
- The manifest has **no placement, icon or pin field**.
- Chat is scoped to the **active mini app and the selected record** (a letter agreement), not to a deal.
- **There is no deal entity or deal list.** The deal container and its stages are a proposal (brief
  SSR-006): Preliminary → Letter Agreement → Acquisition → Payment → Closed.

### 5.2 Target layout

| Region | San Saba content |
|---|---|
| Navigator | Search; **Deals** grouped by stage with a stage chip and "waiting on you"; New deal; settings. The "Core App" and "Agentic Modules" sections leave the navigator. |
| Thread | One conversation per deal (more can be added). Breadcrumb `Deal · stage`. Quick actions come from the deal stage and the active mini app. Human-review cards for valuation, send / no-send and document release. Small A2UI cards inline (for example a checklist item or a generated letter summary). |
| Canvas | Tabs for the open mini apps and for Deal documents. Embedded apps run in the same `RemoteMiniAppHost`; A2UI surfaces render with the Forge catalog. |
| Rail | Right edge: Mineral Rights Manager, Reports, Letter Agreement, Documents; Sourcing and Curative Tracker greyed out with "unavailable" until enabled. Click opens or focuses the tab; ⌥-click pops it out (desktop). |

**Behaviour**

1. **Selecting a deal** loads its conversation in the thread and sends `entity.apply` for that deal to every
   open mini app, so each embedded app shows the same deal.
2. **A mini app reporting a selection** (`entity.intent`) updates the thread's context line ("Looking at
   letter agreement LA-2026-118") without changing the deal.
3. **An agent action can open a tool**: for example "Draft the letter agreement" opens the Letter Agreement
   tab on the right record and shows the staged result there, while the thread keeps the approval card.
4. **The core app gets room**: Mineral Rights Manager opens **expanded** by default (canvas takes the main
   column, thread narrows to a strip with the composer), and the user can dock it back to the side.
5. **Narrow screens**: below the docking breakpoint the canvas is a slide-over from the right; on phones the
   existing one-plane switcher stays, with Deals · Chat · Apps.

### 5.3 Proposed manifest additions

The placement rule is a shell rule, but modules need to state their preferences. Proposed for
`mini-app.schema.json` (all optional, with defaults that match the rule above):

| Field | Values | Default | Purpose |
|---|---|---|---|
| `presentation.region` | `canvas` · `thread-card` | `canvas` | Where the module's main surface renders. `thread-card` is for small A2UI cards only |
| `presentation.initialSize` | `docked` · `expanded` | `docked` (`expanded` for `kind: core`) | Starting canvas state |
| `presentation.minWidth` | px number | `360` | Below this the shell opens it expanded |
| `presentation.popOut` | boolean | `true` | Whether desktop can pop it into its own window |
| `rail.icon` | icon name from the shared set | required for `canvas` | Rail and tab icon |
| `rail.order`, `rail.pinned` | number, boolean | manifest order, `true` | Rail ordering and default pinning (users can unpin) |
| `context.container` | `deal` · `none` | `deal` | Whether the shell sends the deal with `entity.apply` |

For A2UI, Forge's assemble call should take the region instead of assuming `main`: `canvas` for a surface,
`thread` for an inline card.

The README describes fields the schema does not have (`navSection`, `protected`, `agent.orchestrator`) and
calls `kind: core | agentic` retired while the schema still uses it. Settle that before adding fields.

### 5.4 Dependencies and open questions

- **The deal entity comes first.** Without a `Deal` table and list there is nothing to scope the navigator
  and thread to. The SSR-006 brief is the starting point.
- **Bridge contract:** confirm that every embedded app accepts `entity.apply` for a deal, and what it shows
  when the deal does not exist on its side.
- **Two embedded apps at once:** with tabs, several iframes may stay mounted. Decide whether hidden tabs
  stay alive (fast switching, more memory) or unload.
- **UAR:** San Saba keeps UAR-backed mini apps unavailable until UAR has tenant claims and partitioned
  storage (SSW-ARCH-001 §0.1). The pattern does not change that.

## 6. What changes in each product

| Product | Change |
|---|---|
| UAR operator shell | None to the layout. Projects (proposed) become the container; the canvas already hosts run, files, browser and plan. |
| PatientGPT | None. It already follows the pattern with patients and cases as containers and the Files and evidence panel as the canvas. |
| San Saba Workspace | Move mini apps from the centre to a right-hand canvas with a rail, move the chat to the centre, add Deals to the navigator, add the manifest fields above. |

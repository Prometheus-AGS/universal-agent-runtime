# Upstream research: A2UI carriage and surface-state ownership

Research date: 2026-10-28. Phase: research input to an architecture decision (no verification run; gate CLOSED).

Sources were read as raw files at pinned commits:

- A2UI: `a2ui-project/a2ui` @ `ccf5dc20d0b0ed041b03e3913d807420ff45e178` (committed 2026-10-28). Base: `https://github.com/a2ui-project/a2ui/blob/ccf5dc20d0b0ed041b03e3913d807420ff45e178/` (written below as `A2UI:`).
- AG-UI: `ag-ui-protocol/ag-ui` @ `dab9c95f8e70f03ebc2a817e9900a245dfca6ddd` (2026-10-27). Base: `https://github.com/ag-ui-protocol/ag-ui/blob/dab9c95f8e70f03ebc2a817e9900a245dfca6ddd/` (written as `AGUI:`).
- CopilotKit: `CopilotKit/CopilotKit` @ `4693af4d398c63da6f185ed7024ca5b196c20958` (2026-10-28). Base: `https://github.com/CopilotKit/CopilotKit/blob/4693af4d398c63da6f185ed7024ca5b196c20958/` (written as `CK:`).
- Local pin: `frontend/node_modules/.pnpm/@a2ui+web_core@0.12.0/node_modules/@a2ui/web_core/` (written as `WC:`).

Method note: the project-mandated `firecrawl developer` index was not usable. `firecrawl developer ...` returned `Error: unknown command 'developer'`. Claims below come from upstream source files read directly, not from that index.

Version status: v0.9.1 is "Current Production" (A2UI:specification/v0_9_1/docs/a2ui_protocol.md#L12-L14) and "closed" (A2UI:specification/v0_9_1/README.md#L5). v0.9 is marked "Stable" (A2UI:specification/v0_9/docs/a2ui_protocol.md#L13-L14). v1.0 (formerly v0.10) is a "Candidate" (A2UI:specification/v1_0/README.md#L3-L5).

---

## 1. A2UI spec: state ownership, message semantics, actions, catalogs, security

### Who owns surface state
- **The renderer/client owns it.** The client builds the UI by applying a stream of messages: "The client parses each object as a distinct message and incrementally builds or updates the UI" (A2UI:specification/v0_9_1/docs/a2ui_protocol.md#L22).
- **The client stores components in a map and rebuilds the tree at render time.** "The client is responsible for storing all components in a map (e.g., `Map<String, Component>`) and recreating the tree structure at render time" (…/a2ui_protocol.md#L324). Updates that arrive before `root` exists are buffered (…#L328).
- **The client-side data model is authoritative for user input.** "Because the local Data Model is the single source of truth" (…#L491). Two-way binding "is **local to the client**". Typing sends nothing to the server; state goes up only when an action fires (…#L497-L501).
- **The server's view converges only at action time.** With `sendDataModel`, "The server treats the received data model as the current state of the client at the time of the action" (…#L593).
- **The agent-side SDK keeps no state by default.** The agent blueprint calls the default check "stateless". It says that "An agent that keeps one `MessageProcessor` across a session and runs its own outbound messages through it holds that tree" (A2UI:blueprints/modules/a2ui_agent.blueprint.md#L401-L405). The core blueprint lists both "Framework Renderers and Server Agents / Agent SDKs" as consumers of `MessageProcessor` (A2UI:blueprints/modules/a2ui_core.blueprint.md#L75). Python `a2ui_core` is described as "used by python-based server agents, rendering backends, and conformance testing frameworks" (A2UI:python/a2ui_core/README.md#L5).

### Server-side state, snapshot, resync, late join, reconnect
- **The spec defines none of these.** I searched the v0.9, v0.9.1 and v1.0 protocol docs and the v0.9.1 extension spec for snapshot, reconnect, resync, late join, replay, resume, history and persist. There were no matches in any of them.
- **The nearest mechanisms:**
  - **Ordered delivery is required.** The transport contract requires in-order delivery because "out-of-order delivery can corrupt the UI state" (A2UI:specification/v0_9_1/docs/a2ui_protocol.md#L88).
  - **An agent may skip `createSurface`** "if it knows the surface has already been created (e.g., by another agent)" (…#L179). v1.0 rewords this as "a preexisting surface that it has permission to modify" (A2UI:specification/v1_0/docs/a2ui_protocol.md#L176).
  - **v1.0 can deliver a whole surface in one message.** "Single-Message UI Instantiation" puts `components` and `dataModel` inside `createSurface` (A2UI:specification/v1_0/docs/a2ui_protocol.md#L38, #L187-L188).
  - **Re-creating an existing surface is an error.** v1.0 calls it `INTEGRITY_ERROR` and requires the `surfaceId` to be "globally unique for the renderer's lifetime" (…v1_0…#L178). In v0.9.1 it is "an error" (…v0_9_1…#L179).

### `updateComponents` semantics
- **Upsert by `id`.** Components are "added to or updated within a specific surface" as a flat adjacency list. Forward references are allowed and are rendered progressively (A2UI:specification/v0_9_1/docs/a2ui_protocol.md#L206).
- **The spec does not say whether an update merges into or replaces an existing component's properties (unverified at spec level).**
- **The local web_core implementation replaces them.** It replaces the whole property bag (`existing.properties = properties`). It recreates the component when the type or catalog changes (WC:src/processing/message-processor.js#L544-L563). It validates every entry before applying any of them (…#L572-L581).

### `updateDataModel` semantics
- **It sets a value at a JSON Pointer path.** It "replaces the value at the specified `path`". A missing `path` or `/` replaces the whole model (A2UI:specification/v0_9_1/docs/a2ui_protocol.md#L243).
- **Upsert rules.** "Strict upsert semantics": the value is created if the path is absent and updated if present. An omitted value removes the key. For arrays, the index is set to `undefined` and the length is kept (…#L530-L536, #L249).

### User actions back to the agent
- **Server actions are named events.** They are declared as `action.event` with a `name` and an optional `context`. Local actions use `functionCall` and stay on the client (…#L357-L393).
- **The `action` message carries:** `name`, `surfaceId`, `sourceComponentId`, `timestamp` and `context`, all required. `context` is "any context provided in the component's `action` property" (…#L822-L832). v1.0 adds an optional `userMessage` and `metadata` and says context comes "after resolving all dynamic data bindings" (A2UI:specification/v1_0/docs/a2ui_protocol.md#L1359-L1371).
- **Data model values travel two ways:**
  - **Through context paths.** "the `context` property of the action can reference the modified data paths" (…v0_9_1…#L501-L520).
  - **Through `sendDataModel`.** When `sendDataModel: true` is set on `createSurface`, the client attaches the **entire** surface data model to transport metadata on every message to the creating server (…#L186, #L586-L593). This is the `a2uiClientDataModel` object with `surfaces: {surfaceId: model}` (…#L851-L857). The data goes "exclusively to the server that created the surface" (…#L590).

### Catalog negotiation and custom catalogs
- **Capabilities travel in transport metadata or handshakes, not as A2UI messages** (…#L834-L836, #L90).
- **Client side.** The client sends `a2uiClientCapabilities.supportedCatalogIds` (required) and optionally `inlineCatalogs` (…#L842-L849). `inlineCatalogs` should be sent only if the agent declares `acceptsInlineCatalogs: true` (A2UI:specification/v0_9_1/json/client_capabilities.json#L17-L23).
- **Agent side.** The agent advertises `supportedCatalogIds` and `acceptsInlineCatalogs` (default `false`) (A2UI:specification/v0_9_1/docs/a2ui_extension_specification.md#L46-L47).
- **The surface's `catalogId` is fixed.** It is set by `createSurface` and is a string identifier, not a resolvable URI (A2UI:specification/v0_9_1/docs/a2ui_protocol.md#L184, #L312-L318).
- **Catalog format is JSON Schema.** Set `$id` and `catalogId` to the same URI (…#L316).
- **Child references must use the shared types.** Use `common_types.json#/$defs/ComponentId` for one child and `ChildList` for many, so validators can check the tree (…#L160-L171).
- **v1.0 adds stricter catalog rules.** These cover naming (UAX #31), schema conventions and mixable catalogs (A2UI:specification/v1_0/docs/a2ui_protocol.md#L506-L736).

### Security guidance
- **Functions are named, not shipped.** Functions are referenced by name, which "avoids sending executable code" (A2UI:specification/v0_9_1/docs/a2ui_protocol.md#L601).
- **Orchestrators should verify displayed identity.** They should set or overwrite `iconUrl` and `agentDisplayName` to prevent "malicious agents from impersonating trusted services" (…#L721-L725).
- **Validate, then return structured errors.** The prompt–generate–validate loop returns a `VALIDATION_FAILED` error with `surfaceId`, `path` and `message` (…#L781-L816).
- **Validate on both sides.** Agents validate before sending. Clients validate again for "Security and stability … protecting against … compromised agent outputs" (A2UI:docs/public/concepts/catalogs.md#L439-L446).
- **Catalog guide rules.** It says to allowlist components, validate properties and sanitize text (A2UI:docs/public/guides/defining-your-own-catalog.md#L50-L56).
- **v1.0 function boundaries.** `callRendererFunction` enforces `allowedCallers` and `requiresUserActivation` and rejects violations with `INVALID_FUNCTION_CALL` (A2UI:specification/v1_0/docs/a2ui_protocol.md#L320-L327).
- **The spec sets no numeric size, rate or depth limits.** I found none in the protocol docs I read. Limits exist only in implementations (see §5).

## 2. A2UI transports and upstream sample state

- **v0.9.1 transports:**
  - **The spec is transport-agnostic** (A2UI:specification/v0_9_1/docs/a2ui_protocol.md#L80-L93).
  - **A2A binding.** Each envelope is one A2A message Part. Capabilities and the data model go in `metadata`. `contextId` maps the session (…#L99-L108).
  - **AG-UI binding.** It is described in prose only: "low latency and shared state message passing" (…#L110-L113).
  - **Other carriers named:** MCP ("tool outputs or resource subscriptions"), SSE with JSON-RPC, WebSockets and REST (…#L115-L122).
- **v1.0 transports:**
  - **AG-UI is "the standard transport binding for Agent-to-User Interaction"** (A2UI:specification/v1_0/docs/a2ui_protocol.md#L100-L102).
  - **MCP can carry A2UI** over "tool calls, tool outputs, or resource subscriptions" (…#L108-L110).
  - **A2A is the only formal "transport extension" document** (A2UI:specification/v1_0/README.md#L19-L23).
- **The docs site status table:**
  - A2A and AG-UI are listed as "Stable". REST is "Planned". WebSockets and SSE are "Proposed" (A2UI:docs/public/concepts/transports.md#L19-L25).
  - It says AG-UI "translates A2UI messages to AG-UI events and handles transport and state sync automatically" (…#L46).
- **Sample state: no evidence the sample agent materializes surface state.** The restaurant_finder sample agent uses `A2uiGenerator`/`A2uiRequestProcessor`, which handle prompt and response parsing per request. It also uses ADK `InMemorySessionService`, whose session state holds `base_url` (A2UI:samples/agent/adk/restaurant_finder/agent.py#L50, #L155, #L206-L237). `A2uiRequestProcessor` "exposes prompt generation and response parsing" (A2UI:python/a2ui_agent/README.md). Neither file I read applies messages to a surface model. Other samples were not surveyed (unverified).

## 3. AG-UI events, snapshot/delta, activity, passthrough, reconnect

- **Event list (AG-UI spec 1.0):** 31 types in eight families (AGUI:docs/spec/1.0/basic/index.mdx#L108-L115; AGUI:docs/spec/1.0/events/index.mdx#L10-L21):
  - **Lifecycle:** `RUN_STARTED`, `RUN_FINISHED`, `RUN_ERROR`, `STEP_STARTED`, `STEP_FINISHED`.
  - **Text:** `TEXT_MESSAGE_START`, `TEXT_MESSAGE_CONTENT`, `TEXT_MESSAGE_END`, `TEXT_MESSAGE_CHUNK`.
  - **Tools:** `TOOL_CALL_START`, `TOOL_CALL_ARGS`, `TOOL_CALL_END`, `TOOL_CALL_CHUNK`, `TOOL_CALL_RESULT`.
  - **Reasoning:** `REASONING_START`, `REASONING_END`, `REASONING_MESSAGE_START`, `REASONING_MESSAGE_CONTENT`, `REASONING_MESSAGE_END`, `REASONING_MESSAGE_CHUNK`, `REASONING_ENCRYPTED_VALUE`.
  - **State:** `STATE_SNAPSHOT`, `STATE_DELTA`, `MESSAGES_SNAPSHOT`.
  - **Activity:** `ACTIVITY_SNAPSHOT`, `ACTIVITY_DELTA`.
  - **Subagents:** `SUBAGENT_STARTED`, `SUBAGENT_FINISHED`, `SUBAGENT_ERROR`.
  - **Passthrough:** `RAW`, `CUSTOM`.
  - **Consumers must accept all families** (…events/index.mdx#L23-L27).
- **`STATE_SNAPSHOT` / `STATE_DELTA`:**
  - **A snapshot replaces state wholesale.** "A consumer MUST replace its state with `snapshot` — no merging" (AGUI:docs/spec/1.0/events/state.mdx#L13-L17).
  - **A delta is an RFC 6902 patch** against the consumer's current state (…#L19-L26). It applies atomically, and a consumer must not keep a partial result (AGUI:docs/spec/1.0/basic/patterns/snapshots.mdx#L34-L35, #L62).
  - **State persists across runs** and returns as the next run's input `state` (…state.mdx#L54-L60).
  - **State is untrusted input.** "State events are remote writes into application state. An application MUST validate what it reads out of state" (…#L89-L96).
- **`ACTIVITY_SNAPSHOT` / `ACTIVITY_DELTA`: these exist in 1.0.**
  - **An activity is UI-rendered material** stored as a message with object `content` and an open `activityType` string (AGUI:docs/spec/1.0/events/activity.mdx#L6-L30).
  - **Snapshots create or replace content.** `replace` defaults to true, and `replace:false` leaves the existing message untouched. "It is not a merge" (…#L31-L38).
  - **Deltas are RFC 6902 patches** that require a prior snapshot. They are resynchronized by sending a fresh snapshot with the same `messageId` (…#L52-L71).
  - **Activity messages never go back to the agent.** "a consumer MUST strip activity messages from the `messages` it sends in run input" (…#L73-L78).
- **`MESSAGES_SNAPSHOT`:**
  - **It carries the producer's complete owned message set.** Messages are replaced in place by id, and new ones are appended (AGUI:docs/spec/1.0/events/state.mdx#L28-L40).
  - **Client-only messages survive by default.** Consumer-held activity and reasoning messages are kept unless the snapshot itself carries activity messages. If it does, it declares the complete activity set (…#L41-L49).
- **`CUSTOM` / `RAW`:**
  - **`CUSTOM` is opt-in.** A consumer that does not recognise the `name` MUST ignore the event. A producer "MUST NOT rely on a `CUSTOM` event to carry semantics this specification assigns to a standard event" (AGUI:docs/spec/1.0/events/passthrough.mdx#L40-L65).
  - **`RAW` carries no protocol behaviour.** A consumer MUST NOT derive protocol behaviour from it (…#L28-L34).
  - **Both are untrusted** (…#L72-L78).
- **Reconnect and resume:**
  - **HTTP+SSE cannot resume.** "SSE's `Last-Event-ID` mechanism is not used, and a broken stream cannot be re-entered. Re-running is a new run with a new `runId`" (AGUI:docs/spec/1.0/basic/transports/http-sse.mdx#L51-L55).
  - **A dropped connection is a truncated run** (AGUI:docs/spec/1.0/basic/transports/index.mdx#L27-L29).
  - **Replays must be snapshots.** A stream MAY replay a thread's history as several runs, and that restatement "MUST restate it as snapshots", which keeps it idempotent (AGUI:docs/spec/1.0/events/lifecycle.mdx#L190-L202).
  - **Producers should send snapshots when baselines may differ.** A producer "SHOULD send one whenever it cannot guarantee that the consumer's baseline matches its own — … or at the start of a run that continues an old thread" (AGUI:docs/spec/1.0/basic/patterns/snapshots.mdx#L26-L29).
  - **The JS client has a persistent `connectAgent()`/`connect()` path.** Its default throws `ConnectNotImplementedError`. Middleware runs in `runAgent()`, while "`connectAgent()` currently calls `connect()` directly" (AGUI:docs/sdk/js/client/abstract-agent.mdx#L127, #L157-L167, #L206-L215).
  - **Serialization concept page (non-normative).** It lists restoring "UI state after reloads or reconnects" and compacting deltas into snapshots as goals (AGUI:docs/concepts/serialization.mdx#L8-L14, #L56-L64).

## 4. A2UI over AG-UI: the de-facto mapping and who reduces

- **A2UI defines no normative A2UI-on-AG-UI mapping document.** A2UI's only formal transport extension is for A2A (A2UI:specification/v1_0/README.md#L19-L23). AG-UI says it "is not a generative UI specification" and "natively supports" A2UI (AGUI:docs/concepts/generative-ui-specs.mdx#L18-L20).
- **The de-facto mapping is `@ag-ui/a2ui-middleware`, which runs on the server.** It wraps the agent, injects catalog context and the `render_a2ui` tool, detects `a2ui_operations` tool results, emits `ACTIVITY_SNAPSHOT` with `activityType: "a2ui-surface"`, and forwards user actions as synthetic `log_a2ui_event` tool messages (AGUI:skills/ag-ui-a2ui-integration/references/a2ui-runtime-and-renderer.md#L6-L11). A2UI's own guide points to that AG-UI skill (A2UI:docs/public/guides/a2ui-with-any-agent-framework.md#L36-L45).
- **Carriage is an activity event, not shared state, a custom event or message text:**
  - **The activity type is fixed.** `A2UIActivityType = "a2ui-surface"` (AGUI:middlewares/a2ui-middleware/src/index.ts#L83).
  - **Content is raw A2UI operations.** It is `{ a2ui_operations: [...ops] }`, grouped per surface, sent as one `ACTIVITY_SNAPSHOT` per surface with `replace: true` "so all operations (createSurface + updateComponents + updateDataModel) are delivered atomically" (…index.ts#L1323-L1377).
  - **Streamed snapshots are cumulative.** "Each emitted snapshot is cumulative (createSurface + updateComponents + updateDataModel-so-far) with replace:true, so any single snapshot is self-sufficient" (…#L494-L497).
  - **Pre-paint lifecycle states share the same activity.** States (`building`, `retrying`, `failed`) ride the same `messageId` on the activity wrapper, never inside the A2UI envelope (…#L207-L235).
- **The server-side source of operations is tool calls and tool results.** In fixed mode, a backend tool returns `{a2ui_operations: [createSurface, updateComponents, updateDataModel]}`. In dynamic mode, a sub-agent streams `render_a2ui` args (AGUI:skills/ag-ui-a2ui-integration/references/a2ui-runtime-and-renderer.md#L55-L104). Catalog choice "belongs to the host/factory, not the subagent" (AGUI:middlewares/a2ui-middleware/src/types.ts#L103-L117).
- **User actions travel in run input, not as A2UI client messages:**
  - **The client sends the action in run input.** It goes in `RunAgentInput.forwardedProps.a2uiAction.userAction` with `{name, surfaceId, sourceComponentId, context, timestamp}` (AGUI:middlewares/a2ui-middleware/src/types.ts#L120-L147).
  - **The middleware turns it into synthetic messages.** It appends an assistant tool call plus a tool result whose text is `User performed action "…" on surface "…" … Context: {…}` (AGUI:middlewares/a2ui-middleware/src/index.ts#L327-L410).
  - **CopilotKit sets the property and starts a new run.** It sets `a2uiAction` via `copilotkit.setProperties` and calls `runAgent` (CK:packages/react-core/src/v2/a2ui/A2UIMessageRenderer.tsx#L464-L482).
  - **No data-model forwarding was found.** A `sendDataModel`/data-model forwarding path did not appear in the middleware `index.ts` or the CopilotKit renderer files I read. `attachDataModel` exists only as a type field (AGUI:middlewares/a2ui-middleware/src/types.ts#L166) (forwarding unverified).
- **The client reduces the A2UI messages:**
  - **CopilotKit renders the activity.** It registers an activity renderer for `activityType: "a2ui-surface"` and reads `content.a2ui_operations` (CK:packages/react-core/src/v2/a2ui/A2UIMessageRenderer.tsx#L252-L266).
  - **The client applies operations through web_core.** The renderer feeds the operations into `@a2ui/web_core/v0_9` `MessageProcessor.processMessages` in the browser (CK:packages/a2ui-renderer/src/react-renderer/core/A2UIProvider.tsx#L3, #L53-L72, #L87-L91).
  - **The client compensates for cumulative snapshots.** Because "content snapshots always include the full operation list" and the processor throws on duplicate `createSurface`, the client drops `createSurface` when the surface already exists and dedups by JSON hash (CK:…/A2UIMessageRenderer.tsx#L563-L582).
  - **The server keeps operation lists, not reduced state.** The middleware keeps per-run, per-tool-call operation lists (the `streamingToolCalls` map, AGUI:…/index.ts#L498-L505). It does not reduce them into component/data state.
- **How a surface is restored after reload was not found (unverified).** I found no handling of history or `MESSAGES_SNAPSHOT` in the middleware `index.ts` (grep for histor/MESSAGES_SNAPSHOT/replay returned no matches). README: "Historical calls are not executed again" (AGUI:middlewares/a2ui-middleware/README.md#L53). The LangGraph helper preserves `render_a2ui` tool results in saved history (AGUI:integrations/langgraph/typescript/src/a2ui-history.ts#L1-L85).

## 5. Local pin: `@a2ui/web_core` 0.12.0

- **Protocol coverage.** `MessageProcessor` processes v0.8, v0.9 and v1.0 messages (WC:src/processing/message-processor.d.ts#L18). The package repository field is `a2ui-project/a2ui`, directory `typescript/web_core` (WC:package.json).
- **No state-to-messages serialization or snapshot API was found.** The public `MessageProcessor` surface is: `processMessages`, `processMessagesAsync`, `processOperation`, `getSurfaces`/`getSurface`, `onSurfaceCreated`/`onSurfaceDeleted`, `getRendererCapabilities`, `getRendererDataModel`, `callAgentFunction` and `dispose` (WC:src/processing/message-processor.d.ts#L82-L206). None of them turns processed state back into A2UI messages.
- **Nearest equivalents:**
  - **`getRendererDataModel(version?)`** returns `{version, surfaces: {id: dataModel.get('/')}}`, but only for surfaces created with `sendDataModel` (WC:src/processing/message-processor.js#L175-L216).
  - **`ComponentModel.componentTree`** returns `{id, type, ...properties}` for one component (WC:src/state/component-model.js#L59-L65).
  - **`DataModel.get('/')`** reads the whole model (WC:src/state/data-model.d.ts#L86).
  - **`ComponentNode.toJSON()`** serializes the *resolved* tree "for debugging and headless assertions", meaning bound values rather than protocol messages (WC:src/resolution/component-node.d.ts#L76-L83).
- **Implication.** Rebuilding `createSurface` + `updateComponents` + `updateDataModel` from these pieces would be caller-written code (inference).
- **The constructor accepts initial data.** `new DataModel(initialData)` (WC:src/state/data-model.d.ts#L29).
- **`createSurface` handling:**
  - **It accepts inline `dataModel` and `components`** (the v1.0 shape) (WC:src/processing/message-processor.js#L485-L499).
  - **It throws on a duplicate surface.** "Surface X already exists." (…#L465-L467).
  - **`updateComponents`/`updateDataModel` on an unknown surface throw `A2uiStateError`** (…#L569-L570, #L586-L588).
- **Limits in code:**
  - `MAX_ARRAY_INDEX = 10000` (WC:src/state/data-model.js#L25).
  - `MAX_GLOBAL_DEPTH = 50` and `MAX_FUNC_CALL_DEPTH = 5` (WC:src/validation/integrity-checker.js#L19-L21).
  - `MAX_DYNAMIC_VALUE_DEPTH = 1000` (WC:src/resolution/data-context.js#L174).
  - `MAX_DYNAMIC_CHILD_LIST_SIZE = 10000` (WC:src/resolution/generic-binder.js#L222).
  - `MAX_EXPRESSION_TEMPLATE_LENGTH = 10000`, `MAX_EXPRESSION_PARTS = 1000` and `ExpressionParser.MAX_DEPTH = 100` (WC:src/expressions/expression_parser.js#L21, #L29, #L294).
  - `MAX_FUNCTION_CALL_ARGS = 1000` (WC:src/types/helpers.js#L28).
  - Forbidden path segments: `__proto__`, `constructor`, `prototype` (WC:src/state/data-model.d.ts#L38-L39).
- **No limit was found on payload bytes, component count, surface count or data-model size.**
- **0.12.0 changelog:** `functionCall` actions now run locally. A `PayloadValidator` was added. Deleting past an array's end is a no-op. `createSurface` assigns `dataModel` at the root (WC:CHANGELOG.md#L3-L80).

## 6. Native and web renderers

- **Maintained table** (A2UI:docs/public/reference/renderers.md#L13-L30):
  - **React, Lit and Angular** (web): v0.8 and v0.9.1 Stable, v1.0 Planned.
  - **Flutter (GenUI SDK, `flutter/genui`)**: v0.8 and v0.9.1 Stable.
  - **SwiftUI**: "—" for v0.8 and v0.9.1, v1.0 Planned.
  - **Jetpack Compose**: v0.9.1 "Alpha", with code in androidx `a2ui/`.
- **In-repo code:**
  - **Web renderers** live in `typescript/renderers/{react,lit,angular}` (repo tree).
  - **Flutter.** `dart/a2ui_flutter` targets v1.0 and v0.9 but "is not published yet" (A2UI:dart/a2ui_flutter/README.md#L1-L14).
  - **Swift.** `swift/core` (`A2UISwiftCore`, a message processor and state engine) and `swift/swiftui` (`A2UISwiftUI`) exist in the repo (A2UI:swift/core/README.md#L1-L25; A2UI:swift/swiftui/README.md#L1-L25). The published table still marks SwiftUI as planned (A2UI:docs/public/roadmap.md#L26-L27). Release status is unverified.
- **No Kotlin/Android renderer in the A2UI repo.** `kotlin/` contains only `agent_sdk_legacy` (repo tree). Android support is the androidx Compose renderer listed above (not inspected; unverified).
- **Official React package.** `@a2ui/react` is called "the core A2UI React renderer maintained by the A2UI team" (A2UI:docs/public/ecosystem/renderers.md#L8-L9).
- **Community renderers are explicitly "not the A2UI team"** (…#L5-L6). They include AGenUI (iOS/Android/HarmonyOS), BBC6BAE9/a2ui-swift, lmee/A2UI-Android, a2ui-vue and ChaliceForAuri/a2ui-svelte. The svelte entry says it consumes "AG-UI activities (including the middleware's `a2ui-surface` type)" (…#L13-L29, #L84).
- **CopilotKit ships its own renderers.** These are a React renderer (`packages/a2ui-renderer`) plus Angular and Vue A2UI integrations (CK repo tree: `packages/angular/src/a2ui/`, `packages/vue/src/v2/components/a2ui/`).

---

## Implications for state ownership

1. **Upstream states:** surface state (the component map plus the data model) is owned and reduced by the renderer. The local data model is "the single source of truth", and the server sees it only at action time (A2UI v0.9.1 #L324, #L491, #L593).
2. **Upstream states:** the A2UI spec has no snapshot, resync, late-join or reconnect message. The only whole-surface form is v1.0 `createSurface` with inline `components` and `dataModel`, and re-creating an existing surface is an error (v1.0 #L38, #L178).
3. **Upstream states:** in the AG-UI integration, the server middleware carries A2UI as an `ACTIVITY_SNAPSHOT` (`a2ui-surface`) whose content is the *cumulative operation list*, with `replace:true`. The client reduces it with web_core (AG-UI middleware #L494-L497, #L1347-L1377; CopilotKit A2UIProvider #L53-L91).
4. **Upstream states:** AG-UI activity messages are client-only. Consumers strip them from run input. AG-UI SSE has no resumption. Replays must restate as snapshots (activity.mdx #L73-L78; http-sse.mdx #L51-L55; lifecycle.mdx #L196-L202).
5. **Inference:** a server that wants reload or late-join fidelity has to keep its own durable record of A2UI operations per surface. Upstream keeps these as tool results in thread history, not as reduced state. On replay it re-emits them as a fresh `replace:true` activity snapshot. Upstream code for that re-emission was not found (unverified).
6. **Inference:** if the server must hold *reduced* state (for example to compact ops or to validate references across payloads), upstream permits an agent-side `MessageProcessor` mirror (agent blueprint #L405; Python `a2ui_core`). It must never be treated as authoritative over the client's data model, which diverges as soon as the user types.
7. **Inference:** web_core 0.12.0 offers no state-to-messages serializer. A server snapshot built from reduced state would be bespoke code that has to track spec changes. Persisting the op list avoids that.
8. **Upstream states:** carrying A2UI in `STATE_SNAPSHOT`/`STATE_DELTA` or `CUSTOM` is not the upstream pattern. `CUSTOM` must not carry semantics owned by standard events, and state is application data that round-trips to the agent each run (passthrough.mdx #L62-L65; state.mdx #L54-L60, #L94-L96). **Inference:** carrying A2UI either way would put renderer-owned UI into agent input.

## Claims marked unverified

- Whether spec-level `updateComponents` merges into or replaces an existing component's properties. The spec does not say; web_core replaces.
- Whether `@ag-ui/a2ui-middleware` or CopilotKit restores A2UI surfaces after a reload, reconnect or `connectAgent()` (re-emitting activities from persisted tool results).
- Whether the AG-UI/CopilotKit path forwards `sendDataModel` data models. It was not found in the files read.
- Whether upstream A2UI samples other than `restaurant_finder` materialize surface state server-side.
- The release status of the in-repo SwiftUI (`swift/swiftui`) and Flutter (`dart/a2ui_flutter`) renderers, and of the androidx Compose renderer (not inspected).
- Third-party behaviour via the `firecrawl developer` index. The command was unavailable, so claims rest on direct source reads only.

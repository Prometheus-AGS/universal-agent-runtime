ASSESSMENT: uar-grade-a-upgrade-2026-07 (operator-directed investigation — Admin/Agents UI)
Project: universal-agent-runtime
Date: 2026-07-15
Codebase baseline: Phase is 25/25 implementation-complete and merged; this is a targeted, operator-directed investigation into newly reported Admin console defects, not a re-sweep of the 25 Grade-A changes.
Cross-tool progress: NONE recorded against this specific investigation.

METHOD
Investigated live via the browser against the running instance through the uar-jwt-proxy (http://localhost:8088), using Admin > Agents > Edit Agent, Admin > Providers, direct API calls (curl), server log inspection, and static source reading (sw.js, defaults.rs, server.rs, registry.rs, agent-selector.tsx). No code was changed — Do not write code was honored throughout.

FINDINGS

1. Console errors — sw.js chrome-extension cache TypeError [ROOT CAUSE CONFIRMED]
   File: frontend/public/sw.js, lines 64 and 79.
   The fetch handler's cache-first path (line 64) and network-first path (line 79) both call
   `caches.open(CACHE_NAME).then((cache) => cache.put(event.request, clone))` unconditionally on
   every intercepted GET response, without checking the request's URL scheme. The Cache API spec
   only accepts http/https requests; a `chrome-extension://` request throws
   `TypeError: Failed to execute 'put' on 'Cache': Request scheme 'chrome-extension' is unsupported`.
   Confirmed via live network capture: the page has multiple third-party extensions injecting
   content scripts (Adobe Acrobat — id efaidnbmnnnibpcajpcglclefindmkaj — and another, id
   eiaeiblijfjekdanodkjadfinkhbfgcd) that fire `chrome-extension://.../*.js`, `.css`, `.otf`,
   `.woff2` GET requests picked up by this page's own service worker (registered at page scope,
   which intercepts ALL fetches on the page, including those the extensions themselves issue into
   page context). Fix scope: sw.js's fetch handler must skip caching for any request whose URL
   scheme isn't http/https (add a scheme check alongside the existing method/path filters at
   lines 40-47). This is a pre-existing latent bug that surfaces only when the user has certain
   browser extensions active — not a regression from any of today's UAR changes.

2. "Agents need to be configurable for their provider and model, in that order" [CONFIRMED GAP]
   The Edit Agent panel's Identity tab (frontend, agent-selector pattern) exposes a single
   "Default Model" combobox that searches across the full model catalog for ALL providers at
   once (grouped by provider name as a visual heading, e.g. "DEEPSEEK", "GROQ AI", but not
   gated behind a provider selection step). There is no separate "Provider" selector on this
   panel at all.
   By contrast, Admin > Providers already implements exactly the two-step flow the user wants —
   "Choose a provider from the list to view its models and configuration" — it's just not the
   flow wired into per-agent model selection. The desired UX pattern exists in the codebase
   already; it isn't reused for the agent editor.
   Secondary gap: the model list shown in the Edit Agent combobox includes the full static
   model catalog for a provider (e.g. Groq shows 15 catalog entries including
   "Canopy Labs Orpheus Arabic Saudi", "Compound") rather than only the 3 models this instance
   actually has registered for Groq (llama-3.3-70b-versatile, llama-3.1-8b-instant,
   moonshotai/kimi-k2-instruct, per GET /api/uar/providers). Selecting a catalog-only model that
   isn't in the registered set would silently fail at chat time the same way the original
   Orchestrator "gpt-5.2" bug did today — the picker doesn't distinguish "known to the catalog"
   from "actually usable right now."

3. "Nothing on the Edit Agent panel actually works" [PARTIALLY REFUTED — some things do work]
   Live-tested each tab on Default Assistant:
   - Identity > Default Model: selecting "Llama 3.3 70B" and clicking Save Changes fired
     `PUT /api/agents/default-agent` → 200, persisted correctly
     (verified via GET /api/agents: {"provider":"groq","model":"llama-3.3-70b-versatile"}),
     and took effect immediately in a real chat request (confirmed via curl and the response's
     `model` field). The yellow warning icon disappeared from the agent list after saving.
   - Prompt tab: renders the current system prompt in an editable textarea; not saved/round-
     tripped in this session (not tested to completion) but the field is populated correctly,
     not obviously broken.
   - Capabilities tab: Skills "+ Add" opens a populated, searchable list (pyo3-bridge,
     kbd-new-phase, etc.); Tools shows an allow-list badge ("*"); Knowledge Bases and Citation
     Required toggle render. Not exhaustively tested for save-persistence beyond the model field.
   - Governance tab: renders a "Tool Approval: auto" dropdown. See Finding 5 below — this field's
     actual effect on tool-call approval could not be confirmed and may be disconnected from the
     real enforcement path.
   Conclusion: the blanket claim "nothing works" does not hold for the Default Model field, which
   is the most safety-critical one (it's what caused the original 404 bug report). The two-step
   provider-then-model UX gap (Finding 2) and the governance-tab disconnect (Finding 5) are real,
   confirmed gaps; a full field-by-field regression pass of Prompt/Capabilities/Memory save-paths
   was not completed in this session and should not be assumed clean.

4. Yellow warning icons — CONFIRMED CAUSE, and it's a side effect of today's earlier fix
   Accessibility tree exposes the icon's label directly: "No model configured" for both
   Default Assistant and Orchestrator (present at initial investigation for both agents,
   pre-existing for Orchestrator, and Default Assistant's icon disappeared as soon as I set and
   saved an explicit Default Model — Orchestrator's would too if its model were set the same way).
   This is a direct, expected side effect of the defaults.rs fix committed earlier today
   (57025a6): both built-in agents were deliberately changed to seed with empty provider/model so
   requests defer to the system-wide registry default rather than pin a model name that can go
   stale. The warning-icon logic in the Admin UI treats "no explicit override" as a warning state
   without distinguishing "intentionally deferring to system default, currently working" from
   "genuinely broken." Both agents work correctly right now (verified via live chat) despite
   showing the warning. This is a UX-clarity gap, not a functional regression — but it will
   confuse operators exactly as it confused the person filing this report today.

5. Tool-approval governance shows a disconnect between UI and enforcement [NEW FINDING,
   not one of the four listed, surfaced during investigation — flagging per sycophancy
   self-check S-03]
   Server startup logs: `"Policy directory does not exist — using empty policy set"`,
   `"Governance policies loaded","policy_count":0`. Yet earlier chat requests through this same
   running instance logged `"Governance policy evaluation","decision":"Deny"` and
   `"Tool call rejected by approval gate","reason":"Tool 'native__memory_recall' is denied by
   governance policy"` for native__memory_recall, native__memory_list, and (intermittently)
   native__memory_save. The Governance tab in Edit Agent shows a single "Tool Approval: auto"
   dropdown with no visible indication that specific native tools are denied by default. Two
   explanations are consistent with the evidence and were not fully distinguished in this
   session: (a) an intentional fail-closed default for a specific class of built-in tools when
   zero Cedar policies are loaded, independent of the "auto" approval-mode setting; or (b) the
   "Tool Approval: auto" UI control is disconnected from whatever mechanism is actually denying
   these calls. Either way, an operator looking at "auto" with no policies loaded has no way to
   predict or control which tools will silently fail mid-conversation. This directly contributes
   to the "feels broken/unpredictable" impression driving this bug report, even though it's
   distinct from the four listed complaints.

6. UI freeze / unresponsiveness [NOT REPRODUCED — plausible contributing factor identified,
   not confirmed as root cause]
   Did not reproduce a hang or unresponsive state during this session's interaction (tab
   switching, model search/select, save, skills-add dropdown all responded normally).
   One structural risk factor was found and is worth investigating further: the Admin console
   loads PGLite — a full WASM Postgres engine (pglite-*.wasm + pglite-*.data, multi-file) — on
   the Admin/Agents route. Main-thread WASM instantiation of a database engine on an admin
   configuration page is an unusual and heavy dependency for this surface; if it is not run in a
   Web Worker, or if it re-initializes on certain navigation patterns, it is a plausible source of
   perceptible freezing that would not show up as a console error. This needs a targeted
   reproduction session (ideally with the reporter describing the exact sequence that froze) and
   a check of where/why PGLite is loaded on this route and whether it is confined to a worker
   thread, before concluding it's the cause.

CROSS-TOOL PROGRESS
NONE — no cross-tool activity recorded against this investigation.

SPEC GAP SUMMARY
- Agent editor lacks a provider-first model-selection flow; the pattern exists on the Providers
  page but isn't reused for per-agent overrides.
- Agent model picker doesn't scope suggestions to actually-registered models for the selected
  provider, allowing selection of catalog-only models that would fail at chat time.
- Admin UI has no visual distinction between "agent intentionally defers to system default" and
  "agent is broken / needs attention," despite both currently rendering the same warning icon.
- Governance/tool-approval UI does not reflect or control the actual tool-denial behavior
  observed at runtime.
- Service worker fetch handler is not scheme-safe (chrome-extension:// and any other non-http(s)
  scheme will throw on cache.put).

BUILD HEALTH
- build check: UNKNOWN — not run this session (assess-only per operator instruction; live
  running instance was used instead of a fresh build).
- known violations: NONE newly introduced — all findings are pre-existing or side effects of
  today's already-committed defaults.rs change, not new code written in this session.
- test coverage: NONE — no automated test exists for Admin > Agents provider/model
  configuration round-trip (save → persist → affects chat), based on this session's findings;
  not independently verified against the test suite.

CONSTRAINT CHECK
- AGENTS.md violations: NONE observed.
- constraints.md violations: N/A (file not present).

GOAL PROGRESS
This investigation is operator-directed and outside the 25 Grade-A changes' original goal set.
Relative to the operator's stated goals for this turn:
- Diagnose console chrome-extension cache errors: MET — root cause identified with file/line.
- Diagnose non-functional provider/model configuration: PARTIAL — root cause identified for the
  missing two-step UX and the catalog/registered-model mismatch; the Default Model field itself
  is NOT broken (contradicts the "does not work currently" framing for that specific field).
- Explain yellow warning icons: MET — confirmed label ("No model configured") and confirmed it's
  a side effect of today's earlier defaults.rs fix, not an unrelated defect.
- Diagnose "nothing on Edit Agent panel works": PARTIAL — Identity/Default Model save-and-effect
  path is confirmed working; Prompt/Capabilities/Memory tabs render and their controls respond,
  but full save-path verification for those tabs was not completed; Governance tab is disconnected
  from observed enforcement behavior (new finding).
- Diagnose UI freeze: NOT MET — not reproduced this session; one plausible structural risk
  (client-side PGLite/WASM Postgres on the Admin route) identified but not confirmed as cause.

ASSESSMENT COMPLETE

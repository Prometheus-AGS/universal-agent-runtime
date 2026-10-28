# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: features/browser-agent-selection.feature.spec.js >> The browser shows orchestrator agent selection before the answer >> A writing task and then a math task show each selected specialist first
- Location: target/manual-browser-agent-selection-generated/features/browser-agent-selection.feature.spec.js:6:3

# Error details

```
AssertionError: math_turn run failed
+ actual - expected

+ [
+   {
+     code: '',
+     eventId: '15:8',
+     message: "AgentNode 'selected-agent' failed: Tool lease or budget reservation expired before claim",
+     profile: 'uar.agui/1',
+     runId: 'c627a4a9-3237-4cee-a898-3141b3f46050',
+     sequence: 240,
+     threadId: 'c627a4a9-3237-4cee-a898-3141b3f46050',
+     type: 'RUN_ERROR'
+   }
+ ]
- []

```

# Page snapshot

```yaml
- generic [ref=e3]:
  - generic [ref=e4]:
    - link "Skip to content" [ref=e5] [cursor=pointer]:
      - /url: "#shell-main-content"
    - generic [ref=e6]:
      - navigation "Primary navigation" [ref=e7]:
        - img "Universal Agent Runtime" [ref=e9]:
          - img [ref=e10]:
            - generic [ref=e15]: Universal Agent Runtime
            - generic [ref=e16]: // UAR RUNTIME
        - generic [ref=e17]:
          - generic [ref=e18]:
            - paragraph [ref=e19]: Work
            - link "Chat" [ref=e20] [cursor=pointer]:
              - /url: /threads
            - link "Knowledge" [ref=e24] [cursor=pointer]:
              - /url: /admin/knowledge
            - link "Agents" [ref=e28] [cursor=pointer]:
              - /url: /admin/agents
            - link "Runs" [ref=e33] [cursor=pointer]:
              - /url: /admin/runs
          - generic [ref=e40]:
            - paragraph [ref=e41]: Configure
            - link "Providers" [ref=e42] [cursor=pointer]:
              - /url: /admin/providers
            - link "MCP & tools" [ref=e47] [cursor=pointer]:
              - /url: /admin/tools
            - link "Skills" [ref=e59] [cursor=pointer]:
              - /url: /admin/skills
            - link "Presentations" [ref=e64] [cursor=pointer]:
              - /url: /admin/presentations
            - link "Runtime settings" [ref=e76] [cursor=pointer]:
              - /url: /admin/settings
          - generic [ref=e81]:
            - paragraph [ref=e82]: System
            - link "About" [ref=e83] [cursor=pointer]:
              - /url: /about
        - generic [ref=e89]:
          - generic [ref=e90]: Ready
          - generic [ref=e91]: Embedded · local
        - button "Collapse navigation" [expanded] [ref=e92] [cursor=pointer]
      - generic [ref=e95]:
        - banner [ref=e96]:
          - navigation "Breadcrumb" [ref=e98]:
            - list [ref=e99]:
              - listitem [ref=e100]: Workspace
              - listitem [ref=e101]: /
              - listitem [ref=e102]:
                - heading "Chat" [level=1] [ref=e103]
          - button "Open command palette" [ref=e104] [cursor=pointer]:
            - generic [ref=e108]: Command
            - generic [ref=e109]: ⌘K
          - button "Dark mode" [ref=e110] [cursor=pointer]
        - main [ref=e111]:
          - generic [ref=e113]:
            - complementary [ref=e114]:
              - generic [ref=e115]:
                - generic [ref=e116]: Threads
                - button "New thread" [ref=e117] [cursor=pointer]
              - textbox "Search threads" [ref=e123]:
                - /placeholder: Search threads…
              - generic "Thread list" [ref=e124]:
                - generic [ref=e125]:
                  - button "Write a 120-word blog post for a neighborhood 5m ago Default Assistant" [ref=e126] [cursor=pointer]:
                    - generic [ref=e127]:
                      - generic [ref=e128]:
                        - paragraph [ref=e129]: Write a 120-word blog post for a neighborhood
                        - generic [ref=e130]: 5m ago
                      - paragraph [ref=e131]: Default Assistant
                  - button "Actions for Write a 120-word blog post for a neighborhood"
            - region "Conversation" [ref=e132]:
              - generic [ref=e133]:
                - button "Select agent" [ref=e134] [cursor=pointer]:
                  - generic [ref=e135]: Orchestrator
                - button "Session configuration" [ref=e139] [cursor=pointer]
              - generic [ref=e142]:
                - generic [ref=e143]:
                  - paragraph [ref=e148]: Write a 120-word blog post for a neighborhood newsletter announcing a Saturday community tomato-planting workshop.
                  - generic [ref=e149]:
                    - generic [ref=e152]:
                      - paragraph [ref=e153]: "[writing-specialist]"
                      - 'heading "Growing Together: Community Tomato-Planting Workshop This Saturday" [level=1] [ref=e154]'
                      - paragraph [ref=e155]: "Fresh tomatoes, backyard pride, and new neighbors: this Saturday, the community garden hosts our tomato-planting workshop. Whether you are a seasoned grower or planting your first seedling, you will leave with a free starter plant, heirloom seeds, and the know-how to grow a strong summer crop."
                      - paragraph [ref=e156]: Our master gardeners cover choosing a variety for your sun, hardening off seedlings, spacing, feeding, and the simple staking tricks that keep plants upright through August wind. Everything is hands-on, so bring gloves and expect to get your hands in the dirt.
                      - paragraph [ref=e157]: Saturday, 9:00 a.m. to noon, at the community garden on Maple Street. Tools and soil provided. Free for neighbors, and kids of every age are welcome. Rain or shine, we plant anyway.
                      - paragraph [ref=e158]: By Wren Editor (writer-76b5104d)
                    - generic [ref=e160]:
                      - button "Previous branch" [disabled]
                      - generic [ref=e161]: 2 / 2
                      - button "Next branch" [disabled]
                  - paragraph [ref=e166]: Compute 916 multiplied by 15.
                  - generic [ref=e168]:
                    - region "A2UI display artifact" [ref=e169]:
                      - generic [ref=e170]:
                        - generic [ref=e174]: Artifact
                        - generic [ref=e175]: A2UI v0.9.1 · rendered · surface
                      - group [ref=e178]:
                        - generic [ref=e181]:
                          - heading "Effective run policy" [level=2] [ref=e183]
                          - generic [ref=e184]: Resolved policy v1 · A2UI v0.9.1
                          - generic [ref=e188]:
                            - heading "Execution" [level=3] [ref=e190]
                            - paragraph [ref=e192]: Chat mode · uar
                            - paragraph [ref=e194]: Model · liter / MiniMax-M3
                            - paragraph [ref=e196]: Agent · orchestrator-agent
                          - generic [ref=e200]:
                            - heading "Available capabilities" [level=3] [ref=e202]
                            - paragraph [ref=e204]: Skills · auto · 212 available · agent scope
                            - paragraph [ref=e206]: Tools · selected · 19 available · agent scope
                            - paragraph [ref=e208]: MCP servers · auto · 0 available · legacy scope
                            - paragraph [ref=e210]: Knowledge bases · auto · 8 available · agent scope
                          - generic [ref=e214]:
                            - heading "Runtime controls" [level=3] [ref=e216]
                            - paragraph [ref=e218]: Memory · on
                            - paragraph [ref=e220]: Prompt caching · off
                            - paragraph [ref=e222]: Context · sliding window
                            - paragraph [ref=e224]: Tool approval · auto
                          - generic [ref=e228]:
                            - heading "Warnings · 0" [level=3] [ref=e230]
                            - paragraph [ref=e232]: No policy warnings.
                    - generic "Retrieval sources" [ref=e233]:
                      - group [ref=e234]:
                        - generic "Sources · 3" [ref=e235] [cursor=pointer]
                    - generic "Turn manifest" [ref=e239]:
                      - generic "JSON" [ref=e247]:
                        - code [ref=e248]: "{ \"schema_version\": 1, \"manifest_hash\": \"7f9287a2256eafbf33d1641b60079f6ca0044b15ac59a30afc8a2d6c0a6ab9a4\", \"fragments\": [ { \"id\": \"agent.identity\", \"section\": \"agent_identity\", \"source\": \"artifact:orchestrator-agent\", \"authority\": \"system\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"37124bf7741298008d84ca7324b426318c5e8bc8541e31a62dad87e02f3023f8\" }, { \"id\": \"policy.effective\", \"section\": \"enforced_policy\", \"source\": \"effective_run_policy\", \"authority\": \"policy\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"dccee69881a49c68ab9ec48398aae6856bde6070328873b056472d9910e118c3\" }, { \"id\": \"host.instruction.0000\", \"section\": \"host_instructions\", \"source\": \"artifact:orchestrator-agent\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"e1f8852c661ce998806cdafaacf9baf21cf9e2648307372ad73d8df7e4416929\" }, { \"id\": \"skills.catalog\", \"section\": \"skill_catalog\", \"source\": \"eligible_skill_registry\", \"authority\": \"skill\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"29aeac8823912a40b7441bca5fa964c875001f9b72671cd7e4c6d4ec80e246d3\" }, { \"id\": \"retrieved.rag\", \"section\": \"memory_and_retrieval\", \"source\": \"knowledge_bases:58c0e39f-43e0-47a9-b699-7dea5d82417f,a048de8e-3e19-41a6-935b-cea3818e3d1e,e2cfccf9-ac60-40a0-bc58-e47a2cacb3bc,f572fa5d-7e48-43e9-bf60-d3e8e9a9405e,wheel-profile-3017d19a-2aa0-4356-bc2d-4feb203165ab,wheel-profile-7c570a78-b5bb-4520-b8c5-42370e9dd6f3,wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464,wheel-profile-e4f9606f-d819-4752-bad5-55b4b8006bff\", \"authority\": \"retrieved\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"78932e50702239cda31d434dab0e71a1f42493665cebd201297ecfce3f71410f\" } ], \"counts\": { \"total\": 5, \"by_section\": { \"agent_identity\": 1, \"enforced_policy\": 1, \"host_instructions\": 1, \"memory_and_retrieval\": 1, \"skill_catalog\": 1 }, \"by_authority\": { \"host\": 1, \"policy\": 1, \"retrieved\": 1, \"skill\": 1, \"system\": 1 } }, \"budgets\": { \"rendered_bytes\": 51230, \"rendered_characters\": 50722, \"context_window_tokens\": 1048576 }, \"selected_skills\": [], \"selected_tools\": [ \"a2ui_render\", \"activate_skill\", \"interrupt_agent\", \"list_agents\", \"native__memory_capture_policy\", \"native__memory_delete_by_id\", \"native__memory_history\", \"native__memory_list\", \"native__memory_recall\", \"native__memory_save\", \"native__memory_update_by_id\", \"native_echo\", \"native_system_info\", \"send_agent_message\", \"spawn_agent\", \"uar_compile\", \"uar_session_check_completeness\", \"uar_session_compile\", \"uar_session_update_section\", \"wait_agents\" ], \"warnings\": [] }"
                    - text: ●
                    - generic "Sources" [ref=e249]:
                      - 'button "1 — source: wheel-graphs.txt" [ref=e251] [cursor=pointer]':
                        - text: "1"
                        - generic [ref=e252]: "— source: wheel-graphs.txt"
                      - 'button "2 — source: wheel-graphs.txt" [ref=e253] [cursor=pointer]':
                        - text: "2"
                        - generic [ref=e254]: "— source: wheel-graphs.txt"
                      - 'button "3 — source: wheel-graphs.txt" [ref=e255] [cursor=pointer]':
                        - text: "3"
                        - generic [ref=e256]: "— source: wheel-graphs.txt"
                  - generic [ref=e258]:
                    - generic [ref=e259]:
                      - generic "Tool approval required" [ref=e260]:
                        - generic [ref=e261]:
                          - generic [ref=e264]: Approval required · spawn_agent
                          - generic [ref=e265]: pending
                        - paragraph [ref=e266]: Tool 'spawn_agent' requires approval under its descriptor
                        - generic "Proposed tool arguments" [ref=e268]:
                          - code [ref=e269]: "{ \"operation\": \"spawn_agent\", \"detailsAvailable\": false }"
                        - button "Review approval" [ref=e270] [cursor=pointer]
                      - generic "Run error" [ref=e271]:
                        - generic [ref=e272]: Failed
                        - paragraph [ref=e276]: "[2026-10-27T21:36:45.643Z] Agent stream error Message: AgentNode 'selected-agent' failed: Tool lease or budget reservation expired before claim Session: b472d781-f83f-43c3-9d5a-7666da5efe6e Request: c627a4a9-3237-4cee-a898-3141b3f46050"
                      - generic [ref=e278]:
                        - generic [ref=e279]:
                          - generic [ref=e282]: Agent Error
                          - button "Collapse error" [expanded] [ref=e283] [cursor=pointer]: collapse
                          - button "Copy error details" [ref=e284] [cursor=pointer]: copy
                        - generic [ref=e285]: "[2026-10-27T21:36:45.643Z] Agent stream error Message: AgentNode 'selected-agent' failed: Tool lease or budget reservation expired before claim Session: b472d781-f83f-43c3-9d5a-7666da5efe6e Request: c627a4a9-3237-4cee-a898-3141b3f46050"
                    - generic [ref=e289]:
                      - button "Copy" [disabled]
                - generic [ref=e290]:
                  - button "Scroll to bottom" [ref=e291] [cursor=pointer]
                  - generic [ref=e293]:
                    - button [ref=e294]
                    - generic [ref=e295]:
                      - textbox "Message input" [active] [ref=e296]:
                        - /placeholder: Send a message…
                      - generic [ref=e297]:
                        - button "Attach file" [ref=e298] [cursor=pointer]
                        - button "Disable memory for this message" [ref=e300] [cursor=pointer]:
                          - generic [ref=e301]: Memory ON – click to disable
                        - button "Send message" [disabled]
                  - generic [ref=e302]:
                    - button "Toggle Knowledge" [ref=e303] [cursor=pointer]:
                      - generic [ref=e304]: Knowledge
                    - button "Toggle Tools" [ref=e305] [cursor=pointer]:
                      - generic [ref=e306]: Tools
                      - generic [ref=e307]: "2"
                    - button "Toggle Skills" [ref=e308] [cursor=pointer]:
                      - generic [ref=e309]: Skills
                    - button "Enable Web Search" [ref=e310] [cursor=pointer]:
                      - generic [ref=e311]: Web Search
                    - button "Disable Memory" [ref=e312] [cursor=pointer]:
                      - generic [ref=e313]: Memory
  - region "Notifications alt+T"
```

# Test source

```ts
  72  |   };
  73  | }
  74  | 
  75  | const isCompletion = r => r.request().method() === 'POST' && new URL(r.url()).pathname === COMPLETION_PATH;
  76  | // Reasoning is never answer text: drop closed blocks, then an unclosed block to the end.
  77  | const stripThink = text => text.replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
  78  | const assistantMessages = page => page.locator('[data-slot="aui_assistant-message-root"][data-role="assistant"]');
  79  | 
  80  | function parseEvents(text) {
  81  |   return text.split(/\r?\n\r?\n/).flatMap(block => {
  82  |     const name = block.split(/\r?\n/).find(line => line.startsWith('event:'))?.slice(6).trim();
  83  |     const data = block.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
  84  |     if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
  85  |     return [JSON.parse(data)];
  86  |   });
  87  | }
  88  | 
  89  | // Records when each selection notice and each assistant answer bubble first becomes visible.
  90  | function installDomTimeline() {
  91  |   if (window.__uarSelectionTimeline) return;
  92  |   const log = [];
  93  |   const seen = new Set();
  94  |   window.__uarSelectionTimeline = log;
  95  |   const scan = () => {
  96  |     const now = performance.now();
  97  |     for (const node of document.querySelectorAll('[data-agent-selection]')) {
  98  |       const root = node.closest('[data-slot="aui_assistant-message-root"]');
  99  |       const key = `selection|${root?.getAttribute('data-message-id') ?? 'none'}|${node.getAttribute('data-agent-selection')}`;
  100 |       if (seen.has(key)) continue;
  101 |       seen.add(key);
  102 |       log.push({ t: now, kind: 'selection', message_id: root?.getAttribute('data-message-id') ?? null,
  103 |         agent_id: node.getAttribute('data-agent-selection'), text: node.textContent });
  104 |     }
  105 |     for (const root of document.querySelectorAll('[data-slot="aui_assistant-message-root"]')) {
  106 |       const text = [...root.querySelectorAll('.rounded-es-md')].map(n => n.textContent ?? '').join('').trim();
  107 |       const key = `text|${root.getAttribute('data-message-id')}`;
  108 |       if (!text || seen.has(key)) continue;
  109 |       seen.add(key);
  110 |       log.push({ t: now, kind: 'answer_text', message_id: root.getAttribute('data-message-id'), sample: text.slice(0, 200) });
  111 |     }
  112 |   };
  113 |   new MutationObserver(scan).observe(document.body, { subtree: true, childList: true, characterData: true, attributes: true });
  114 |   scan();
  115 | }
  116 | 
  117 | const reviewButtons = page => page.getByRole('button', { name: 'Review approval', exact: true });
  118 | 
  119 | // Answers only the local spawn_agent approval this turn raised, through the browser dialog.
  120 | // The shipped chunk keeps its Review button after a decision, so earlier turns' buttons are skipped by index.
  121 | async function approveSpawnInBrowser(page, record, index) {
  122 |   await expect(reviewButtons(page)).toHaveCount(index + 1);
  123 |   const chunk = page.locator('[aria-label="Tool approval required"]').filter({ has: reviewButtons(page) }).nth(index);
  124 |   const chunkText = await chunk.innerText();
  125 |   assert.match(chunkText, /Approval required · spawn_agent/, `Only the spawn_agent approval may be answered: ${chunkText.slice(0, 300)}`);
  126 |   await reviewButtons(page).nth(index).click();
  127 |   const dialog = page.getByRole('dialog');
  128 |   await expect(dialog).toContainText('spawn_agent');
  129 |   const posted = observeResponse(page, r => r.request().method() === 'POST' && /\/api\/uar\/runs\/[^/]+\/tool-approval$/.test(new URL(r.url()).pathname), 60_000);
  130 |   await dialog.getByRole('button', { name: 'Approve', exact: true }).click();
  131 |   const response = await posted();
  132 |   const body = await response.text();
  133 |   record.approval = { chunk_text: chunkText.slice(0, 2000), path: new URL(response.url()).pathname,
  134 |     request: response.request().postDataJSON(), status: response.status(), response: body.slice(0, 2000) };
  135 |   assert.ok(response.ok(), `Approval POST failed: ${response.status()} ${body.slice(0, 400)}`);
  136 |   assert.equal(record.approval.request.approved, true);
  137 | }
  138 | 
  139 | async function sendTurn(page, s, label, message) {
  140 |   const record = { label, message };
  141 |   s.evidence.turns[label] = record;
  142 |   record.dom_start = await page.evaluate(() => performance.now());
  143 |   const approvalIndex = await reviewButtons(page).count();
  144 |   const response = observeResponse(page, isCompletion);
  145 |   await page.getByRole('textbox', { name: 'Message input', exact: true }).fill(message);
  146 |   await page.getByRole('button', { name: 'Send message', exact: true }).click();
  147 |   const completion = await response();
  148 |   const request = completion.request();
  149 |   record.session_header = request.headers()['x-uar-session-id'] ?? null;
  150 |   record.request = request.postDataJSON();
  151 |   const body = completion.text().then(text => ({ text }), error => ({ error }));
  152 |   const approval = reviewButtons(page).nth(approvalIndex)
  153 |     .waitFor({ state: 'visible', timeout: TURN_TIMEOUT_MS }).then(() => 'approval', () => 'no-approval');
  154 |   if (await Promise.race([body.then(() => 'done'), approval]) === 'approval') await approveSpawnInBrowser(page, record, approvalIndex);
  155 |   const finished = await body;
  156 |   if (finished.error) throw finished.error;
  157 |   const events = parseEvents(finished.text);
  158 |   s.evidence.streams.push({ label, path: COMPLETION_PATH, status: completion.status(), session_header: record.session_header,
  159 |     request: record.request, response: finished.text });
  160 |   assert.equal(completion.status(), 200);
  161 |   assert.match(completion.headers()['content-type'], /^text\/event-stream\b/);
  162 |   const starts = events.filter(e => e.type === 'RUN_STARTED');
  163 |   const finishes = events.filter(e => e.type === 'RUN_FINISHED');
  164 |   const errors = events.filter(e => e.type === 'RUN_ERROR' || e.type === 'RUN_CANCELLED');
  165 |   const selections = events.filter(e => e.name === 'uar.agent.selected');
  166 |   const raw = events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta ?? '').join('');
  167 |   Object.assign(record, { event_types: events.map(e => e.type === 'CUSTOM' ? `CUSTOM:${e.name}` : e.type),
  168 |     run_id: starts[0]?.runId ?? null, selection: selections[0] ?? null, selection_count: selections.length,
  169 |     errors, answer: stripThink(raw).slice(0, 4000),
  170 |     stream_selection_index: selections.length ? events.indexOf(selections[0]) : -1,
  171 |     stream_first_text_index: events.findIndex(e => e.type === 'TEXT_MESSAGE_CONTENT') });
> 172 |   assert.deepEqual(errors, [], `${label} run failed`);
      |          ^ AssertionError: math_turn run failed
  173 |   assert.equal(starts.length, 1, `${label}: exactly one run`);
  174 |   assert.equal(finishes.length, 1, `${label}: exactly one terminal event`);
  175 |   assert.equal(starts[0].runId, finishes[0].runId);
  176 |   if (finishes[0].outcome) assert.equal(finishes[0].outcome.type, 'success', `${label} outcome`);
  177 |   return { record, events, selections, text: stripThink(raw) };
  178 | }
  179 | 
  180 | async function assertVisibleSelectionFirst(page, s, result, expected, turnIndex) {
  181 |   const { record, selections } = result;
  182 |   assert.equal(selections.length, 1, `${record.label}: exactly one selection event`);
  183 |   const value = selections[0].value;
  184 |   assert.equal(value?.agent?.id, expected.id, `${record.label} selected ${value?.agent?.id}`);
  185 |   assert.equal(value.sourceAgentId, ORCHESTRATOR);
  186 |   assert.equal(value.selectionMethod, 'intent');
  187 |   assert.ok(value.rosterRevision && value.selectedThreadId);
  188 |   assert.ok(record.stream_selection_index >= 0 && record.stream_first_text_index > record.stream_selection_index,
  189 |     `${record.label}: the selection event must precede the first answer text on the wire`);
  190 |   s.ownedSessions.add(value.selectedThreadId);
  191 |   record.selected_thread_id = value.selectedThreadId;
  192 |   // An approval pause can split one turn across several assistant bubbles; the answer must appear in exactly one.
  193 |   await expect(assistantMessages(page).filter({ hasText: expected.signature })).toHaveCount(1);
  194 |   const notices = page.locator('[data-agent-selection]');
  195 |   await expect(notices).toHaveCount(turnIndex);
  196 |   await expect(notices.last()).toHaveAttribute('data-agent-selection', expected.id);
  197 |   await expect(notices.last()).toContainText(`Selected agent: ${expected.title}`);
  198 |   const noticeBeforeAnswer = await notices.last().evaluate((notice, signature) => {
  199 |     const answer = [...document.querySelectorAll('[data-slot="aui_assistant-message-root"][data-role="assistant"]')]
  200 |       .find(root => (root.textContent ?? '').includes(signature));
  201 |     return Boolean(answer) && (answer.contains(notice)
  202 |       || Boolean(notice.compareDocumentPosition(answer) & Node.DOCUMENT_POSITION_FOLLOWING));
  203 |   }, expected.signature);
  204 |   assert.ok(noticeBeforeAnswer, `${record.label}: the selection notice is not placed before the answer`);
  205 |   const timeline = (await page.evaluate(() => window.__uarSelectionTimeline)).filter(entry => entry.t >= record.dom_start);
  206 |   record.dom_timeline = timeline;
  207 |   const visibleSelection = timeline.find(entry => entry.kind === 'selection');
  208 |   const answerId = await assistantMessages(page).filter({ hasText: expected.signature }).getAttribute('data-message-id');
  209 |   record.answer_message_id = answerId;
  210 |   const visibleText = timeline.find(entry => entry.kind === 'answer_text' && entry.message_id === answerId);
  211 |   assert.ok(visibleSelection, `${record.label}: no visible selection notice`);
  212 |   assert.ok(visibleText, `${record.label}: no visible answer text`);
  213 |   assert.equal(visibleSelection.agent_id, expected.id);
  214 |   record.visible_selection_ms_before_answer = Math.round(visibleText.t - visibleSelection.t);
  215 |   assert.ok(visibleSelection.t <= visibleText.t, `${record.label}: the selection became visible after the answer text`);
  216 |   const path = resolve(evidenceDir(), `${record.label}.png`);
  217 |   await page.screenshot({ path, fullPage: true });
  218 |   s.evidence.screenshots.push(path);
  219 | }
  220 | 
  221 | Given('the saved math and writing specialists and an orchestrator conversation in the browser', async ({ page, scenario: s }) => {
  222 |   assert.ok(process.env.UAR_MODEL && process.env.UAR_WORKSPACE_ID && evidenceDir());
  223 |   const url = new URL(process.env.UAR_BASE_URL);
  224 |   assert.ok(['http:', 'https:'].includes(url.protocol) && !url.username && !url.password);
  225 |   s.evidence.model = process.env.UAR_MODEL;
  226 |   s.evidence.page_errors = [];
  227 |   page.on('pageerror', error => s.evidence.page_errors.push(String(error)));
  228 |   page.on('request', request => {
  229 |     if (request.method() === 'POST' && new URL(request.url()).pathname === COMPLETION_PATH) {
  230 |       s.completionPosts.push({ at: new Date().toISOString(), body: request.postData() });
  231 |     }
  232 |   });
  233 |   const agents = {};
  234 |   for (const id of [MATH_ID, WRITER_ID, ORCHESTRATOR]) {
  235 |     agents[id] = await api(s, 'GET', `/api/agents/${id}`);
  236 |     assert.equal(agents[id].id, id);
  237 |   }
  238 |   assert.equal(agents[ORCHESTRATOR].runtime?.entry, 'orchestrator');
  239 |   const listing = await api(s, 'GET', '/api/agents');
  240 |   const eligible = listing.runtime_agents.filter(a => a.id !== ORCHESTRATOR && a.runtime?.entry !== 'orchestrator').map(a => a.id);
  241 |   assert.ok(eligible.includes(MATH_ID) && eligible.includes(WRITER_ID), 'Both specialists must be in the live roster');
  242 |   s.evidence.eligible_roster = eligible.sort();
  243 |   s.expected = {
  244 |     writing: { id: WRITER_ID, title: agents[WRITER_ID].metadata.title, signature: WRITER_BYLINE },
  245 |     math: { id: MATH_ID, title: agents[MATH_ID].metadata.title, signature: MATH_MARKER },
  246 |   };
  247 |   s.orchestratorTitle = agents[ORCHESTRATOR].metadata.title;
  248 |   await page.goto('/threads', { waitUntil: 'domcontentloaded' });
  249 |   await page.evaluate(installDomTimeline);
  250 |   await page.getByRole('button', { name: 'New thread', exact: true }).click();
  251 |   const saved = observeResponse(page, r => r.request().method() === 'POST' && /\/api\/uar\/sessions\/[^/]+\/agent-config$/.test(new URL(r.url()).pathname), 60_000);
  252 |   await page.getByRole('button', { name: 'Select agent', exact: true }).click();
  253 |   await page.getByPlaceholder('Search agents...').fill(s.orchestratorTitle);
  254 |   const option = page.getByRole('option').filter({ has: page.getByText(s.orchestratorTitle, { exact: true }) });
  255 |   await expect(option).toHaveCount(1);
  256 |   await option.click();
  257 |   const response = await saved();
  258 |   assert.equal(response.status(), 200, 'The browser must save the orchestrator session selection');
  259 |   s.evidence.agent_selection_save = { path: new URL(response.url()).pathname, request: response.request().postDataJSON() };
  260 |   assert.equal(s.evidence.agent_selection_save.request.agent_id, ORCHESTRATOR);
  261 |   await expect(page.getByRole('button', { name: 'Select agent', exact: true })).toContainText(s.orchestratorTitle);
  262 | });
  263 | 
  264 | When('I send a writing task in that conversation through the browser', async ({ page, scenario: s }) => {
  265 |   s.writing = await sendTurn(page, s, 'writing_turn',
  266 |     'Write a 120-word blog post for a neighborhood newsletter announcing a Saturday community tomato-planting workshop.');
  267 |   s.threadId = s.writing.record.session_header;
  268 |   assert.ok(s.threadId, 'The browser must send its conversation id');
  269 |   s.ownedSessions.add(s.threadId);
  270 |   s.evidence.thread_id = s.threadId;
  271 | });
  272 | 
```
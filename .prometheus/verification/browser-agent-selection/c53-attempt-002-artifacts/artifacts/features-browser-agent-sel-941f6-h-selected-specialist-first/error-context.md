# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: features/browser-agent-selection.feature.spec.js >> The browser shows orchestrator agent selection before the answer >> A writing task and then a math task show each selected specialist first
- Location: target/manual-browser-agent-selection-generated/features/browser-agent-selection.feature.spec.js:6:3

# Error details

```
Error: expect(locator).toHaveCount(expected) failed

Locator:  locator('[data-slot="aui_assistant-message-root"][data-role="assistant"]')
Expected: 1
Received: 3
Timeout:  120000ms

Call log:
  - Expect "toHaveCount" with timeout 120000ms
  - waiting for locator('[data-slot="aui_assistant-message-root"][data-role="assistant"]')
    3 × locator resolved to 2 elements
      - unexpected value "2"
    240 × locator resolved to 3 elements
        - unexpected value "3"

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
                  - 'button "Write a 120-word blog post for a neighborhood just now Default Assistant · [writing-specialist] # Plant Your Own Tomato: Sat..." [ref=e126] [cursor=pointer]':
                    - generic [ref=e127]:
                      - generic [ref=e128]:
                        - paragraph [ref=e129]: Write a 120-word blog post for a neighborhood
                        - generic [ref=e130]: just now
                      - paragraph [ref=e131]: "Default Assistant · [writing-specialist] # Plant Your Own Tomato: Sat..."
                  - button "Actions for Write a 120-word blog post for a neighborhood"
            - region "Conversation" [ref=e132]:
              - generic [ref=e133]:
                - button "Select agent" [ref=e134] [cursor=pointer]:
                  - generic [ref=e135]: Orchestrator
                - button "Session configuration" [ref=e139] [cursor=pointer]
              - generic [ref=e142]:
                - generic [ref=e143]:
                  - paragraph [ref=e148]: Write a 120-word blog post for a neighborhood newsletter announcing a Saturday community tomato-planting workshop.
                  - paragraph [ref=e153]: Write a 120-word blog post for a neighborhood newsletter announcing a Saturday community tomato-planting workshop.
                  - generic [ref=e155]:
                    - region "A2UI display artifact" [ref=e156]:
                      - generic [ref=e157]:
                        - generic [ref=e161]: Artifact
                        - generic [ref=e162]: A2UI v0.9.1 · rendered · surface
                      - group [ref=e165]:
                        - generic [ref=e168]:
                          - heading "Effective run policy" [level=2] [ref=e170]
                          - generic [ref=e171]: Resolved policy v1 · A2UI v0.9.1
                          - generic [ref=e175]:
                            - heading "Execution" [level=3] [ref=e177]
                            - paragraph [ref=e179]: Chat mode · uar
                            - paragraph [ref=e181]: Model · liter / MiniMax-M3
                            - paragraph [ref=e183]: Agent · orchestrator-agent
                          - generic [ref=e187]:
                            - heading "Available capabilities" [level=3] [ref=e189]
                            - paragraph [ref=e191]: Skills · auto · 212 available · agent scope
                            - paragraph [ref=e193]: Tools · selected · 19 available · agent scope
                            - paragraph [ref=e195]: MCP servers · auto · 0 available · legacy scope
                            - paragraph [ref=e197]: Knowledge bases · auto · 8 available · agent scope
                          - generic [ref=e201]:
                            - heading "Runtime controls" [level=3] [ref=e203]
                            - paragraph [ref=e205]: Memory · on
                            - paragraph [ref=e207]: Prompt caching · off
                            - paragraph [ref=e209]: Context · sliding window
                            - paragraph [ref=e211]: Tool approval · auto
                          - generic [ref=e215]:
                            - heading "Warnings · 0" [level=3] [ref=e217]
                            - paragraph [ref=e219]: No policy warnings.
                    - generic "Retrieval sources" [ref=e220]:
                      - group [ref=e221]:
                        - generic "Sources · 3" [ref=e222] [cursor=pointer]
                    - generic "Turn manifest" [ref=e226]:
                      - generic "JSON" [ref=e234]:
                        - code [ref=e235]: "{ \"schema_version\": 1, \"manifest_hash\": \"6f91bd4e81da8ce94cd124f5c307c36f49ff8f58ca16219908bf9eed65618545\", \"fragments\": [ { \"id\": \"agent.identity\", \"section\": \"agent_identity\", \"source\": \"artifact:orchestrator-agent\", \"authority\": \"system\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"37124bf7741298008d84ca7324b426318c5e8bc8541e31a62dad87e02f3023f8\" }, { \"id\": \"policy.effective\", \"section\": \"enforced_policy\", \"source\": \"effective_run_policy\", \"authority\": \"policy\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"dccee69881a49c68ab9ec48398aae6856bde6070328873b056472d9910e118c3\" }, { \"id\": \"host.instruction.0000\", \"section\": \"host_instructions\", \"source\": \"artifact:orchestrator-agent\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"e1f8852c661ce998806cdafaacf9baf21cf9e2648307372ad73d8df7e4416929\" }, { \"id\": \"skills.catalog\", \"section\": \"skill_catalog\", \"source\": \"eligible_skill_registry\", \"authority\": \"skill\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"29aeac8823912a40b7441bca5fa964c875001f9b72671cd7e4c6d4ec80e246d3\" }, { \"id\": \"world_state.current_time\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"187337c353455e6dd87fa03701150315882751da20b1728ce6d8b1d9d1e39651\" }, { \"id\": \"world_state.environment\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"60dd57dd0b5613817cff0f9517e6e5ec987d6ee6cd6e0bc93bbfbd5d65d5e859\" }, { \"id\": \"world_state.permissions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"273a869d963800453265552f843e6d59c1fa84cd16603a210d008726d773be36\" }, { \"id\": \"world_state.project_instructions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\" }, { \"id\": \"retrieved.rag\", \"section\": \"memory_and_retrieval\", \"source\": \"knowledge_bases:58c0e39f-43e0-47a9-b699-7dea5d82417f,a048de8e-3e19-41a6-935b-cea3818e3d1e,e2cfccf9-ac60-40a0-bc58-e47a2cacb3bc,f572fa5d-7e48-43e9-bf60-d3e8e9a9405e,wheel-profile-3017d19a-2aa0-4356-bc2d-4feb203165ab,wheel-profile-7c570a78-b5bb-4520-b8c5-42370e9dd6f3,wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464,wheel-profile-e4f9606f-d819-4752-bad5-55b4b8006bff\", \"authority\": \"retrieved\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"8fa16288279c40b73409a8c0637d93470158c084c1f57bce4836b2ed13e13660\" } ], \"counts\": { \"total\": 9, \"by_section\": { \"agent_identity\": 1, \"enforced_policy\": 1, \"host_instructions\": 1, \"memory_and_retrieval\": 1, \"skill_catalog\": 1, \"world_state\": 4 }, \"by_authority\": { \"host\": 5, \"policy\": 1, \"retrieved\": 1, \"skill\": 1, \"system\": 1 } }, \"budgets\": { \"rendered_bytes\": 57459, \"rendered_characters\": 56951, \"context_window_tokens\": 1048576 }, \"selected_skills\": [], \"selected_tools\": [ \"a2ui_render\", \"activate_skill\", \"interrupt_agent\", \"list_agents\", \"native__memory_capture_policy\", \"native__memory_delete_by_id\", \"native__memory_history\", \"native__memory_list\", \"native__memory_recall\", \"native__memory_save\", \"native__memory_update_by_id\", \"native_echo\", \"native_system_info\", \"send_agent_message\", \"spawn_agent\", \"uar_compile\", \"uar_session_check_completeness\", \"uar_session_compile\", \"uar_session_update_section\", \"wait_agents\" ], \"warnings\": [] }"
                    - text: ●
                    - generic "Sources" [ref=e236]:
                      - 'button "1 — source: wheel-graphs.txt" [ref=e238] [cursor=pointer]':
                        - text: "1"
                        - generic [ref=e239]: "— source: wheel-graphs.txt"
                      - 'button "2 — source: wheel-graphs.txt" [ref=e240] [cursor=pointer]':
                        - text: "2"
                        - generic [ref=e241]: "— source: wheel-graphs.txt"
                      - 'button "3 — source: wheel-graphs.txt" [ref=e242] [cursor=pointer]':
                        - text: "3"
                        - generic [ref=e243]: "— source: wheel-graphs.txt"
                  - generic [ref=e245]:
                    - generic [ref=e246]:
                      - generic "Tool approval required" [ref=e247]:
                        - generic [ref=e248]:
                          - generic [ref=e251]: Approval required · spawn_agent
                          - generic [ref=e252]: pending
                        - paragraph [ref=e253]: Tool 'spawn_agent' requires approval under its descriptor
                        - generic "Proposed tool arguments" [ref=e255]:
                          - code [ref=e256]: "{ \"operation\": \"spawn_agent\", \"detailsAvailable\": false }"
                        - button "Review approval" [active] [ref=e257] [cursor=pointer]
                      - generic "Agent selection" [ref=e258]:
                        - generic [ref=e263]:
                          - paragraph [ref=e264]: "Selected agent: Blog and Article Writer"
                          - paragraph [ref=e265]: Intent-based routing
                        - status [ref=e266]: "Selected agent: Blog and Article Writer"
                      - text: ●
                    - generic [ref=e268]:
                      - button "Copy" [disabled]
                  - generic [ref=e269]:
                    - generic [ref=e272]:
                      - paragraph [ref=e273]: "[writing-specialist]"
                      - 'heading "Plant Your Own Tomato: Saturday Workshop" [level=1] [ref=e274]'
                      - paragraph [ref=e275]: Spring's heat has arrived, and it's time to start tomatoes. Join us Saturday, June 14, at 9:30 a.m. in the garden's south plot for a hands-on planting workshop led by longtime gardener Rosa Delgado.
                      - paragraph [ref=e276]: We'll cover choosing the right variety for your space, soil preparation, transplant timing, spacing, staking, and how to water deeply without wasting a drop. The advice suits raised beds, containers, and small backyards alike.
                      - paragraph [ref=e277]: Everyone leaves with two seedlings, a printed care guide, and a summer garden plot assignment. Gloves and tools are provided; bring a hat, water bottle, and sturdy, comfortable shoes.
                      - paragraph [ref=e278]: Registration is free, but space is limited to twenty gardeners. Sign up at the library front desk. We'll see you under the sun.
                      - paragraph [ref=e279]: By Wren Editor (writer-76b5104d)
                    - button "Copy" [ref=e282] [cursor=pointer]
                - generic [ref=e284]:
                  - button "Scroll to bottom" [ref=e285] [cursor=pointer]
                  - generic [ref=e287]:
                    - button [ref=e288]
                    - generic [ref=e289]:
                      - textbox "Message input" [ref=e290]:
                        - /placeholder: Send a message…
                      - generic [ref=e291]:
                        - button "Attach file" [ref=e292] [cursor=pointer]
                        - button "Disable memory for this message" [ref=e294] [cursor=pointer]:
                          - generic [ref=e295]: Memory ON – click to disable
                        - button "Send message" [disabled]
                  - generic [ref=e296]:
                    - button "Toggle Knowledge" [ref=e297] [cursor=pointer]:
                      - generic [ref=e298]: Knowledge
                    - button "Toggle Tools" [ref=e299] [cursor=pointer]:
                      - generic [ref=e300]: Tools
                      - generic [ref=e301]: "2"
                    - button "Toggle Skills" [ref=e302] [cursor=pointer]:
                      - generic [ref=e303]: Skills
                    - button "Enable Web Search" [ref=e304] [cursor=pointer]:
                      - generic [ref=e305]: Web Search
                    - button "Disable Memory" [ref=e306] [cursor=pointer]:
                      - generic [ref=e307]: Memory
  - region "Notifications alt+T"
```

# Test source

```ts
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
  172 |   assert.deepEqual(errors, [], `${label} run failed`);
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
> 192 |   await expect(assistantMessages(page)).toHaveCount(turnIndex);
      |                                         ^ Error: expect(locator).toHaveCount(expected) failed
  193 |   const last = assistantMessages(page).last();
  194 |   await expect(last).toContainText(expected.signature);
  195 |   const notices = page.locator('[data-agent-selection]');
  196 |   await expect(notices).toHaveCount(turnIndex);
  197 |   await expect(notices.last()).toHaveAttribute('data-agent-selection', expected.id);
  198 |   await expect(notices.last()).toContainText(`Selected agent: ${expected.title}`);
  199 |   const timeline = (await page.evaluate(() => window.__uarSelectionTimeline)).filter(entry => entry.t >= record.dom_start);
  200 |   record.dom_timeline = timeline;
  201 |   const visibleSelection = timeline.find(entry => entry.kind === 'selection');
  202 |   const visibleText = timeline.find(entry => entry.kind === 'answer_text');
  203 |   assert.ok(visibleSelection, `${record.label}: no visible selection notice`);
  204 |   assert.ok(visibleText, `${record.label}: no visible answer text`);
  205 |   assert.equal(visibleSelection.agent_id, expected.id);
  206 |   record.visible_selection_ms_before_answer = Math.round(visibleText.t - visibleSelection.t);
  207 |   assert.ok(visibleSelection.t <= visibleText.t, `${record.label}: the selection became visible after the answer text`);
  208 |   const path = resolve(evidenceDir(), `${record.label}.png`);
  209 |   await page.screenshot({ path, fullPage: true });
  210 |   s.evidence.screenshots.push(path);
  211 | }
  212 | 
  213 | Given('the saved math and writing specialists and an orchestrator conversation in the browser', async ({ page, scenario: s }) => {
  214 |   assert.ok(process.env.UAR_MODEL && process.env.UAR_WORKSPACE_ID && evidenceDir());
  215 |   const url = new URL(process.env.UAR_BASE_URL);
  216 |   assert.ok(['http:', 'https:'].includes(url.protocol) && !url.username && !url.password);
  217 |   s.evidence.model = process.env.UAR_MODEL;
  218 |   s.evidence.page_errors = [];
  219 |   page.on('pageerror', error => s.evidence.page_errors.push(String(error)));
  220 |   page.on('request', request => {
  221 |     if (request.method() === 'POST' && new URL(request.url()).pathname === COMPLETION_PATH) {
  222 |       s.completionPosts.push({ at: new Date().toISOString(), body: request.postData() });
  223 |     }
  224 |   });
  225 |   const agents = {};
  226 |   for (const id of [MATH_ID, WRITER_ID, ORCHESTRATOR]) {
  227 |     agents[id] = await api(s, 'GET', `/api/agents/${id}`);
  228 |     assert.equal(agents[id].id, id);
  229 |   }
  230 |   assert.equal(agents[ORCHESTRATOR].runtime?.entry, 'orchestrator');
  231 |   const listing = await api(s, 'GET', '/api/agents');
  232 |   const eligible = listing.runtime_agents.filter(a => a.id !== ORCHESTRATOR && a.runtime?.entry !== 'orchestrator').map(a => a.id);
  233 |   assert.ok(eligible.includes(MATH_ID) && eligible.includes(WRITER_ID), 'Both specialists must be in the live roster');
  234 |   s.evidence.eligible_roster = eligible.sort();
  235 |   s.expected = {
  236 |     writing: { id: WRITER_ID, title: agents[WRITER_ID].metadata.title, signature: WRITER_BYLINE },
  237 |     math: { id: MATH_ID, title: agents[MATH_ID].metadata.title, signature: MATH_MARKER },
  238 |   };
  239 |   s.orchestratorTitle = agents[ORCHESTRATOR].metadata.title;
  240 |   await page.goto('/threads', { waitUntil: 'domcontentloaded' });
  241 |   await page.evaluate(installDomTimeline);
  242 |   await page.getByRole('button', { name: 'New thread', exact: true }).click();
  243 |   const saved = observeResponse(page, r => r.request().method() === 'POST' && /\/api\/uar\/sessions\/[^/]+\/agent-config$/.test(new URL(r.url()).pathname), 60_000);
  244 |   await page.getByRole('button', { name: 'Select agent', exact: true }).click();
  245 |   await page.getByPlaceholder('Search agents...').fill(s.orchestratorTitle);
  246 |   const option = page.getByRole('option').filter({ has: page.getByText(s.orchestratorTitle, { exact: true }) });
  247 |   await expect(option).toHaveCount(1);
  248 |   await option.click();
  249 |   const response = await saved();
  250 |   assert.equal(response.status(), 200, 'The browser must save the orchestrator session selection');
  251 |   s.evidence.agent_selection_save = { path: new URL(response.url()).pathname, request: response.request().postDataJSON() };
  252 |   assert.equal(s.evidence.agent_selection_save.request.agent_id, ORCHESTRATOR);
  253 |   await expect(page.getByRole('button', { name: 'Select agent', exact: true })).toContainText(s.orchestratorTitle);
  254 | });
  255 | 
  256 | When('I send a writing task in that conversation through the browser', async ({ page, scenario: s }) => {
  257 |   s.writing = await sendTurn(page, s, 'writing_turn',
  258 |     'Write a 120-word blog post for a neighborhood newsletter announcing a Saturday community tomato-planting workshop.');
  259 |   s.threadId = s.writing.record.session_header;
  260 |   assert.ok(s.threadId, 'The browser must send its conversation id');
  261 |   s.ownedSessions.add(s.threadId);
  262 |   s.evidence.thread_id = s.threadId;
  263 | });
  264 | 
  265 | Then('the browser shows the writing specialist selection before its specialist answer', async ({ page, scenario: s }) => {
  266 |   assert.ok(s.writing.text.includes(WRITER_BYLINE), 'Missing the saved writer byline');
  267 |   await assertVisibleSelectionFirst(page, s, s.writing, s.expected.writing, 1);
  268 | });
  269 | 
  270 | When('I send a math task in the same conversation through the browser', async ({ page, scenario: s }) => {
  271 |   s.left = randomInt(101, 999);
  272 |   s.right = randomInt(11, 99);
  273 |   s.evidence.math_expected = s.left * s.right;
  274 |   s.math = await sendTurn(page, s, 'math_turn', `Compute ${s.left} multiplied by ${s.right}.`);
  275 |   assert.equal(s.math.record.session_header, s.threadId, 'The math task must continue the same conversation');
  276 | });
  277 | 
  278 | Then('the browser shows the math specialist selection before its specialist answer', async ({ page, scenario: s }) => {
  279 |   assert.ok(s.math.text.includes(MATH_MARKER), 'Missing the saved math specialist marker');
  280 |   assert.ok(s.math.text.replace(/,/g, '').includes(String(s.left * s.right)), `Math answer lacks ${s.left * s.right}`);
  281 |   await assertVisibleSelectionFirst(page, s, s.math, s.expected.math, 2);
  282 | });
  283 | 
  284 | Then('each selected specialist matches execution with no duplicate answer or run', async ({ page, scenario: s }) => {
  285 |   assert.equal(s.completionPosts.length, 2, 'Exactly one browser run per task');
  286 |   await expect(page.locator('[data-slot="aui_user-message-root"][data-role="user"]')).toHaveCount(2);
  287 |   await expect(assistantMessages(page)).toHaveCount(2);
  288 |   const turns = [[s.writing, s.expected.writing], [s.math, s.expected.math]];
  289 |   assert.notEqual(s.writing.record.run_id, s.math.record.run_id);
  290 |   assert.notEqual(s.writing.record.selected_thread_id, s.math.record.selected_thread_id);
  291 |   const runs = await api(s, 'GET', '/api/uar/runs');
  292 |   for (const [turn, expected] of turns) {
```
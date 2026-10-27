# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: features/a2ui-browser.feature.spec.js >> Interact with a real A2UI response in the browser >> A rendered calculator sends fresh input back to its agent
- Location: target/manual-a2ui-browser-generated/features/a2ui-browser.feature.spec.js:6:3

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: getByRole('region', { name: 'A2UI display artifact', exact: true }).locator('[data-a2ui-surface-state="ready"]')
Expected: visible
Timeout: 120000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 120000ms
  - waiting for getByRole('region', { name: 'A2UI display artifact', exact: true }).locator('[data-a2ui-surface-state="ready"]')

```

```yaml
- banner:
  - text: Universal Agent Runtime
  - navigation "Main navigation":
    - link "Chat":
      - /url: /threads
    - link "Admin":
      - /url: /admin
    - link "About":
      - /url: /about
- complementary:
  - text: Threads
  - button "New thread"
  - textbox "Search threads":
    - /placeholder: Search threads…
  - button "Please render the interactive parcel calculator. just now Default Assistant · <think>The initial request requires rendering the ...":
    - paragraph: Please render the interactive parcel calculator.
    - text: just now
    - paragraph: Default Assistant · <think>The initial request requires rendering the ...
  - button "Actions for Please render the interactive parcel calculator."
- main:
  - button "Select agent": Browser calculator 0a754465 liter/MiniMax-M3
  - button "Session configuration"
  - paragraph: Please render the interactive parcel calculator.
  - text: You Agent
  - region "A2UI display artifact":
    - text: Artifact · a2ui
    - heading "Effective run policy" [level=3]
    - paragraph: "[{\"version\":\"v0.9.1\",\"createSurface\":{\"surfaceId\":\"run-policy-79091d1d-cff5-46f7-aa9b-4dd1f1de76a9\",\"catalogId\":\"urn:uar:a2ui:catalog:1\"}},{\"version\":\"v0.9.1\",\"updateComponents\":{\"surfaceId\":\"run-policy-79091d1d-cff5-46f7-aa9b-4dd1f1de76a9\",\"components\":[{\"id\":\"title\",\"component\":\"Text\",\"text\":{\"path\":\"/title\"},\"variant\":\"h2\"},{\"id\":\"subtitle\",\"component\":\"Text\",\"text\":{\"path\":\"/subtitle\"},\"variant\":\"caption\"},{\"id\":\"overview-heading\",\"component\":\"Text\",\"text\":\"Execution\",\"variant\":\"h3\"},{\"id\":\"chat-mode\",\"component\":\"Text\",\"text\":{\"path\":\"/chatMode\"},\"variant\":\"body\"},{\"id\":\"model\",\"component\":\"Text\",\"text\":{\"path\":\"/model\"},\"variant\":\"body\"},{\"id\":\"agent\",\"component\":\"Text\",\"text\":{\"path\":\"/agent\"},\"variant\":\"body\"},{\"id\":\"overview-column\",\"component\":\"Column\",\"children\":[\"overview-heading\",\"chat-mode\",\"model\",\"agent\"]},{\"id\":\"overview-card\",\"component\":\"Card\",\"child\":\"overview-column\"},{\"id\":\"capabilities-heading\",\"component\":\"Text\",\"text\":\"Available capabilities\",\"variant\":\"h3\"},{\"id\":\"skills\",\"component\":\"Text\",\"text\":{\"path\":\"/skills\"},\"variant\":\"body\"},{\"id\":\"tools\",\"component\":\"Text\",\"text\":{\"path\":\"/tools\"},\"variant\":\"body\"},{\"id\":\"mcp-servers\",\"component\":\"Text\",\"text\":{\"path\":\"/mcpServers\"},\"variant\":\"body\"},{\"id\":\"knowledge-bases\",\"component\":\"Text\",\"text\":{\"path\":\"/knowledgeBases\"},\"variant\":\"body\"},{\"id\":\"capabilities-column\",\"component\":\"Column\",\"children\":[\"capabilities-heading\",\"skills\",\"tools\",\"mcp-servers\",\"knowledge-bases\"]},{\"id\":\"capabilities-card\",\"component\":\"Card\",\"child\":\"capabilities-column\"},{\"id\":\"runtime-heading\",\"component\":\"Text\",\"text\":\"Runtime controls\",\"variant\":\"h3\"},{\"id\":\"memory\",\"component\":\"Text\",\"text\":{\"path\":\"/memory\"},\"variant\":\"body\"},{\"id\":\"prompt-caching\",\"component\":\"Text\",\"text\":{\"path\":\"/promptCaching\"},\"variant\":\"body\"},{\"id\":\"context\",\"component\":\"Text\",\"text\":{\"path\":\"/context\"},\"variant\":\"body\"},{\"id\":\"approval\",\"component\":\"Text\",\"text\":{\"path\":\"/approval\"},\"variant\":\"body\"},{\"id\":\"runtime-column\",\"component\":\"Column\",\"children\":[\"runtime-heading\",\"memory\",\"prompt-caching\",\"context\",\"approval\"]},{\"id\":\"runtime-card\",\"component\":\"Card\",\"child\":\"runtime-column\"},{\"id\":\"warnings-heading\",\"component\":\"Text\",\"text\":{\"path\":\"/warningsHeading\"},\"variant\":\"h3\"},{\"id\":\"warnings\",\"component\":\"Text\",\"text\":{\"path\":\"/warnings\"},\"variant\":\"body\"},{\"id\":\"warnings-column\",\"component\":\"Column\",\"children\":[\"warnings-heading\",\"warnings\"]},{\"id\":\"warnings-card\",\"component\":\"Card\",\"child\":\"warnings-column\"},{\"id\":\"root\",\"component\":\"Column\",\"children\":[\"title\",\"subtitle\",\"overview-card\",\"capabilities-card\",\"runtime-card\",\"warnings-card\"]}]}},{\"version\":\"v0.9.1\",\"updateDataModel\":{\"surfaceId\":\"run-policy-79091d1d-cff5-46f7-aa9b-4dd1f1de76a9\",\"path\":\"/\",\"value\":{\"title\":\"Effective run policy\",\"subtitle\":\"Resolved policy v1 · A2UI v0.9.1\",\"chatMode\":\"Chat mode · agent\",\"model\":\"Model · liter / MiniMax-M3\",\"agent\":\"Agent · disposable-a2ui-browser-90dd5001-2027-4a2f-b053-1f423ba67bf8\",\"skills\":\"Skills · auto · 210 available · agent scope\",\"tools\":\"Tools · selected · 1 available · agent scope\",\"mcpServers\":\"MCP servers · auto · 0 available · legacy scope\",\"knowledgeBases\":\"Knowledge bases · none · 0 available · agent scope\",\"memory\":\"Memory · on\",\"promptCaching\":\"Prompt caching · off\",\"context\":\"Context · sliding window\",\"approval\":\"Tool approval · auto\",\"warningsHeading\":\"Warnings · 0\",\"warnings\":\"No policy warnings.\"}}}]"
    - status
  - region "A2UI display artifact":
    - text: Artifact · json
    - heading "Turn manifest" [level=3]
    - paragraph: "{\"schema_version\":1,\"manifest_hash\":\"748f610bd04f59eae5af919de40819fb6d36aae539b93338b677357f650c8b8a\",\"fragments\":[{\"id\":\"agent.identity\",\"section\":\"agent_identity\",\"source\":\"artifact:disposable-a2ui-browser-90dd5001-2027-4a2f-b053-1f423ba67bf8\",\"authority\":\"system\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"5bff8ccbe118e221b7633ecf8b5805387ec07d75408bee7cadd2ae45bd454eec\"},{\"id\":\"policy.effective\",\"section\":\"enforced_policy\",\"source\":\"effective_run_policy\",\"authority\":\"policy\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"a5c80076cbe9c70572db9954e54b1632f5e5a0b2a98ab585f99db7e662e2ac65\"},{\"id\":\"skills.catalog\",\"section\":\"skill_catalog\",\"source\":\"eligible_skill_registry\",\"authority\":\"skill\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"d77ed6dbeef567fdfafab070e307d297b37ad4baaa1ddd06bcacff342b99ce29\"},{\"id\":\"world_state.current_time\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"351d63402bd8b925f6a0150759e386a89220a504a5abbbe75c031bec1424430b\"},{\"id\":\"world_state.environment\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"60dd57dd0b5613817cff0f9517e6e5ec987d6ee6cd6e0bc93bbfbd5d65d5e859\"},{\"id\":\"world_state.permissions\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"24ebd0230f6d7165bb84398bb8f16f1b21ca7592192806ef347039d0e2a8bd3d\"},{\"id\":\"world_state.project_instructions\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\"}],\"counts\":{\"total\":7,\"by_section\":{\"agent_identity\":1,\"enforced_policy\":1,\"skill_catalog\":1,\"world_state\":4},\"by_authority\":{\"host\":4,\"policy\":1,\"skill\":1,\"system\":1}},\"budgets\":{\"rendered_bytes\":55785,\"rendered_characters\":55281,\"context_window_tokens\":1048576},\"selected_skills\":[],\"selected_tools\":[\"a2ui_render\",\"activate_skill\"],\"warnings\":[]}"
    - status
  - region "A2UI display artifact":
    - text: Artifact · json
    - heading "resolved_step" [level=3]
    - paragraph: "{\"source\":\"uar.turn\",\"payload\":{\"step\":1,\"model\":\"MiniMax-M3\",\"mcp_catalog\":[],\"manifest\":{\"schema_version\":1,\"manifest_hash\":\"d2dc26f65016c01743632ee860f1b032720e4a32b438ac9cb7e26036a7c81e35\",\"fragments\":[{\"id\":\"agent.identity\",\"section\":\"agent_identity\",\"source\":\"artifact:disposable-a2ui-browser-90dd5001-2027-4a2f-b053-1f423ba67bf8\",\"authority\":\"system\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"5bff8ccbe118e221b7633ecf8b5805387ec07d75408bee7cadd2ae45bd454eec\"},{\"id\":\"policy.effective\",\"section\":\"enforced_policy\",\"source\":\"effective_run_policy\",\"authority\":\"policy\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"a5c80076cbe9c70572db9954e54b1632f5e5a0b2a98ab585f99db7e662e2ac65\"},{\"id\":\"skills.catalog\",\"section\":\"skill_catalog\",\"source\":\"eligible_skill_registry\",\"authority\":\"skill\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"d77ed6dbeef567fdfafab070e307d297b37ad4baaa1ddd06bcacff342b99ce29\"},{\"id\":\"world_state.current_time\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"351d63402bd8b925f6a0150759e386a89220a504a5abbbe75c031bec1424430b\"},{\"id\":\"world_state.environment\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"60dd57dd0b5613817cff0f9517e6e5ec987d6ee6cd6e0bc93bbfbd5d65d5e859\"},{\"id\":\"world_state.permissions\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"24ebd0230f6d7165bb84398bb8f16f1b21ca7592192806ef347039d0e2a8bd3d\"},{\"id\":\"world_state.project_instructions\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\"}],\"counts\":{\"total\":7,\"by_section\":{\"agent_identity\":1,\"enforced_policy\":1,\"skill_catalog\":1,\"world_state\":4},\"by_authority\":{\"host\":4,\"policy\":1,\"skill\":1,\"system\":1}},\"budgets\":{\"rendered_bytes\":55554,\"rendered_characters\":55050,\"context_window_tokens\":1048576},\"selected_skills\":[],\"selected_tools\":[\"a2ui_render\",\"activate_skill\"],\"warnings\":[]}}}"
    - status
  - region "A2UI display artifact":
    - text: Artifact · json
    - heading "Provider attempt manifest" [level=3]
    - paragraph: "{\"schema_version\":\"uar.attempt-manifest.v1\",\"destination_model\":\"MiniMax-M3\",\"budgeting\":{\"label\":\"legacy_unbudgeted\",\"contract_present\":false,\"fit_guarantee\":false},\"warning\":\"Provider dispatch is using the legacy unbudgeted path\"}"
    - status
  - region "A2UI display artifact":
    - text: Artifact · json
    - heading "budget_receipt" [level=3]
    - paragraph: "{\"source\":\"uar.request\",\"payload\":{\"schema_version\":\"uar.request-budget.v1\",\"representation\":\"final_provider_request\",\"destination_model\":\"MiniMax-M3\",\"serialized_request\":true,\"count_quality\":\"approximate\",\"counting_revision\":\"approximate-cl100k-json-v1\",\"counted_input_tokens\":13931,\"serialized_bytes\":61029,\"serialized_sha256\":\"ca0a81a506c28e903b52f75abec626309df93c349b56f99420021b4076ba4335\",\"tools_included\":true,\"instructions_included\":true,\"fit_guarantee\":false,\"reserved_output_tokens\":null,\"output_ceiling_enforced\":true}}"
    - status
  - paragraph: <think>The initial request requires rendering the A2UI surface exactly once with the specified template.</think>
  - button "a2ui_render complete"
  - region "A2UI display artifact":
    - text: Artifact · application/a2ui+json
    - heading "Interactive UI" [level=3]
    - paragraph: "{\"version\":\"v0.9.1\",\"createSurface\":{\"surfaceId\":\"parcel-calculator\",\"catalogId\":\"https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json\"}} {\"version\":\"v0.9.1\",\"updateComponents\":{\"surfaceId\":\"parcel-calculator\",\"components\":[{\"id\":\"root\",\"component\":\"Column\",\"children\":[\"heading\",\"parcel\",\"calculate\"]},{\"id\":\"heading\",\"component\":\"Text\",\"text\":\"Parcel calculator\"},{\"id\":\"parcel\",\"component\":\"TextField\",\"label\":\"Parcel number\",\"value\":{\"path\":\"/parcel\"}},{\"id\":\"calculate\",\"component\":\"Button\",\"child\":\"button-label\",\"action\":{\"event\":{\"name\":\"calculate_parcel\"}}},{\"id\":\"button-label\",\"component\":\"Text\",\"text\":\"Calculate parcel\"}]}} {\"version\":\"v0.9.1\",\"updateDataModel\":{\"surfaceId\":\"parcel-calculator\",\"path\":\"/\",\"value\":{\"parcel\":\"\"}}}"
    - status
  - region "A2UI display artifact":
    - text: Artifact · json
    - heading "resolved_step" [level=3]
    - paragraph: "{\"source\":\"uar.turn\",\"payload\":{\"step\":2,\"model\":\"MiniMax-M3\",\"mcp_catalog\":[],\"manifest\":{\"schema_version\":1,\"manifest_hash\":\"53805722426049c72e602abec192df8fcc8e8b7941fe5c3572682fe075c90120\",\"fragments\":[{\"id\":\"agent.identity\",\"section\":\"agent_identity\",\"source\":\"artifact:disposable-a2ui-browser-90dd5001-2027-4a2f-b053-1f423ba67bf8\",\"authority\":\"system\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"5bff8ccbe118e221b7633ecf8b5805387ec07d75408bee7cadd2ae45bd454eec\"},{\"id\":\"policy.effective\",\"section\":\"enforced_policy\",\"source\":\"effective_run_policy\",\"authority\":\"policy\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"a5c80076cbe9c70572db9954e54b1632f5e5a0b2a98ab585f99db7e662e2ac65\"},{\"id\":\"skills.catalog\",\"section\":\"skill_catalog\",\"source\":\"eligible_skill_registry\",\"authority\":\"skill\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"d77ed6dbeef567fdfafab070e307d297b37ad4baaa1ddd06bcacff342b99ce29\"}],\"counts\":{\"total\":3,\"by_section\":{\"agent_identity\":1,\"enforced_policy\":1,\"skill_catalog\":1},\"by_authority\":{\"policy\":1,\"skill\":1,\"system\":1}},\"budgets\":{\"rendered_bytes\":55893,\"rendered_characters\":55389,\"context_window_tokens\":1048576},\"selected_skills\":[],\"selected_tools\":[\"a2ui_render\",\"activate_skill\"],\"warnings\":[]}}}"
    - status
  - region "A2UI display artifact":
    - text: Artifact · json
    - heading "Provider attempt manifest" [level=3]
    - paragraph: "{\"schema_version\":\"uar.attempt-manifest.v1\",\"destination_model\":\"MiniMax-M3\",\"budgeting\":{\"label\":\"legacy_unbudgeted\",\"contract_present\":false,\"fit_guarantee\":false},\"warning\":\"Provider dispatch is using the legacy unbudgeted path\"}"
    - status
  - region "A2UI display artifact":
    - text: Artifact · json
    - heading "budget_receipt" [level=3]
    - paragraph: "{\"source\":\"uar.request\",\"payload\":{\"schema_version\":\"uar.request-budget.v1\",\"representation\":\"final_provider_request\",\"destination_model\":\"MiniMax-M3\",\"serialized_request\":true,\"count_quality\":\"approximate\",\"counting_revision\":\"approximate-cl100k-json-v1\",\"counted_input_tokens\":14302,\"serialized_bytes\":62527,\"serialized_sha256\":\"530dcbd1336d9165d16947dc1efc286b39c33f7db1a4b714f65270b0bc6a80b7\",\"tools_included\":true,\"instructions_included\":true,\"fit_guarantee\":false,\"reserved_output_tokens\":null,\"output_ceiling_enforced\":true}}"
    - status
  - paragraph: Calculator ready.
  - button "Copy"
  - text: Browser calculator 0a754465 MiniMax-M3 29038507
  - button "Scroll to bottom"
  - textbox "Message input":
    - /placeholder: Send a message to the agent…
  - button "Attach file"
  - button "Disable memory for this message": Memory ON – click to disable
  - button "Send message" [disabled]
  - button "Toggle Knowledge": Knowledge
  - button "Toggle Tools": Tools 1
  - button "Toggle Skills": Skills
  - button "Enable Web Search": Web Search
  - button "Disable Memory": Memory
- region "Notifications alt+T"
```

# Test source

```ts
  31  | 
  32  | async function api(state, method, path, payload, expected = 200) {
  33  |   const response = await fetch(new URL(path, process.env.UAR_BASE_URL), {
  34  |     method, redirect: 'error', signal: AbortSignal.timeout(15_000),
  35  |     headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': process.env.UAR_WORKSPACE_ID },
  36  |     body: payload === undefined ? undefined : JSON.stringify(payload),
  37  |   });
  38  |   if (method === 'POST' && path === '/api/agents' && response.status === 201) state.created = true;
  39  |   const text = await response.text();
  40  |   state.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text });
  41  |   assert.equal(response.status, expected, `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  42  |   return text ? JSON.parse(text) : null;
  43  | }
  44  | 
  45  | function observeResponse(page, predicate) {
  46  |   // Attach rejection handling immediately so an earlier UI failure stays primary.
  47  |   const pending = page.waitForResponse(predicate).then(response => ({ response }), error => ({ error }));
  48  |   return async () => {
  49  |     const result = await pending;
  50  |     if (result.error) throw result.error;
  51  |     return result.response;
  52  |   };
  53  | }
  54  | 
  55  | async function stream(state, response) {
  56  |   const text = await response.text();
  57  |   const record = { path: new URL(response.url()).pathname, status: response.status(),
  58  |     request: response.request().postDataJSON(), response: text };
  59  |   state.evidence.streams.push(record);
  60  |   assert.equal(response.status(), 200);
  61  |   assert.match(response.headers()['content-type'], /^text\/event-stream\b/);
  62  |   const events = text.split(/\r?\n\r?\n/).flatMap(block => {
  63  |     const name = block.split(/\r?\n/).find(line => line.startsWith('event:'))?.slice(6).trim();
  64  |     const data = block.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
  65  |     if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
  66  |     return [JSON.parse(data)];
  67  |   });
  68  |   record.events = events;
  69  |   assert.ok(!events.some(event => ['RUN_ERROR', 'RUN_CANCELLED'].includes(event.type)), 'Stream must succeed');
  70  |   const starts = events.filter(event => event.type === 'RUN_STARTED');
  71  |   const finishes = events.filter(event => event.type === 'RUN_FINISHED');
  72  |   assert.equal(starts.length, 1);
  73  |   assert.equal(finishes.length, 1);
  74  |   assert.equal(starts[0].runId, finishes[0].runId);
  75  |   return { runId: starts[0].runId, events,
  76  |     text: events.filter(event => event.type === 'TEXT_MESSAGE_CONTENT').map(event => event.delta).join('') };
  77  | }
  78  | 
  79  | Given('a disposable A2UI calculator agent is selected in the browser', async ({ page, scenario: s }) => {
  80  |   assert.ok(process.env.UAR_MODEL && process.env.UAR_WORKSPACE_ID);
  81  |   const url = new URL(process.env.UAR_BASE_URL);
  82  |   assert.ok(['http:', 'https:'].includes(url.protocol) && !url.username && !url.password);
  83  |   s.agentId = `disposable-a2ui-browser-${randomUUID()}`;
  84  |   s.title = `Browser calculator ${randomUUID().slice(0, 8)}`;
  85  |   s.marker = `BROWSER-${randomUUID()}`;
  86  |   s.evidence.agent_id = s.agentId;
  87  |   s.evidence.model = process.env.UAR_MODEL;
  88  |   s.evidence.page_errors = [];
  89  |   page.on('pageerror', error => s.evidence.page_errors.push(String(error)));
  90  |   const agent = await api(s, 'GET', '/api/agents/default-agent');
  91  |   agent.id = s.agentId;
  92  |   agent.version = '1.0.0';
  93  |   agent.metadata = { ...agent.metadata, title: s.title, description: 'Disposable interactive parcel calculator.', tags: ['verification'] };
  94  |   agent.runtime.entry = 'default';
  95  |   const template = {"messages":[{"version":"v0.9.1","createSurface":{"surfaceId":"parcel-calculator","catalogId":"https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json"}},{"version":"v0.9.1","updateComponents":{"surfaceId":"parcel-calculator","components":[{"id":"root","component":"Column","children":["heading","parcel","calculate"]},{"id":"heading","component":"Text","text":"Parcel calculator"},{"id":"parcel","component":"TextField","label":"Parcel number","value":{"path":"/parcel"}},{"id":"calculate","component":"Button","child":"button-label","action":{"event":{"name":"calculate_parcel"}}},{"id":"button-label","component":"Text","text":"Calculate parcel"}]}},{"version":"v0.9.1","updateDataModel":{"surfaceId":"parcel-calculator","path":"/","value":{"parcel":""}}}]};
  96  |   agent.prompt = { system: 'On the initial request call a2ui_render exactly once with this valid template verbatim: '
  97  |     + JSON.stringify(template) + '. Preserve JSON arrays exactly: messages, components and children are arrays, NEVER objects with an item property. Use native JSON tool arguments, not XML wrappers. Do not alter the template, use another tool or print it as prose. '
  98  |     + 'After publication say only Calculator ready. '
  99  |     + 'On an a2ui.user_action follow-up, do not render again. Read the fresh parcel value from '
  100 |     + 'interaction.a2uiClientDataModel[interaction.surfaceId].parcel. '
  101 |     + `Return only JSON with marker "${s.marker}" and numeric result equal to that fresh parcel number plus 19.`, instructions: [] };
  102 |   const split = process.env.UAR_MODEL.indexOf('/');
  103 |   assert.ok(split > 0);
  104 |   agent.policy.provider = { default: { provider: process.env.UAR_MODEL.slice(0, split), model: process.env.UAR_MODEL.slice(split + 1) }, fallbacks: [] };
  105 |   agent.policy.tools.allow = ['a2ui_render'];
  106 |   agent.policy.skills.prefer = [];
  107 |   agent.memory.kb.enabled = false;
  108 |   const saved = await api(s, 'POST', '/api/agents', agent, 201);
  109 |   assert.equal(saved.id, s.agentId);
  110 |   await page.goto('/threads', { waitUntil: 'domcontentloaded' });
  111 |   await page.getByRole('button', { name: 'New thread', exact: true }).click();
  112 |   await page.getByRole('button', { name: 'Select agent', exact: true }).click();
  113 |   await page.getByPlaceholder('Search agents...').fill(s.title);
  114 |   await page.getByRole('option').filter({ hasText: s.title }).click();
  115 | });
  116 | 
  117 | When('I ask the agent to render its calculator', async ({ page, scenario: s }) => {
  118 |   const response = observeResponse(page, r => r.request().method() === 'POST' && new URL(r.url()).pathname === '/api/chat/completion');
  119 |   await page.getByRole('textbox', { name: 'Message input', exact: true }).fill('Please render the interactive parcel calculator.');
  120 |   await page.getByRole('button', { name: 'Send message', exact: true }).click();
  121 |   s.initial = await stream(s, await response());
  122 |   const artifacts = s.initial.events.filter(event => event.type === 'CUSTOM' && event.name === 'uar.artifact.available' && event.value?.artifactType === 'a2ui');
  123 |   assert.equal(artifacts.length, 1, 'Require genuine published A2UI artifact');
  124 |   assert.equal(artifacts[0].value.metadata?.sourceTool, 'a2ui_render');
  125 |   assert.equal(artifacts[0].value.metadata?.publication_status, 'published');
  126 |   s.evidence.source_run_id = s.initial.runId;
  127 | });
  128 | 
  129 | Then('the browser displays the generated input and action button', async ({ page, scenario: s }) => {
  130 |   s.surface = page.getByRole('region', { name: 'A2UI display artifact', exact: true });
> 131 |   await expect(s.surface.locator('[data-a2ui-surface-state="ready"]')).toBeVisible();
      |                                                                        ^ Error: expect(locator).toBeVisible() failed
  132 |   await expect(s.surface).toContainText('Parcel calculator');
  133 |   await expect(s.surface.getByRole('textbox', { name: 'Parcel number', exact: true })).toBeVisible();
  134 |   await expect(s.surface.getByRole('button', { name: 'Calculate parcel', exact: true })).toBeVisible();
  135 |   await page.screenshot({ path: resolve(process.env.UAR_BROWSER_EVIDENCE_DIR, 'rendered-surface.png'), fullPage: true });
  136 | });
  137 | 
  138 | When('I enter a fresh number and press the rendered calculate button', async ({ page, scenario: s }) => {
  139 |   s.number = randomInt(100, 900);
  140 |   s.evidence.fresh_input = s.number;
  141 |   await s.surface.getByRole('textbox', { name: 'Parcel number', exact: true }).fill(String(s.number));
  142 |   const action = observeResponse(page, r => r.request().method() === 'POST' && /\/api\/uar\/runs\/[^/]+\/a2ui\/actions$/.test(new URL(r.url()).pathname));
  143 |   const continuation = observeResponse(page, r => r.request().method() === 'GET' && /\/api\/uar\/runs\/[^/]+\/stream$/.test(new URL(r.url()).pathname));
  144 |   await s.surface.getByRole('button', { name: 'Calculate parcel', exact: true }).click();
  145 |   const response = await action();
  146 |   const body = response.request().postDataJSON();
  147 |   const ack = await response.json();
  148 |   s.evidence.action = { path: new URL(response.url()).pathname, request: body, status: response.status(), response: ack };
  149 |   assert.equal(response.status(), 202);
  150 |   assert.equal(body.name, 'calculate_parcel');
  151 |   assert.equal(body.a2uiClientDataModel?.[body.surfaceId]?.parcel, String(s.number));
  152 |   assert.equal(ack.status, 'continued');
  153 |   assert.equal(ack.run_id, s.initial.runId);
  154 |   assert.ok(ack.continuation_run_id && ack.continuation_run_id !== ack.run_id);
  155 |   s.continuation = await stream(s, await continuation());
  156 |   assert.equal(s.continuation.runId, ack.continuation_run_id);
  157 | });
  158 | 
  159 | Then('the agent answers using that browser input and its saved instructions', async ({ page, scenario: s }) => {
  160 |   const final = s.continuation.text.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim();
  161 |   let answer;
  162 |   try { answer = JSON.parse(final); } catch {
  163 |     const fences = [...final.matchAll(/(?:^|\n)```(?:json)?[ \t]*\r?\n([\s\S]*?)\r?\n```(?=\s|$)/g)];
  164 |     assert.equal(fences.length, 1);
  165 |     answer = JSON.parse(fences[0][1]);
  166 |   }
  167 |   assert.deepEqual(answer, { marker: s.marker, result: s.number + 19 });
  168 |   const last = page.locator('[data-slot="aui_assistant-message-root"][data-role="assistant"]').last();
  169 |   await expect(last).toContainText(s.marker);
  170 |   await expect(last).toContainText(String(s.number + 19));
  171 |   s.evidence.answer = answer;
  172 |   s.evidence.continuation_run_id = s.continuation.runId;
  173 |   s.evidence.result = 'VERIFIED';
  174 |   await page.screenshot({ path: resolve(process.env.UAR_BROWSER_EVIDENCE_DIR, 'continued-answer.png'), fullPage: true });
  175 | });
  176 | 
```
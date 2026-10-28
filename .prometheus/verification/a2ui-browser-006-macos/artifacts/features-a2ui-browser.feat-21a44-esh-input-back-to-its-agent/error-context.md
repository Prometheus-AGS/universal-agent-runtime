# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: features/a2ui-browser.feature.spec.js >> Interact with a real A2UI response in the browser >> A rendered calculator sends fresh input back to its agent
- Location: target/manual-a2ui-browser-generated/features/a2ui-browser.feature.spec.js:6:3

# Error details

```
Error: expect(locator).toContainText(expected) failed

Locator: getByRole('region', { name: 'A2UI display artifact', exact: true })
Expected substring: "Parcel calculator"
Error: strict mode violation: getByRole('region', { name: 'A2UI display artifact', exact: true }) resolved to 2 elements:
    1) <section aria-label="A2UI display artifact" class="my-2 rounded-xl bg-card px-3 py-3">…</section> aka getByText('ArtifactA2UI v0.9.1 · rendered · surfaceEffective run policyResolved policy v1')
    2) <section aria-label="A2UI display artifact" class="my-2 rounded-xl bg-card px-3 py-3">…</section> aka getByText('ArtifactInvalid surface · surface · application/a2ui+jsonInteractive UI could')

Call log:
  - Expect "toContainText" with timeout 120000ms
  - waiting for getByRole('region', { name: 'A2UI display artifact', exact: true })

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
                  - paragraph [ref=e126]: No threads yet
                  - paragraph [ref=e127]: Start a new thread above
            - region "Conversation" [ref=e128]:
              - generic [ref=e129]:
                - button "Select agent" [ref=e130] [cursor=pointer]:
                  - generic [ref=e131]:
                    - generic [ref=e132]: Browser calculator dabc6cf1
                    - generic [ref=e133]: liter/MiniMax-M3
                - button "Session configuration" [ref=e136] [cursor=pointer]
              - generic [ref=e139]:
                - generic [ref=e140]:
                  - paragraph [ref=e145]: Please render the interactive parcel calculator.
                  - paragraph [ref=e150]: Please render the interactive parcel calculator.
                  - generic [ref=e152]:
                    - region "A2UI display artifact" [ref=e153]:
                      - generic [ref=e154]:
                        - generic [ref=e158]: Artifact
                        - generic [ref=e159]: A2UI v0.9.1 · rendered · surface
                      - group [ref=e162]:
                        - generic [ref=e165]:
                          - heading "Effective run policy" [level=2] [ref=e167]
                          - generic [ref=e168]: Resolved policy v1 · A2UI v0.9.1
                          - generic [ref=e172]:
                            - heading "Execution" [level=3] [ref=e174]
                            - paragraph [ref=e176]: Chat mode · agent
                            - paragraph [ref=e178]: Model · liter / MiniMax-M3
                            - paragraph [ref=e180]: Agent · disposable-a2ui-browser-9e344d2a-11ed-44a3-8e2e-d9f31aacf54d
                          - generic [ref=e184]:
                            - heading "Available capabilities" [level=3] [ref=e186]
                            - paragraph [ref=e188]: Skills · auto · 210 available · agent scope
                            - paragraph [ref=e190]: Tools · selected · 1 available · agent scope
                            - paragraph [ref=e192]: MCP servers · auto · 0 available · legacy scope
                            - paragraph [ref=e194]: Knowledge bases · none · 0 available · agent scope
                          - generic [ref=e198]:
                            - heading "Runtime controls" [level=3] [ref=e200]
                            - paragraph [ref=e202]: Memory · on
                            - paragraph [ref=e204]: Prompt caching · off
                            - paragraph [ref=e206]: Context · sliding window
                            - paragraph [ref=e208]: Tool approval · auto
                          - generic [ref=e212]:
                            - heading "Warnings · 0" [level=3] [ref=e214]
                            - paragraph [ref=e216]: No policy warnings.
                    - generic "Turn manifest" [ref=e217]:
                      - generic "JSON" [ref=e225]:
                        - code [ref=e226]: "{ \"schema_version\": 1, \"manifest_hash\": \"9dd473a682cc496e55bd8f90d19a8bc5b0130a6c8c1bd07a04f604631e95cea4\", \"fragments\": [ { \"id\": \"agent.identity\", \"section\": \"agent_identity\", \"source\": \"artifact:disposable-a2ui-browser-9e344d2a-11ed-44a3-8e2e-d9f31aacf54d\", \"authority\": \"system\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"be10f67d10e9d0147f09c723bb7f94b9d2e2d5145e986adc19e867b54c41b6cf\" }, { \"id\": \"policy.effective\", \"section\": \"enforced_policy\", \"source\": \"effective_run_policy\", \"authority\": \"policy\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"6feeb99001f84b3a7809ab761daf88bd6620f37997fda67d84ce317c64e79bf5\" }, { \"id\": \"skills.catalog\", \"section\": \"skill_catalog\", \"source\": \"eligible_skill_registry\", \"authority\": \"skill\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"d77ed6dbeef567fdfafab070e307d297b37ad4baaa1ddd06bcacff342b99ce29\" }, { \"id\": \"world_state.current_time\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"02a01cdd5b975ccaffc5eaf0e7c62bba31193c214b3ac7e1b37bbe5e1c1d2cc7\" }, { \"id\": \"world_state.environment\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"60dd57dd0b5613817cff0f9517e6e5ec987d6ee6cd6e0bc93bbfbd5d65d5e859\" }, { \"id\": \"world_state.permissions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"24ebd0230f6d7165bb84398bb8f16f1b21ca7592192806ef347039d0e2a8bd3d\" }, { \"id\": \"world_state.project_instructions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\" } ], \"counts\": { \"total\": 7, \"by_section\": { \"agent_identity\": 1, \"enforced_policy\": 1, \"skill_catalog\": 1, \"world_state\": 4 }, \"by_authority\": { \"host\": 4, \"policy\": 1, \"skill\": 1, \"system\": 1 } }, \"budgets\": { \"rendered_bytes\": 55786, \"rendered_characters\": 55282, \"context_window_tokens\": 1048576 }, \"selected_skills\": [], \"selected_tools\": [ \"a2ui_render\", \"activate_skill\" ], \"warnings\": [] }"
                    - generic "resolved_step" [ref=e227]:
                      - generic "JSON" [ref=e235]:
                        - code [ref=e236]: "{ \"source\": \"uar.turn\", \"payload\": { \"step\": 1, \"model\": \"MiniMax-M3\", \"mcp_catalog\": [], \"manifest\": { \"schema_version\": 1, \"manifest_hash\": \"ac496fae90d0e3579950e7d959dba07935c5e367d36c1fdd14ae201131f8abdf\", \"fragments\": [ { \"id\": \"agent.identity\", \"section\": \"agent_identity\", \"source\": \"artifact:disposable-a2ui-browser-9e344d2a-11ed-44a3-8e2e-d9f31aacf54d\", \"authority\": \"system\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"be10f67d10e9d0147f09c723bb7f94b9d2e2d5145e986adc19e867b54c41b6cf\" }, { \"id\": \"policy.effective\", \"section\": \"enforced_policy\", \"source\": \"effective_run_policy\", \"authority\": \"policy\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"6feeb99001f84b3a7809ab761daf88bd6620f37997fda67d84ce317c64e79bf5\" }, { \"id\": \"skills.catalog\", \"section\": \"skill_catalog\", \"source\": \"eligible_skill_registry\", \"authority\": \"skill\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"d77ed6dbeef567fdfafab070e307d297b37ad4baaa1ddd06bcacff342b99ce29\" }, { \"id\": \"world_state.current_time\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"02a01cdd5b975ccaffc5eaf0e7c62bba31193c214b3ac7e1b37bbe5e1c1d2cc7\" }, { \"id\": \"world_state.environment\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"60dd57dd0b5613817cff0f9517e6e5ec987d6ee6cd6e0bc93bbfbd5d65d5e859\" }, { \"id\": \"world_state.permissions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"24ebd0230f6d7165bb84398bb8f16f1b21ca7592192806ef347039d0e2a8bd3d\" }, { \"id\": \"world_state.project_instructions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\" } ], \"counts\": { \"total\": 7, \"by_section\": { \"agent_identity\": 1, \"enforced_policy\": 1, \"skill_catalog\": 1, \"world_state\": 4 }, \"by_authority\": { \"host\": 4, \"policy\": 1, \"skill\": 1, \"system\": 1 } }, \"budgets\": { \"rendered_bytes\": 55555, \"rendered_characters\": 55051, \"context_window_tokens\": 1048576 }, \"selected_skills\": [], \"selected_tools\": [ \"a2ui_render\", \"activate_skill\" ], \"warnings\": [] } } }"
                    - generic "Provider attempt manifest" [ref=e237]:
                      - generic "JSON" [ref=e245]:
                        - code [ref=e246]: "{ \"schema_version\": \"uar.attempt-manifest.v1\", \"destination_model\": \"MiniMax-M3\", \"budgeting\": { \"label\": \"legacy_unbudgeted\", \"contract_present\": false, \"fit_guarantee\": false }, \"warning\": \"Provider dispatch is using the legacy unbudgeted path\" }"
                    - generic "budget_receipt" [ref=e247]:
                      - generic "JSON" [ref=e255]:
                        - code [ref=e256]: "{ \"source\": \"uar.request\", \"payload\": { \"schema_version\": \"uar.request-budget.v1\", \"representation\": \"final_provider_request\", \"destination_model\": \"MiniMax-M3\", \"serialized_request\": true, \"count_quality\": \"approximate\", \"counting_revision\": \"approximate-cl100k-json-v1\", \"counted_input_tokens\": 13938, \"serialized_bytes\": 61030, \"serialized_sha256\": \"ea07afab6afc9d664da96ee6ee78673905562e6805b85b238507af02d5cc85da\", \"tools_included\": true, \"instructions_included\": true, \"fit_guarantee\": false, \"reserved_output_tokens\": null, \"output_ceiling_enforced\": true } }"
                    - paragraph [ref=e259]: I'll render the parcel calculator surface now.
                    - generic "Tool a2ui_render" [ref=e260]:
                      - group [ref=e261]:
                        - generic "a2ui_render complete" [ref=e262] [cursor=pointer]:
                          - generic [ref=e265]: a2ui_render
                          - generic [ref=e266]: complete
                    - region "A2UI display artifact" [ref=e267]:
                      - generic [ref=e268]:
                        - generic [ref=e272]: Artifact
                        - generic [ref=e273]: Invalid surface · surface · application/a2ui+json
                      - generic [ref=e274]:
                        - alert [ref=e275]:
                          - paragraph [ref=e276]: Interactive UI could not be rendered.
                          - paragraph [ref=e277]: Invalid input
                        - group [ref=e278]:
                          - generic "View A2UI source" [ref=e279] [cursor=pointer]
                    - generic "resolved_step" [ref=e280]:
                      - generic "JSON" [ref=e288]:
                        - code [ref=e289]: "{ \"source\": \"uar.turn\", \"payload\": { \"step\": 2, \"model\": \"MiniMax-M3\", \"mcp_catalog\": [], \"manifest\": { \"schema_version\": 1, \"manifest_hash\": \"49696d577a73d1c70e81805cfbb93e467ba14da7897cbe137cff95e3b1ca43ca\", \"fragments\": [ { \"id\": \"agent.identity\", \"section\": \"agent_identity\", \"source\": \"artifact:disposable-a2ui-browser-9e344d2a-11ed-44a3-8e2e-d9f31aacf54d\", \"authority\": \"system\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"be10f67d10e9d0147f09c723bb7f94b9d2e2d5145e986adc19e867b54c41b6cf\" }, { \"id\": \"policy.effective\", \"section\": \"enforced_policy\", \"source\": \"effective_run_policy\", \"authority\": \"policy\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"6feeb99001f84b3a7809ab761daf88bd6620f37997fda67d84ce317c64e79bf5\" }, { \"id\": \"skills.catalog\", \"section\": \"skill_catalog\", \"source\": \"eligible_skill_registry\", \"authority\": \"skill\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"d77ed6dbeef567fdfafab070e307d297b37ad4baaa1ddd06bcacff342b99ce29\" } ], \"counts\": { \"total\": 3, \"by_section\": { \"agent_identity\": 1, \"enforced_policy\": 1, \"skill_catalog\": 1 }, \"by_authority\": { \"policy\": 1, \"skill\": 1, \"system\": 1 } }, \"budgets\": { \"rendered_bytes\": 55826, \"rendered_characters\": 55322, \"context_window_tokens\": 1048576 }, \"selected_skills\": [], \"selected_tools\": [ \"a2ui_render\", \"activate_skill\" ], \"warnings\": [] } } }"
                    - generic "Provider attempt manifest" [ref=e290]:
                      - generic "JSON" [ref=e298]:
                        - code [ref=e299]: "{ \"schema_version\": \"uar.attempt-manifest.v1\", \"destination_model\": \"MiniMax-M3\", \"budgeting\": { \"label\": \"legacy_unbudgeted\", \"contract_present\": false, \"fit_guarantee\": false }, \"warning\": \"Provider dispatch is using the legacy unbudgeted path\" }"
                    - generic "budget_receipt" [ref=e300]:
                      - generic "JSON" [ref=e308]:
                        - code [ref=e309]: "{ \"source\": \"uar.request\", \"payload\": { \"schema_version\": \"uar.request-budget.v1\", \"representation\": \"final_provider_request\", \"destination_model\": \"MiniMax-M3\", \"serialized_request\": true, \"count_quality\": \"approximate\", \"counting_revision\": \"approximate-cl100k-json-v1\", \"counted_input_tokens\": 14270, \"serialized_bytes\": 62434, \"serialized_sha256\": \"579fd0adfa9a1b03b4a037457374ddf72579ed49f370de6f5757151f25b2270c\", \"tools_included\": true, \"instructions_included\": true, \"fit_guarantee\": false, \"reserved_output_tokens\": null, \"output_ceiling_enforced\": true } }"
                    - paragraph [ref=e312]: Calc●
                - generic "Thinking..." [ref=e315]
                - generic [ref=e325]:
                  - button "Scroll to bottom" [ref=e326] [cursor=pointer]
                  - generic [ref=e328]:
                    - button [ref=e329]
                    - generic [ref=e330]:
                      - textbox "Message input" [active] [ref=e331]:
                        - /placeholder: Send a message…
                      - generic [ref=e332]:
                        - button "Attach file" [ref=e333] [cursor=pointer]
                        - button "Disable memory for this message" [ref=e335] [cursor=pointer]:
                          - generic [ref=e336]: Memory ON – click to disable
                        - generic [ref=e337]: Generating…
                        - button "Stop generating" [ref=e342] [cursor=pointer]
                  - generic [ref=e343]:
                    - button "Toggle Knowledge" [ref=e344] [cursor=pointer]:
                      - generic [ref=e345]: Knowledge
                    - button "Toggle Tools" [ref=e346] [cursor=pointer]:
                      - generic [ref=e347]: Tools
                      - generic [ref=e348]: "1"
                    - button "Toggle Skills" [ref=e349] [cursor=pointer]:
                      - generic [ref=e350]: Skills
                    - button "Enable Web Search" [ref=e351] [cursor=pointer]:
                      - generic [ref=e352]: Web Search
                    - button "Disable Memory" [ref=e353] [cursor=pointer]:
                      - generic [ref=e354]: Memory
  - region "Notifications alt+T"
```

# Test source

```ts
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
  131 |   await expect(s.surface.locator('[data-a2ui-surface-state="ready"]')).toBeVisible();
> 132 |   await expect(s.surface).toContainText('Parcel calculator');
      |                           ^ Error: expect(locator).toContainText(expected) failed
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
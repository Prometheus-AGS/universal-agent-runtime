# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: features/browser-history-reload.feature.spec.js >> A browser conversation reloads from saved history >> Reopen a saved conversation after reload and continue from its earlier fact
- Location: target/manual-browser-history-reload-generated/features/browser-history-reload.feature.spec.js:6:3

# Error details

```
Error: expect(locator).toHaveCount(expected) failed

Locator:  locator('[data-slot="aui_user-message-root"][data-role="user"]')
Expected: 2
Received: 3
Timeout:  120000ms

Call log:
  - Expect "toHaveCount" with timeout 120000ms
  - waiting for locator('[data-slot="aui_user-message-root"][data-role="user"]')
    243 × locator resolved to 3 elements
        - unexpected value "3"

```

# Page snapshot

```yaml
- generic [ref=f1e3]:
  - generic [ref=f1e4]:
    - link "Skip to content" [ref=f1e5] [cursor=pointer]:
      - /url: "#shell-main-content"
    - generic [ref=f1e6]:
      - navigation "Primary navigation" [ref=f1e7]:
        - img "Universal Agent Runtime" [ref=f1e9]:
          - img [ref=f1e10]:
            - generic [ref=f1e15]: Universal Agent Runtime
            - generic [ref=f1e16]: // UAR RUNTIME
        - generic [ref=f1e17]:
          - generic [ref=f1e18]:
            - paragraph [ref=f1e19]: Work
            - link "Chat" [ref=f1e20] [cursor=pointer]:
              - /url: /threads
            - link "Knowledge" [ref=f1e24] [cursor=pointer]:
              - /url: /admin/knowledge
            - link "Agents" [ref=f1e28] [cursor=pointer]:
              - /url: /admin/agents
            - link "Runs" [ref=f1e33] [cursor=pointer]:
              - /url: /admin/runs
          - generic [ref=f1e40]:
            - paragraph [ref=f1e41]: Configure
            - link "Providers" [ref=f1e42] [cursor=pointer]:
              - /url: /admin/providers
            - link "MCP & tools" [ref=f1e47] [cursor=pointer]:
              - /url: /admin/tools
            - link "Skills" [ref=f1e59] [cursor=pointer]:
              - /url: /admin/skills
            - link "Presentations" [ref=f1e64] [cursor=pointer]:
              - /url: /admin/presentations
            - link "Runtime settings" [ref=f1e76] [cursor=pointer]:
              - /url: /admin/settings
          - generic [ref=f1e81]:
            - paragraph [ref=f1e82]: System
            - link "About" [ref=f1e83] [cursor=pointer]:
              - /url: /about
        - generic [ref=f1e89]:
          - generic [ref=f1e90]: Ready
          - generic [ref=f1e91]: Embedded · local
        - button "Collapse navigation" [expanded] [ref=f1e92] [cursor=pointer]
      - generic [ref=f1e95]:
        - banner [ref=f1e96]:
          - navigation "Breadcrumb" [ref=f1e98]:
            - list [ref=f1e99]:
              - listitem [ref=f1e100]: Workspace
              - listitem [ref=f1e101]: /
              - listitem [ref=f1e102]:
                - heading "Chat" [level=1] [ref=f1e103]
          - button "Open command palette" [ref=f1e104] [cursor=pointer]:
            - generic [ref=f1e108]: Command
            - generic [ref=f1e109]: ⌘K
          - button "Dark mode" [ref=f1e110] [cursor=pointer]
        - main [ref=f1e111]:
          - generic [ref=f1e113]:
            - complementary [ref=f1e114]:
              - generic [ref=f1e115]:
                - generic [ref=f1e116]: Threads
                - button "New thread" [ref=f1e117] [cursor=pointer]
              - textbox "Search threads" [ref=f1e123]:
                - /placeholder: Search threads…
              - generic "Thread list" [ref=f1e124]:
                - generic [ref=f1e125]:
                  - button "New conversation just now Default Assistant · <think>We need answer only JSON with reference and..." [ref=f1e126] [cursor=pointer]:
                    - generic [ref=f1e127]:
                      - generic [ref=f1e128]:
                        - paragraph [ref=f1e129]: New conversation
                        - generic [ref=f1e130]: just now
                      - paragraph [ref=f1e131]: Default Assistant · <think>We need answer only JSON with reference and...
                  - button "Actions for New conversation"
            - region "Conversation" [ref=f1e132]:
              - generic [ref=f1e133]:
                - button "Select agent" [ref=f1e134] [cursor=pointer]:
                  - generic [ref=f1e135]:
                    - generic [ref=f1e136]: Browser history 83e4619c
                    - generic [ref=f1e137]: liter/MiniMax-M3
                - button "Session configuration" [ref=f1e140] [cursor=pointer]
              - generic [ref=f1e143]:
                - generic [ref=f1e144]:
                  - paragraph [ref=f1e149]: "Remember for this conversation: reference CASE-f19b6bbd-d187-46da-9026-67de81b1c809; first quantity 342; second quantity 834. Acknowledge only READY without repeating any facts."
                  - generic [ref=f1e151]:
                    - region "A2UI display artifact" [ref=f1e152]:
                      - generic [ref=f1e153]:
                        - generic [ref=f1e157]: Artifact
                        - generic [ref=f1e158]: A2UI v0.9.1 · rendered · surface
                      - group [ref=f1e161]:
                        - generic [ref=f1e164]:
                          - heading "Effective run policy" [level=2] [ref=f1e166]
                          - generic [ref=f1e167]: Resolved policy v1 · A2UI v0.9.1
                          - generic [ref=f1e171]:
                            - heading "Execution" [level=3] [ref=f1e173]
                            - paragraph [ref=f1e175]: Chat mode · agent
                            - paragraph [ref=f1e177]: Model · liter / MiniMax-M3
                            - paragraph [ref=f1e179]: Agent · disposable-browser-history-83e4619c-87dc-4bed-9dc1-ff8156f8d022
                          - generic [ref=f1e183]:
                            - heading "Available capabilities" [level=3] [ref=f1e185]
                            - paragraph [ref=f1e187]: Skills · none · 0 available · agent scope
                            - paragraph [ref=f1e189]: Tools · none · 0 available · agent scope
                            - paragraph [ref=f1e191]: MCP servers · none · 0 available · agent scope
                            - paragraph [ref=f1e193]: Knowledge bases · none · 0 available · agent scope
                          - generic [ref=f1e197]:
                            - heading "Runtime controls" [level=3] [ref=f1e199]
                            - paragraph [ref=f1e201]: Memory · off
                            - paragraph [ref=f1e203]: Prompt caching · off
                            - paragraph [ref=f1e205]: Context · none
                            - paragraph [ref=f1e207]: Tool approval · auto
                          - generic [ref=f1e211]:
                            - heading "Warnings · 0" [level=3] [ref=f1e213]
                            - paragraph [ref=f1e215]: No policy warnings.
                    - generic "Turn manifest" [ref=f1e216]:
                      - generic "JSON" [ref=f1e224]:
                        - code [ref=f1e225]: "{ \"schema_version\": 1, \"manifest_hash\": \"d64021227d14ba397624b1bc16e5e628eda8115c60fbeb3499ed4b9d9891afd3\", \"fragments\": [ { \"id\": \"agent.identity\", \"section\": \"agent_identity\", \"source\": \"artifact:disposable-browser-history-83e4619c-87dc-4bed-9dc1-ff8156f8d022\", \"authority\": \"system\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"37cfc681cda8f116945b86f56941533388f97cb53a9d64d412fc62f1e66f0bb0\" }, { \"id\": \"policy.effective\", \"section\": \"enforced_policy\", \"source\": \"effective_run_policy\", \"authority\": \"policy\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"fd49500cde39a390c18b2ddf7b0268890be82e464d0a25005ea1a71659172314\" }, { \"id\": \"world_state.current_time\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"1d723cd58ebf2b918f9ce6aaeecfe950df92ee74f4fc9f94cb99ed97f17bd59d\" }, { \"id\": \"world_state.environment\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"60dd57dd0b5613817cff0f9517e6e5ec987d6ee6cd6e0bc93bbfbd5d65d5e859\" }, { \"id\": \"world_state.permissions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"1086bd5826885dcb64b18e69a018500cec2e60dcae9d0473139c3ca6d1a9425a\" }, { \"id\": \"world_state.project_instructions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\" } ], \"counts\": { \"total\": 6, \"by_section\": { \"agent_identity\": 1, \"enforced_policy\": 1, \"world_state\": 4 }, \"by_authority\": { \"host\": 4, \"policy\": 1, \"system\": 1 } }, \"budgets\": { \"rendered_bytes\": 2542, \"rendered_characters\": 2542, \"context_window_tokens\": 1048576 }, \"selected_skills\": [], \"selected_tools\": [], \"warnings\": [] }"
                    - generic "resolved_step" [ref=f1e226]:
                      - generic "JSON" [ref=f1e234]:
                        - code [ref=f1e235]: "{ \"source\": \"uar.turn\", \"payload\": { \"step\": 1, \"model\": \"MiniMax-M3\", \"mcp_catalog\": [], \"manifest\": { \"schema_version\": 1, \"manifest_hash\": \"0db31baf04f4e7c4d62a3ef84a76ecf6c51903c10c6219a89ac3775cb115b2f6\", \"fragments\": [ { \"id\": \"agent.identity\", \"section\": \"agent_identity\", \"source\": \"artifact:disposable-browser-history-83e4619c-87dc-4bed-9dc1-ff8156f8d022\", \"authority\": \"system\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"37cfc681cda8f116945b86f56941533388f97cb53a9d64d412fc62f1e66f0bb0\" }, { \"id\": \"policy.effective\", \"section\": \"enforced_policy\", \"source\": \"effective_run_policy\", \"authority\": \"policy\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"fd49500cde39a390c18b2ddf7b0268890be82e464d0a25005ea1a71659172314\" }, { \"id\": \"world_state.current_time\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"1d723cd58ebf2b918f9ce6aaeecfe950df92ee74f4fc9f94cb99ed97f17bd59d\" }, { \"id\": \"world_state.environment\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"60dd57dd0b5613817cff0f9517e6e5ec987d6ee6cd6e0bc93bbfbd5d65d5e859\" }, { \"id\": \"world_state.permissions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"1086bd5826885dcb64b18e69a018500cec2e60dcae9d0473139c3ca6d1a9425a\" }, { \"id\": \"world_state.project_instructions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\" } ], \"counts\": { \"total\": 6, \"by_section\": { \"agent_identity\": 1, \"enforced_policy\": 1, \"world_state\": 4 }, \"by_authority\": { \"host\": 4, \"policy\": 1, \"system\": 1 } }, \"budgets\": { \"rendered_bytes\": 2440, \"rendered_characters\": 2440, \"context_window_tokens\": 1048576 }, \"selected_skills\": [], \"selected_tools\": [], \"warnings\": [] } } }"
                    - generic "Provider attempt manifest" [ref=f1e236]:
                      - generic "JSON" [ref=f1e244]:
                        - code [ref=f1e245]: "{ \"schema_version\": \"uar.attempt-manifest.v1\", \"destination_model\": \"MiniMax-M3\", \"budgeting\": { \"label\": \"legacy_unbudgeted\", \"contract_present\": false, \"fit_guarantee\": false }, \"warning\": \"Provider dispatch is using the legacy unbudgeted path\" }"
                    - generic "budget_receipt" [ref=f1e246]:
                      - generic "JSON" [ref=f1e254]:
                        - code [ref=f1e255]: "{ \"source\": \"uar.request\", \"payload\": { \"schema_version\": \"uar.request-budget.v1\", \"representation\": \"final_provider_request\", \"destination_model\": \"MiniMax-M3\", \"serialized_request\": true, \"count_quality\": \"approximate\", \"counting_revision\": \"approximate-cl100k-json-v1\", \"counted_input_tokens\": 721, \"serialized_bytes\": 2971, \"serialized_sha256\": \"99c6fa5a4616a6e14caedaf3fa5110e50c32746acc9bc0cccb0eaaaab31542cf\", \"tools_included\": true, \"instructions_included\": true, \"fit_guarantee\": false, \"reserved_output_tokens\": null, \"output_ceiling_enforced\": true } }"
                    - paragraph [ref=f1e258]: READY
                    - paragraph [ref=f1e259]: 847 input · 3 output · 850 total · MiniMax-M3
                  - paragraph [ref=f1e265]: What reference did I previously give you, and what is the sum of the two quantities? Return only JSON with reference and total.
                  - paragraph [ref=f1e270]: What reference did I previously give you, and what is the sum of the two quantities? Return only JSON with reference and total.
                  - generic [ref=f1e271]:
                    - generic [ref=f1e272]:
                      - region "A2UI display artifact" [ref=f1e273]:
                        - generic [ref=f1e274]:
                          - generic [ref=f1e278]: Artifact
                          - generic [ref=f1e279]: A2UI v0.9.1 · rendered · surface
                        - group [ref=f1e282]:
                          - generic [ref=f1e285]:
                            - heading "Effective run policy" [level=2] [ref=f1e287]
                            - generic [ref=f1e288]: Resolved policy v1 · A2UI v0.9.1
                            - generic [ref=f1e292]:
                              - heading "Execution" [level=3] [ref=f1e294]
                              - paragraph [ref=f1e296]: Chat mode · agent
                              - paragraph [ref=f1e298]: Model · liter / MiniMax-M3
                              - paragraph [ref=f1e300]: Agent · disposable-browser-history-83e4619c-87dc-4bed-9dc1-ff8156f8d022
                            - generic [ref=f1e304]:
                              - heading "Available capabilities" [level=3] [ref=f1e306]
                              - paragraph [ref=f1e308]: Skills · none · 0 available · agent scope
                              - paragraph [ref=f1e310]: Tools · none · 0 available · agent scope
                              - paragraph [ref=f1e312]: MCP servers · none · 0 available · agent scope
                              - paragraph [ref=f1e314]: Knowledge bases · none · 0 available · agent scope
                            - generic [ref=f1e318]:
                              - heading "Runtime controls" [level=3] [ref=f1e320]
                              - paragraph [ref=f1e322]: Memory · off
                              - paragraph [ref=f1e324]: Prompt caching · off
                              - paragraph [ref=f1e326]: Context · none
                              - paragraph [ref=f1e328]: Tool approval · auto
                            - generic [ref=f1e332]:
                              - heading "Warnings · 0" [level=3] [ref=f1e334]
                              - paragraph [ref=f1e336]: No policy warnings.
                      - generic "Turn manifest" [ref=f1e337]:
                        - generic "JSON" [ref=f1e345]:
                          - code [ref=f1e346]: "{ \"schema_version\": 1, \"manifest_hash\": \"8446b2b0f2c2615c37f3cea064e488204836c5644b617645005cb8946c0289cc\", \"fragments\": [ { \"id\": \"agent.identity\", \"section\": \"agent_identity\", \"source\": \"artifact:disposable-browser-history-83e4619c-87dc-4bed-9dc1-ff8156f8d022\", \"authority\": \"system\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"37cfc681cda8f116945b86f56941533388f97cb53a9d64d412fc62f1e66f0bb0\" }, { \"id\": \"policy.effective\", \"section\": \"enforced_policy\", \"source\": \"effective_run_policy\", \"authority\": \"policy\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"fd49500cde39a390c18b2ddf7b0268890be82e464d0a25005ea1a71659172314\" }, { \"id\": \"world_state.current_time\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"af65b5c90e1aa4ffcee9cd463340aefc8a5ae9cbfc6cc9c00de5515f6179cb20\" } ], \"counts\": { \"total\": 3, \"by_section\": { \"agent_identity\": 1, \"enforced_policy\": 1, \"world_state\": 1 }, \"by_authority\": { \"host\": 1, \"policy\": 1, \"system\": 1 } }, \"budgets\": { \"rendered_bytes\": 1395, \"rendered_characters\": 1395, \"context_window_tokens\": 1048576 }, \"selected_skills\": [], \"selected_tools\": [], \"warnings\": [] }"
                      - generic "resolved_step" [ref=f1e347]:
                        - generic "JSON" [ref=f1e355]:
                          - code [ref=f1e356]: "{ \"source\": \"uar.turn\", \"payload\": { \"step\": 1, \"model\": \"MiniMax-M3\", \"mcp_catalog\": [], \"manifest\": { \"schema_version\": 1, \"manifest_hash\": \"6edd571c6d17ccb112aa44aec212938e5fb740695aebb74ceb68e65acc962ec6\", \"fragments\": [ { \"id\": \"agent.identity\", \"section\": \"agent_identity\", \"source\": \"artifact:disposable-browser-history-83e4619c-87dc-4bed-9dc1-ff8156f8d022\", \"authority\": \"system\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"37cfc681cda8f116945b86f56941533388f97cb53a9d64d412fc62f1e66f0bb0\" }, { \"id\": \"policy.effective\", \"section\": \"enforced_policy\", \"source\": \"effective_run_policy\", \"authority\": \"policy\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"fd49500cde39a390c18b2ddf7b0268890be82e464d0a25005ea1a71659172314\" }, { \"id\": \"world_state.current_time\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"af65b5c90e1aa4ffcee9cd463340aefc8a5ae9cbfc6cc9c00de5515f6179cb20\" } ], \"counts\": { \"total\": 3, \"by_section\": { \"agent_identity\": 1, \"enforced_policy\": 1, \"world_state\": 1 }, \"by_authority\": { \"host\": 1, \"policy\": 1, \"system\": 1 } }, \"budgets\": { \"rendered_bytes\": 2734, \"rendered_characters\": 2734, \"context_window_tokens\": 1048576 }, \"selected_skills\": [], \"selected_tools\": [], \"warnings\": [] } } }"
                      - generic "Provider attempt manifest" [ref=f1e357]:
                        - generic "JSON" [ref=f1e365]:
                          - code [ref=f1e366]: "{ \"schema_version\": \"uar.attempt-manifest.v1\", \"destination_model\": \"MiniMax-M3\", \"budgeting\": { \"label\": \"legacy_unbudgeted\", \"contract_present\": false, \"fit_guarantee\": false }, \"warning\": \"Provider dispatch is using the legacy unbudgeted path\" }"
                      - generic "budget_receipt" [ref=f1e367]:
                        - generic "JSON" [ref=f1e375]:
                          - code [ref=f1e376]: "{ \"source\": \"uar.request\", \"payload\": { \"schema_version\": \"uar.request-budget.v1\", \"representation\": \"final_provider_request\", \"destination_model\": \"MiniMax-M3\", \"serialized_request\": true, \"count_quality\": \"approximate\", \"counting_revision\": \"approximate-cl100k-json-v1\", \"counted_input_tokens\": 810, \"serialized_bytes\": 3362, \"serialized_sha256\": \"fb107b8925547840dad89564d977842e0b8aa81c3b3c56a0821868e34c37b2a3\", \"tools_included\": true, \"instructions_included\": true, \"fit_guarantee\": false, \"reserved_output_tokens\": null, \"output_ceiling_enforced\": true } }"
                      - generic [ref=f1e378]:
                        - paragraph [ref=f1e379]: "We need answer only JSON with reference and total. Facts: reference CASE-f19b6bbd-d187-46da-9026-67de81b1c809; quantities 342 and 834. Sum 342+834=1176. Need exact JSON fields requested likely {\"reference\":\"...\",\"total\":1176}. Ensure only JSON no markdown."
                        - paragraph [ref=f1e380]: "{\"reference\":\"CASE-f19b6bbd-d187-46da-9026-67de81b1c809\",\"total\":1176}"
                      - paragraph [ref=f1e381]: 936 input · 110 output · 1,046 total · MiniMax-M3
                    - generic [ref=f1e383]:
                      - button "Copy" [ref=f1e384] [cursor=pointer]
                      - generic [ref=f1e386]:
                        - 'generic "Answered by agent: Browser history 83e4619c" [ref=f1e387]': Browser history 83e4619c
                        - 'generic "Model: MiniMax-M3" [ref=f1e391]': MiniMax-M3
                        - 'generic "Tokens — in: 936, out: 110, total: 1046" [ref=f1e395]': "936110"
                - generic [ref=f1e400]:
                  - button "Scroll to bottom" [ref=f1e401] [cursor=pointer]
                  - generic [ref=f1e403]:
                    - button [ref=f1e404]
                    - generic [ref=f1e405]:
                      - textbox "Message input" [active] [ref=f1e406]:
                        - /placeholder: Send a message…
                      - generic [ref=f1e407]:
                        - button "Attach file" [ref=f1e408] [cursor=pointer]
                        - button "Disable memory for this message" [ref=f1e410] [cursor=pointer]:
                          - generic [ref=f1e411]: Memory ON – click to disable
                        - button "Send message" [disabled]
                  - generic [ref=f1e412]:
                    - button "Toggle Knowledge" [ref=f1e413] [cursor=pointer]:
                      - generic [ref=f1e414]: Knowledge
                    - button "Toggle Tools" [ref=f1e415] [cursor=pointer]:
                      - generic [ref=f1e416]: Tools
                      - generic [ref=f1e417]: "1"
                    - button "Toggle Skills" [ref=f1e418] [cursor=pointer]:
                      - generic [ref=f1e419]: Skills
                    - button "Enable Web Search" [ref=f1e420] [cursor=pointer]:
                      - generic [ref=f1e421]: Web Search
                    - button "Disable Memory" [ref=f1e422] [cursor=pointer]:
                      - generic [ref=f1e423]: Memory
  - region "Notifications alt+T"
```

# Test source

```ts
  140 |   agent.policy.provider = { default: { provider: process.env.UAR_MODEL.slice(0, split), model: process.env.UAR_MODEL.slice(split + 1) }, fallbacks: [] };
  141 |   agent.policy.skills.prefer = [];
  142 |   agent.memory.kb.enabled = false;
  143 |   // Long-term memory and retrieval are off so a recalled fact can only come from the session history.
  144 |   agent.extensions = { ...agent.extensions, 'uar.run_policy': { version: 1,
  145 |     context_strategy: { type: 'none' }, memory_enabled: false,
  146 |     skills: { mode: 'none' }, tools: { mode: 'none' },
  147 |     mcp_servers: { mode: 'none' }, knowledge_bases: { mode: 'none' }, presentations: { mode: 'none' } } };
  148 |   const saved = await api(s, 'POST', '/api/agents', agent, 201);
  149 |   assert.equal(saved.id, s.agentId);
  150 |   await page.goto('/threads', { waitUntil: 'domcontentloaded' });
  151 |   await page.getByRole('button', { name: 'New thread', exact: true }).click();
  152 |   const selection = observeResponse(page, r => r.request().method() === 'POST' && /\/api\/uar\/sessions\/[^/]+\/agent-config$/.test(new URL(r.url()).pathname));
  153 |   await page.getByRole('button', { name: 'Select agent', exact: true }).click();
  154 |   await page.getByPlaceholder('Search agents...').fill(s.title);
  155 |   await page.getByRole('option').filter({ hasText: s.title }).click();
  156 |   const saveResponse = await selection();
  157 |   assert.equal(saveResponse.status(), 200, 'The browser must save the session agent selection');
  158 |   s.evidence.agent_selection_save = { path: new URL(saveResponse.url()).pathname, request: saveResponse.request().postDataJSON() };
  159 |   await expect(page.getByRole('button', { name: 'Select agent', exact: true })).toContainText(s.title);
  160 | });
  161 | 
  162 | When('I give the agent a fresh fact in the browser', async ({ page, scenario: s }) => {
  163 |   s.reference = `CASE-${randomUUID()}`;
  164 |   s.left = randomInt(101, 400);
  165 |   s.right = randomInt(501, 900);
  166 |   s.expected = { reference: s.reference, total: s.left + s.right };
  167 |   s.factMessage = `Remember for this conversation: reference ${s.reference}; first quantity ${s.left}; second quantity ${s.right}. Acknowledge only READY without repeating any facts.`;
  168 |   s.evidence.expected = s.expected;
  169 |   const response = observeResponse(page, isCompletion);
  170 |   const title = observeResponse(page, r => r.request().method() === 'POST' && new URL(r.url()).pathname === '/api/generate-title', 240_000);
  171 |   await page.getByRole('textbox', { name: 'Message input', exact: true }).fill(s.factMessage);
  172 |   await page.getByRole('button', { name: 'Send message', exact: true }).click();
  173 |   s.first = await stream(s, 'fact', await response());
  174 |   s.threadId = s.first.sessionHeader;
  175 |   assert.ok(s.threadId, 'The browser must send its conversation id');
  176 |   assert.equal(s.first.request.message, s.factMessage);
  177 |   assert.match(s.first.text, /^READY\b/i, `Fact acknowledgement: ${s.first.text.slice(0, 200)}`);
  178 |   s.evidence.thread_id = s.threadId;
  179 |   s.evidence.fact_run_id = s.first.runId;
  180 |   s.evidence.fact_answer = s.first.text;
  181 |   await expect(assistantMessages(page).last()).toContainText(/READY/i);
  182 |   try { s.evidence.title_generation_status = (await title()).status(); }
  183 |   catch (error) { s.evidence.title_generation_status = `not observed: ${String(error).slice(0, 200)}`; }
  184 |   const active = threadList(page).locator('button[aria-current="page"] p.truncate');
  185 |   await expect(active).toBeVisible();
  186 |   await expect(active).not.toHaveText('New conversation', { timeout: 30_000 }).catch(() => undefined);
  187 |   s.threadTitle = (await active.textContent()).trim();
  188 |   s.evidence.thread_title = s.threadTitle;
  189 |   s.evidence.before_reload = { user_messages: await userMessages(page).count(), assistant_messages: await assistantMessages(page).count() };
  190 |   await shot(page, s, 'before-reload.png');
  191 | });
  192 | 
  193 | When('I reload the page and reopen that conversation from the thread list', async ({ page, scenario: s }) => {
  194 |   s.postsBeforeReload = s.completionPosts.length;
  195 |   await page.reload({ waitUntil: 'domcontentloaded' });
  196 |   await expect(threadList(page)).toHaveAttribute('aria-busy', 'false');
  197 |   const entry = threadList(page).getByRole('button').filter({ has: page.getByText(s.threadTitle, { exact: true }) });
  198 |   await expect(entry).toHaveCount(1);
  199 |   s.evidence.thread_list_after_reload = await threadList(page).innerText();
  200 |   const configLoad = observeResponse(page, r => r.request().method() === 'GET' && new URL(r.url()).pathname === `/api/uar/sessions/${s.threadId}/agent-config`, 30_000);
  201 |   await entry.click();
  202 |   await expect(entry).toHaveAttribute('aria-current', 'page');
  203 |   try { s.evidence.agent_config_reload_status = (await configLoad()).status(); }
  204 |   catch (error) { s.evidence.agent_config_reload_status = `not observed after click: ${String(error).slice(0, 200)}`; }
  205 | });
  206 | 
  207 | Then('the reopened conversation shows the saved turns without a new run', async ({ page, scenario: s }) => {
  208 |   await expect(userMessages(page)).toHaveCount(1);
  209 |   await expect(assistantMessages(page)).toHaveCount(1);
  210 |   await expect(userMessages(page).first()).toContainText(s.reference);
  211 |   await expect(assistantMessages(page).first()).toContainText(/READY/i);
  212 |   await expect(page.getByRole('button', { name: 'Select agent', exact: true })).toContainText(s.title);
  213 |   assert.equal(s.completionPosts.length, s.postsBeforeReload, 'Reload and reopen must not start a run');
  214 |   s.evidence.reopened = { user_messages: await userMessages(page).allInnerTexts(),
  215 |     assistant_messages: await assistantMessages(page).allInnerTexts(),
  216 |     agent_selector: await page.getByRole('button', { name: 'Select agent', exact: true }).innerText() };
  217 |   await shot(page, s, 'reopened-after-reload.png');
  218 | });
  219 | 
  220 | When('I send a follow-up that contains none of the earlier facts', async ({ page, scenario: s }) => {
  221 |   s.followUp = 'What reference did I previously give you, and what is the sum of the two quantities? Return only JSON with reference and total.';
  222 |   for (const fact of [s.reference, String(s.left), String(s.right)]) assert.ok(!s.followUp.includes(fact));
  223 |   const response = observeResponse(page, isCompletion);
  224 |   await page.getByRole('textbox', { name: 'Message input', exact: true }).fill(s.followUp);
  225 |   await page.getByRole('button', { name: 'Send message', exact: true }).click();
  226 |   s.second = await stream(s, 'follow_up', await response());
  227 |   const body = JSON.stringify(s.second.request);
  228 |   for (const fact of [s.reference, String(s.left), String(s.right)]) assert.ok(!body.includes(fact), `Follow-up request replayed ${fact}`);
  229 |   assert.equal(s.second.request.message, s.followUp);
  230 |   assert.equal(s.second.sessionHeader, s.threadId, 'The follow-up must continue the reopened conversation');
  231 |   assert.notEqual(s.second.runId, s.first.runId);
  232 |   s.evidence.follow_up_request = s.second.request;
  233 |   s.evidence.follow_up_run_id = s.second.runId;
  234 | });
  235 | 
  236 | Then('the follow-up answer uses the earlier fact from the saved server history', async ({ page, scenario: s }) => {
  237 |   const answer = parseJsonAnswer(s.second.text);
  238 |   s.evidence.follow_up_answer = answer;
  239 |   assert.deepEqual({ reference: answer.reference, total: answer.total }, s.expected);
> 240 |   await expect(userMessages(page)).toHaveCount(2);
      |                                    ^ Error: expect(locator).toHaveCount(expected) failed
  241 |   await expect(assistantMessages(page)).toHaveCount(2);
  242 |   await expect(assistantMessages(page).last()).toContainText(s.reference);
  243 |   await expect(assistantMessages(page).last()).toContainText(String(s.expected.total));
  244 |   assert.equal(s.completionPosts.length, 2, 'Exactly two browser runs: the fact and the follow-up');
  245 |   for (const [label, runId] of [['fact_run', s.first.runId], ['follow_up_run', s.second.runId]]) {
  246 |     const run = await api(s, 'GET', `/api/uar/runs/${runId}`);
  247 |     assert.equal(run.conversation_id, s.threadId, `${label} conversation`);
  248 |     assert.equal(run.agent_id, s.agentId, `${label} agent`);
  249 |     assert.equal(run.status, 'done', `${label} status`);
  250 |     s.evidence[label] = run;
  251 |   }
  252 |   const runs = await api(s, 'GET', '/api/uar/runs');
  253 |   const owned = runs.filter(run => run.conversation_id === s.threadId).map(run => run.run_id).sort();
  254 |   s.evidence.conversation_runs = owned;
  255 |   // The run listing holds only retained active-run state, so it can omit finished runs; it must never add one.
  256 |   const known = new Set([s.first.runId, s.second.runId]);
  257 |   assert.ok(owned.every(id => known.has(id)), `Unexpected extra run in the conversation: ${owned.join(', ')}`);
  258 |   s.evidence.fact_source = 'server session history: follow-up request carried only the new question; memory, retrieval and context strategy disabled by run policy';
  259 |   s.evidence.result = 'VERIFIED';
  260 |   await shot(page, s, 'follow-up-answer.png');
  261 | });
  262 | 
```
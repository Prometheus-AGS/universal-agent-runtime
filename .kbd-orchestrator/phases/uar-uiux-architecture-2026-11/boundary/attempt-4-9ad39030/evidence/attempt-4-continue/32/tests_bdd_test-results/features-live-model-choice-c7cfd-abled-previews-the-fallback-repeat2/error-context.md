# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: features/live-model-choice-controls.feature.spec.js >> Live model choice, negative controls >> Clearing an override while the default's provider is disabled previews the fallback
- Location: tests/bdd/.features-gen/features/live-model-choice-controls.feature.spec.js:69:3

# Error details

```
Test timeout of 120000ms exceeded.
```

```
Error: locator.fill: Test timeout of 120000ms exceeded.
Call log:
  - waiting for getByRole('dialog').getByLabel('Thread name')

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
          - img [aria-hidden] [ref=e10]:
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
            - link "Teams" [ref=e33] [cursor=pointer]:
              - /url: /admin/teams
            - link "Runs" [ref=e40] [cursor=pointer]:
              - /url: /admin/runs
          - generic [ref=e47]:
            - paragraph [ref=e48]: Configure
            - link "Providers" [ref=e49] [cursor=pointer]:
              - /url: /admin/providers
            - link "MCP & tools" [ref=e54] [cursor=pointer]:
              - /url: /admin/tools
            - link "Skills" [ref=e66] [cursor=pointer]:
              - /url: /admin/skills
            - link "Presentations" [ref=e71] [cursor=pointer]:
              - /url: /admin/presentations
            - link "Runtime settings" [ref=e83] [cursor=pointer]:
              - /url: /admin/settings
          - generic [ref=e88]:
            - paragraph [ref=e89]: System
            - link "About" [ref=e90] [cursor=pointer]:
              - /url: /about
        - generic [ref=e96]:
          - generic [ref=e97]: Ready
          - generic [ref=e98]: Embedded · local
        - button "Collapse navigation" [expanded] [ref=e99] [cursor=pointer]
      - generic [ref=e102]:
        - banner [ref=e103]:
          - navigation "Breadcrumb" [ref=e105]:
            - list [ref=e106]:
              - listitem [ref=e107]: Workspace
              - listitem [aria-hidden] [ref=e108]: /
              - listitem [ref=e109]:
                - heading "Chat" [level=1] [ref=e110]
          - button "Open command palette" [ref=e111] [cursor=pointer]:
            - generic [ref=e115]: Command
            - generic [ref=e116]: ⌘K
          - button "Dark mode" [ref=e117] [cursor=pointer]
        - main [ref=e118]:
          - generic [ref=e120]:
            - complementary [ref=e121]:
              - generic [ref=e122]:
                - generic [ref=e123]: Threads
                - button "New thread" [ref=e124] [cursor=pointer]
              - textbox "Search threads" [ref=e130]:
                - /placeholder: Search threads…
              - generic "Thread list" [ref=e131]:
                - generic [ref=e132]:
                  - button "S1 c012 just now Default Assistant · bdd-reply-gpt-5.4-mini" [ref=e133] [cursor=pointer]:
                    - generic [ref=e134]:
                      - generic [ref=e135]:
                        - paragraph [ref=e136]: S1 c012
                        - generic [ref=e137]: just now
                      - paragraph [ref=e138]: Default Assistant · bdd-reply-gpt-5.4-mini
                  - button "Actions for S1 c012" [active]
            - region "Conversation" [ref=e139]:
              - generic [ref=e140]:
                - button "Select agent" [ref=e141] [cursor=pointer]:
                  - generic [ref=e142]: Default Assistant
                - button "Session configuration" [ref=e146] [cursor=pointer]
              - generic [ref=e149]:
                - generic [ref=e150]:
                  - paragraph [ref=e155]: S1 c012
                  - generic [ref=e156]:
                    - generic [ref=e157]:
                      - region "A2UI display artifact" [ref=e158]:
                        - generic [ref=e159]:
                          - generic [ref=e163]: Artifact
                          - generic [ref=e164]: A2UI 1.0 · rendered · effective_run_policy
                        - generic [ref=e166]:
                          - group [ref=e167]:
                            - generic [ref=e170]:
                              - paragraph [ref=e172]: Effective run policy
                              - generic [ref=e173]: Resolved policy v1 · A2UI 1.0
                              - generic [ref=e177]:
                                - paragraph [ref=e179]: Execution
                                - paragraph [ref=e181]: Chat mode · uar
                                - paragraph [ref=e183]: Model · openai / gpt-5.4-mini
                                - paragraph [ref=e185]: Agent · default-agent
                              - generic [ref=e189]:
                                - paragraph [ref=e191]: Available capabilities
                                - paragraph [ref=e193]: Skills · auto · 288 available · agent scope
                                - paragraph [ref=e195]: Tools · selected · 15 available · agent scope
                                - paragraph [ref=e197]: MCP servers · auto · 0 available · legacy scope
                                - paragraph [ref=e199]: Knowledge bases · auto · 0 available · agent scope
                              - generic [ref=e203]:
                                - paragraph [ref=e205]: Runtime controls
                                - paragraph [ref=e207]: Memory · on
                                - paragraph [ref=e209]: Prompt caching · off
                                - paragraph [ref=e211]: Context · sliding window
                                - paragraph [ref=e213]: Tool approval · auto
                              - generic [ref=e217]:
                                - paragraph [ref=e219]: Warnings · 0
                                - paragraph [ref=e221]: No policy warnings.
                          - status [ref=e222]: This surface has no publication identity. Request a fresh surface before acting.
                          - button "Recover action status" [ref=e223] [cursor=pointer]
                      - generic "Turn manifest" [ref=e224]:
                        - generic "JSON" [ref=e232]:
                          - code [ref=e233]: "{ \"schema_version\": 1, \"manifest_hash\": \"30e7bdf73537961bead9b88d79fe9f034a5c4a7f8f6161993d21eb9a0521d5b8\", \"fragments\": [ { \"id\": \"agent.identity\", \"section\": \"agent_identity\", \"source\": \"artifact:default-agent\", \"authority\": \"system\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"e9f6bc3f98833b312aa5f17108f5459fab5e8cfce631a3a48ed23a35ed5b413e\" }, { \"id\": \"policy.effective\", \"section\": \"enforced_policy\", \"source\": \"effective_run_policy\", \"authority\": \"policy\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"344e1f4e5fbfe62ad80659bce528ebdfa1ea098d403c9579db867609b918cfe6\" }, { \"id\": \"skills.catalog\", \"section\": \"skill_catalog\", \"source\": \"eligible_skill_registry\", \"authority\": \"skill\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"b1682ed3688a5f9d6f0fb040ba707bc068f05aaf207b88f588a2c1422e86d79e\" }, { \"id\": \"world_state.current_time\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"b7ad1fda1344b77151d89c44e40cb0a56ff357350d46d2f0f573534a16c2933c\" }, { \"id\": \"world_state.environment\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"f122476abc931a95b2343d1442fc641a99dcd84c974aa29aed030eb7236c49b9\" }, { \"id\": \"world_state.permissions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"9d48b593b41db02f2604616d5af722c6b1ea5e0c86695e3492a0707d9aebfe40\" }, { \"id\": \"world_state.project_instructions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\" } ], \"counts\": { \"total\": 7, \"by_section\": { \"agent_identity\": 1, \"enforced_policy\": 1, \"skill_catalog\": 1, \"world_state\": 4 }, \"by_authority\": { \"host\": 4, \"policy\": 1, \"skill\": 1, \"system\": 1 } }, \"budgets\": { \"rendered_bytes\": 50905, \"rendered_characters\": 50525, \"context_window_tokens\": 400000 }, \"selected_skills\": [], \"selected_tools\": [ \"a2ui_render\", \"activate_skill\", \"native__memory_capture_policy\", \"native__memory_delete_by_id\", \"native__memory_history\", \"native__memory_list\", \"native__memory_recall\", \"native__memory_save\", \"native__memory_update_by_id\", \"native_echo\", \"native_system_info\", \"session_search\", \"uar_compile\", \"uar_session_check_completeness\", \"uar_session_compile\", \"uar_session_update_section\" ], \"warnings\": [] }"
                      - generic "resolved_step" [ref=e234]:
                        - generic "JSON" [ref=e242]:
                          - code [ref=e243]: "{ \"source\": \"uar.turn\", \"payload\": { \"step\": 1, \"model\": \"gpt-5.4-mini\", \"mcp_catalog\": [], \"manifest\": { \"schema_version\": 1, \"manifest_hash\": \"452cbe4ff93f8a58fcb60ab815fa36956e68a03d33d379822fe1514933498dd9\", \"fragments\": [ { \"id\": \"agent.identity\", \"section\": \"agent_identity\", \"source\": \"artifact:default-agent\", \"authority\": \"system\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"e9f6bc3f98833b312aa5f17108f5459fab5e8cfce631a3a48ed23a35ed5b413e\" }, { \"id\": \"policy.effective\", \"section\": \"enforced_policy\", \"source\": \"effective_run_policy\", \"authority\": \"policy\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"344e1f4e5fbfe62ad80659bce528ebdfa1ea098d403c9579db867609b918cfe6\" }, { \"id\": \"skills.catalog\", \"section\": \"skill_catalog\", \"source\": \"eligible_skill_registry\", \"authority\": \"skill\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"b1682ed3688a5f9d6f0fb040ba707bc068f05aaf207b88f588a2c1422e86d79e\" }, { \"id\": \"world_state.current_time\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"b7ad1fda1344b77151d89c44e40cb0a56ff357350d46d2f0f573534a16c2933c\" }, { \"id\": \"world_state.environment\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"f122476abc931a95b2343d1442fc641a99dcd84c974aa29aed030eb7236c49b9\" }, { \"id\": \"world_state.permissions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"9d48b593b41db02f2604616d5af722c6b1ea5e0c86695e3492a0707d9aebfe40\" }, { \"id\": \"world_state.project_instructions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\" } ], \"counts\": { \"total\": 7, \"by_section\": { \"agent_identity\": 1, \"enforced_policy\": 1, \"skill_catalog\": 1, \"world_state\": 4 }, \"by_authority\": { \"host\": 4, \"policy\": 1, \"skill\": 1, \"system\": 1 } }, \"budgets\": { \"rendered_bytes\": 50909, \"rendered_characters\": 50529, \"context_window_tokens\": 400000 }, \"selected_skills\": [], \"selected_tools\": [ \"a2ui_render\", \"activate_skill\", \"native__memory_capture_policy\", \"native__memory_delete_by_id\", \"native__memory_history\", \"native__memory_list\", \"native__memory_recall\", \"native__memory_save\", \"native__memory_update_by_id\", \"native_echo\", \"native_system_info\", \"session_search\", \"uar_compile\", \"uar_session_check_completeness\", \"uar_session_compile\", \"uar_session_update_section\" ], \"warnings\": [] } } }"
                      - generic "Provider attempt manifest" [ref=e244]:
                        - generic "JSON" [ref=e252]:
                          - code [ref=e253]: "{ \"schema_version\": \"uar.attempt-manifest.v1\", \"destination_model\": \"openai/gpt-5.4-mini\", \"budgeting\": { \"label\": \"legacy_unbudgeted\", \"contract_present\": false, \"fit_guarantee\": false }, \"warning\": \"Provider dispatch is using the legacy unbudgeted path\" }"
                      - generic "budget_receipt" [ref=e254]:
                        - generic "JSON" [ref=e262]:
                          - code [ref=e263]: "{ \"source\": \"uar.request\", \"payload\": { \"schema_version\": \"uar.request-budget.v1\", \"representation\": \"final_provider_request\", \"destination_model\": \"openai/gpt-5.4-mini\", \"serialized_request\": true, \"count_quality\": \"approximate\", \"counting_revision\": \"approximate-cl100k-json-v1\", \"counted_input_tokens\": 15878, \"serialized_bytes\": 69268, \"serialized_sha256\": \"45f774a757ae595ef35ce89e528b6ab557b5a400893cd136a8d7d1e97ae44a44\", \"tools_included\": true, \"instructions_included\": true, \"fit_guarantee\": false, \"reserved_output_tokens\": null, \"output_ceiling_enforced\": true } }"
                      - paragraph [ref=e266]: bdd-reply-gpt-5.4-mini
                      - paragraph [ref=e267]:
                        - generic [ref=e268]: Answered by
                        - generic [ref=e269]: openai/gpt-5.4-mini
                        - generic [aria-hidden] [ref=e270]: ·
                        - generic [ref=e271]: configured fallback
                    - generic [ref=e273]:
                      - button "Copy" [ref=e274] [cursor=pointer]
                      - generic [ref=e276]:
                        - 'generic "Answered by agent: Default Assistant" [ref=e277]': Default Assistant
                        - 'generic "Model: gpt-5.4-mini" [ref=e281]': gpt-5.4-mini
                - generic [ref=e285]:
                  - button "Scroll to bottom" [ref=e286] [cursor=pointer]
                  - generic [ref=e288]:
                    - button [aria-hidden] [ref=e289]
                    - generic [ref=e290]:
                      - textbox "Message input" [ref=e291]:
                        - /placeholder: Send a message…
                      - generic [ref=e292]:
                        - button "Attach file" [ref=e293] [cursor=pointer]
                        - button "Disable memory for this message" [ref=e295] [cursor=pointer]:
                          - generic [ref=e296]: Memory ON – click to disable
                        - button "Send message" [disabled]
                  - generic [ref=e297]:
                    - button "Toggle Knowledge" [ref=e298] [cursor=pointer]:
                      - generic [ref=e299]: Knowledge
                    - button "Toggle Tools" [ref=e300] [cursor=pointer]:
                      - generic [ref=e301]: Tools
                      - generic [ref=e302]: "1"
                    - button "Toggle Skills" [ref=e303] [cursor=pointer]:
                      - generic [ref=e304]: Skills
                    - button "Enable Web Search" [ref=e305] [cursor=pointer]:
                      - generic [ref=e306]: Web Search
                    - button "Disable Memory" [ref=e307] [cursor=pointer]:
                      - generic [ref=e308]: Memory
  - region "Notifications alt+T"
```

# Test source

```ts
  70  |   return page.locator(CONVERSATION).innerText();
  71  | }
  72  | 
  73  | function repliesIn(text: string): string[] {
  74  |   return (text.match(REPLY_PATTERN) ?? []).map((reply) => reply.replace(/[._-]+$/, ''));
  75  | }
  76  | 
  77  | /** The newest `bdd-reply-...` text in the conversation transcript, or null. */
  78  | export async function newestReply(page: Page): Promise<string | null> {
  79  |   return repliesIn(await conversationText(page)).at(-1) ?? null;
  80  | }
  81  | 
  82  | /**
  83  |  * Notes how many replies the transcript already holds, and which assistant turns it shows,
  84  |  * before a message is sent.
  85  |  */
  86  | export async function noteRepliesBefore(page: Page, s1: S1Context, message: string): Promise<void> {
  87  |   s1.replyCountBefore.set(message, repliesIn(await conversationText(page)).length);
  88  |   const ids = (await assistantTurns(page)).map((turn) => turn.id).filter((id): id is string => id !== null);
  89  |   lastSent.set(s1, { message, assistantIdsBefore: new Set(ids) });
  90  | }
  91  | 
  92  | /**
  93  |  * Records the reply text the transcript shows for `message`: waits until the
  94  |  * transcript holds more `bdd-reply-...` strings than it did at send time, then
  95  |  * takes the newest. A later step compares it with the served model's reply text.
  96  |  */
  97  | export async function captureReply(page: Page, s1: S1Context, message: string): Promise<void> {
  98  |   const before = s1.replyCountBefore.get(message) ?? 0;
  99  |   await expect
  100 |     .poll(async () => repliesIn(await conversationText(page)).length, {
  101 |       message: `the transcript shows a reply to "${message}"`,
  102 |       timeout: 15_000,
  103 |     })
  104 |     .toBeGreaterThan(before);
  105 |   const reply = await newestReply(page);
  106 |   if (reply === null) throw new Error(`no reply text was found in the transcript for "${message}"`);
  107 |   s1.replyShown.set(message, reply);
  108 | }
  109 | 
  110 | /**
  111 |  * Sends a message and waits for the reply, as the shared `I send the message`
  112 |  * step does; during an S1 scenario it also records the reply text and, after a
  113 |  * conversation's first message, names the conversation's thread.
  114 |  */
  115 | export async function sendAndCapture(page: Page, s1: S1Context, message: string): Promise<void> {
  116 |   if (s1.active) await noteRepliesBefore(page, s1, message);
  117 |   await sendMessageAndWait(page, message);
  118 |   if (!s1.active) return;
  119 |   await finishSend(page, s1, message);
  120 | }
  121 | 
  122 | /** After a reply has finished: records its text and names the conversation's thread after its first message. */
  123 | export async function finishSend(page: Page, s1: S1Context, message: string): Promise<void> {
  124 |   await captureReply(page, s1, message);
  125 |   await materializeCurrentConversation(page, s1);
  126 | }
  127 | 
  128 | /** Sends a message and returns as soon as the run has started (the stop control is shown). */
  129 | export async function sendWithoutWaiting(page: Page, s1: S1Context, message: string): Promise<void> {
  130 |   await noteRepliesBefore(page, s1, message);
  131 |   await page.locator('[aria-label="Message input"]').fill(message);
  132 |   await page.keyboard.press('Enter');
  133 |   await expect(page.locator('[aria-label="Stop generating"]')).toBeVisible({ timeout: 10_000 });
  134 | }
  135 | 
  136 | // ---------------------------------------------------------------------------
  137 | // Conversations
  138 | //
  139 | // A thread is hidden from the sidebar until its first message is sent, so the
  140 | // harness keeps a name -> state handle per conversation and names the thread
  141 | // (through the sidebar's Rename dialog) after its first message.
  142 | // ---------------------------------------------------------------------------
  143 | 
  144 | /** Starts a fresh thread, loading the chat page first when it is not open yet. */
  145 | export async function startFreshThread(page: Page): Promise<void> {
  146 |   if (page.url().includes('/threads')) await startNewConversation(page);
  147 |   else await openFreshThread(page);
  148 | }
  149 | 
  150 | function threadRow(page: Page, name: string): Locator {
  151 |   return page
  152 |     .locator(THREAD_LIST)
  153 |     .locator('div.group')
  154 |     .filter({ has: page.getByText(name, { exact: true }) });
  155 | }
  156 | 
  157 | async function renameActiveThread(page: Page, name: string): Promise<void> {
  158 |   const list = page.locator(THREAD_LIST);
  159 |   const row = list.locator('div.group').filter({ has: page.locator('button[aria-current="page"]') }).first();
  160 |   await expect(row).toBeVisible({ timeout: 10_000 });
  161 |   // The runtime titles a thread after its first reply; give that a moment so it
  162 |   // cannot land after the rename. A thread that stays untitled is renamed anyway.
  163 |   await expect(row)
  164 |     .not.toContainText('New conversation', { timeout: 3_000 })
  165 |     .catch(() => undefined);
  166 |   await row.hover();
  167 |   await row.getByRole('button', { name: /^Actions for / }).click();
  168 |   await page.getByRole('menuitem', { name: 'Rename' }).click();
  169 |   const dialog = page.getByRole('dialog');
> 170 |   await dialog.getByLabel('Thread name').fill(name);
      |                                          ^ Error: locator.fill: Test timeout of 120000ms exceeded.
  171 |   await dialog.getByRole('button', { name: 'Save', exact: true }).click();
  172 |   await expect(list.getByText(name, { exact: true })).toBeVisible();
  173 | }
  174 | 
  175 | /** After a conversation's first message, names its thread so it can be switched to. */
  176 | async function materializeCurrentConversation(page: Page, s1: S1Context): Promise<void> {
  177 |   const conversation = s1.currentConversationState();
  178 |   if (!conversation || conversation.materialized) return;
  179 |   await renameActiveThread(page, conversation.name);
  180 |   conversation.materialized = true;
  181 | }
  182 | 
  183 | /** Creates a conversation on a fresh thread and makes it the current one. */
  184 | export async function createConversation(
  185 |   page: Page,
  186 |   s1: S1Context,
  187 |   name: string,
  188 |   options: { agent?: string; override?: string },
  189 | ): Promise<void> {
  190 |   await startFreshThread(page);
  191 |   if (options.agent) await switchAgentViaUI(page, options.agent);
  192 |   s1.conversations.set(name, {
  193 |     name,
  194 |     agent: options.agent ?? null,
  195 |     override: null,
  196 |     materialized: false,
  197 |   });
  198 |   s1.currentConversation = name;
  199 |   if (options.override) await setOverride(page, s1, options.override);
  200 | }
  201 | 
  202 | /**
  203 |  * Makes a conversation current. A named one is selected in the sidebar. One that
  204 |  * has not received its first message has no sidebar row, so its handle is moved
  205 |  * onto a fresh thread and its recorded agent and override are applied again.
  206 |  */
  207 | export async function switchToConversation(page: Page, s1: S1Context, name: string): Promise<void> {
  208 |   const conversation = s1.conversations.get(name);
  209 |   if (!conversation) throw new Error(`no conversation named "${name}" was created in this scenario`);
  210 |   if (s1.currentConversation === name) return;
  211 |   if (conversation.materialized) {
  212 |     await threadRow(page, name).getByRole('button').first().click();
  213 |     await expect(threadRow(page, name).locator('button[aria-current="page"]')).toBeVisible();
  214 |   } else {
  215 |     await startNewConversation(page);
  216 |     if (conversation.agent) await switchAgentViaUI(page, conversation.agent);
  217 |     if (conversation.override) await setOverrideOnThread(page, conversation.override);
  218 |   }
  219 |   s1.currentConversation = name;
  220 | }
  221 | 
  222 | // ---------------------------------------------------------------------------
  223 | // Session Configuration sheet
  224 | // ---------------------------------------------------------------------------
  225 | 
  226 | function sessionSheet(page: Page): Locator {
  227 |   return page.getByRole('dialog').filter({ hasText: 'Session Configuration' });
  228 | }
  229 | 
  230 | export async function openSessionSheet(page: Page): Promise<Locator> {
  231 |   await page.getByRole('button', { name: 'Session configuration', exact: true }).click();
  232 |   const sheet = sessionSheet(page);
  233 |   await expect(sheet).toBeVisible();
  234 |   await expect(sheet.getByRole('button', { name: 'Save Configuration' })).toBeEnabled({ timeout: 15_000 });
  235 |   return sheet;
  236 | }
  237 | 
  238 | export async function closeSessionSheet(page: Page): Promise<void> {
  239 |   const sheet = sessionSheet(page);
  240 |   if (await sheet.isVisible()) await page.keyboard.press('Escape');
  241 |   await expect(sheet).toBeHidden();
  242 | }
  243 | 
  244 | // The model pickers (W5): with seven or fewer options the trigger is a Base UI
  245 | // Select with no search box; with more it is a searchable combobox. In both modes
  246 | // the trigger carries the `testId` the call site passed, and every option carries
  247 | // `data-testid="model-option"` and `data-model-id="<provider>/<model>"`;
  248 | // non-selectable options also carry `aria-disabled="true"` and `data-disabled`.
  249 | 
  250 | function modelOption(page: Page, qualified: string): Locator {
  251 |   return page.locator(`[data-testid="${TID.modelOption}"][data-model-id="${qualified}"]`);
  252 | }
  253 | 
  254 | /**
  255 |  * Opens a picker from its trigger and narrows it to a model when the picker has
  256 |  * a search box. The popup is open once either a search box or an option shows.
  257 |  */
  258 | async function openPickerFor(page: Page, trigger: Locator, qualified: string): Promise<void> {
  259 |   const { modelId } = splitQualifiedModel(qualified);
  260 |   await trigger.click();
  261 |   const search = page.getByPlaceholder(/search models/i);
  262 |   await search.or(page.locator(`[data-testid="${TID.modelOption}"]`).first()).first().waitFor({ state: 'visible', timeout: 15_000 });
  263 |   if ((await search.count()) > 0) await search.first().fill(modelId);
  264 | }
  265 | 
  266 | /** Opens the picker and clicks the option for a provider-qualified model. */
  267 | async function chooseModel(page: Page, trigger: Locator, qualified: string): Promise<void> {
  268 |   await openPickerFor(page, trigger, qualified);
  269 |   const option = modelOption(page, qualified);
  270 |   await expect(option, `the picker offers ${qualified}`).toBeVisible({ timeout: 10_000 });
```
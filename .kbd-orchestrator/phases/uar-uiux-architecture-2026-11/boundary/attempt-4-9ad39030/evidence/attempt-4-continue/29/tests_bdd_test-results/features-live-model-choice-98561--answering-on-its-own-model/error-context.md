# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: features/live-model-choice.feature.spec.js >> Live model choice governs the next eligible response >> A conversation override keeps answering on its own model
- Location: tests/bdd/.features-gen/features/live-model-choice.feature.spec.js:41:3

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
                  - button "S1 r3 override just now Default Assistant · bdd-reply-qwen-5-turbo" [ref=e133] [cursor=pointer]:
                    - generic [ref=e134]:
                      - generic [ref=e135]:
                        - paragraph [ref=e136]: S1 r3 override
                        - generic [ref=e137]: just now
                      - paragraph [ref=e138]: Default Assistant · bdd-reply-qwen-5-turbo
                  - button "Actions for S1 r3 override" [active]
            - region "Conversation" [ref=e139]:
              - generic [ref=e140]:
                - button "Select agent" [ref=e141] [cursor=pointer]:
                  - generic [ref=e142]:
                    - generic [ref=e143]: Default Assistant
                    - generic [ref=e144]: "Session: bdd-provider-c/qwen-5-turbo"
                - button "Session configuration" [ref=e147] [cursor=pointer]
              - generic [ref=e150]:
                - generic [ref=e151]:
                  - paragraph [ref=e156]: S1 r3 override
                  - generic [ref=e157]:
                    - generic [ref=e158]:
                      - region "A2UI display artifact" [ref=e159]:
                        - generic [ref=e160]:
                          - generic [ref=e164]: Artifact
                          - generic [ref=e165]: A2UI 1.0 · rendered · effective_run_policy
                        - generic [ref=e167]:
                          - group [ref=e168]:
                            - generic [ref=e171]:
                              - paragraph [ref=e173]: Effective run policy
                              - generic [ref=e174]: Resolved policy v1 · A2UI 1.0
                              - generic [ref=e178]:
                                - paragraph [ref=e180]: Execution
                                - paragraph [ref=e182]: Chat mode · uar
                                - paragraph [ref=e184]: Model · bdd-provider-c / qwen-5-turbo
                                - paragraph [ref=e186]: Agent · default-agent
                              - generic [ref=e190]:
                                - paragraph [ref=e192]: Available capabilities
                                - paragraph [ref=e194]: Skills · auto · 288 available · agent scope
                                - paragraph [ref=e196]: Tools · selected · 15 available · agent scope
                                - paragraph [ref=e198]: MCP servers · auto · 0 available · legacy scope
                                - paragraph [ref=e200]: Knowledge bases · auto · 0 available · agent scope
                              - generic [ref=e204]:
                                - paragraph [ref=e206]: Runtime controls
                                - paragraph [ref=e208]: Memory · on
                                - paragraph [ref=e210]: Prompt caching · off
                                - paragraph [ref=e212]: Context · sliding window
                                - paragraph [ref=e214]: Tool approval · auto
                              - generic [ref=e218]:
                                - paragraph [ref=e220]: Warnings · 0
                                - paragraph [ref=e222]: No policy warnings.
                          - status [ref=e223]: This surface has no publication identity. Request a fresh surface before acting.
                          - button "Recover action status" [ref=e224] [cursor=pointer]
                      - generic "Turn manifest" [ref=e225]:
                        - generic "JSON" [ref=e233]:
                          - code [ref=e234]: "{ \"schema_version\": 1, \"manifest_hash\": \"e7b2d22d8ac4c019bbe656fc0d4290799cabc2449cd483c5ea83ee3c339cf6b9\", \"fragments\": [ { \"id\": \"agent.identity\", \"section\": \"agent_identity\", \"source\": \"artifact:default-agent\", \"authority\": \"system\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"e9f6bc3f98833b312aa5f17108f5459fab5e8cfce631a3a48ed23a35ed5b413e\" }, { \"id\": \"policy.effective\", \"section\": \"enforced_policy\", \"source\": \"effective_run_policy\", \"authority\": \"policy\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"7cb1decf1bf4efa9a938d5f45902f15f5c40e0ab9e643163cba1e84cedc7d747\" }, { \"id\": \"skills.catalog\", \"section\": \"skill_catalog\", \"source\": \"eligible_skill_registry\", \"authority\": \"skill\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"5daae18a687a25edbd36f948e3ff6255baba2e5325bbf2bac08a811b0d6282f6\" }, { \"id\": \"world_state.current_time\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"d07b2eb63dd5561de731315ff12fae3ea630b32614c33320eec79704c0da3fa4\" }, { \"id\": \"world_state.environment\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"f122476abc931a95b2343d1442fc641a99dcd84c974aa29aed030eb7236c49b9\" }, { \"id\": \"world_state.permissions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"9d48b593b41db02f2604616d5af722c6b1ea5e0c86695e3492a0707d9aebfe40\" }, { \"id\": \"world_state.project_instructions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\" } ], \"counts\": { \"total\": 7, \"by_section\": { \"agent_identity\": 1, \"enforced_policy\": 1, \"skill_catalog\": 1, \"world_state\": 4 }, \"by_authority\": { \"host\": 4, \"policy\": 1, \"skill\": 1, \"system\": 1 } }, \"budgets\": { \"rendered_bytes\": 27350, \"rendered_characters\": 27350, \"context_window_tokens\": 128000 }, \"selected_skills\": [], \"selected_tools\": [ \"a2ui_render\", \"activate_skill\", \"native__memory_capture_policy\", \"native__memory_delete_by_id\", \"native__memory_history\", \"native__memory_list\", \"native__memory_recall\", \"native__memory_save\", \"native__memory_update_by_id\", \"native_echo\", \"native_system_info\", \"session_search\", \"uar_compile\", \"uar_session_check_completeness\", \"uar_session_compile\", \"uar_session_update_section\" ], \"warnings\": [] }"
                      - generic "resolved_step" [ref=e235]:
                        - generic "JSON" [ref=e243]:
                          - code [ref=e244]: "{ \"source\": \"uar.turn\", \"payload\": { \"step\": 1, \"model\": \"qwen-5-turbo\", \"mcp_catalog\": [], \"manifest\": { \"schema_version\": 1, \"manifest_hash\": \"610be11ed9413dcf2412a38ab9c3e8c41d05a27ebac3905fba7bc938b73d5511\", \"fragments\": [ { \"id\": \"agent.identity\", \"section\": \"agent_identity\", \"source\": \"artifact:default-agent\", \"authority\": \"system\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"e9f6bc3f98833b312aa5f17108f5459fab5e8cfce631a3a48ed23a35ed5b413e\" }, { \"id\": \"policy.effective\", \"section\": \"enforced_policy\", \"source\": \"effective_run_policy\", \"authority\": \"policy\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"7cb1decf1bf4efa9a938d5f45902f15f5c40e0ab9e643163cba1e84cedc7d747\" }, { \"id\": \"skills.catalog\", \"section\": \"skill_catalog\", \"source\": \"eligible_skill_registry\", \"authority\": \"skill\", \"role\": \"system\", \"retention\": \"turn\", \"content_hash\": \"5daae18a687a25edbd36f948e3ff6255baba2e5325bbf2bac08a811b0d6282f6\" }, { \"id\": \"world_state.current_time\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"d07b2eb63dd5561de731315ff12fae3ea630b32614c33320eec79704c0da3fa4\" }, { \"id\": \"world_state.environment\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"f122476abc931a95b2343d1442fc641a99dcd84c974aa29aed030eb7236c49b9\" }, { \"id\": \"world_state.permissions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"9d48b593b41db02f2604616d5af722c6b1ea5e0c86695e3492a0707d9aebfe40\" }, { \"id\": \"world_state.project_instructions\", \"section\": \"world_state\", \"source\": \"host.world_state\", \"authority\": \"host\", \"role\": \"system\", \"retention\": \"session\", \"content_hash\": \"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\" } ], \"counts\": { \"total\": 7, \"by_section\": { \"agent_identity\": 1, \"enforced_policy\": 1, \"skill_catalog\": 1, \"world_state\": 4 }, \"by_authority\": { \"host\": 4, \"policy\": 1, \"skill\": 1, \"system\": 1 } }, \"budgets\": { \"rendered_bytes\": 27361, \"rendered_characters\": 27361, \"context_window_tokens\": 128000 }, \"selected_skills\": [], \"selected_tools\": [ \"a2ui_render\", \"activate_skill\", \"native__memory_capture_policy\", \"native__memory_delete_by_id\", \"native__memory_history\", \"native__memory_list\", \"native__memory_recall\", \"native__memory_save\", \"native__memory_update_by_id\", \"native_echo\", \"native_system_info\", \"session_search\", \"uar_compile\", \"uar_session_check_completeness\", \"uar_session_compile\", \"uar_session_update_section\" ], \"warnings\": [] } } }"
                      - generic "Provider attempt manifest" [ref=e245]:
                        - generic "JSON" [ref=e253]:
                          - code [ref=e254]: "{ \"schema_version\": \"uar.attempt-manifest.v1\", \"destination_model\": \"bdd-provider-c/qwen-5-turbo\", \"budgeting\": { \"label\": \"legacy_unbudgeted\", \"contract_present\": false, \"fit_guarantee\": false }, \"warning\": \"Provider dispatch is using the legacy unbudgeted path\" }"
                      - generic "budget_receipt" [ref=e255]:
                        - generic "JSON" [ref=e263]:
                          - code [ref=e264]: "{ \"source\": \"uar.request\", \"payload\": { \"schema_version\": \"uar.request-budget.v1\", \"representation\": \"final_provider_request\", \"destination_model\": \"bdd-provider-c/qwen-5-turbo\", \"serialized_request\": true, \"count_quality\": \"approximate\", \"counting_revision\": \"approximate-cl100k-json-v1\", \"counted_input_tokens\": 10492, \"serialized_bytes\": 45669, \"serialized_sha256\": \"b4a7bcfd2058805da70c62a4be74ad0090887b3182136c4371f2538c6f70a528\", \"tools_included\": true, \"instructions_included\": true, \"fit_guarantee\": false, \"reserved_output_tokens\": null, \"output_ceiling_enforced\": true } }"
                      - paragraph [ref=e267]: bdd-reply-qwen-5-turbo
                      - paragraph [ref=e268]:
                        - generic [ref=e269]: Answered by
                        - generic [ref=e270]: bdd-provider-c/qwen-5-turbo
                        - generic [aria-hidden] [ref=e271]: ·
                        - generic [ref=e272]: this conversation
                    - generic [ref=e274]:
                      - button "Copy" [ref=e275] [cursor=pointer]
                      - generic [ref=e277]:
                        - 'generic "Answered by agent: Default Assistant" [ref=e278]': Default Assistant
                        - 'generic "Model: qwen-5-turbo" [ref=e282]': qwen-5-turbo
                - generic [ref=e286]:
                  - button "Scroll to bottom" [ref=e287] [cursor=pointer]
                  - generic [ref=e289]:
                    - button [aria-hidden] [ref=e290]
                    - generic [ref=e291]:
                      - textbox "Message input" [ref=e292]:
                        - /placeholder: Send a message…
                      - generic [ref=e293]:
                        - button "Attach file" [ref=e294] [cursor=pointer]
                        - button "Disable memory for this message" [ref=e296] [cursor=pointer]:
                          - generic [ref=e297]: Memory ON – click to disable
                        - button "Send message" [disabled]
                  - generic [ref=e298]:
                    - button "Toggle Knowledge" [ref=e299] [cursor=pointer]:
                      - generic [ref=e300]: Knowledge
                    - button "Toggle Tools" [ref=e301] [cursor=pointer]:
                      - generic [ref=e302]: Tools
                      - generic [ref=e303]: "1"
                    - button "Toggle Skills" [ref=e304] [cursor=pointer]:
                      - generic [ref=e305]: Skills
                    - button "Enable Web Search" [ref=e306] [cursor=pointer]:
                      - generic [ref=e307]: Web Search
                    - button "Disable Memory" [ref=e308] [cursor=pointer]:
                      - generic [ref=e309]: Memory
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
import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { expect } from '@playwright/test';
import { test as base, createBdd } from 'playwright-bdd';

const COMPLETION_PATH = '/api/chat/completion';
const evidenceDir = () => process.env.UAR_BROWSER_EVIDENCE_DIR;

export const test = base.extend({
  scenario: async ({}, use, testInfo) => {
    const id = randomUUID();
    const state = { id, started: performance.now(), completionPosts: [],
      record: resolve(`.prometheus/verification/browser-history-reload/${id}.json`),
      evidence: { scenario: 'C35-BROWSER-HISTORY-RELOAD-01', platform: process.platform,
        authentication_supplied_by_test: false, previous_scenarios_rerun: false,
        service_kept_running: true, artifacts_dir: evidenceDir(), screenshots: [],
        exchanges: [], streams: [] } };
    const errors = [];
    try { await use(state); } finally {
      if (state.historyContext) {
        try { await state.historyContext.close(); } catch (error) { errors.push(`Fresh context cleanup: ${error}`); }
      }
      if (state.created) {
        try {
          await api(state, 'DELETE', `/api/agents/${state.agentId}`, undefined, 204);
          state.evidence.disposable_agent_deleted = true;
        } catch (error) { errors.push(`Agent cleanup: ${error}`); }
      }
      if (state.threadId) {
        try { await api(state, 'DELETE', `/api/uar/sessions/${state.threadId}`, undefined, [204, 404]); state.evidence.session_deleted = true; }
        catch (error) { errors.push(`Session cleanup ${state.threadId}: ${error}`); }
      }
      state.evidence.completion_posts = state.completionPosts;
      state.evidence.cleanup_errors = errors;
      state.evidence.status = errors.length ? 'failed' : testInfo.status;
      state.evidence.error = testInfo.error?.message?.slice(0, 4000) ?? null;
      state.evidence.duration_ms = Math.round(performance.now() - state.started);
      await mkdir(dirname(state.record), { recursive: true });
      await writeFile(state.record, JSON.stringify(state.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
      console.log(`EVIDENCE ${state.record} status=${state.evidence.status}`);
      if (errors.length) throw new Error(errors.join('; '));
    }
  },
});
const { Given, When, Then } = createBdd(test);

async function api(state, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, process.env.UAR_BASE_URL), {
    method, redirect: 'error', signal: AbortSignal.timeout(30_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': process.env.UAR_WORKSPACE_ID },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) state.created = true;
  const text = await response.text();
  state.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 4000) });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

function observeResponse(page, predicate, timeout = 180_000) {
  // Attach rejection handling immediately so an earlier UI failure stays primary.
  const pending = page.waitForResponse(predicate, { timeout }).then(response => ({ response }), error => ({ error }));
  return async () => {
    const result = await pending;
    if (result.error) throw result.error;
    return result.response;
  };
}

const isCompletion = r => r.request().method() === 'POST' && new URL(r.url()).pathname === COMPLETION_PATH;

// Reasoning is never answer text: drop closed blocks, then an unclosed block to the end.
const stripThink = text => text.replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();

async function stream(state, label, response) {
  const text = await response.text();
  const request = response.request();
  const record = { label, path: new URL(response.url()).pathname, status: response.status(),
    session_header: request.headers()['x-uar-session-id'] ?? null, request: request.postDataJSON(), response: text };
  state.evidence.streams.push(record);
  assert.equal(response.status(), 200);
  assert.match(response.headers()['content-type'], /^text\/event-stream\b/);
  const events = text.split(/\r?\n\r?\n/).flatMap(block => {
    const name = block.split(/\r?\n/).find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = block.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
    return [JSON.parse(data)];
  });
  assert.ok(!events.some(event => ['RUN_ERROR', 'RUN_CANCELLED'].includes(event.type)), `${label} stream must succeed`);
  const starts = events.filter(event => event.type === 'RUN_STARTED');
  const finishes = events.filter(event => event.type === 'RUN_FINISHED');
  assert.equal(starts.length, 1, `${label}: exactly one run`);
  assert.equal(finishes.length, 1, `${label}: exactly one terminal event`);
  assert.equal(starts[0].runId, finishes[0].runId);
  const raw = events.filter(event => event.type === 'TEXT_MESSAGE_CONTENT').map(event => event.delta ?? '').join('');
  Object.assign(record, { run_id: starts[0].runId, event_types: events.map(e => e.type === 'CUSTOM' ? `CUSTOM:${e.name}` : e.type),
    answer: stripThink(raw) });
  return { runId: starts[0].runId, request: record.request, sessionHeader: record.session_header, text: stripThink(raw) };
}

function parseJsonAnswer(final) {
  try { return JSON.parse(final); } catch {
    const fences = [...final.matchAll(/(?:^|\n)```(?:json)?[ \t]*\r?\n([\s\S]*?)\r?\n```(?=\s|$)/g)];
    assert.equal(fences.length, 1, `Answer is not JSON: ${final.slice(0, 400)}`);
    return JSON.parse(fences[0][1]);
  }
}

async function shot(page, s, name) {
  const path = resolve(evidenceDir(), name);
  await page.screenshot({ path, fullPage: true });
  s.evidence.screenshots.push(path);
}

const userMessages = page => page.locator('[data-slot="aui_user-message-root"][data-role="user"]');
const assistantMessages = page => page.locator('[data-slot="aui_assistant-message-root"][data-role="assistant"]');
const threadList = page => page.locator('[aria-label="Thread list"]');

Given('a disposable custom agent is selected in a fresh browser conversation', async ({ page, scenario: s }) => {
  assert.ok(process.env.UAR_MODEL && process.env.UAR_WORKSPACE_ID && evidenceDir());
  const url = new URL(process.env.UAR_BASE_URL);
  assert.ok(['http:', 'https:'].includes(url.protocol) && !url.username && !url.password);
  s.agentId = `disposable-browser-history-${s.id}`;
  s.title = `Browser history ${s.id.slice(0, 8)}`;
  Object.assign(s.evidence, { agent_id: s.agentId, agent_title: s.title, model: process.env.UAR_MODEL, page_errors: [] });
  page.on('pageerror', error => s.evidence.page_errors.push(String(error)));
  page.on('request', request => {
    if (request.method() === 'POST' && new URL(request.url()).pathname === COMPLETION_PATH) {
      s.completionPosts.push({ at: new Date().toISOString(), body: request.postData() });
    }
  });
  const agent = await api(s, 'GET', '/api/agents/default-agent');
  agent.id = s.agentId;
  agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: s.title, description: 'Disposable browser conversation history assistant.', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Remember the facts the user gives you for subsequent questions in this conversation. Acknowledge the initial facts with only READY, without repeating them. Answer later questions using the earlier user facts and the exact JSON fields requested.', instructions: [] };
  const split = process.env.UAR_MODEL.indexOf('/');
  assert.ok(split > 0);
  agent.policy.provider = { default: { provider: process.env.UAR_MODEL.slice(0, split), model: process.env.UAR_MODEL.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.memory.kb.enabled = false;
  // Long-term memory and retrieval are off so a recalled fact can only come from the session history.
  agent.extensions = { ...agent.extensions, 'uar.run_policy': { version: 1,
    context_strategy: { type: 'none' }, memory_enabled: false,
    skills: { mode: 'none' }, tools: { mode: 'none' },
    mcp_servers: { mode: 'none' }, knowledge_bases: { mode: 'none' }, presentations: { mode: 'none' } } };
  const saved = await api(s, 'POST', '/api/agents', agent, 201);
  assert.equal(saved.id, s.agentId);
  await page.goto('/threads', { waitUntil: 'domcontentloaded' });
  await page.getByRole('button', { name: 'New thread', exact: true }).click();
  const selection = observeResponse(page, r => r.request().method() === 'POST' && /\/api\/uar\/sessions\/[^/]+\/agent-config$/.test(new URL(r.url()).pathname));
  await page.getByRole('button', { name: 'Select agent', exact: true }).click();
  await page.getByPlaceholder('Search agents...').fill(s.title);
  await page.getByRole('option').filter({ hasText: s.title }).click();
  const saveResponse = await selection();
  assert.equal(saveResponse.status(), 200, 'The browser must save the session agent selection');
  s.evidence.agent_selection_save = { path: new URL(saveResponse.url()).pathname, request: saveResponse.request().postDataJSON() };
  await expect(page.getByRole('button', { name: 'Select agent', exact: true })).toContainText(s.title);
});

When('I give the agent a fresh fact in the browser', async ({ page, scenario: s }) => {
  s.reference = `CASE-${randomUUID()}`;
  s.left = randomInt(101, 400);
  s.right = randomInt(501, 900);
  s.expected = { reference: s.reference, total: s.left + s.right };
  s.factMessage = `Remember for this conversation: reference ${s.reference}; first quantity ${s.left}; second quantity ${s.right}. Acknowledge only READY without repeating any facts.`;
  s.evidence.expected = s.expected;
  const response = observeResponse(page, isCompletion);
  const title = observeResponse(page, r => r.request().method() === 'POST' && new URL(r.url()).pathname === '/api/generate-title', 240_000);
  await page.getByRole('textbox', { name: 'Message input', exact: true }).fill(s.factMessage);
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  s.first = await stream(s, 'fact', await response());
  s.threadId = s.first.sessionHeader;
  assert.ok(s.threadId, 'The browser must send its conversation id');
  assert.equal(s.first.request.message, s.factMessage);
  assert.ok(s.first.request.client_turn_id && s.first.request.user_message_id, "Original turn and message identities must be sent");
  assert.match(s.first.text, /^READY\b/i, `Fact acknowledgement: ${s.first.text.slice(0, 200)}`);
  s.evidence.thread_id = s.threadId;
  s.evidence.fact_run_id = s.first.runId;
  s.evidence.fact_answer = s.first.text;
  await expect(assistantMessages(page).last()).toContainText(/READY/i);
  try { s.evidence.title_generation_status = (await title()).status(); }
  catch (error) { s.evidence.title_generation_status = `not observed: ${String(error).slice(0, 200)}`; }
  const active = threadList(page).locator('button[aria-current="page"] p.truncate');
  await expect(active).toBeVisible();
  await expect(active).not.toHaveText('New conversation', { timeout: 30_000 }).catch(() => undefined);
  s.threadTitle = (await active.textContent()).trim();
  s.evidence.thread_title = s.threadTitle;
  s.evidence.before_reload = { user_messages: await userMessages(page).count(), assistant_messages: await assistantMessages(page).count() };
  await shot(page, s, 'before-reload.png');
});

When('I reload the page and reopen that conversation from the thread list', async ({ page, scenario: s }) => {
  s.postsBeforeReload = s.completionPosts.length;
  await page.reload({ waitUntil: 'domcontentloaded' });
  await expect(threadList(page)).toHaveAttribute('aria-busy', 'false');
  const entry = threadList(page).getByRole('button').filter({ has: page.getByText(s.threadTitle, { exact: true }) });
  await expect(entry).toHaveCount(1);
  s.evidence.thread_list_after_reload = await threadList(page).innerText();
  const configLoad = observeResponse(page, r => r.request().method() === 'GET' && new URL(r.url()).pathname === `/api/uar/sessions/${s.threadId}/agent-config`, 30_000);
  await entry.click();
  await expect(entry).toHaveAttribute('aria-current', 'page');
  try { s.evidence.agent_config_reload_status = (await configLoad()).status(); }
  catch (error) { s.evidence.agent_config_reload_status = `not observed after click: ${String(error).slice(0, 200)}`; }
});

Then('the reopened conversation shows the saved turns without a new run', async ({ page, scenario: s }) => {
  await expect(userMessages(page)).toHaveCount(1);
  await expect(assistantMessages(page)).toHaveCount(1);
  await expect(userMessages(page).first()).toContainText(s.reference);
  await expect(assistantMessages(page).first()).toContainText(/READY/i);
  await expect(page.getByRole('button', { name: 'Select agent', exact: true })).toContainText(s.title);
  assert.equal(s.completionPosts.length, s.postsBeforeReload, 'Reload and reopen must not start a run');
  s.evidence.reopened = { user_messages: await userMessages(page).allInnerTexts(),
    assistant_messages: await assistantMessages(page).allInnerTexts(),
    agent_selector: await page.getByRole('button', { name: 'Select agent', exact: true }).innerText() };
  await shot(page, s, 'reopened-after-reload.png');
});

When('I send a follow-up that contains none of the earlier facts', async ({ page, scenario: s }) => {
  s.followUp = 'What reference did I previously give you, and what is the sum of the two quantities? Return only JSON with reference and total.';
  for (const fact of [s.reference, String(s.left), String(s.right)]) assert.ok(!s.followUp.includes(fact));
  const response = observeResponse(page, isCompletion);
  await page.getByRole('textbox', { name: 'Message input', exact: true }).fill(s.followUp);
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  s.second = await stream(s, 'follow_up', await response());
  const body = JSON.stringify(s.second.request);
  for (const fact of [s.reference, String(s.left), String(s.right)]) assert.ok(!body.includes(fact), `Follow-up request replayed ${fact}`);
  assert.equal(s.second.request.message, s.followUp);
  assert.equal(s.second.sessionHeader, s.threadId, 'The follow-up must continue the reopened conversation');
  assert.notEqual(s.second.runId, s.first.runId);
  assert.notEqual(s.second.request.client_turn_id, s.first.request.client_turn_id);
  assert.notEqual(s.second.request.user_message_id, s.first.request.user_message_id);
  s.evidence.follow_up_request = s.second.request;
  s.evidence.follow_up_run_id = s.second.runId;
});

Then('the follow-up answer uses the earlier fact from the saved server history', async ({ page, scenario: s }) => {
  const answer = parseJsonAnswer(s.second.text);
  s.evidence.follow_up_answer = answer;
  assert.deepEqual({ reference: answer.reference, total: answer.total }, s.expected);
  await expect(userMessages(page)).toHaveCount(2);
  await expect(assistantMessages(page)).toHaveCount(2);
  await expect(assistantMessages(page).last()).toContainText(s.reference);
  await expect(assistantMessages(page).last()).toContainText(String(s.expected.total));
  assert.equal(s.completionPosts.length, 2, 'Exactly two browser runs: the fact and the follow-up');
  for (const [label, runId] of [['fact_run', s.first.runId], ['follow_up_run', s.second.runId]]) {
    const run = await api(s, 'GET', `/api/uar/runs/${runId}`);
    assert.equal(run.conversation_id, s.threadId, `${label} conversation`);
    assert.equal(run.agent_id, s.agentId, `${label} agent`);
    assert.equal(run.status, 'done', `${label} status`);
    s.evidence[label] = run;
  }
  const runs = await api(s, 'GET', '/api/uar/runs');
  const owned = runs.filter(run => run.conversation_id === s.threadId).map(run => run.run_id).sort();
  s.evidence.conversation_runs = owned;
  // The run listing holds only retained active-run state, so it can omit finished runs; it must never add one.
  const known = new Set([s.first.runId, s.second.runId]);
  assert.ok(owned.every(id => known.has(id)), `Unexpected extra run in the conversation: ${owned.join(', ')}`);
  s.evidence.fact_source = 'server session history: follow-up request carried only the new question; memory, retrieval and context strategy disabled by run policy';
  s.evidence.result = 'VERIFIED';
  await shot(page, s, 'follow-up-answer.png');
});

When('the server admits my browser turn but its response is lost', async ({ page, scenario: s }) => {
  s.reference = `LOST-${randomUUID()}`;
  s.factMessage = `Remember reference ${s.reference}. Acknowledge only READY.`;
  let delivered;
  const admitted = new Promise((resolve, reject) => { delivered = { resolve, reject }; });
  await page.route('**/api/chat/completion', async route => {
    try {
      const request = route.request();
      const body = request.postDataJSON();
      assert.ok(body.client_turn_id && body.user_message_id, 'Stable original IDs must be sent');
      s.threadId = request.headers()['x-uar-session-id'];
      assert.ok(s.threadId);
      s.lostTurn = body;
      const response = await route.fetch();
      assert.equal(response.status(), 200);
      const text = await response.text();
      assert.ok(text.includes('RUN_FINISHED'), 'The actual server run must finish before its response is lost');
      s.evidence.lost_response = { request: body, session_id: s.threadId, server_response: text };
      await route.abort('failed');
      delivered.resolve();
    } catch (error) { delivered.reject(error); }
  }, { times: 1 });
  await page.getByRole('textbox', { name: 'Message input', exact: true }).fill(s.factMessage);
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  await admitted;
  s.postsBeforeReload = s.completionPosts.length;
  assert.equal(s.postsBeforeReload, 1);
});
When('I reload after losing that admission response', async ({ page }) => {
  await page.reload({ waitUntil: 'domcontentloaded' });
  await expect(threadList(page)).toHaveAttribute('aria-busy', 'false');
});
Then('the original turn receipt and history recover without a second completion POST', async ({ page, scenario: s }) => {
  const receipt = await api(s, 'GET', `/api/uar/chat-turns/${encodeURIComponent(s.lostTurn.client_turn_id)}?session_id=${encodeURIComponent(s.threadId)}`);
  assert.equal(receipt.client_turn_id, s.lostTurn.client_turn_id);
  assert.equal(receipt.user_message_id, s.lostTurn.user_message_id);
  assert.equal(receipt.retry_allowed, false);
  assert.equal(receipt.state, 'complete');
  const history = await api(s, 'GET', `/api/uar/sessions/${encodeURIComponent(s.threadId)}`);
  const user = history.history.filter(row => row.message_id === s.lostTurn.user_message_id);
  assert.equal(user.length, 1);
  assert.equal(user[0].client_turn_id, s.lostTurn.client_turn_id);
  assert.ok(history.history.some(row => row.run_id === receipt.run_id && row.role === 'assistant' && row.status === 'complete'));
  await expect(userMessages(page).filter({ hasText: s.reference })).toHaveCount(1);
  await expect(assistantMessages(page).filter({ hasText: /READY/i })).toHaveCount(1);
  assert.equal(s.completionPosts.length, s.postsBeforeReload, 'Receipt/history recovery cannot start a second run');
  s.evidence.recovered_turn = { receipt, history, completion_posts: s.completionPosts.length };
  await shot(page, s, 'lost-admission-recovered.png');
});

// These cases consume the existing externally provisioned native fixture. They
// never start peers, change provider configuration or carry authentication.
const historyText = row => typeof row.content === 'string' ? row.content
  : row.content.filter(block => block.type === 'text').map(block => block.text).join('');
const conversationMessages = page => page.locator('[data-slot="aui_user-message-root"][data-role="user"], [data-slot="aui_assistant-message-root"][data-role="assistant"]');
async function historyJournals(s) {
  const lines = async path => (await readFile(path, 'utf8')).split('\n').filter(Boolean).map(line => JSON.parse(line));
  return { provider: await lines(s.fixture.peer.journal), effects: await lines(s.fixture.peer.recordLog) };
}
function observeHistoryPage(page, s) {
  page.on('pageerror', error => s.evidence.page_errors.push(String(error)));
  page.on('request', request => {
    if (request.method() === 'POST' && new URL(request.url()).pathname === COMPLETION_PATH)
      s.completionPosts.push({ at: new Date().toISOString(), body: request.postData() });
  });
}

Given('the externally provisioned governed tool agent is selected in a fresh browser conversation', async ({ page, scenario: s }) => {
  assert.ok(process.env.UAR_NATIVE_CHAT_FIXTURE && process.env.UAR_BASE_URL && process.env.UAR_WORKSPACE_ID && evidenceDir());
  s.fixture = JSON.parse(await readFile(process.env.UAR_NATIVE_CHAT_FIXTURE, 'utf8'));
  assert.equal(s.fixture.protocol, 'uar.native-chat-admission-fixture/v1');
  assert.equal(s.fixture.dedicated, true);
  assert.match(s.fixture.namespace, /^native-chat-[A-Za-z0-9_-]{1,64}$/);
  assert.match(s.fixture.identity.sourceCommit, /^[a-f0-9]{40}$/);
  assert.match(s.fixture.identity.imageDigest, /^sha256:[a-f0-9]{64}$/);
  assert.match(s.fixture.identity.configDigest, /^[a-f0-9]{64}$/);
  assert.equal(s.fixture.identity.platform, 'linux/amd64');
  assert.equal(s.fixture.peer.protocol, 'uar.native-chat-admission-peer/v1');
  for (const flag of ['retriesDisabled', 'fallbacksDisabled', 'visionEnabled']) assert.equal(s.fixture.peer[flag], true);
  for (const path of [s.fixture.peer.journal, s.fixture.peer.recordLog]) assert.ok(path?.startsWith('/'));
  assert.ok(s.fixture.peer.toolName && s.fixture.peer.wireModel && s.fixture.model && s.fixture.agents.effect);
  const base = new URL(process.env.UAR_BASE_URL);
  assert.ok(['http:', 'https:'].includes(base.protocol) && ['127.0.0.1', '[::1]'].includes(base.hostname));
  assert.ok(!base.username && !base.password && !base.search && !base.hash);
  s.agentId = s.fixture.agents.effect;
  const agent = await api(s, 'GET', `/api/agents/${encodeURIComponent(s.agentId)}`);
  assert.equal(agent.id, s.agentId);
  s.inheritedModel = { provider_id: agent.policy.provider.default.provider, model_id: agent.policy.provider.default.model };
  assert.equal(`${s.inheritedModel.provider_id}/${s.inheritedModel.model_id}`, s.fixture.model);
  s.title = agent.metadata.title;
  assert.ok(s.title);
  Object.assign(s.evidence, { scenario: 'C35-CANONICAL-TOOL-LOOP-RELOAD', identity: s.fixture.identity,
    namespace: s.fixture.namespace, external_services_started_by_test: false, page_errors: [] });
  observeHistoryPage(page, s);
  await page.goto('/threads', { waitUntil: 'domcontentloaded' });
  await page.getByRole('button', { name: 'New thread', exact: true }).click();
  const selected = observeResponse(page, r => r.request().method() === 'POST' && /\/api\/uar\/sessions\/[^/]+\/agent-config$/.test(new URL(r.url()).pathname));
  await page.getByRole('button', { name: 'Select agent', exact: true }).click();
  await page.getByPlaceholder('Search agents...').fill(s.title);
  await page.getByRole('option').filter({ hasText: s.title }).click();
  const selectionResponse = await selected();
  assert.equal(selectionResponse.status(), 200);
  const savedConfig = await selectionResponse.json();
  assert.equal(savedConfig.agent_id, s.agentId);
  assert.equal(savedConfig.model, null, 'The fresh session must inherit the selected agent model policy');
  s.evidence.saved_model_policy = { path: new URL(selectionResponse.url()).pathname, config: savedConfig,
    agent_default: s.inheritedModel };
  await expect(page.getByRole('button', { name: 'Select agent', exact: true })).toContainText(s.title);
  s.journalsBefore = await historyJournals(s);
  s.historyPage = page;
});

When('I complete a controlled {string} tool-loop and a later browser turn', async ({ page, scenario: s }, intermediate) => {
  assert.ok(['preamble', 'tool-only'].includes(intermediate));
  s.intermediate = intermediate;
  s.toolCase = randomUUID();
  s.followCase = randomUUID();
  const mode = intermediate === 'preamble' ? 'effect-preamble' : 'effect';
  s.toolPrompt = `UAR_NATIVE_CHAT:${s.fixture.namespace}:${s.toolCase}:${mode}`;
  s.finalText = `NATIVE-CHAT-OK:${s.toolCase}`;
  s.preambleText = `NATIVE-CHAT-PREAMBLE:${s.toolCase}`;
  const title = observeResponse(page, r => r.request().method() === 'POST' && new URL(r.url()).pathname === '/api/generate-title');
  // Capture the actual durable pre-dispatch UI state before any server reply.
  // The shipped DB uses idb://uar-threads, so IndexedDB must be included.
  await page.route('**/api/chat/completion', async route => {
    assert.equal(route.request().postDataJSON().message, s.toolPrompt);
    await expect(assistantMessages(page)).toHaveCount(0);
    assert.deepEqual(await historyJournals(s), s.journalsBefore, 'No provider arrival or tool effect may precede the captured original dispatch');
    const snapshot = await page.context().storageState({ indexedDB: true });
    const origin = snapshot.origins.find(row => row.origin === new URL(process.env.UAR_BASE_URL).origin);
    assert.ok(origin?.indexedDB?.length, 'The pre-reply snapshot must capture the shipped IndexedDB backing store');
    // The gateway supplies identity; transfer only local browser data, no cookies.
    s.preReplySnapshot = { cookies: [], origins: snapshot.origins };
    s.evidence.pre_reply_snapshot = { assistant_messages: 0, indexed_databases: origin.indexedDB.length,
      captured_before_completion_dispatch: true, run_text_events: 0, event_basis: 'original POST held before dispatch in fresh scenario context', includes_registry_user_intent: 'asserted again in fresh context before history/receipt reads' };
    await route.continue();
  }, { times: 1 });
  const completed = observeResponse(page, isCompletion);
  await page.getByRole('textbox', { name: 'Message input', exact: true }).fill(s.toolPrompt);
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  s.toolTurn = await stream(s, 'controlled-tool-loop', await completed());
  s.threadId = s.toolTurn.sessionHeader;
  assert.ok(s.threadId && s.toolTurn.request.client_turn_id && s.toolTurn.request.user_message_id);
  assert.equal(s.toolTurn.request.agent_id, s.agentId);
  assert.equal(Object.hasOwn(s.toolTurn.request, 'model'), false, 'Normal Send must not override the saved model policy');
  assert.equal(s.evidence.saved_model_policy.path, `/api/uar/sessions/${encodeURIComponent(s.threadId)}/agent-config`);
  await expect(assistantMessages(page).filter({ hasText: s.finalText })).toHaveCount(1);
  assert.equal((await title()).status(), 200);
  const active = threadList(page).locator('button[aria-current="page"] p.truncate');
  await expect(active).not.toHaveText('New conversation');
  s.threadTitle = (await active.innerText()).trim();
  s.followPrompt = `UAR_NATIVE_CHAT:${s.fixture.namespace}:${s.followCase}:success`;
  const follow = observeResponse(page, isCompletion);
  await page.getByRole('textbox', { name: 'Message input', exact: true }).fill(s.followPrompt);
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  s.followTurn = await stream(s, 'later-turn', await follow());
  assert.equal(s.followTurn.sessionHeader, s.threadId);
  assert.notEqual(s.followTurn.runId, s.toolTurn.runId);
  assert.equal(s.followTurn.text, `NATIVE-CHAT-OK:${s.followCase}`);
  assert.equal(s.completionPosts.length, 2);
  s.evidence.admitted_model_policy = [];
  for (const turn of [s.toolTurn, s.followTurn]) {
    assert.equal(Object.hasOwn(turn.request, 'model'), false, 'Both normal Send turns must inherit model policy');
    const admitted = await api(s, 'GET', `/api/uar/runs/${encodeURIComponent(turn.runId)}`);
    assert.equal(admitted.run_id, turn.runId);
    assert.equal(admitted.agent_id, s.agentId);
    assert.equal(admitted.conversation_id, s.threadId);
    assert.deepEqual(admitted.effective_model, s.inheritedModel, 'The admitted logical model must match the selected agent policy');
    s.evidence.admitted_model_policy.push({ run_id: admitted.run_id, effective_model: admitted.effective_model });
  }
  const history = await api(s, 'GET', `/api/uar/sessions/${encodeURIComponent(s.threadId)}`);
  assert.equal(history.history_version, 1);
  const segments = history.history.filter(row => row.role === 'assistant' && row.run_id === s.toolTurn.runId);
  assert.deepEqual(segments.map(row => row.message_id), [`run-${s.toolTurn.runId}-assistant-0`, `run-${s.toolTurn.runId}-assistant-1`]);
  assert.deepEqual(segments.map(historyText), [intermediate === 'preamble' ? s.preambleText : '', s.finalText]);
  assert.ok(segments.every(row => row.status === 'complete'));
  assert.ok(segments.every(row => typeof row.content === 'string' || row.content.every(block => ['text', 'image_url', 'image'].includes(block.type))),
    'Use the real canonical text/image history projection, not portable tool argument/result mocks');
  s.canonicalIds = history.history.map(row => row.message_id);
  assert.equal(history.history.filter(row => row.message_id === s.toolTurn.request.user_message_id).length, 1);
  assert.equal(history.history.filter(row => row.message_id === s.followTurn.request.user_message_id).length, 1);
  const journals = await historyJournals(s);
  const arrivals = journals.provider.slice(s.journalsBefore.provider.length).filter(row => row.kind === 'provider');
  assert.deepEqual(arrivals.map(row => [row.case_id, row.stage]), [[s.toolCase, 'tool_request'], [s.toolCase, 'after_tool'], [s.followCase, 'success']]);
  assert.ok(arrivals.every(row => row.namespace === s.fixture.namespace && row.model === s.fixture.peer.wireModel));
  const effects = journals.effects.slice(s.journalsBefore.effects.length).filter(row => row.tool === 'record_write' && !row.response_dropped);
  assert.equal(effects.length, 1);
  assert.equal(effects[0].arguments.key, `${s.fixture.namespace}:${s.toolCase}`);
  assert.equal(effects[0].result.stored, true);
  assert.equal(effects[0].result.revision, 1);
  s.journalsSettled = journals;
  s.postsSettled = s.completionPosts.length;
  Object.assign(s.evidence, { intermediate, thread_id: s.threadId, canonical_before_reload: history,
    independent_before_reload: journals, original_completion_posts: s.postsSettled });
  await shot(page, s, `tool-loop-${intermediate}-before-reload-${s.id}.png`);
});

When('I reopen that tool-loop conversation with {string} browser history', async ({ browser, scenario: s }, cache) => {
  assert.ok(['warm-cache', 'fresh-context'].includes(cache));
  let page = s.historyPage;
  if (cache === 'fresh-context') {
    assert.ok(s.preReplySnapshot, 'Use the actual pre-reply local snapshot, never a completed-history snapshot');
    s.historyContext = await browser.newContext({ baseURL: process.env.UAR_BASE_URL,
      viewport: { width: 1280, height: 900 }, extraHTTPHeaders: { 'x-uar-workspace-id': process.env.UAR_WORKSPACE_ID },
      storageState: s.preReplySnapshot });
    page = await s.historyContext.newPage();
    s.historyPage = page;
    observeHistoryPage(page, s);
    let release;
    const readsReleased = new Promise(done => { release = done; });
    const sessionPath = `/api/uar/sessions/${encodeURIComponent(s.threadId)}`;
    await page.route('**/api/uar/**', async route => {
      const request = route.request();
      const url = new URL(request.url());
      const receipt = url.pathname.startsWith('/api/uar/chat-turns/') && url.searchParams.get('session_id') === s.threadId;
      if (request.method() === 'GET' && (url.pathname === sessionPath || receipt)) await readsReleased;
      await route.continue();
    });
    try {
      await page.goto('/threads', { waitUntil: 'domcontentloaded' });
      await expect(threadList(page)).toHaveAttribute('aria-busy', 'false');
      const entry = threadList(page).getByRole('button');
      await expect(entry).toHaveCount(1);
      await entry.click();
      await expect(entry).toHaveAttribute('aria-current', 'page');
      // This positive registry/user assertion prevents treating a failed or empty
      // IndexedDB export as an empty-assistant-cache acceptance result.
      await expect(userMessages(page).filter({ hasText: s.toolPrompt })).toHaveCount(1);
      await expect(assistantMessages(page)).toHaveCount(0);
      s.evidence.fresh_context_pre_recovery = { registry_entries: 1, original_user_visible: true,
        assistant_messages: 0, server_reads_held: true, snapshot: 'pre-reply local IndexedDB; not zero local data' };
    } finally { release(); }
  } else {
    await page.reload({ waitUntil: 'domcontentloaded' });
    await expect(threadList(page)).toHaveAttribute('aria-busy', 'false');
    const entry = threadList(page).getByRole('button').filter({ has: page.getByText(s.threadTitle, { exact: true }) });
    await expect(entry).toHaveCount(1);
    await entry.click();
    await expect(entry).toHaveAttribute('aria-current', 'page');
  }
  s.lastHistoryCache = cache;
});

Then('every canonical assistant segment and final answer appears once in order without another run or effect', async ({ scenario: s }) => {
  const page = s.historyPage;
  const messages = conversationMessages(page);
  await expect(messages).toHaveCount(s.canonicalIds.length);
  await expect.poll(async () => messages.evaluateAll(rows => rows.map(row => row.getAttribute('data-message-id')))).toEqual(s.canonicalIds);
  const final = assistantMessages(page).filter({ hasText: s.finalText });
  await expect(final).toHaveCount(1);
  await expect(final).toBeVisible();
  const visibleAnswers = await assistantMessages(page).allInnerTexts();
  assert.equal(visibleAnswers.join('\n').split(s.finalText).length - 1, 1, 'Final answer text must occur exactly once');
  if (s.intermediate === 'preamble') {
    await expect(assistantMessages(page).filter({ hasText: s.preambleText })).toHaveCount(1);
    assert.equal(visibleAnswers.join('\n').split(s.preambleText).length - 1, 1);
  }
  await expect(userMessages(page)).toHaveCount(2);
  await expect(assistantMessages(page)).toHaveCount(3);
  const history = await api(s, 'GET', `/api/uar/sessions/${encodeURIComponent(s.threadId)}`);
  assert.deepEqual(history.history.map(row => row.message_id), s.canonicalIds);
  for (const turn of [s.toolTurn, s.followTurn]) {
    const receipt = await api(s, 'GET', `/api/uar/chat-turns/${encodeURIComponent(turn.request.client_turn_id)}?session_id=${encodeURIComponent(s.threadId)}`);
    assert.equal(receipt.run_id, turn.runId);
    assert.equal(receipt.state, 'complete');
    assert.equal(receipt.retry_allowed, false);
    assert.equal(receipt.user_message_id, turn.request.user_message_id);
  }
  assert.equal(s.completionPosts.length, s.postsSettled, 'Read-only history/receipt recovery cannot issue another completion POST');
  const runs = await api(s, 'GET', '/api/uar/runs');
  const owned = runs.filter(row => row.conversation_id === s.threadId).map(row => row.run_id);
  assert.ok(owned.every(id => [s.toolTurn.runId, s.followTurn.runId].includes(id)), 'Reload must not create any extra run');
  const journals = await historyJournals(s);
  assert.deepEqual(journals.provider, s.journalsSettled.provider, 'Reload/recovery must cause zero provider arrivals');
  assert.deepEqual(journals.effects, s.journalsSettled.effects, 'Reload/recovery must cause zero extra effects');
  s.evidence.reloads ??= [];
  s.evidence.reloads.push({ cache: s.lastHistoryCache, visible_ids: s.canonicalIds, assistant_text: visibleAnswers,
    completion_posts: s.completionPosts.length, retained_runs: owned, provider_rows: journals.provider.length,
    effect_rows: journals.effects.length, canonical_history: history });
  await shot(page, s, `tool-loop-${s.intermediate}-${s.lastHistoryCache}-${s.evidence.reloads.length}-${s.id}.png`);
});

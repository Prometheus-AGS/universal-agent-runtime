import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
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

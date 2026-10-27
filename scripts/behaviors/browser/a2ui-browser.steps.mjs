import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { expect } from '@playwright/test';
import { test as base, createBdd } from 'playwright-bdd';

export const test = base.extend({
  scenario: async ({}, use, testInfo) => {
    const state = { started: performance.now(), evidence: { scenario: 'A2UI-BROWSER-01',
      platform: process.platform, authentication_supplied_by_test: false,
      previous_scenarios_rerun: false, service_kept_running: true, exchanges: [], streams: [] } };
    let cleanupError;
    try { await use(state); } finally {
      if (state.created) {
        try {
          await api(state, 'DELETE', `/api/agents/${state.agentId}`, undefined, 204);
          state.evidence.disposable_agent_deleted = true;
        } catch (error) { cleanupError = error; state.evidence.cleanup_error = String(error); }
      }
      state.evidence.status = cleanupError ? 'failed' : testInfo.status;
      state.evidence.duration_ms = Math.round(performance.now() - state.started);
      const dir = process.env.UAR_BROWSER_EVIDENCE_DIR;
      await mkdir(dir, { recursive: true });
      await writeFile(resolve(dir, 'evidence.json'), JSON.stringify(state.evidence, null, 2) + '\n', { flag: 'wx' });
      if (cleanupError) throw cleanupError;
    }
  },
});
const { Given, When, Then } = createBdd(test);

async function api(state, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, process.env.UAR_BASE_URL), {
    method, redirect: 'error', signal: AbortSignal.timeout(15_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': process.env.UAR_WORKSPACE_ID },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) state.created = true;
  const text = await response.text();
  state.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text });
  assert.equal(response.status, expected, `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}

function observeResponse(page, predicate) {
  // Attach rejection handling immediately so an earlier UI failure stays primary.
  const pending = page.waitForResponse(predicate).then(response => ({ response }), error => ({ error }));
  return async () => {
    const result = await pending;
    if (result.error) throw result.error;
    return result.response;
  };
}

async function stream(state, response) {
  const text = await response.text();
  const record = { path: new URL(response.url()).pathname, status: response.status(),
    request: response.request().postDataJSON(), response: text };
  state.evidence.streams.push(record);
  assert.equal(response.status(), 200);
  assert.match(response.headers()['content-type'], /^text\/event-stream\b/);
  const events = text.split(/\r?\n\r?\n/).flatMap(block => {
    const name = block.split(/\r?\n/).find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = block.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
    return [JSON.parse(data)];
  });
  record.events = events;
  assert.ok(!events.some(event => ['RUN_ERROR', 'RUN_CANCELLED'].includes(event.type)), 'Stream must succeed');
  const starts = events.filter(event => event.type === 'RUN_STARTED');
  const finishes = events.filter(event => event.type === 'RUN_FINISHED');
  assert.equal(starts.length, 1);
  assert.equal(finishes.length, 1);
  assert.equal(starts[0].runId, finishes[0].runId);
  return { runId: starts[0].runId, events,
    text: events.filter(event => event.type === 'TEXT_MESSAGE_CONTENT').map(event => event.delta).join('') };
}

Given('a disposable A2UI calculator agent is selected in the browser', async ({ page, scenario: s }) => {
  assert.ok(process.env.UAR_MODEL && process.env.UAR_WORKSPACE_ID);
  const url = new URL(process.env.UAR_BASE_URL);
  assert.ok(['http:', 'https:'].includes(url.protocol) && !url.username && !url.password);
  s.agentId = `disposable-a2ui-browser-${randomUUID()}`;
  s.title = `Browser calculator ${randomUUID().slice(0, 8)}`;
  s.marker = `BROWSER-${randomUUID()}`;
  s.evidence.agent_id = s.agentId;
  s.evidence.model = process.env.UAR_MODEL;
  s.evidence.page_errors = [];
  page.on('pageerror', error => s.evidence.page_errors.push(String(error)));
  const agent = await api(s, 'GET', '/api/agents/default-agent');
  agent.id = s.agentId;
  agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: s.title, description: 'Disposable interactive parcel calculator.', tags: ['verification'] };
  agent.runtime.entry = 'default';
  const template = {"messages":[{"version":"v0.9.1","createSurface":{"surfaceId":"parcel-calculator","catalogId":"urn:uar:a2ui:catalog:1"}},{"version":"v0.9.1","updateComponents":{"surfaceId":"parcel-calculator","components":[{"id":"root","component":"Column","children":["heading","parcel","calculate"]},{"id":"heading","component":"Text","text":"Parcel calculator"},{"id":"parcel","component":"TextField","label":"Parcel number","value":{"path":"/parcel"}},{"id":"calculate","component":"Button","child":"button-label","action":{"event":{"name":"calculate_parcel"}}},{"id":"button-label","component":"Text","text":"Calculate parcel"}]}},{"version":"v0.9.1","updateDataModel":{"surfaceId":"parcel-calculator","path":"/","value":{"parcel":""}}}]};
  agent.prompt = { system: 'On the initial request call a2ui_render exactly once with this valid template verbatim: '
    + JSON.stringify(template) + '. Preserve JSON arrays exactly: messages, components and children are arrays, NEVER objects with an item property. Use native JSON tool arguments, not XML wrappers. Do not alter the template, use another tool or print it as prose. '
    + 'After publication say only Calculator ready. '
    + 'On an a2ui.user_action follow-up, do not render again. Read the fresh parcel value from '
    + 'interaction.a2uiClientDataModel[interaction.surfaceId].parcel. '
    + `Return only JSON with marker "${s.marker}" and numeric result equal to that fresh parcel number plus 19.`, instructions: [] };
  const split = process.env.UAR_MODEL.indexOf('/');
  assert.ok(split > 0);
  agent.policy.provider = { default: { provider: process.env.UAR_MODEL.slice(0, split), model: process.env.UAR_MODEL.slice(split + 1) }, fallbacks: [] };
  agent.policy.tools.allow = ['a2ui_render'];
  agent.policy.skills.prefer = [];
  agent.memory.kb.enabled = false;
  const saved = await api(s, 'POST', '/api/agents', agent, 201);
  assert.equal(saved.id, s.agentId);
  await page.goto('/threads', { waitUntil: 'domcontentloaded' });
  await page.getByRole('button', { name: 'New thread', exact: true }).click();
  await page.getByRole('button', { name: 'Select agent', exact: true }).click();
  await page.getByPlaceholder('Search agents...').fill(s.title);
  await page.getByRole('option').filter({ hasText: s.title }).click();
});

When('I ask the agent to render its calculator', async ({ page, scenario: s }) => {
  const response = observeResponse(page, r => r.request().method() === 'POST' && new URL(r.url()).pathname === '/api/chat/completion');
  await page.getByRole('textbox', { name: 'Message input', exact: true }).fill('Please render the interactive parcel calculator.');
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  s.initial = await stream(s, await response());
  const artifacts = s.initial.events.filter(event => event.type === 'CUSTOM' && event.name === 'uar.artifact.available' && event.value?.artifactType === 'a2ui');
  assert.equal(artifacts.length, 1, 'Require genuine published A2UI artifact');
  assert.equal(artifacts[0].value.metadata?.sourceTool, 'a2ui_render');
  assert.equal(artifacts[0].value.metadata?.publication_status, 'published');
  s.evidence.source_run_id = s.initial.runId;
});

Then('the browser displays the generated input and action button', async ({ page, scenario: s }) => {
  s.surface = page.getByRole('region', { name: 'A2UI display artifact', exact: true }).filter({ hasText: 'Parcel calculator' });
  await expect(s.surface.locator('[data-a2ui-surface-state="ready"]')).toBeVisible();
  await expect(s.surface).toContainText('Parcel calculator');
  await expect(s.surface.getByRole('textbox', { name: 'Parcel number', exact: true })).toBeVisible();
  await expect(s.surface.getByRole('button', { name: 'Calculate parcel', exact: true })).toBeVisible();
  await page.screenshot({ path: resolve(process.env.UAR_BROWSER_EVIDENCE_DIR, 'rendered-surface.png'), fullPage: true });
});

When('I enter a fresh number and press the rendered calculate button', async ({ page, scenario: s }) => {
  s.number = randomInt(100, 900);
  s.evidence.fresh_input = s.number;
  await s.surface.getByRole('textbox', { name: 'Parcel number', exact: true }).fill(String(s.number));
  const action = observeResponse(page, r => r.request().method() === 'POST' && /\/api\/uar\/runs\/[^/]+\/a2ui\/actions$/.test(new URL(r.url()).pathname));
  const continuation = observeResponse(page, r => r.request().method() === 'GET' && /\/api\/uar\/runs\/[^/]+\/stream$/.test(new URL(r.url()).pathname));
  await s.surface.getByRole('button', { name: 'Calculate parcel', exact: true }).click();
  const response = await action();
  const body = response.request().postDataJSON();
  const ack = await response.json();
  s.evidence.action = { path: new URL(response.url()).pathname, request: body, status: response.status(), response: ack };
  assert.equal(response.status(), 202);
  assert.equal(body.name, 'calculate_parcel');
  assert.equal(body.a2uiClientDataModel?.[body.surfaceId]?.parcel, String(s.number));
  assert.equal(ack.status, 'continued');
  assert.equal(ack.run_id, s.initial.runId);
  assert.ok(ack.continuation_run_id && ack.continuation_run_id !== ack.run_id);
  s.continuation = await stream(s, await continuation());
  assert.equal(s.continuation.runId, ack.continuation_run_id);
});

Then('the agent answers using that browser input and its saved instructions', async ({ page, scenario: s }) => {
  const final = s.continuation.text.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim();
  let answer;
  try { answer = JSON.parse(final); } catch {
    const fences = [...final.matchAll(/(?:^|\n)```(?:json)?[ \t]*\r?\n([\s\S]*?)\r?\n```(?=\s|$)/g)];
    assert.equal(fences.length, 1);
    answer = JSON.parse(fences[0][1]);
  }
  assert.deepEqual(answer, { marker: s.marker, result: s.number + 19 });
  const last = page.locator('[data-slot="aui_assistant-message-root"][data-role="assistant"]').last();
  await expect(last).toContainText(s.marker);
  await expect(last).toContainText(String(s.number + 19));
  s.evidence.answer = answer;
  s.evidence.continuation_run_id = s.continuation.runId;
  s.evidence.result = 'VERIFIED';
  await page.screenshot({ path: resolve(process.env.UAR_BROWSER_EVIDENCE_DIR, 'continued-answer.png'), fullPage: true });
});

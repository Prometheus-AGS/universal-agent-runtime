import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { expect } from '@playwright/test';
import { test as base, createBdd } from 'playwright-bdd';

const COMPLETION_PATH = '/api/chat/completion';
const ORCHESTRATOR = 'orchestrator-agent';
const MATH_ID = 'math-specialist-db64c2a8';
const WRITER_ID = 'writing-specialist';
// Private signatures from the saved specialists' own prompts prove which agent executed.
const MATH_MARKER = 'MATH-ce9324ee-0d63-4714-982f-f099e8c8f378';
const WRITER_BYLINE = 'By Wren Editor (writer-76b5104d)';
const TURN_TIMEOUT_MS = 600_000;
const evidenceDir = () => process.env.UAR_BROWSER_EVIDENCE_DIR;

export const test = base.extend({
  scenario: async ({}, use, testInfo) => {
    const id = randomUUID();
    const state = { id, started: performance.now(), completionPosts: [], ownedSessions: new Set(),
      record: resolve(`.prometheus/verification/browser-agent-selection/${id}.json`),
      evidence: { scenario: 'C53-BROWSER-AGENT-SELECTION-01', platform: process.platform,
        authentication_supplied_by_test: false, previous_scenarios_rerun: false,
        service_kept_running: true, artifacts_dir: evidenceDir(), screenshots: [],
        turns: {}, exchanges: [], streams: [] } };
    const errors = [];
    try { await use(state); } finally {
      for (const session of state.ownedSessions) {
        try { await api(state, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); }
        catch (error) { errors.push(`Session cleanup ${session}: ${error}`); }
      }
      state.evidence.cleaned_sessions = [...state.ownedSessions];
      try {
        for (const agent of [MATH_ID, WRITER_ID, ORCHESTRATOR]) await api(state, 'GET', `/api/agents/${agent}`);
        state.evidence.saved_agents_retained = true;
      } catch (error) { errors.push(`Saved agent check: ${error}`); }
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
  const text = await response.text();
  state.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 4000) });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

function observeResponse(page, predicate, timeout = TURN_TIMEOUT_MS) {
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
const assistantMessages = page => page.locator('[data-slot="aui_assistant-message-root"][data-role="assistant"]');

function parseEvents(text) {
  return text.split(/\r?\n\r?\n/).flatMap(block => {
    const name = block.split(/\r?\n/).find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = block.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
    return [JSON.parse(data)];
  });
}

// Records when each selection notice and each assistant answer bubble first becomes visible.
function installDomTimeline() {
  if (window.__uarSelectionTimeline) return;
  const log = [];
  const seen = new Set();
  window.__uarSelectionTimeline = log;
  const scan = () => {
    const now = performance.now();
    for (const node of document.querySelectorAll('[data-agent-selection]')) {
      const root = node.closest('[data-slot="aui_assistant-message-root"]');
      const key = `selection|${root?.getAttribute('data-message-id') ?? 'none'}|${node.getAttribute('data-agent-selection')}`;
      if (seen.has(key)) continue;
      seen.add(key);
      log.push({ t: now, kind: 'selection', message_id: root?.getAttribute('data-message-id') ?? null,
        agent_id: node.getAttribute('data-agent-selection'), text: node.textContent });
    }
    for (const root of document.querySelectorAll('[data-slot="aui_assistant-message-root"]')) {
      const text = [...root.querySelectorAll('.rounded-es-md')].map(n => n.textContent ?? '').join('').trim();
      const key = `text|${root.getAttribute('data-message-id')}`;
      if (!text || seen.has(key)) continue;
      seen.add(key);
      log.push({ t: now, kind: 'answer_text', message_id: root.getAttribute('data-message-id'), sample: text.slice(0, 200) });
    }
  };
  new MutationObserver(scan).observe(document.body, { subtree: true, childList: true, characterData: true, attributes: true });
  scan();
}

// The first Review button placed after this turn's user message: earlier turns' approval
// cards may keep or lose their buttons, so only document position identifies this turn's approval.
const turnReviewButton = page => page.locator('[data-slot="aui_user-message-root"][data-role="user"]').last()
  .locator('xpath=following::button[normalize-space()="Review approval"]').first();

// Answers only the local spawn_agent approval this turn raised, through the browser dialog.
async function approveSpawnInBrowser(page, record, button) {
  const chunk = button.locator('xpath=ancestor::*[@aria-label="Tool approval required"][1]');
  const chunkText = await chunk.innerText();
  assert.match(chunkText, /Approval required · spawn_agent/, `Only the spawn_agent approval may be answered: ${chunkText.slice(0, 300)}`);
  await button.click();
  const dialog = page.getByRole('dialog');
  await expect(dialog).toContainText('spawn_agent');
  const posted = observeResponse(page, r => r.request().method() === 'POST' && /\/api\/uar\/runs\/[^/]+\/tool-approval$/.test(new URL(r.url()).pathname), 60_000);
  await dialog.getByRole('button', { name: 'Approve', exact: true }).click();
  const response = await posted();
  const body = await response.text();
  record.approval = { chunk_text: chunkText.slice(0, 2000), path: new URL(response.url()).pathname,
    request: response.request().postDataJSON(), status: response.status(), response: body.slice(0, 2000) };
  assert.ok(response.ok(), `Approval POST failed: ${response.status()} ${body.slice(0, 400)}`);
  assert.equal(record.approval.request.approved, true);
}

async function sendTurn(page, s, label, message) {
  const record = { label, message };
  s.evidence.turns[label] = record;
  record.dom_start = await page.evaluate(() => performance.now());
  const response = observeResponse(page, isCompletion);
  await page.getByRole('textbox', { name: 'Message input', exact: true }).fill(message);
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  const completion = await response();
  const request = completion.request();
  record.session_header = request.headers()['x-uar-session-id'] ?? null;
  record.request = request.postDataJSON();
  const body = completion.text().then(text => ({ text }), error => ({ error }));
  const button = turnReviewButton(page);
  const approval = button
    .waitFor({ state: 'visible', timeout: TURN_TIMEOUT_MS }).then(() => 'approval', () => 'no-approval');
  if (await Promise.race([body.then(() => 'done'), approval]) === 'approval') await approveSpawnInBrowser(page, record, button);
  const finished = await body;
  if (finished.error) throw finished.error;
  const events = parseEvents(finished.text);
  s.evidence.streams.push({ label, path: COMPLETION_PATH, status: completion.status(), session_header: record.session_header,
    request: record.request, response: finished.text });
  assert.equal(completion.status(), 200);
  assert.match(completion.headers()['content-type'], /^text\/event-stream\b/);
  const starts = events.filter(e => e.type === 'RUN_STARTED');
  const finishes = events.filter(e => e.type === 'RUN_FINISHED');
  const errors = events.filter(e => e.type === 'RUN_ERROR' || e.type === 'RUN_CANCELLED');
  const selections = events.filter(e => e.name === 'uar.agent.selected');
  const raw = events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta ?? '').join('');
  Object.assign(record, { event_types: events.map(e => e.type === 'CUSTOM' ? `CUSTOM:${e.name}` : e.type),
    run_id: starts[0]?.runId ?? null, selection: selections[0] ?? null, selection_count: selections.length,
    errors, answer: stripThink(raw).slice(0, 4000),
    stream_selection_index: selections.length ? events.indexOf(selections[0]) : -1,
    stream_first_text_index: events.findIndex(e => e.type === 'TEXT_MESSAGE_CONTENT') });
  assert.deepEqual(errors, [], `${label} run failed`);
  assert.equal(starts.length, 1, `${label}: exactly one run`);
  assert.equal(finishes.length, 1, `${label}: exactly one terminal event`);
  assert.equal(starts[0].runId, finishes[0].runId);
  if (finishes[0].outcome) assert.equal(finishes[0].outcome.type, 'success', `${label} outcome`);
  return { record, events, selections, text: stripThink(raw) };
}

async function assertVisibleSelectionFirst(page, s, result, expected, turnIndex) {
  const { record, selections } = result;
  assert.equal(selections.length, 1, `${record.label}: exactly one selection event`);
  const value = selections[0].value;
  assert.equal(value?.agent?.id, expected.id, `${record.label} selected ${value?.agent?.id}`);
  assert.equal(value.sourceAgentId, ORCHESTRATOR);
  assert.equal(value.selectionMethod, 'intent');
  assert.ok(value.rosterRevision && value.selectedThreadId);
  assert.ok(record.stream_selection_index >= 0 && record.stream_first_text_index > record.stream_selection_index,
    `${record.label}: the selection event must precede the first answer text on the wire`);
  s.ownedSessions.add(value.selectedThreadId);
  record.selected_thread_id = value.selectedThreadId;
  // An approval pause can split one turn across several assistant bubbles; the answer must appear in exactly one.
  await expect(assistantMessages(page).filter({ hasText: expected.signature })).toHaveCount(1);
  const notices = page.locator('[data-agent-selection]');
  await expect(notices).toHaveCount(turnIndex);
  await expect(notices.last()).toHaveAttribute('data-agent-selection', expected.id);
  await expect(notices.last()).toContainText(`Selected agent: ${expected.title}`);
  const noticeBeforeAnswer = await notices.last().evaluate((notice, signature) => {
    const answer = [...document.querySelectorAll('[data-slot="aui_assistant-message-root"][data-role="assistant"]')]
      .find(root => (root.textContent ?? '').includes(signature));
    return Boolean(answer) && (answer.contains(notice)
      || Boolean(notice.compareDocumentPosition(answer) & Node.DOCUMENT_POSITION_FOLLOWING));
  }, expected.signature);
  assert.ok(noticeBeforeAnswer, `${record.label}: the selection notice is not placed before the answer`);
  const timeline = (await page.evaluate(() => window.__uarSelectionTimeline)).filter(entry => entry.t >= record.dom_start);
  record.dom_timeline = timeline;
  const visibleSelection = timeline.find(entry => entry.kind === 'selection');
  const answerId = await assistantMessages(page).filter({ hasText: expected.signature }).getAttribute('data-message-id');
  record.answer_message_id = answerId;
  const visibleText = timeline.find(entry => entry.kind === 'answer_text' && entry.message_id === answerId);
  assert.ok(visibleSelection, `${record.label}: no visible selection notice`);
  assert.ok(visibleText, `${record.label}: no visible answer text`);
  assert.equal(visibleSelection.agent_id, expected.id);
  record.visible_selection_ms_before_answer = Math.round(visibleText.t - visibleSelection.t);
  assert.ok(visibleSelection.t <= visibleText.t, `${record.label}: the selection became visible after the answer text`);
  const path = resolve(evidenceDir(), `${record.label}.png`);
  await page.screenshot({ path, fullPage: true });
  s.evidence.screenshots.push(path);
}

Given('the saved math and writing specialists and an orchestrator conversation in the browser', async ({ page, scenario: s }) => {
  assert.ok(process.env.UAR_MODEL && process.env.UAR_WORKSPACE_ID && evidenceDir());
  const url = new URL(process.env.UAR_BASE_URL);
  assert.ok(['http:', 'https:'].includes(url.protocol) && !url.username && !url.password);
  s.evidence.model = process.env.UAR_MODEL;
  s.evidence.page_errors = [];
  page.on('pageerror', error => s.evidence.page_errors.push(String(error)));
  page.on('request', request => {
    if (request.method() === 'POST' && new URL(request.url()).pathname === COMPLETION_PATH) {
      s.completionPosts.push({ at: new Date().toISOString(), body: request.postData() });
    }
  });
  const agents = {};
  for (const id of [MATH_ID, WRITER_ID, ORCHESTRATOR]) {
    agents[id] = await api(s, 'GET', `/api/agents/${id}`);
    assert.equal(agents[id].id, id);
  }
  assert.equal(agents[ORCHESTRATOR].runtime?.entry, 'orchestrator');
  const listing = await api(s, 'GET', '/api/agents');
  const eligible = listing.runtime_agents.filter(a => a.id !== ORCHESTRATOR && a.runtime?.entry !== 'orchestrator').map(a => a.id);
  assert.ok(eligible.includes(MATH_ID) && eligible.includes(WRITER_ID), 'Both specialists must be in the live roster');
  s.evidence.eligible_roster = eligible.sort();
  s.expected = {
    writing: { id: WRITER_ID, title: agents[WRITER_ID].metadata.title, signature: WRITER_BYLINE },
    math: { id: MATH_ID, title: agents[MATH_ID].metadata.title, signature: MATH_MARKER },
  };
  s.orchestratorTitle = agents[ORCHESTRATOR].metadata.title;
  await page.goto('/threads', { waitUntil: 'domcontentloaded' });
  await page.evaluate(installDomTimeline);
  await page.getByRole('button', { name: 'New thread', exact: true }).click();
  const saved = observeResponse(page, r => r.request().method() === 'POST' && /\/api\/uar\/sessions\/[^/]+\/agent-config$/.test(new URL(r.url()).pathname), 60_000);
  await page.getByRole('button', { name: 'Select agent', exact: true }).click();
  await page.getByPlaceholder('Search agents...').fill(s.orchestratorTitle);
  const option = page.getByRole('option').filter({ has: page.getByText(s.orchestratorTitle, { exact: true }) });
  await expect(option).toHaveCount(1);
  await option.click();
  const response = await saved();
  assert.equal(response.status(), 200, 'The browser must save the orchestrator session selection');
  s.evidence.agent_selection_save = { path: new URL(response.url()).pathname, request: response.request().postDataJSON() };
  assert.equal(s.evidence.agent_selection_save.request.agent_id, ORCHESTRATOR);
  await expect(page.getByRole('button', { name: 'Select agent', exact: true })).toContainText(s.orchestratorTitle);
});

When('I send a writing task in that conversation through the browser', async ({ page, scenario: s }) => {
  s.writing = await sendTurn(page, s, 'writing_turn',
    'Write a 120-word blog post for a neighborhood newsletter announcing a Saturday community tomato-planting workshop.');
  s.threadId = s.writing.record.session_header;
  assert.ok(s.threadId, 'The browser must send its conversation id');
  s.ownedSessions.add(s.threadId);
  s.evidence.thread_id = s.threadId;
});

Then('the browser shows the writing specialist selection before its specialist answer', async ({ page, scenario: s }) => {
  assert.ok(s.writing.text.includes(WRITER_BYLINE), 'Missing the saved writer byline');
  await assertVisibleSelectionFirst(page, s, s.writing, s.expected.writing, 1);
});

When('I send a math task in the same conversation through the browser', async ({ page, scenario: s }) => {
  s.left = randomInt(101, 999);
  s.right = randomInt(11, 99);
  s.evidence.math_expected = s.left * s.right;
  s.math = await sendTurn(page, s, 'math_turn', `Compute ${s.left} multiplied by ${s.right}.`);
  assert.equal(s.math.record.session_header, s.threadId, 'The math task must continue the same conversation');
});

Then('the browser shows the math specialist selection before its specialist answer', async ({ page, scenario: s }) => {
  assert.ok(s.math.text.includes(MATH_MARKER), 'Missing the saved math specialist marker');
  assert.ok(s.math.text.replace(/,/g, '').includes(String(s.left * s.right)), `Math answer lacks ${s.left * s.right}`);
  await assertVisibleSelectionFirst(page, s, s.math, s.expected.math, 2);
});

Then('each selected specialist matches execution with no duplicate answer or run', async ({ page, scenario: s }) => {
  assert.equal(s.completionPosts.length, 2, 'Exactly one browser run per task');
  await expect(page.locator('[data-slot="aui_user-message-root"][data-role="user"]')).toHaveCount(2);
  for (const expected of [s.expected.writing, s.expected.math]) {
    await expect(assistantMessages(page).filter({ hasText: expected.signature })).toHaveCount(1);
  }
  const turns = [[s.writing, s.expected.writing], [s.math, s.expected.math]];
  assert.notEqual(s.writing.record.run_id, s.math.record.run_id);
  assert.notEqual(s.writing.record.selected_thread_id, s.math.record.selected_thread_id);
  const runs = await api(s, 'GET', '/api/uar/runs');
  for (const [turn, expected] of turns) {
    const root = await api(s, 'GET', `/api/uar/runs/${turn.record.run_id}`);
    assert.equal(root.agent_id, ORCHESTRATOR);
    assert.equal(root.conversation_id, s.threadId);
    assert.equal(root.status, 'done');
    turn.record.root_run = root;
    // The run listing holds only retained active-run state; a listed child run must match the selection.
    const children = runs.filter(run => run.conversation_id === turn.record.selected_thread_id);
    turn.record.listed_child_runs = children.map(run => ({ run_id: run.run_id, agent_id: run.agent_id, status: run.status }));
    for (const child of children) assert.equal(child.agent_id, expected.id, `${turn.record.label} child executed as ${child.agent_id}`);
    assert.ok(children.length <= 1, `${turn.record.label}: duplicate child runs`);
    assert.ok(turn.text.includes(expected.signature), `${turn.record.label}: execution signature missing`);
  }
  const rootRuns = runs.filter(run => run.conversation_id === s.threadId).map(run => run.run_id);
  const known = new Set([s.writing.record.run_id, s.math.record.run_id]);
  s.evidence.listed_root_runs = rootRuns;
  assert.ok(rootRuns.every(id => known.has(id)), `Unexpected extra root run: ${rootRuns.join(', ')}`);
  s.evidence.result = 'VERIFIED';
});

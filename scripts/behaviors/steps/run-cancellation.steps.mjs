import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(150_000);
const AGENT = 'default-agent';
const POLL_MS = 500;
const START_DEADLINE_MS = 90_000;
const TERMINAL_DEADLINE_MS = 60_000;
const TERMINAL = new Set(['done', 'error', 'cancelled']);

async function request(w, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 2000) });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

const pause = ms => new Promise(done => setTimeout(done, ms));
const events = async (w, runId) => (await request(w, 'GET', `/api/uar/runs/${runId}/events`)).events;
const text = list => list.filter(e => e.eventName === 'agui.message.delta').map(e => e.data?.delta?.text ?? '').join('');

async function startRun(w, input) {
  const session = randomUUID();
  w.sessions.push(session);
  const created = await request(w, 'POST', '/api/uar/runs', { agent_id: AGENT, input, session_id: session }, [200, 201, 202]);
  const runId = created.run_id ?? created.runId;
  assert.ok(runId, `The run was created: ${JSON.stringify(created).slice(0, 300)}`);
  return runId;
}

async function waitTerminal(w, runId, deadlineMs) {
  const deadline = Date.now() + deadlineMs;
  const statuses = [];
  let status;
  while (Date.now() < deadline) {
    status = (await request(w, 'GET', `/api/uar/runs/${runId}`)).status;
    if (statuses.at(-1) !== status) statuses.push(status);
    if (TERMINAL.has(status)) break;
    await pause(POLL_MS);
  }
  return { status, statuses };
}

Given('the saved default agent is available', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.record = resolve(`.prometheus/verification/run-cancellation/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.sessions = [];
  this.evidence = { scenario: 'C50-RUN-CANCELLATION-01', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.base);
  assert.equal((await request(this, 'GET', `/api/agents/${AGENT}`)).id, AGENT);
});

When('I start a long genuine run and it begins streaming', async function () {
  this.runId = await startRun(this, 'Write a very long, detailed essay of at least 3000 words on the history of lighthouses, '
    + 'with many numbered sections covering every century from antiquity to today.');
  const deadline = Date.now() + START_DEADLINE_MS;
  let streamed = '';
  while (Date.now() < deadline) {
    streamed = text(await events(this, this.runId));
    if (streamed.length > 0) break;
    const status = (await request(this, 'GET', `/api/uar/runs/${this.runId}`)).status;
    assert.ok(!TERMINAL.has(status), `The run ended before it could be cancelled: ${status}`);
    await pause(POLL_MS);
  }
  assert.ok(streamed.length > 0, 'Live execution started streaming before cancellation');
  this.evidence.run_id = this.runId;
  this.evidence.streamed_chars_before_cancel = streamed.length;
});

When('I cancel that run through the cancellation endpoint', async function () {
  this.evidence.cancel_response = await request(this, 'POST', `/api/uar/runs/${this.runId}/cancel`);
  assert.equal(this.evidence.cancel_response.cancelled, true);
  this.cancelledAt = Date.now();
});

Then('the run ends cancelled with no later success', async function () {
  const terminal = await waitTerminal(this, this.runId, TERMINAL_DEADLINE_MS);
  Object.assign(this.evidence, { terminal_status: terminal.status, statuses_after_cancel: terminal.statuses });
  assert.equal(terminal.status, 'cancelled', `Terminal status after cancel: ${terminal.status}`);
  const first = await events(this, this.runId);
  await pause(3_000);
  const later = await events(this, this.runId);
  const names = later.map(e => e.eventName);
  Object.assign(this.evidence, { event_names: [...new Set(names)], event_count: later.length,
    events_added_after_settle: later.length - first.length, final_events: later.slice(-5),
    streamed_chars_total: text(later).length });
  assert.equal(later.length, first.length, 'No events arrive after the cancelled terminal');
  assert.equal((await request(this, 'GET', `/api/uar/runs/${this.runId}`)).status, 'cancelled', 'Status stays cancelled');
  const done = later.filter(e => e.eventName === 'agui.done');
  for (const event of done) assert.notEqual(event.data?.outcome?.type ?? event.data?.status, 'success', 'No success outcome after cancel');
});

Then('its model calls are settled or explicitly uncertain', async function () {
  const calls = await request(this, 'GET', `/api/uar/accounting/calls/${this.runId}`);
  this.evidence.calls = calls;
  assert.ok(Array.isArray(calls) && calls.length > 0, 'The cancelled run admitted at least one model call');
  // Explicit uncertainty: the run itself reports that durable usage awaits reconciliation.
  const diagnostics = (await events(this, this.runId)).filter(e => e.eventName === 'agui.custom'
    && e.data?.name === 'uar.presentation.diagnostic').map(e => e.data.value);
  this.evidence.accounting_diagnostics = diagnostics;
  const reportedUncertain = diagnostics.some(d => d?.code === 'accounting_unresolved');
  for (const call of calls) {
    const settled = call.state === 'settled';
    const uncertain = call.state === 'pending' && (Boolean(call.reconciliation_evidence) || reportedUncertain);
    assert.ok(settled || uncertain, `Call ${call.admission?.identity?.call_id} is ${call.state} without an explicit uncertainty report`);
  }
});

Then('a fresh ordinary prompt still succeeds', async function () {
  const runId = await startRun(this, 'What is 17 plus 25? Reply with only the number.');
  const terminal = await waitTerminal(this, runId, 120_000);
  const answer = text(await events(this, runId)).replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
  this.evidence.follow_up = { run_id: runId, status: terminal.status, answer: answer.slice(0, 500) };
  assert.equal(terminal.status, 'done');
  assert.match(answer, /\b42\b/);
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 60_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  for (const session of this.sessions) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${session}: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});

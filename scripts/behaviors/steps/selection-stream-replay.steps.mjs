import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(180_000);
const ORCHESTRATOR = 'orchestrator-agent';
const MATH_ID = 'math-specialist-db64c2a8';
const ROUTE_PATH = `/api/uar/ag-ui/agents/${ORCHESTRATOR}`;

async function request(w, method, path, payload, expected = [200]) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 2000) });
  assert.ok(expected.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

// One AG-UI exchange: POST a run input, or GET a reconnect with an optional cursor; frames keep their SSE ids.
async function sse(w, label, { method, path, payload, cursor }) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(150_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace,
      ...(cursor && { 'Last-Event-ID': cursor }) },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  assert.equal(response.status, 200, `${label} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  const frames = text.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const id = lines.find(line => line.startsWith('id:'))?.slice(3).trim() ?? null;
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).replace(/^ /, '')).join('\n');
    return data && data !== '[DONE]' ? [{ id, event: JSON.parse(data) }] : [];
  });
  w.evidence.streams[label] = frames.map(f => ({ id: f.id, type: f.event.type, name: f.event.name, runId: f.event.runId }));
  return frames;
}

const answerText = frames => frames.filter(f => f.event.type === 'TEXT_MESSAGE_CONTENT').map(f => f.event.delta ?? '').join('')
  .replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();

// Run ids created since the scenario began, with each run's accounting-call count.
async function dispatches(w) {
  const runs = await request(w, 'GET', '/api/uar/runs?limit=200');
  const fresh = runs.filter(run => !w.baselineRuns.has(run.run_id));
  const result = {};
  for (const run of fresh) {
    const calls = await request(w, 'GET', `/api/uar/accounting/calls/${encodeURIComponent(run.run_id)}`, undefined, [200, 404]);
    result[run.run_id] = { agent_id: run.agent_id, status: run.status, calls: Array.isArray(calls) ? calls.length : 0 };
  }
  return result;
}

// Route one turn on a thread; answer only the spawn_agent interrupt this run raised.
async function turn(w, label, threadId, task, parentRunId) {
  const firstRunId = randomUUID();
  const firstPayload = { threadId, runId: firstRunId, ...(parentRunId && { parentRunId }), protocolVersion: '1.0',
    messages: [{ id: randomUUID(), role: 'user', content: task }], tools: [], context: [],
    forwardedProps: { uar: { model: w.model } } };
  let payload = firstPayload;
  let frames = await sse(w, `${label}.first`, { method: 'POST', path: ROUTE_PATH, payload });
  const finished = frames.find(f => f.event.type === 'RUN_FINISHED')?.event;
  if (finished?.outcome?.type === 'interrupt') {
    const interrupts = finished.outcome.interrupts;
    assert.equal(interrupts.length, 1, 'Expected one local delegation approval');
    const [interrupt] = interrupts;
    assert.equal(interrupt.reason, 'tool_approval');
    assert.ok(frames.some(f => f.event.type === 'TOOL_CALL_START' && f.event.toolCallId === interrupt.toolCallId
      && f.event.toolCallName === 'spawn_agent'), 'Only the requested local agent-spawn approval may be answered');
    payload = { threadId, runId: randomUUID(), parentRunId: firstRunId, protocolVersion: '1.0',
      messages: [], tools: [], context: [], forwardedProps: { uar: {} },
      resume: [{ interruptId: interrupt.id, status: 'resolved', payload: { approved: true } }] };
    frames = await sse(w, `${label}.resumed`, { method: 'POST', path: ROUTE_PATH, payload });
  }
  const selections = frames.filter(f => f.event.type === 'CUSTOM' && f.event.name === 'uar.agent.selected');
  const done = frames.filter(f => f.event.type === 'RUN_FINISHED');
  return { payload, frames, selections, done, runId: payload.runId, text: answerText(frames) };
}

function sameFrames(actual, expected) {
  return actual.map(f => [f.id, JSON.stringify(f.event)]).join('\n') === expected.map(f => [f.id, JSON.stringify(f.event)]).join('\n');
}

Given('the saved mathematics specialist is in the live roster', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.record = resolve(`.prometheus/verification/selection-stream-replay/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.threadId = randomUUID();
  this.evidence = { scenario: 'E10-SELECTION-STREAM-REPLAY', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, model: this.model,
    thread_id: this.threadId, dispatch_evidence: 'new run ids since start and per-run accounting call counts', streams: {}, exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model?.includes('/'));
  assert.equal((await request(this, 'GET', `/api/agents/${MATH_ID}`)).id, MATH_ID);
  this.baselineRuns = new Set((await request(this, 'GET', '/api/uar/runs?limit=200')).map(run => run.run_id));
});

When('the orchestrator routes a fresh calculation and I capture its selection stream and dispatches', async function () {
  const left = randomInt(21, 89), right = randomInt(11, 49);
  this.expected = [left * right];
  this.original = await turn(this, 'original', this.threadId, `Compute ${left} multiplied by ${right}. Reply with the number.`);
  assert.equal(this.original.selections.length, 1, 'Expected one agent-selection event');
  assert.equal(this.original.selections[0].event.value.agent.id, MATH_ID);
  assert.equal(this.original.done.length, 1);
  assert.equal(this.original.done[0].event.outcome?.type, 'success');
  assert.ok(this.original.text.includes(String(this.expected[0])), `Answer: ${this.original.text.slice(0, 300)}`);
  this.evidence.selection = { frame_id: this.original.selections[0].id, value: this.original.selections[0].event.value, run_id: this.original.runId };
  this.before = await dispatches(this);
  this.evidence.dispatches_after_original = this.before;
  // Model calls are accounted to the root run; a dispatch is a new specialist child run.
  const runs = Object.values(this.before);
  assert.equal(runs.filter(run => run.agent_id === MATH_ID).length, 1, 'The specialist was dispatched once');
  assert.ok(runs.reduce((sum, run) => sum + run.calls, 0) > 0, 'The routed turn made model calls');
});

When("I reconnect to that run from the selection event's cursor", async function () {
  const cursor = this.original.selections[0].id;
  this.reconnected = await sse(this, 'reconnect', { method: 'GET', cursor,
    path: `${ROUTE_PATH}/threads/${this.threadId}/runs/${this.original.runId}/stream` });
});

When('I replay the identical admitted request', async function () {
  this.replayed = await sse(this, 'replay', { method: 'POST', path: ROUTE_PATH, payload: this.original.payload });
});

Then('both replays preserve the run and selection identity and no second dispatch occurred', async function () {
  const at = this.original.frames.indexOf(this.original.selections[0]);
  const suffix = this.original.frames.slice(at + 1);
  assert.ok(this.reconnected.length > 0 && !this.reconnected.some(f => f.event.type === 'RUN_STARTED'), 'A cursor reconnect resumes after the cursor');
  assert.ok(sameFrames(this.reconnected, suffix), 'The reconnect replays exactly the frames after the selection cursor');
  assert.ok(sameFrames(this.replayed, this.original.frames), 'The identical request replays the original frames and identities');
  for (const frames of [this.reconnected, this.replayed]) {
    assert.ok(frames.every(f => f.event.runId === undefined || f.event.runId === this.original.runId));
  }
  const after = await dispatches(this);
  this.evidence.dispatches_after_replays = after;
  assert.deepEqual(after, this.before, 'Reconnect and replay must not create or call anything');
});

When('I send a distinct next turn on the same thread', async function () {
  const left = randomInt(21, 89), right = randomInt(11, 49);
  this.expected.push(left * right);
  this.next = await turn(this, 'next', this.threadId, `Now compute ${left} multiplied by ${right}. Reply with the number.`, this.original.runId);
});

Then('the next turn performs a fresh selection with its own dispatch and a correct answer', async function () {
  assert.equal(this.next.selections.length, 1, 'Expected a fresh selection event');
  const selection = this.next.selections[0];
  assert.notEqual(this.next.runId, this.original.runId);
  assert.equal(selection.event.runId, this.next.runId);
  assert.notEqual(selection.id, this.original.selections[0].id);
  assert.equal(selection.event.value.agent.id, MATH_ID);
  assert.equal(this.next.done[0]?.event.outcome?.type, 'success');
  assert.ok(this.next.text.includes(String(this.expected[1])), `Answer: ${this.next.text.slice(0, 300)}`);
  const after = await dispatches(this);
  this.evidence.dispatches_after_next = after;
  const added = Object.keys(after).filter(id => !(id in this.before));
  assert.equal(added.filter(id => after[id].agent_id === MATH_ID).length, 1, 'The next turn dispatches the specialist once more');
  assert.ok(added.reduce((sum, id) => sum + after[id].calls, 0) > 0, 'The next turn made model calls');
  for (const id of Object.keys(this.before)) assert.deepEqual(after[id], this.before[id], 'Earlier runs stay unchanged');
  this.evidence.next_selection = { frame_id: selection.id, value: selection.event.value, run_id: this.next.runId };
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 120_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  try { await request(this, 'DELETE', `/api/uar/sessions/${this.threadId}`, undefined, [204, 404]); }
  catch (error) { errors.push(`Session cleanup: ${error}`); }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});

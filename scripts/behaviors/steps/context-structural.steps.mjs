import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(180_000);
const POLL_MS = 500;
const TERMINAL = new Set(['done', 'error', 'cancelled']);
const PAIRS = 5;

async function request(w, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 3000) });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

const pause = ms => new Promise(done => setTimeout(done, ms));
const code = () => `DEPOT-${randomUUID().slice(0, 6).toUpperCase()}`;

function setup(w, scenario) {
  w.started = performance.now();
  w.id = randomUUID();
  w.short = w.id.slice(0, 8);
  w.record = resolve(`.prometheus/verification/context-structural/${w.id}.json`);
  w.base = new URL(process.env.UAR_BASE_URL);
  w.workspace = process.env.UAR_WORKSPACE_ID;
  w.model = process.env.UAR_MODEL;
  w.sessions = [];
  w.evidence = { scenario, platform: process.platform, model: w.model,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, runs: {}, exchanges: [] };
  assert.ok(w.workspace && w.model?.includes('/'));
}

async function createAgent(w, strategy) {
  const agent = await request(w, 'GET', '/api/agents/default-agent');
  w.agentId = `disposable-context-${w.short}`;
  Object.assign(agent, { id: w.agentId, version: '1.0.0', extensions: { 'uar.run_policy': { context_strategy: strategy } } });
  agent.metadata = { ...agent.metadata, title: 'Disposable context agent', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Answer only from this conversation. Return only JSON with the requested keys. Use the string UNKNOWN for any value the conversation does not contain. Never guess.', instructions: [] };
  const split = w.model.indexOf('/');
  agent.policy.provider = { default: { provider: w.model.slice(0, split), model: w.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.tools.allow = [];
  agent.memory.kb.enabled = false;
  agent.memory.conversation = { enabled: false };
  await request(w, 'POST', '/api/agents', agent, 201);
  w.agentCreated = true;
  w.evidence.strategy = strategy;
}

// Seeded history: pair i states one depot code; each pair is one user/assistant group.
function seededHistory(codes) {
  return codes.flatMap((value, index) => [
    { role: 'user', content: `Record ${index + 1}: depot ${String.fromCharCode(65 + index)} uses code ${value}.` },
    { role: 'assistant', content: `Noted: depot ${String.fromCharCode(65 + index)} uses code ${value}.` },
  ]);
}

async function runWithHistory(w, label, codes, question) {
  const session = randomUUID();
  w.sessions.push(session);
  const created = await request(w, 'POST', '/api/uar/runs', { agent_id: w.agentId, session_id: session, input: question,
    history: { session_id: session, messages: seededHistory(codes) } }, [200, 201, 202]);
  const runId = created.run_id ?? created.runId;
  const deadline = Date.now() + 150_000;
  let status;
  while (Date.now() < deadline) {
    status = (await request(w, 'GET', `/api/uar/runs/${runId}`)).status;
    if (TERMINAL.has(status)) break;
    await pause(POLL_MS);
  }
  const run = await request(w, 'GET', `/api/uar/runs/${runId}`);
  const events = (await request(w, 'GET', `/api/uar/runs/${runId}/events`)).events;
  const final = events.filter(e => e.eventName === 'agui.message.delta').map(e => e.data?.delta?.text ?? '').join('')
    .replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
  let answer = null;
  try { answer = JSON.parse(final); } catch {
    const blocks = [...final.matchAll(/```(?:json)?\s*([\s\S]*?)```/g)];
    if (blocks.length === 1) answer = JSON.parse(blocks[0][1]);
  }
  const record = { run_id: runId, status, history: created.history, seeded_messages: created.seeded_messages,
    context_reduction: run.context_reduction, answer, final: final.slice(0, 600) };
  w.evidence.runs[label] = record;
  assert.equal(status, 'done');
  assert.equal(created.seeded_messages, codes.length * 2, 'The full history was seeded');
  assert.ok(answer, `Require one JSON answer: ${final.slice(0, 300)}`);
  return record;
}

// The model may name keys "A" or "depot_a_code"; select the value by depot letter.
const pick = (answer, letter) => {
  const key = Object.keys(answer).find(k => k === letter || new RegExp(`(^|_)(depot_?)?${letter}(_|$)`, 'i').test(k));
  return key === undefined ? undefined : answer[key];
};

const question = letters => `What codes do depots ${letters.join(' and ')} use? Return only JSON with keys ${letters.map(l => `"${l}"`).join(', ')}.`;

Given('a disposable agent using a sliding window of 4 messages', async function () {
  setup(this, 'C45-SLIDING-WINDOW-01');
  await createAgent(this, { type: 'sliding_window', max_messages: 4 });
});

When('I continue a seeded 10-message conversation that names an old and a recent depot code', async function () {
  this.codes = Array.from({ length: PAIRS }, code);
  this.result = await runWithHistory(this, 'sliding', this.codes, question(['A', 'E']));
});

Then('the reduction receipt shows old groups left the prepared input', function () {
  const receipt = this.result.context_reduction;
  assert.equal(receipt.requested_strategy.type, 'sliding_window');
  assert.ok(receipt.source_messages > receipt.prepared_messages, `Receipt: ${JSON.stringify(receipt).slice(0, 400)}`);
  assert.equal(receipt.changed, true);
  assert.ok(receipt.prepared_messages <= 4 + 2, 'Prepared input is bounded by the window (plus system and current turn)');
});

Then('the answer uses the recent fact and cannot see the old one', function () {
  assert.equal(pick(this.result.answer, 'E'), this.codes[4], 'The most recent group is retained and used');
  assert.equal(pick(this.result.answer, 'A'), 'UNKNOWN', 'The oldest group left the provider input');
  this.evidence.result = 'VERIFIED';
});

Given('a disposable agent using truncate-middle keeping the first 2 and last 2 messages', async function () {
  setup(this, 'C44-TRUNCATE-MIDDLE-01');
  await createAgent(this, { type: 'truncate_middle', keep_first: 2, keep_last: 2 });
});

When('I continue a seeded 10-message conversation that names first, middle and latest depot codes', async function () {
  this.codes = Array.from({ length: PAIRS }, code);
  this.result = await runWithHistory(this, 'truncate_middle', this.codes, question(['A', 'C', 'E']));
});

Then('the reduction receipt shows the middle removed and the first and latest groups retained', function () {
  const receipt = this.result.context_reduction;
  assert.equal(receipt.requested_strategy.type, 'truncate_middle');
  assert.equal(receipt.changed, true);
  assert.ok(receipt.source_messages > receipt.prepared_messages, `Receipt: ${JSON.stringify(receipt).slice(0, 400)}`);
});

Then('the answer uses the first and latest facts and cannot see the middle one', function () {
  assert.equal(pick(this.result.answer, 'A'), this.codes[0], 'The first group is retained and used');
  assert.equal(pick(this.result.answer, 'E'), this.codes[4], 'The latest group is retained and used');
  assert.equal(pick(this.result.answer, 'C'), 'UNKNOWN', 'The middle group left the provider input');
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 60_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  for (const session of this.sessions) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${session}: ${error}`); }
  }
  if (this.agentCreated) {
    try { await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Agent cleanup: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});

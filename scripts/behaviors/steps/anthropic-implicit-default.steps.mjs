import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { stripReasoning } from '../support/answer-oracle.mjs';

setDefaultTimeout(240_000);
const HEADERS = { 'Content-Type': 'application/json', 'anthropic-version': '2023-06-01' };

async function call(w, payload) {
  const exchange = { method: 'POST', path: '/v1/messages', request: payload };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL('/v1/messages', w.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(170_000),
    headers: { ...HEADERS, ...(payload.stream && { Accept: 'text/event-stream' }), 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify(payload),
  });
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.equal(response.status, 200, `POST /v1/messages: HTTP ${response.status}: ${exchange.response.slice(0, 800)}`);
  return { response, text: exchange.response };
}

async function get(w, path) {
  const response = await fetch(new URL(path, w.base), { redirect: 'error', signal: AbortSignal.timeout(30_000),
    headers: { 'x-uar-workspace-id': w.workspace } });
  const text = await response.text();
  w.evidence.exchanges.push({ method: 'GET', path, status: response.status, response: text.slice(0, 3000) });
  assert.equal(response.status, 200, `GET ${path}: ${response.status}`);
  return JSON.parse(text);
}

// Messages omit agent_id; the session id finds the run that UAR created for each turn.
async function runsFor(w, session) {
  const runs = await get(w, '/api/uar/runs?limit=100');
  return runs.filter(run => run.conversation_id === session);
}

function numberIn(text) {
  const final = stripReasoning(text);
  const numbers = [...final.matchAll(/-?\d[\d,]*/g)].map(m => Number(m[0].replaceAll(',', '')));
  return { final, numbers };
}

async function turnJson(w, messages) {
  const { text } = await call(w, { model: w.model, session_id: w.session, max_tokens: 4096, stream: false, messages });
  const message = JSON.parse(text);
  assert.equal(message.type, 'message');
  assert.equal(message.role, 'assistant');
  assert.equal(message.stop_reason, 'end_turn');
  assert.match(message.id, /^msg_.+/);
  const content = message.content.filter(block => block.type === 'text').map(block => block.text).join('');
  return { content, deltas: 0 };
}

async function turnStream(w, messages) {
  const { response, text } = await call(w, { model: w.model, session_id: w.session, max_tokens: 4096, stream: true, messages });
  assert.match(response.headers.get('content-type') ?? '', /^text\/event-stream\b/);
  const events = text.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const event = lines.find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    return data ? [{ event, data: JSON.parse(data) }] : [];
  });
  const names = events.map(e => e.event ?? e.data.type);
  assert.equal(names[0], 'message_start');
  assert.equal(names.at(-1), 'message_stop');
  const deltas = events.filter(e => (e.event ?? e.data.type) === 'content_block_delta' && e.data.delta?.type === 'text_delta');
  const stop = events.find(e => (e.event ?? e.data.type) === 'message_delta');
  assert.equal(stop?.data.delta?.stop_reason, 'end_turn');
  return { content: deltas.map(e => e.data.delta.text).join(''), deltas: deltas.length };
}

Given('an Anthropic-compatible request that omits agent_id', async function () {
  this.started = performance.now();
  this.session = randomUUID();
  this.model = process.env.UAR_MODEL;
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.base = new URL(process.env.UAR_BASE_URL);
  assert.ok(this.model && this.workspace, 'Set UAR_MODEL and UAR_WORKSPACE_ID');
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  this.record = resolve(`.prometheus/verification/anthropic-implicit-default/${this.session}.json`);
  this.evidence = { scenario: 'I20-I05-ANTHROPIC-IMPLICIT-DEFAULT', client_platform: process.platform, session_id: this.session,
    model: this.model, agent_id_sent: false, authentication_supplied_by_test: false,
    follow_up_contract: 'client-supplied transcript (Anthropic Messages is stateless)', exchanges: [] };
});

async function twoTurns(w, turn) {
  const left = randomInt(21, 80), right = randomInt(11, 40), extra = randomInt(101, 500), more = randomInt(11, 90);
  const first = `What is (${left} multiplied by ${right}) plus ${extra}? Reply with only the number.`;
  w.expected = [left * right + extra, left * right + extra + more];
  w.first = await turn(w, [{ role: 'user', content: first }]);
  const followUp = `Add ${more} to that result. Reply with only the number.`;
  w.second = await turn(w, [{ role: 'user', content: first }, { role: 'assistant', content: stripReasoning(w.first.content) },
    { role: 'user', content: followUp }]);
  w.evidence.expected = w.expected;
  w.evidence.turns = [w.first, w.second].map(t => ({ deltas: t.deltas, final: numberIn(t.content).final.slice(0, 200) }));
}

When('I send a fresh calculation and then a dependent follow-up as JSON Messages requests', async function () {
  this.mode = 'json';
  await twoTurns(this, turnJson);
});

When('I send a fresh calculation and then a dependent follow-up as streamed Messages requests', async function () {
  this.mode = 'stream';
  await twoTurns(this, turnStream);
});

async function assertTurns(w) {
  assert.ok(numberIn(w.first.content).numbers.includes(w.expected[0]), `First answer must be ${w.expected[0]}`);
  assert.ok(numberIn(w.second.content).numbers.includes(w.expected[1]), `Follow-up answer must be ${w.expected[1]}`);
  const runs = await runsFor(w, w.session);
  w.evidence.runs = runs.map(run => ({ run_id: run.run_id, agent_id: run.agent_id, status: run.status, effective_model: run.effective_model }));
  assert.equal(runs.length, 2, 'One run per Messages request');
  for (const run of runs) {
    assert.equal(run.agent_id, 'default-agent', 'An omitted agent_id must resolve the protected default agent');
    assert.equal(run.status, 'done');
  }
  w.evidence.result = 'VERIFIED';
}

Then('both answers are correct and complete and the runs used the default agent', async function () {
  await assertTurns(this);
});

Then('both streams deliver incremental text and complete with correct answers from the default agent', async function () {
  assert.ok(this.first.deltas >= 1 && this.second.deltas >= 1, 'Each stream must deliver content deltas before message_stop');
  await assertTurns(this);
});

After(async function ({ result }) {
  if (!this.evidence) return;
  this.evidence.mode = this.mode;
  this.evidence.scenario_status = result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: result.status }), 'application/json');
});

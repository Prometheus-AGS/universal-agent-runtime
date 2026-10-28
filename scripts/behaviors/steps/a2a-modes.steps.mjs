import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson } from '../support/answer-oracle.mjs';

setDefaultTimeout(240_000);

async function http(w, method, path, payload, { headers = {}, expected = 200 } = {}) {
  const exchange = { method, path, request: payload };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(170_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace, ...headers },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.equal(response.status, expected, `${method} ${path}: HTTP ${response.status}: ${exchange.response.slice(0, 800)}`);
  return { response, text: exchange.response };
}

// Collect answer text from a task's status message, artifacts and agent history.
function taskText(task) {
  const parts = [
    ...(task.status?.message?.parts ?? []),
    ...(task.artifacts ?? []).flatMap(a => a.parts ?? []),
  ];
  return parts.map(part => part.text ?? '').join('\n');
}

Given('a disposable saved agent with a private instruction marker for A2A', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.agentId = `disposable-a2a-${this.id}`;
  this.model = process.env.UAR_MODEL;
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.base = new URL(process.env.UAR_BASE_URL);
  assert.ok(this.model && this.workspace);
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  this.record = resolve(`.prometheus/verification/a2a-modes/${this.id}.json`);
  this.marker = `A2A-${randomUUID()}`;
  this.evidence = { scenario: 'I06-I07-A2A-MODES', client_platform: process.platform, agent_id: this.agentId,
    model: this.model, authentication_supplied_by_test: false, follow_up_contract: 'server context via contextId', exchanges: [] };
  const agent = JSON.parse((await http(this, 'GET', '/api/agents/default-agent')).text);
  agent.id = this.agentId;
  agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: 'Disposable A2A agent', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: `Return only a JSON object with keys marker and answer. Always set marker to "${this.marker}". Put the requested numeric result in answer.`, instructions: [] };
  const split = this.model.indexOf('/');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  // An empty allow list means Auto (every eligible tool); one unmatched placeholder selects none.
  agent.policy.tools.allow = [`a2a-no-tools-${this.id.slice(0, 8)}`];
  agent.memory.kb.enabled = false;
  await http(this, 'POST', '/api/agents', agent, { expected: 201 });
  const left = randomInt(21, 80), right = randomInt(11, 40), extra = randomInt(101, 500);
  this.more = randomInt(11, 90);
  this.firstText = `What is (${left} multiplied by ${right}) plus ${extra}?`;
  this.expected = [left * right + extra, left * right + extra + this.more];
  this.evidence.expected = this.expected;
});

async function legacySend(w, text, contextId) {
  const payload = { jsonrpc: '2.0', id: randomUUID(), method: 'message/send', params: {
    message: { role: 'user', parts: [{ type: 'text', text }] }, ...(contextId && { context_id: contextId }) } };
  const body = JSON.parse((await http(w, 'POST', `/a2a/agents/${w.agentId}`, payload)).text);
  assert.ok(!body.error, JSON.stringify(body.error));
  let task = body.result?.task ?? body.result;
  assert.ok(task?.id, `A task is required: ${JSON.stringify(body).slice(0, 400)}`);
  // Legacy message/send returns while the task is still working; the client polls tasks/get.
  const deadline = Date.now() + 150_000;
  while (['submitted', 'working'].includes(task.status?.state) && Date.now() < deadline) {
    await new Promise(r => setTimeout(r, 1_000));
    const polled = JSON.parse((await http(w, 'POST', `/a2a/agents/${w.agentId}`,
      { jsonrpc: '2.0', id: randomUUID(), method: 'tasks/get', params: { id: task.id } })).text);
    assert.ok(!polled.error, JSON.stringify(polled.error));
    task = polled.result?.task ?? polled.result;
  }
  return task;
}

When('I send a fresh calculation and then a dependent follow-up through legacy A2A message\\/send', async function () {
  this.mode = 'legacy-json';
  this.first = await legacySend(this, this.firstText);
  this.second = await legacySend(this, `Add ${this.more} to your previous answer.`, this.first.context_id);
});

Then("both legacy tasks complete with the agent's marker and the correct answers in one context", function () {
  for (const [task, expected] of [[this.first, this.expected[0]], [this.second, this.expected[1]]]) {
    assert.equal(task.status?.state, 'completed', `Task state: ${task.status?.state}`);
    const answer = parseAnswerJson(taskText(task));
    assert.equal(answer.marker, this.marker);
    assert.equal(Number(answer.answer), expected);
  }
  assert.ok(this.first.context_id, 'The first task must report a context');
  assert.equal(this.second.context_id, this.first.context_id, 'The follow-up continues the same context');
  this.evidence.tasks = [this.first, this.second].map(t => ({ id: t.id, context_id: t.context_id, state: t.status?.state }));
  this.evidence.result = 'VERIFIED';
});

async function streamSend(w, text, contextId) {
  const payload = { jsonrpc: '2.0', id: randomUUID(), method: 'SendStreamingMessage', params: {
    message: { messageId: randomUUID(), role: 'ROLE_USER', parts: [{ text }], ...(contextId && { contextId }) },
    configuration: {} } };
  const { response, text: raw } = await http(w, 'POST', `/a2a/v1/agents/${w.agentId}`, payload,
    { headers: { 'a2a-version': '1.0', Accept: 'text/event-stream' } });
  assert.match(response.headers.get('content-type') ?? '', /^text\/event-stream\b/);
  const envelopes = raw.split(/\r?\n\r?\n/).flatMap(block => {
    const data = block.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    return data ? [JSON.parse(data)] : [];
  });
  assert.ok(envelopes.every(e => !e.error), JSON.stringify(envelopes.find(e => e.error)?.error));
  const results = envelopes.map(e => e.result);
  assert.ok(results[0]?.task, 'The stream starts with the task');
  const finalStatus = results.filter(r => r.statusUpdate).at(-1)?.statusUpdate;
  const task = results[0].task;
  // The answer repeats across snapshots; read it once, from the final status message or else the last artifact.
  const partsText = parts => (parts ?? []).map(p => p.text ?? '').join('');
  const lastArtifact = results.filter(r => r.artifactUpdate).at(-1)?.artifactUpdate.artifact?.parts;
  const textOut = partsText(finalStatus?.status?.message?.parts) || partsText(lastArtifact) || taskText(task);
  return { task, finalStatus, events: results.map(r => Object.keys(r)[0]), text: textOut };
}

When('I send a fresh calculation and then a dependent follow-up through A2A v1 SendStreamingMessage', async function () {
  this.mode = 'v1-stream';
  this.first = await streamSend(this, this.firstText);
  this.second = await streamSend(this, `Add ${this.more} to your previous answer.`, this.first.task.contextId);
});

Then("both streams progress to a completed task with the agent's marker and the correct answers in one context", function () {
  for (const [turn, expected] of [[this.first, this.expected[0]], [this.second, this.expected[1]]]) {
    assert.ok(turn.events.length >= 2, `The stream must progress beyond the initial task: ${turn.events.join(',')}`);
    assert.equal(turn.finalStatus?.status?.state, 'TASK_STATE_COMPLETED', `Final state: ${JSON.stringify(turn.finalStatus?.status?.state)}`);
    const answer = parseAnswerJson(turn.text);
    assert.equal(answer.marker, this.marker);
    assert.equal(Number(answer.answer), expected);
  }
  assert.ok(this.first.task.contextId);
  assert.equal(this.second.task.contextId, this.first.task.contextId, 'The follow-up continues the same context');
  this.evidence.streams = [this.first, this.second].map(t => ({ task_id: t.task.id, context_id: t.task.contextId, events: t.events }));
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  let cleanupError;
  if (this.created) {
    try { await http(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, { expected: 204 }); }
    catch (error) { cleanupError = error; }
  }
  this.evidence.mode = this.mode;
  this.evidence.scenario_status = cleanupError ? 'FAILED' : result.status;
  if (cleanupError) this.evidence.cleanup_error = cleanupError.message;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.scenario_status }), 'application/json');
  if (cleanupError) throw cleanupError;
});

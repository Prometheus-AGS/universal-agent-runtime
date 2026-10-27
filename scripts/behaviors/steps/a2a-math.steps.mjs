import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(125_000);

async function request(world, method, path, payload) {
  const response = await fetch(new URL(path, world.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', 'a2a-version': '1.0',
      'x-uar-workspace-id': world.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  world.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text });
  assert.equal(response.status, 200, `${method} ${path}: HTTP ${response.status}: ${text.slice(0, 1000)}`);
  return JSON.parse(text);
}

Given('the saved math agent is available through A2A', async function () {
  this.started = performance.now();
  this.requestId = randomUUID();
  this.agentId = 'math-specialist-db64c2a8';
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.record = resolve(`.prometheus/verification/a2a/${this.requestId}.json`);
  this.evidence = { scenario: 'A2A-01', client_platform: process.platform,
    agent_id: this.agentId, protocol: 'A2A v1 JSON-RPC SendMessage',
    authentication_supplied_by_test: false, service_kept_running: true, exchanges: [] };
  assert.ok(this.workspace && process.env.UAR_MODEL, 'Set UAR_WORKSPACE_ID and UAR_MODEL');
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol));
  assert.ok(!this.base.username && !this.base.password, 'Use an endpoint without URL credentials');
  const agent = await request(this, 'GET', `/api/agents/${this.agentId}`);
  assert.equal(agent.id, this.agentId);
  const route = agent.policy.provider.default;
  this.evidence.configured_model = `${route.provider}/${route.model}`;
  assert.equal(this.evidence.configured_model, process.env.UAR_MODEL);
  this.marker = agent.prompt.system.match(/MATH-[a-f0-9-]+/)?.[0];
  assert.ok(this.marker, 'Saved mathematics identity is missing');
});

When('I send it a fresh arithmetic question through A2A SendMessage', async function () {
  const left = randomInt(21, 80);
  const right = randomInt(11, 40);
  const extra = randomInt(101, 500);
  this.evidence.expected = { specialist: this.marker, answer: left * right + extra };
  this.response = await request(this, 'POST', `/a2a/v1/agents/${this.agentId}`, {
    jsonrpc: '2.0', id: this.requestId, method: 'SendMessage',
    params: { message: { messageId: randomUUID(), role: 'ROLE_USER',
      parts: [{ text: `What is (${left} multiplied by ${right}) plus ${extra}? Follow your saved specialist/answer JSON instructions.` }] },
    configuration: { returnImmediately: false } },
  });
});

Then('the completed A2A task contains its saved identity and correct answer', function () {
  assert.equal(this.response.jsonrpc, '2.0');
  assert.equal(this.response.id, this.requestId);
  assert.ok(!Object.hasOwn(this.response, 'error'), JSON.stringify(this.response.error));
  const task = this.response.result?.task;
  assert.ok(task && typeof task.id === 'string' && task.id.length > 0);
  assert.ok(typeof task.contextId === 'string' && task.contextId.length > 0);
  assert.equal(task.status?.state, 'TASK_STATE_COMPLETED');
  assert.equal(task.metadata?.agent_id, this.agentId);
  const message = task.status.message;
  assert.equal(message.role, 'ROLE_AGENT');
  assert.equal(message.taskId, task.id);
  assert.equal(message.contextId, task.contextId);
  assert.ok(Array.isArray(message.parts) && message.parts.length > 0);
  assert.ok(message.parts.every(part => typeof part.text === 'string'));
  const content = message.parts.map(part => part.text).join('');
  const final = content.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim();
  assert.ok(final && !final.includes('<think>'), 'A completed final answer is required');
  let answer;
  try { answer = JSON.parse(final); } catch {
    const fences = [...final.matchAll(/(?:^|\n)```(?:json)?[ \t]*\r?\n([\s\S]*?)\r?\n```(?=\s|$)/g)];
    assert.equal(fences.length, 1, 'Expected one unambiguous final JSON answer');
    answer = JSON.parse(fences[0][1]);
  }
  assert.deepEqual(answer, this.evidence.expected);
  this.evidence.answer = answer;
  this.evidence.task_id = task.id;
  this.evidence.context_id = task.contextId;
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  this.evidence.scenario_status = result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify(this.evidence), 'application/json');
});

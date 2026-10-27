import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(125_000);

async function request(world, method, path, payload) {
  const response = await fetch(new URL(path, world.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': world.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  world.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text });
  assert.equal(response.status, 200, `${method} ${path}: HTTP ${response.status}: ${text.slice(0, 1000)}`);
  return JSON.parse(text);
}

async function prompt(world, content) {
  const response = await request(world, 'POST', '/v1/chat/completions', {
    agent_id: world.agentId, model: world.model, session_id: world.session,
    stream: false, memory_enabled: false, max_completion_tokens: 4096,
    messages: [{ role: 'user', content }],
  });
  assert.equal(response.session_id, world.session);
  assert.equal(response.model, world.model);
  assert.equal(response.choices?.length, 1);
  assert.equal(response.choices[0].finish_reason, 'stop');
  assert.equal(response.choices[0].message.role, 'assistant');
  assert.match(response.id, /^chatcmpl-.+/);
  return response;
}

function answer(response) {
  const content = response.choices[0].message.content;
  assert.equal(typeof content, 'string');
  let final = content.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim();
  assert.ok(final && !final.includes('<think>'), 'A completed final answer is required');
  const fenced = final.match(/^```(?:json)?\s*\n([\s\S]*?)\n```$/);
  if (fenced) final = fenced[1];
  return JSON.parse(final);
}

Given('the saved math agent and a fresh conversation', async function () {
  this.started = performance.now();
  this.session = randomUUID();
  this.agentId = 'math-specialist-db64c2a8';
  this.model = process.env.UAR_MODEL;
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.evidence = { scenario: 'CONTINUITY-01', client_platform: process.platform,
    agent_id: this.agentId, session_id: this.session, model: this.model,
    authentication_supplied_by_test: false, client_replays_history: false,
    service_kept_running: true, exchanges: [] };
  this.record = resolve(`.prometheus/verification/conversations/${this.session}.json`);
  assert.ok(this.model && this.workspace, 'Set UAR_MODEL and UAR_WORKSPACE_ID');
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol));
  assert.ok(!this.base.username && !this.base.password, 'Use an endpoint without URL credentials');
  const agent = await request(this, 'GET', `/api/agents/${this.agentId}`);
  assert.equal(agent.id, this.agentId);
  this.marker = agent.prompt.system.match(/MATH-[a-f0-9-]+/)?.[0];
  assert.ok(this.marker, 'Saved mathematics identity is missing');
});

When('I ask it to add two fresh numbers', async function () {
  this.firstNumber = randomInt(1000, 5000);
  this.secondNumber = randomInt(5000, 10000);
  this.sum = this.firstNumber + this.secondNumber;
  this.evidence.expected_first = this.sum;
  this.first = await prompt(this, `The first number is ${this.firstNumber} and the second number is ${this.secondNumber}. Add them. Apply your saved specialist/answer JSON instructions.`);
});

Then('its first answer is correct and uses its saved identity', function () {
  assert.deepEqual(answer(this.first), { specialist: this.marker, answer: this.sum });
});

When('I ask a follow-up using only references to the previous exchange', async function () {
  this.multiplier = randomInt(3, 10);
  this.expected = this.sum * this.multiplier - this.secondNumber;
  this.evidence.expected_follow_up = this.expected;
  this.second = await prompt(this, `Multiply your previous answer by ${this.multiplier}, then subtract the second number I originally gave you. Apply your saved specialist/answer JSON instructions.`);
});

Then('it answers using the remembered numbers and previous result', function () {
  assert.notEqual(this.first.id, this.second.id, 'Follow-up must be a new completion');
  assert.deepEqual(answer(this.second), { specialist: this.marker, answer: this.expected });
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

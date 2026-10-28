import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(125_000);

async function request(world, method, path, payload) {
  const response = await fetch(new URL(path, world.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', 'anthropic-version': '2023-06-01',
      'x-uar-workspace-id': world.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  world.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text });
  assert.equal(response.status, 200, `${method} ${path}: HTTP ${response.status}: ${text.slice(0, 1000)}`);
  return JSON.parse(text);
}

Given('the saved math agent is available for an Anthropic-compatible request', async function () {
  this.started = performance.now();
  this.session = randomUUID();
  this.agentId = 'math-specialist-db64c2a8';
  this.model = process.env.UAR_MODEL;
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.record = resolve(`.prometheus/verification/anthropic/${this.session}.json`);
  this.evidence = { scenario: 'ANTHROPIC-01', client_platform: process.platform,
    agent_id: this.agentId, session_id: this.session, model: this.model,
    authentication_supplied_by_test: false, service_kept_running: true, exchanges: [] };
  assert.ok(this.model && this.workspace, 'Set UAR_MODEL and UAR_WORKSPACE_ID');
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol));
  assert.ok(!this.base.username && !this.base.password, 'Use an endpoint without URL credentials');
  const agent = await request(this, 'GET', `/api/agents/${this.agentId}`);
  assert.equal(agent.id, this.agentId);
  this.marker = agent.prompt.system.match(/MATH-[a-f0-9-]+/)?.[0];
  assert.ok(this.marker, 'Saved mathematics identity is missing');
});

When('I send it a fresh arithmetic question through the Messages API', async function () {
  const left = randomInt(21, 80);
  const right = randomInt(11, 40);
  const extra = randomInt(101, 500);
  this.evidence.expected = { specialist: this.marker, answer: left * right + extra };
  this.response = await request(this, 'POST', '/v1/messages', {
    agent_id: this.agentId, model: this.model, session_id: this.session,
    max_tokens: 4096, stream: false,
    messages: [{ role: 'user', content: `What is (${left} multiplied by ${right}) plus ${extra}? Follow your saved specialist/answer JSON instructions.` }],
  });
});

Then('the Anthropic response completes with the saved specialist identity and correct answer', function () {
  assert.equal(this.response.type, 'message');
  assert.equal(this.response.role, 'assistant');
  assert.equal(this.response.model, this.model);
  assert.equal(this.response.stop_reason, 'end_turn');
  assert.match(this.response.id, /^msg_.+/);
  assert.ok(Array.isArray(this.response.content));
  const blocks = this.response.content.filter(block => block.type === 'text');
  assert.ok(blocks.length > 0 && blocks.every(block => typeof block.text === 'string'));
  const content = blocks.map(block => block.text).join('');
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

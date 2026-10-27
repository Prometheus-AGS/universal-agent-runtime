import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(125_000);

async function request(world, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, world.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': world.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  // Preserve confirmed ownership even if reading the creation response fails.
  if (method === 'POST' && path === '/api/agents' && response.status === 201) world.created = true;
  const text = await response.text();
  world.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text });
  assert.equal(response.status, expected, `${method} ${path}: HTTP ${response.status}: ${text.slice(0, 1000)}`);
  return expected === 200 || expected === 201 ? JSON.parse(text) : text;
}

async function prompt(world) {
  const session = randomUUID();
  const response = await request(world, 'POST', '/v1/chat/completions', {
    agent_id: world.agentId, model: world.model, session_id: session,
    stream: false, memory_enabled: false, max_completion_tokens: 4096,
    messages: [{ role: 'user', content: `Apply your saved rule to the number ${world.number}.` }],
  });
  assert.equal(response.session_id, session);
  assert.equal(response.model, world.model);
  assert.equal(response.choices?.length, 1);
  assert.equal(response.choices[0].finish_reason, 'stop');
  assert.equal(response.choices[0].message.role, 'assistant');
  const content = response.choices[0].message.content;
  assert.equal(typeof content, 'string');
  let final = content.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim();
  assert.ok(final && !final.includes('<think>'), 'A completed final answer is required');
  let parsed;
  try { parsed = JSON.parse(final); } catch {
    const fences = [...final.matchAll(/(?:^|\n)```(?:json)?[ \t]*\r?\n([\s\S]*?)\r?\n```(?=\s|$)/g)];
    assert.equal(fences.length, 1, 'Expected one unambiguous final JSON answer');
    parsed = JSON.parse(fences[0][1]);
  }
  return { session, answer: parsed };
}

async function removeOwned(world) {
  assert.ok(world.created, 'Only this scenario-created agent may be deleted');
  world.deletionAttempted = true;
  await request(world, 'DELETE', world.agentPath, undefined, 204);
  const missing = await request(world, 'GET', world.agentPath, undefined, 404);
  assert.equal(missing, 'Persisted agent not found');
  world.evidence.deleted = true;
}

Given('a new disposable agent with an original instruction rule', async function () {
  this.started = performance.now();
  this.agentId = `disposable-lifecycle-${randomUUID()}`;
  this.agentPath = `/api/agents/${this.agentId}`;
  this.record = resolve(`.prometheus/verification/agent-lifecycle/${this.agentId}.json`);
  this.model = process.env.UAR_MODEL;
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.evidence = { scenario: 'AGENT-LIFECYCLE-01', client_platform: process.platform,
    agent_id: this.agentId, model: this.model, authentication_supplied_by_test: false,
    service_kept_running: true, deleted: false, exchanges: [] };
  assert.ok(this.model && this.workspace, 'Set UAR_MODEL and UAR_WORKSPACE_ID');
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol));
  assert.ok(!this.base.username && !this.base.password, 'Use an endpoint without URL credentials');
  this.number = randomInt(100, 1000);
  this.offset = randomInt(11, 30);
  this.factor = randomInt(3, 10);
  this.oldMarker = `ORIGINAL-${randomUUID()}`;
  this.newMarker = `UPDATED-${randomUUID()}`;
  this.originalInstructions = `Return only a JSON object with keys marker and answer. Set marker to "${this.oldMarker}". Your rule is to add ${this.offset} to the user's number and put that numeric result in answer.`;
  this.updatedInstructions = `Return only a JSON object with keys marker and answer. Set marker to "${this.newMarker}". Your rule is to multiply the user's number by ${this.factor} and put that numeric result in answer.`;
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  agent.id = this.agentId;
  agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: 'Disposable instruction-update agent',
    description: 'Temporary agent owned by one manual lifecycle scenario.', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: this.originalInstructions, instructions: [] };
  const split = this.model.indexOf('/');
  assert.ok(split > 0 && split < this.model.length - 1, 'UAR_MODEL must be provider/model');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.tools.allow = [];
  agent.memory.kb.enabled = false;
  const saved = await request(this, 'POST', '/api/agents', agent, 201);
  assert.equal(saved.id, this.agentId);
  assert.equal(saved.prompt.system, this.originalInstructions);
});

When('I prompt it using the original rule', async function () {
  this.before = await prompt(this);
});

Then('its answer follows its original instructions', function () {
  this.evidence.expected_original = { marker: this.oldMarker, answer: this.number + this.offset };
  assert.deepEqual(this.before.answer, this.evidence.expected_original);
});

When('I save different instructions on that agent', async function () {
  const updated = await request(this, 'PATCH', this.agentPath,
    { prompt: { system: this.updatedInstructions, instructions: [] } });
  assert.equal(updated.id, this.agentId);
  assert.deepEqual(updated.prompt, { system: this.updatedInstructions, instructions: [] });
  const saved = await request(this, 'GET', this.agentPath);
  assert.deepEqual(saved.prompt, updated.prompt);
});

When('I prompt it again in a fresh conversation', async function () {
  this.after = await prompt(this);
  assert.notEqual(this.after.session, this.before.session);
});

Then('its answer follows only the updated instructions', function () {
  this.evidence.expected_updated = { marker: this.newMarker, answer: this.number * this.factor };
  assert.deepEqual(this.after.answer, this.evidence.expected_updated);
  assert.notDeepEqual(this.after.answer, this.before.answer);
});

When('I delete the disposable agent', async function () {
  await removeOwned(this);
});

Then('its saved definition is no longer available', function () {
  assert.equal(this.evidence.deleted, true);
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  let cleanupError;
  if (this.created && !this.deletionAttempted) {
    try { await removeOwned(this); } catch (error) { cleanupError = error; }
  }
  this.evidence.scenario_status = cleanupError ? 'FAILED' : result.status;
  if (cleanupError) this.evidence.cleanup_error = cleanupError.message;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify(this.evidence), 'application/json');
  if (cleanupError) throw cleanupError;
});

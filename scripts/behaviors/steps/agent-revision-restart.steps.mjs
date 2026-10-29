import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(125_000);
const TERMINAL = new Set(['completed', 'error', 'failed', 'cancelled', 'canceled']);

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
  const allowed = Array.isArray(expected) ? expected : [expected];
  assert.ok(allowed.includes(response.status), `${method} ${path}: HTTP ${response.status}: ${text.slice(0, 1000)}`);
  return { status: response.status, text, json: () => JSON.parse(text) };
}

async function prompt(world) {
  const session = randomUUID();
  const response = (await request(world, 'POST', '/v1/chat/completions', {
    agent_id: world.agentId, model: world.model, session_id: session,
    stream: false, memory_enabled: false, max_completion_tokens: 4096,
    messages: [{ role: 'user', content: `Apply your saved rule to the number ${world.number}.` }],
  })).json();
  assert.equal(response.session_id, session);
  assert.equal(response.choices?.length, 1);
  assert.equal(response.choices[0].finish_reason, 'stop');
  const content = response.choices[0].message.content;
  assert.equal(typeof content, 'string');
  const final = content.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim();
  assert.ok(final && !final.includes('<think>'), 'A completed final answer is required');
  try { return JSON.parse(final); } catch {
    const fences = [...final.matchAll(/(?:^|\n)```(?:json)?[ \t]*\r?\n([\s\S]*?)\r?\n```(?=\s|$)/g)];
    assert.equal(fences.length, 1, 'Expected one unambiguous final JSON answer');
    return JSON.parse(fences[0][1]);
  }
}

async function restart(world, label) {
  const record = {};
  world.evidence.restarts[label] = record;
  await restartOwnedUar(record);
  world.evidence.restarts[label].endpoint_binding = await assertOwnedEndpoint(world.base);
  assert.equal(record.previous_process_exited, true);
  assert.equal(record.new_process_ready, true);
  assert.notEqual(record.previous_pid, record.new_pid);
}

Given('a disposable agent whose instructions were changed after creation', async function () {
  this.started = performance.now();
  this.agentId = `disposable-revision-${randomUUID()}`;
  this.agentPath = `/api/agents/${this.agentId}`;
  this.record = resolve(`.prometheus/verification/agent-revision-restart/${this.agentId}.json`);
  this.model = process.env.UAR_MODEL;
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.evidence = { scenario: 'E11-AGENT-REVISION-RESTART', client_platform: process.platform,
    agent_id: this.agentId, model: this.model, authentication_supplied_by_test: false,
    previous_scenarios_rerun: false, restarts: {}, deleted: false, exchanges: [] };
  assert.ok(this.model && this.workspace, 'Set UAR_MODEL and UAR_WORKSPACE_ID');
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol));
  assert.ok(!this.base.username && !this.base.password, 'Use an endpoint without URL credentials');
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.base);
  this.number = randomInt(100, 1000);
  this.factor = randomInt(3, 10);
  this.marker = `REVISED-${randomUUID()}`;
  const original = `Return only a JSON object with keys marker and answer. Set marker to "ORIGINAL-${randomUUID()}". Your rule is to add 17 to the user's number and put that numeric result in answer.`;
  this.updatedInstructions = `Return only a JSON object with keys marker and answer. Set marker to "${this.marker}". Your rule is to multiply the user's number by ${this.factor} and put that numeric result in answer.`;
  const agent = (await request(this, 'GET', '/api/agents/default-agent')).json();
  agent.id = this.agentId;
  agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: 'Disposable revision-restart agent',
    description: 'Temporary agent owned by one manual restart scenario.', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: original, instructions: [] };
  const split = this.model.indexOf('/');
  assert.ok(split > 0 && split < this.model.length - 1, 'UAR_MODEL must be provider/model');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.tools.allow = [];
  agent.memory.kb.enabled = false;
  const saved = (await request(this, 'POST', '/api/agents', agent, 201)).json();
  assert.equal(saved.prompt.system, original);
  const updated = (await request(this, 'PATCH', this.agentPath,
    { prompt: { system: this.updatedInstructions, instructions: [] } })).json();
  assert.deepEqual(updated.prompt, { system: this.updatedInstructions, instructions: [] });
  this.revisionBefore = (await request(this, 'GET', this.agentPath)).json();
  assert.deepEqual(this.revisionBefore.prompt, updated.prompt);
  this.evidence.revision_before_restart = this.revisionBefore;
});

When('UAR restarts with the existing agent store', { timeout: 360_000 }, async function () {
  await restart(this, 'after_revision');
});

Then("the agent's saved definition is the changed revision", async function () {
  const after = (await request(this, 'GET', this.agentPath)).json();
  this.evidence.revision_after_restart = after;
  assert.deepEqual(after.prompt, { system: this.updatedInstructions, instructions: [] });
  assert.equal(after.id, this.agentId);
});

Then('a fresh prompt follows only the changed instructions', async function () {
  const answer = await prompt(this);
  this.evidence.expected_answer = { marker: this.marker, answer: this.number * this.factor };
  this.evidence.answer_after_restart = answer;
  assert.deepEqual(answer, this.evidence.expected_answer);
});

When('I delete the disposable agent', async function () {
  this.deletionAttempted = true;
  await request(this, 'DELETE', this.agentPath, undefined, 204);
  assert.equal((await request(this, 'GET', this.agentPath, undefined, 404)).text, 'Persisted agent not found');
  this.evidence.deleted = true;
});

When('UAR restarts with the existing agent store again', { timeout: 360_000 }, async function () {
  await restart(this, 'after_delete');
});

Then('the deleted agent cannot be read or listed after cold startup', async function () {
  assert.equal((await request(this, 'GET', this.agentPath, undefined, 404)).text, 'Persisted agent not found');
  const listing = await request(this, 'GET', '/api/agents');
  this.evidence.listed_after_cold_start = listing.text.includes(this.agentId);
  assert.equal(this.evidence.listed_after_cold_start, false, 'The deleted agent must not be listed');
});

Then('selecting the deleted agent dispatches no model call', async function () {
  const admission = await request(this, 'POST', '/api/uar/runs', {
    agent_id: this.agentId, session_id: randomUUID(), input: `Apply your saved rule to ${this.number}.`,
  }, [200, 201, 202, 400, 403, 404, 409, 422]);
  this.evidence.selection_status = admission.status;
  if (admission.status >= 400) {
    this.evidence.selection_outcome = 'refused_at_admission';
  } else {
    const created = admission.json();
    const runId = created.run_id ?? created.runId;
    assert.ok(runId, 'An admitted run must report its id');
    let status;
    const deadline = Date.now() + 90_000;
    while (Date.now() < deadline) {
      status = (await request(this, 'GET', `/api/uar/runs/${runId}`)).json().status;
      if (TERMINAL.has(status)) break;
      await delay(500);
    }
    const calls = (await request(this, 'GET', `/api/uar/accounting/calls/${runId}`)).json();
    Object.assign(this.evidence, { selection_outcome: 'admitted_run', selection_run_id: runId,
      selection_run_status: status, selection_accounting_calls: calls });
    assert.notEqual(status, 'completed', 'A deleted agent must not complete a run');
    assert.deepEqual(calls ?? [], [], 'No model call may be dispatched for a deleted agent');
  }
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  let cleanupError;
  if (this.created && !this.deletionAttempted) {
    try {
      this.deletionAttempted = true;
      await request(this, 'DELETE', this.agentPath, undefined, 204);
    } catch (error) { cleanupError = error; }
  }
  this.evidence.scenario_status = cleanupError ? 'FAILED' : result.status;
  if (cleanupError) this.evidence.cleanup_error = cleanupError.message;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.scenario_status }), 'application/json');
  if (cleanupError) throw cleanupError;
});

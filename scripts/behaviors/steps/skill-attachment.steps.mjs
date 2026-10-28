import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(150_000);
const POLL_MS = 500;
const TERMINAL = new Set(['done', 'error', 'cancelled']);

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

Given('a disposable agent and an eligible skill whose triggers cannot match the request', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.short = this.id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/skill-attachment/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.owned = [];
  this.sessions = [];
  this.evidence = { scenario: 'C41-SKILL-ATTACHMENT-01', platform: process.platform, model: this.model,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model?.includes('/'));
  this.factor = randomInt(7, 20);
  this.fee = randomInt(31, 80);
  this.skillId = `vault-fee-${this.short}`;
  const saved = await request(this, 'POST', '/api/uar/skills', { name: this.skillId,
    description: 'Private vault storage fee rule.',
    triggers: { keywords: [`zz-unmatched-${this.short}`] },
    prompt_overlay: `Vault storage fee rule (private): fee = crates x ${this.factor} + ${this.fee}. `
      + 'When asked for a vault fee, reply with only the resulting number.' }, 201);
  assert.equal(saved.skill_id, this.skillId);
  this.owned.push(`/api/uar/skills/${this.skillId}`);
  this.agentId = `disposable-skill-attach-${this.short}`;
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  Object.assign(agent, { id: this.agentId, version: '1.0.0', extensions: {} });
  agent.metadata = { ...agent.metadata, title: 'Disposable attachment assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Answer briefly. Follow any activated skill instructions exactly.', instructions: [] };
  const split = this.model.indexOf('/');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.policy.tools.allow = [];
  agent.memory.kb.enabled = false;
  await request(this, 'POST', '/api/agents', agent, 201);
  this.owned.push(`/api/agents/${this.agentId}`);
  Object.assign(this.evidence, { skill_id: this.skillId, agent_id: this.agentId });
});

When('I send a fresh request that attaches the skill', async function () {
  this.crates = randomInt(3, 30);
  this.expected = this.crates * this.factor + this.fee;
  const input = `What is the vault fee for ${this.crates} crates? Reply with only the number.`;
  assert.ok(!input.includes(String(this.factor)) && !input.includes(this.skillId));
  const session = randomUUID();
  this.sessions.push(session);
  const created = await request(this, 'POST', '/api/uar/runs',
    { agent_id: this.agentId, input, session_id: session, skill_attachments: [this.skillId] }, [200, 201, 202]);
  this.evidence.create_response = created;
  this.runId = created.run_id ?? created.runId;
  assert.ok(runIdOk(this.runId));
  const deadline = Date.now() + 120_000;
  let status;
  while (Date.now() < deadline) {
    status = (await request(this, 'GET', `/api/uar/runs/${this.runId}`)).status;
    if (TERMINAL.has(status)) break;
    await pause(POLL_MS);
  }
  this.run = await request(this, 'GET', `/api/uar/runs/${this.runId}`);
  this.events = (await request(this, 'GET', `/api/uar/runs/${this.runId}/events`)).events;
  Object.assign(this.evidence, { run_id: this.runId, status, expected: this.expected,
    activated_skills: this.run.activated_skills, skill_matching: this.run.skill_matching,
    skill_events: this.events.filter(e => /skill/i.test(e.eventName) || /skill/i.test(e.data?.name ?? '')).map(e => ({ name: e.eventName, data: e.data })) });
  assert.equal(status, 'done');
});

function runIdOk(id) { return typeof id === 'string' && id.length > 0; }

Then('the skill is activated as an attachment rather than by matching', async function () {
  assert.deepEqual(this.evidence.create_response.activation_failures ?? [], [], 'The attachment was not rejected');
  assert.ok((this.run.activated_skills ?? []).includes(this.skillId), `Activated: ${JSON.stringify(this.run.activated_skills)}`);
  const accepted = (this.run.skill_matching?.candidates ?? []).filter(c => c.accepted).map(c => c.skill_id);
  assert.ok(!accepted.includes(this.skillId), 'The matcher did not select the skill on its own');
});

Then('the answer follows the skill\'s private calculation rule', async function () {
  const answer = this.events.filter(e => e.eventName === 'agui.message.delta').map(e => e.data?.delta?.text ?? '').join('')
    .replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
  this.evidence.answer = answer.slice(0, 1000);
  assert.match(answer, new RegExp(`\\b${this.expected}\\b`), `Expected ${this.expected}, got: ${answer.slice(0, 200)}`);
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 60_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  for (const session of this.sessions) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${session}: ${error}`); }
  }
  for (const path of [...this.owned].reverse()) {
    try { await request(this, 'DELETE', path, undefined, [200, 204, 404]); }
    catch (error) { errors.push(`${path}: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});

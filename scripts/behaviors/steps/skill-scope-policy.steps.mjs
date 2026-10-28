import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(180_000);
const POLL_MS = 500;
const TERMINAL = new Set(['done', 'error', 'cancelled']);

async function request(w, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 2500) });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

const pause = ms => new Promise(done => setTimeout(done, ms));

async function configureConversation(w, session, skills) {
  if (!w.sessions.includes(session)) w.sessions.push(session);
  await request(w, 'POST', `/api/uar/sessions/${session}/agent-config`, { agent_id: w.agentId, skills });
}

// One run in a conversation, attaching the given skills; attachments are
// intersected with effective eligibility, so a rejected one reports a failure.
async function ask(w, label, session, attachments) {
  const crates = randomInt(3, 15);
  const created = await request(w, 'POST', '/api/uar/runs', { agent_id: w.agentId, session_id: session,
    input: `What is the crate tariff for ${crates} crates? Reply with only the number, or UNKNOWN if no tariff rule is available.`,
    skill_attachments: attachments }, [200, 201, 202]);
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
  const answer = events.filter(e => e.eventName === 'agui.message.delta').map(e => e.data?.delta?.text ?? '').join('')
    .replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
  const record = { run_id: runId, session, status, crates, attachments,
    activation_failures: created.activation_failures ?? [],
    effective_skills: run.effective_run_policy?.skills ?? null, activated_skills: run.activated_skills ?? [],
    activation_events: events.filter(e => e.eventName === 'agui.skill.activated').map(e => e.data?.skill?.id),
    answer: answer.slice(0, 300) };
  w.evidence.runs[label] = record;
  assert.equal(status, 'done');
  return record;
}

const onlyNumber = (answer, value) => new RegExp(`\\b${value}\\b`).test(answer);

Given('skills A, B and C with private rules and an agent whose parent selection is A and B with B denied', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.short = this.id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/skill-scope-policy/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.sessions = [];
  this.owned = [];
  this.evidence = { scenario: 'C42-SKILL-SCOPE-POLICY-01', platform: process.platform, model: this.model,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, runs: {}, exchanges: [] };
  assert.ok(this.workspace && this.model?.includes('/'));
  this.factors = { A: randomInt(11, 30), B: randomInt(31, 50), C: randomInt(51, 70) };
  this.ids = Object.fromEntries(Object.keys(this.factors).map(letter => [letter, `c42-skill-${letter.toLowerCase()}-${this.short}`]));
  for (const [letter, factor] of Object.entries(this.factors)) {
    const saved = await request(this, 'POST', '/api/uar/skills', { name: this.ids[letter], description: `Private crate tariff rule ${letter}.`,
      triggers: { keywords: [`zz-unmatched-${this.short}-${letter}`] },
      prompt_overlay: `Crate tariff rule (private): tariff = crates x ${factor}. When asked for a crate tariff, reply with only the resulting number.` }, 201);
    assert.equal(saved.skill_id, this.ids[letter]);
    this.owned.push(`/api/uar/skills/${this.ids[letter]}`);
  }
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `disposable-skill-scope-${this.short}`;
  Object.assign(agent, { id: this.agentId, version: '1.0.0',
    extensions: { 'uar.run_policy': { skills: { mode: 'selected', ids: [this.ids.A, this.ids.B], denied_ids: [this.ids.B] } } } });
  agent.metadata = { ...agent.metadata, title: 'Disposable scope assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Answer briefly. Follow an activated skill instruction exactly. If no tariff rule is available, reply UNKNOWN.', instructions: [] };
  const split = this.model.indexOf('/');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.policy.tools.allow = [];
  agent.memory.kb.enabled = false;
  await request(this, 'POST', '/api/agents', agent, 201);
  this.owned.push(`/api/agents/${this.agentId}`);
  Object.assign(this.evidence, { skill_ids: this.ids, factors: this.factors, agent_id: this.agentId });
});

When('a conversation selects A and C and the run attaches A, B and C', async function () {
  this.first = randomUUID();
  await configureConversation(this, this.first, [this.ids.A, this.ids.C]);
  this.narrowed = await ask(this, 'selected_a_c', this.first, [this.ids.A, this.ids.B, this.ids.C]);
});

Then('only A is eligible and active, and the answer follows A\'s rule', function () {
  const run = this.narrowed;
  assert.deepEqual(run.activated_skills, [this.ids.A], `Activated: ${JSON.stringify(run.activated_skills)}`);
  assert.ok(!run.activation_events.includes(this.ids.B) && !run.activation_events.includes(this.ids.C));
  const rejected = run.activation_failures.map(failure => failure.skill_id ?? failure.id ?? failure);
  for (const letter of ['B', 'C']) assert.ok(JSON.stringify(rejected).includes(this.ids[letter]), `${letter} attachment was rejected`);
  assert.ok(onlyNumber(run.answer, run.crates * this.factors.A), `Expected A's rule: ${run.answer}`);
});

When('a conversation selects no skills', async function () {
  // A fresh conversation: an earlier answer in the same history would let the
  // model infer the private rule without any skill access.
  this.empty = randomUUID();
  await configureConversation(this, this.empty, []);
  this.none = await ask(this, 'none', this.empty, [this.ids.A]);
});

Then('no skill is eligible or active', function () {
  assert.deepEqual(this.none.activated_skills, []);
  assert.deepEqual(this.none.activation_events, []);
  assert.ok(JSON.stringify(this.none.activation_failures).includes(this.ids.A), 'The A attachment was rejected');
  assert.ok(!onlyNumber(this.none.answer, this.none.crates * this.factors.A), 'No private rule was used');
});

When('A is disabled for a second conversation that selects A', async function () {
  this.disabled = randomUUID();
  await configureConversation(this, this.disabled, [this.ids.A]);
  this.evidence.scoped_disable = await request(this, 'POST', `/api/uar/skills/${this.ids.A}/toggle`,
    { enabled: false, scope: { kind: 'conversation', id: this.disabled } });
  this.blocked = await ask(this, 'disabled_context', this.disabled, [this.ids.A]);
});

Then('that conversation never loads A', function () {
  assert.deepEqual(this.blocked.activated_skills, []);
  assert.ok(!this.blocked.activation_events.includes(this.ids.A));
  assert.ok(!onlyNumber(this.blocked.answer, this.blocked.crates * this.factors.A), 'A\'s rule was not used');
});

Then('a third conversation that selects A uses A\'s private rule', async function () {
  this.enabled = randomUUID();
  await configureConversation(this, this.enabled, [this.ids.A]);
  const run = await ask(this, 'enabled_context', this.enabled, [this.ids.A]);
  assert.deepEqual(run.activated_skills, [this.ids.A]);
  assert.ok(onlyNumber(run.answer, run.crates * this.factors.A), `Expected A's rule: ${run.answer}`);
});

Then('after an owned restart the second conversation\'s disable of A is still saved', { timeout: 420_000 }, async function () {
  this.evidence.restart = {};
  await restartOwnedUar(this.evidence.restart);
  // The skill view does not expose scoped configuration; read the saved
  // disable through behavior: the disabled conversation still cannot load A.
  const after = await ask(this, 'disabled_after_restart', this.disabled, [this.ids.A]);
  assert.deepEqual(after.activated_skills, []);
  assert.ok(after.activation_failures.some(f => f.skill_id === this.ids.A && f.code === 'disabled'),
    `A must remain disabled for this conversation: ${JSON.stringify(after.activation_failures)}`);
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 90_000 }, async function ({ result }) {
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

import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson } from '../support/answer-oracle.mjs';

setDefaultTimeout(180_000);

async function request(w, method, path, payload, expected = [200]) {
  const res = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(150_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && res.status === 201) {
    if (path === '/api/agents') w.owned.push(`/api/agents/${payload.id}`);
    if (path === '/api/uar/skills') w.owned.push(`/api/uar/skills/${payload.name}`);
  }
  const text = await res.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: res.status, response: text.slice(0, 2000) });
  assert.ok(expected.includes(res.status), `${method} ${path}: ${res.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}

Given('an installed private crate-quote skill and an unrelated installed skill', async function () {
  this.started = performance.now();
  this.owned = [];
  const id = randomUUID();
  this.record = resolve(`.prometheus/verification/skill-all-policy/${id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model);
  this.evidence = { scenario: 'E26-ALL-SKILLS-POLICY', platform: process.platform, model: this.model,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, exchanges: [] };
  this.marker = `CRATE-${randomUUID()}`;
  this.factor = randomInt(7, 19);
  this.fee = randomInt(31, 90);
  this.count = randomInt(3, 30);
  this.skillId = `orchard-crate-quote-${id}`;
  await request(this, 'POST', '/api/uar/skills', { name: this.skillId,
    description: 'Quote the price of an orchard crate shipment with the private company formula.',
    triggers: { keywords: ['orchard crate quote'] },
    prompt_overlay: `Orchard crate quote formula: multiply the crate count by ${this.factor} and add ${this.fee}. Return only JSON {"marker":"${this.marker}","quote":<number>}.` }, [201]);
  await request(this, 'POST', '/api/uar/skills', { name: `greenhouse-watering-${id}`,
    description: 'Plan a watering schedule for greenhouse orchids.',
    triggers: { keywords: ['greenhouse watering schedule'] },
    prompt_overlay: 'For any request, return only JSON {"marker":"WATERING","litres":12}.' }, [201]);
  this.agentId = `disposable-all-skills-${id}`;
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  Object.assign(agent, { id: this.agentId, version: '1.0.0' });
  agent.metadata = { ...agent.metadata, title: 'Disposable all-skills assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Follow any loaded skill instructions exactly and return only the JSON they request. Do not call tools.', instructions: [] };
  const split = this.model.indexOf('/');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  // An empty allow list means Auto (every eligible tool); one unmatched placeholder selects none.
  agent.policy.tools.allow = [`e26-no-tools-${id.slice(0, 8)}`];
  agent.memory.kb.enabled = false;
  await request(this, 'POST', '/api/agents', agent, [201]);
});

When('I ask for an orchard crate quote in a turn whose skill policy is all', async function () {
  this.session = randomUUID();
  const payload = { agent_id: this.agentId, model: this.model, session_id: this.session, stream: true, stream_mode: 'agui_spec',
    memory_enabled: false, max_completion_tokens: 4096, run_policy: { version: 1, skills: { mode: 'all' } },
    messages: [{ role: 'user', content: `Can I get an orchard crate quote for ${this.count} crates?` }] };
  const res = await fetch(new URL('/v1/chat/completions', this.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(170_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': this.workspace },
    body: JSON.stringify(payload),
  });
  const raw = await res.text();
  assert.equal(res.status, 200, raw.slice(0, 600));
  const events = raw.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const name = lines.find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    return !data || data === '[DONE]' || name?.startsWith('runtime.') ? [] : [JSON.parse(data)];
  });
  assert.ok(!events.some(e => ['RUN_ERROR', 'RUN_CANCELLED'].includes(e.type)), 'The turn must succeed');
  const runId = events.find(e => e.type === 'RUN_STARTED')?.runId;
  this.run = await request(this, 'GET', `/api/uar/runs/${encodeURIComponent(runId)}`);
  this.text = events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta).join('');
  Object.assign(this.evidence, { run_id: runId, effective_skills_mode: this.run.effective_run_policy?.skills?.mode,
    eligible_skill_count: this.run.effective_run_policy?.skills?.ids?.length, activated_skills: this.run.activated_skills,
    accepted: (this.run.skill_matching?.candidates ?? []).filter(c => c.accepted).map(c => c.skill_id), answer_text: this.text });
});

Then("the effective skill policy is all and the answer applies the crate-quote skill's private formula", function () {
  assert.equal(this.run.effective_run_policy?.skills?.mode, 'all');
  const answer = parseAnswerJson(this.text);
  assert.equal(answer.marker, this.marker);
  assert.equal(Number(answer.quote), this.count * this.factor + this.fee);
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.session) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${this.session}`, undefined, [204, 404]); } catch (error) { errors.push(String(error)); }
  }
  for (const path of [...this.owned].reverse()) {
    try { await request(this, 'DELETE', path, undefined, [204]); } catch (error) { errors.push(String(error)); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx' });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});

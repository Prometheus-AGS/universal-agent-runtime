import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(125_000);

async function request(w, method, path, payload, expected = 200) {
  const res = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(15_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  // Record acknowledged ownership before parsing so later failures still clean up.
  if (method === 'POST' && res.status === 201) {
    if (path === '/api/agents') w.owned.push(`/api/agents/${payload.id}`);
    if (path === '/api/uar/skills') w.owned.push(`/api/uar/skills/${payload.name.toLowerCase().replaceAll(' ', '-')}`);
  }
  const text = await res.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: res.status, response: text });
  assert.equal(res.status, expected, `${method} ${path}: ${res.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}

Given('a disposable agent and distinct parcel and gardening skills', async function () {
  this.started = performance.now();
  this.owned = [];
  const id = randomUUID();
  this.record = resolve(`.prometheus/verification/skill-detection/${id}.json`);
  this.evidence = { scenario: 'SKILL-DETECTION-01', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, exchanges: [] };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model);
  this.evidence.matching_config = await request(this, 'GET', '/api/uar/skills/config');
  assert.equal(this.evidence.matching_config.engine, 'service');
  assert.equal(this.evidence.matching_config.algorithm, 'keyword');
  this.marker = `PARCEL-${randomUUID()}`;
  this.factor = randomInt(7, 20);
  this.fee = randomInt(31, 80);
  this.skillId = `parcel-pricing-${id}`;
  this.decoyId = `garden-watering-${id}`;
  for (const skill of [
    { name: this.skillId, description: 'Calculate an opal parcel shipping quote.',
      triggers: { keywords: ['opal parcel'] },
      prompt_overlay: `For opal parcel quotes, multiply the parcel count by ${this.factor} and add ${this.fee}. Return only JSON with marker "${this.marker}" and numeric quote. This is the private pricing recipe; do not substitute general shipping rates.` },
    { name: this.decoyId, description: 'Schedule garden watering.',
      triggers: { keywords: ['garden watering'] },
      prompt_overlay: 'For garden watering requests, return only JSON {"marker":"GARDEN-ONLY","litres":12}.' },
  ]) {
    const saved = await request(this, 'POST', '/api/uar/skills', skill, 201);
    assert.equal(saved.skill_id, skill.name);
  }
  this.agentId = `disposable-skill-detection-${id}`;
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'Disposable service assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Help with the requested task using relevant available skills. Follow the selected skill instructions and return its requested answer format.', instructions: [] };
  const split = this.model.indexOf('/');
  assert.ok(split > 0);
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.policy.tools.allow = [];
  agent.memory.kb.enabled = false;
  await request(this, 'POST', '/api/agents', agent, 201);
  this.evidence.agent_id = this.agentId;
  this.evidence.skill_id = this.skillId;
  this.evidence.decoy_id = this.decoyId;
  this.evidence.model = this.model;
});

When('I ask for a parcel quote without naming a skill', async function () {
  this.count = randomInt(3, 30);
  this.evidence.expected = { marker: this.marker, quote: this.count * this.factor + this.fee };
  const prompt = `Please give me an opal parcel quote for ${this.count} parcels.`;
  const payload = { agent_id: this.agentId, model: this.model, session_id: randomUUID(),
    stream: true, stream_mode: 'agui_spec', memory_enabled: false, max_completion_tokens: 64000,
    messages: [{ role: 'user', content: prompt }] };
  assert.ok(!JSON.stringify(payload).includes(this.marker));
  assert.ok(!prompt.includes(this.skillId) && !prompt.includes(this.decoyId));
  const exchange = { method: 'POST', path: '/v1/chat/completions', request: payload, response: '' };
  this.evidence.exchanges.push(exchange);
  const res = await fetch(new URL(exchange.path, this.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': this.workspace },
    body: JSON.stringify(payload),
  });
  exchange.status = res.status;
  exchange.response = await res.text();
  assert.equal(res.status, 200);
  assert.match(res.headers.get('content-type') ?? '', /^text\/event-stream\b/);
  this.events = exchange.response.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const name = lines.find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
    return [JSON.parse(data)];
  });
  this.evidence.events = this.events;
  assert.ok(!this.events.some(e => ['RUN_ERROR', 'RUN_CANCELLED'].includes(e.type)), 'Run must succeed');
  const starts = this.events.filter(e => e.type === 'RUN_STARTED');
  const ends = this.events.filter(e => e.type === 'RUN_FINISHED');
  assert.equal(starts.length, 1);
  assert.equal(ends.length, 1);
  assert.equal(starts[0].runId, ends[0].runId);
  this.evidence.run_id = starts[0].runId;
});

Then('the runtime selects and activates the parcel skill instead of gardening', async function () {
  const run = await request(this, 'GET', `/api/uar/runs/${this.evidence.run_id}`);
  this.evidence.run_inspection = run;
  assert.equal(run.agent_id, this.agentId);
  assert.equal(run.skill_matching.effective_method, 'skill_service.keyword');
  const selected = run.skill_matching.candidates.filter(c => c.accepted).map(c => c.skill_id);
  assert.deepEqual(selected, [this.skillId]);
  assert.deepEqual(run.activated_skills, [this.skillId]);
  const activations = this.events.filter(e => e.type === 'CUSTOM' && e.name === 'uar.skill.activated');
  assert.ok(activations.some(e => e.value?.skill?.id === this.skillId));
  assert.ok(!activations.some(e => e.value?.skill?.id === this.decoyId));
});

Then('the answer uses the recipe and marker stored only in that skill', function () {
  const raw = this.events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta).join('');
  const final = raw.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim();
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
  const errors = [];
  for (const path of [...this.owned].reverse()) {
    try { await request(this, 'DELETE', path, undefined, 204); }
    catch (error) { errors.push(`${path}: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.disposable_resources_deleted = errors.length === 0;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx' });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Disposable cleanup must succeed');
});

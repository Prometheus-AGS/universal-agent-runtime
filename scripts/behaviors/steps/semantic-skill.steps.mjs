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

Given('a disposable agent with delivery and gardening skills under semantic matching', async function () {
  this.started = performance.now();
  this.owned = [];
  const id = randomUUID();
  this.record = resolve(`.prometheus/verification/semantic-skill/${id}.json`);
  this.evidence = { scenario: 'SEMANTIC-SKILL-01', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, exchanges: [] };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model);
  this.evidence.matching_config = await request(this, 'GET', '/api/uar/skills/config');
  this.originalConfig = this.evidence.matching_config;
  this.semanticConfig = { ...this.originalConfig, engine: 'service', algorithm: 'local_embedding', threshold: 0.65, model_name: null };
  this.marker = `PARCEL-${randomUUID()}`;
  this.factor = randomInt(7, 20);
  this.fee = randomInt(31, 80);
  this.skillId = `delivery-pricing-${id}`;
  this.decoyId = `garden-watering-${id}`;
  this.skills = [
    { name: this.skillId, description: 'Determine the courier delivery charge for sending packages based on the number of boxes. Apply the company shipping cost formula.',
      triggers: { keywords: ['freight tariff', 'consignment valuation'], semantic: 'Calculate the total price to dispatch packages to a recipient.' },
      prompt_overlay: `For delivery charges, multiply the box count by ${this.factor} and add ${this.fee}. Return only JSON with marker "${this.marker}" and numeric quote. This is the private pricing recipe; do not substitute general shipping rates.` },
    { name: this.decoyId, description: 'Schedule garden watering.',
      triggers: { keywords: ['garden watering'] },
      prompt_overlay: 'For garden watering requests, return only JSON {"marker":"GARDEN-ONLY","litres":12}.' },
  ];
  for (const skill of this.skills) {
    const saved = await request(this, 'POST', '/api/uar/skills', skill, 201);
    assert.equal(saved.skill_id, skill.name);
  }
  this.agentId = `disposable-semantic-skill-${id}`;
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'Disposable service assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Help with the requested task using relevant available skills. Follow the selected skill instructions and return its requested answer format.', instructions: [] };
  const split = this.model.indexOf('/');
  assert.ok(split > 0);
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.extensions = { ...agent.extensions, 'uar.run_policy': { version: 1,
    skills: { mode: 'selected', ids: [this.skillId, this.decoyId], denied_ids: [] } } };
  agent.policy.tools.allow = [];
  agent.memory.kb.enabled = false;
  await request(this, 'POST', '/api/agents', agent, 201);
  this.evidence.agent_id = this.agentId;
  this.evidence.skill_id = this.skillId;
  this.evidence.decoy_id = this.decoyId;
  this.evidence.model = this.model;
  // Preserve the original configuration before any global mutation.
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(`${this.record}.config-before.json`, JSON.stringify(this.originalConfig, null, 2) + '\n', { flag: 'wx' });
  this.configAttempted = true;
  await request(this, 'PUT', '/api/uar/skills/config', this.semanticConfig);
  assert.deepEqual(await request(this, 'GET', '/api/uar/skills/config'), this.semanticConfig);
});

When('I describe a delivery task without any exact skill keywords', async function () {
  this.count = randomInt(3, 30);
  this.evidence.expected = { marker: this.marker, quote: this.count * this.factor + this.fee };
  const prompt = `How much should I charge to send ${this.count} boxes to a customer? Please work out the delivery fee.`;
  this.evidence.keyword_controls = this.skills.map(skill => {
    const query = prompt.toLowerCase();
    const matches = skill.triggers.keywords.filter(keyword => query.includes(keyword.toLowerCase()));
    assert.deepEqual(matches, [], 'No exact keyword trigger may match');
    assert.ok(!skill.name.toLowerCase().includes(query));
    assert.ok(!skill.description.toLowerCase().includes(query));
    return { skill_id: skill.name, keywords: skill.triggers.keywords, matched_keywords: matches, derived_keyword_score: 0 };
  });
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

Then('semantic matching selects and activates the delivery skill instead of gardening', async function () {
  const run = await request(this, 'GET', `/api/uar/runs/${this.evidence.run_id}`);
  this.evidence.run_inspection = run;
  assert.equal(run.agent_id, this.agentId);
  assert.equal(run.skill_matching.effective_method, 'skill_service.local_embedding');
  assert.deepEqual([...run.effective_run_policy.skills.ids].sort(), [this.skillId, this.decoyId].sort());
  const selected = run.skill_matching.candidates.filter(c => c.accepted).map(c => c.skill_id);
  assert.deepEqual(selected, [this.skillId]);
  assert.deepEqual(run.activated_skills, [this.skillId]);
  const activations = this.events.filter(e => e.type === 'CUSTOM' && e.name === 'uar.skill.activated');
  assert.ok(activations.some(e => e.value?.skill?.id === this.skillId));
  assert.ok(!activations.some(e => e.value?.skill?.id === this.decoyId));
});

Then('the answer follows the privately stored delivery recipe', function () {
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
  if (this.configAttempted) {
    try {
      const current = await request(this, 'GET', '/api/uar/skills/config');
      if (JSON.stringify(current) !== JSON.stringify(this.originalConfig)) {
        assert.deepEqual(current, this.semanticConfig, 'Do not overwrite a concurrent configuration change');
        await request(this, 'PUT', '/api/uar/skills/config', this.originalConfig);
      }
      assert.deepEqual(await request(this, 'GET', '/api/uar/skills/config'), this.originalConfig);
      this.evidence.matching_config_restored = true;
    } catch (error) { errors.push(`Matching config restoration: ${error}`); }
  }
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

import assert from 'node:assert/strict';
import { createHash, randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson } from '../support/answer-oracle.mjs';

setDefaultTimeout(180_000);

async function request(w, method, path, payload, expected = 200) {
  const res = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(150_000),
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

// Streaming avoids the short non-streaming response deadline, which a slow
// reasoning turn can exceed even though the run itself completes.
async function ask(w, label) {
  const payload = { agent_id: w.agentId, model: w.model, session_id: randomUUID(),
    stream: true, stream_mode: 'agui_spec', memory_enabled: false, max_completion_tokens: 4096,
    messages: [{ role: 'user', content: w.prompt }] };
  for (const marker of [w.targetMarker, w.decoyMarker]) assert.ok(!JSON.stringify(payload).includes(marker));
  const exchange = { method: 'POST', path: '/v1/chat/completions', request: payload, response: '' };
  w.evidence.exchanges.push(exchange);
  const res = await fetch(new URL(exchange.path, w.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(170_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify(payload),
  });
  exchange.status = res.status;
  exchange.response = await res.text();
  assert.equal(res.status, 200, exchange.response.slice(0, 600));
  const events = exchange.response.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const name = lines.find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
    return [JSON.parse(data)];
  });
  assert.ok(!events.some(e => ['RUN_ERROR', 'RUN_CANCELLED'].includes(e.type)), 'Run must succeed');
  const starts = events.filter(e => e.type === 'RUN_STARTED');
  assert.equal(starts.length, 1);
  assert.equal(events.filter(e => e.type === 'RUN_FINISHED').length, 1);
  const runId = starts[0].runId;
  const run = await request(w, 'GET', `/api/uar/runs/${encodeURIComponent(runId)}`);
  assert.equal(run.agent_id, w.agentId);
  const answer = parseAnswerJson(events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta).join(''));
  w.evidence[label] = { prompt_sha256: createHash('sha256').update(w.prompt).digest('hex'), run_id: runId,
    config: await request(w, 'GET', '/api/uar/skills/config'), skill_matching: run.skill_matching,
    activated_skills: run.activated_skills, answer };
  return { run, answer };
}

// Shape-agnostic: the winner's marker and its computed value must appear somewhere
// in the JSON answer (as a key or a value); the other skill's marker must not.
function consumesRule(answer, marker, value, otherMarker) {
  const text = JSON.stringify(answer);
  const numbers = [];
  JSON.stringify(answer, (_key, item) => { if (typeof item === 'number') numbers.push(item); return item; });
  return text.includes(marker) && numbers.includes(value) && !text.includes(otherMarker);
}

function winner(run) {
  return (run.skill_matching?.candidates ?? []).filter(c => c.accepted).map(c => c.skill_id);
}

Given('a disposable agent restricted to a lexical decoy skill and a semantic target skill', async function () {
  this.started = performance.now();
  this.owned = [];
  const id = randomUUID();
  const short = id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/skill-lexical-semantic/${id}.json`);
  this.evidence = { scenario: 'E15-SKILL-LEXICAL-SEMANTIC', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, exchanges: [] };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model);
  this.originalConfig = await request(this, 'GET', '/api/uar/skills/config');
  this.evidence.original_config = this.originalConfig;
  assert.equal(this.originalConfig.engine, 'service');
  assert.equal(this.originalConfig.algorithm, 'keyword');
  this.semanticConfig = { ...this.originalConfig, algorithm: 'local_embedding', threshold: 0.65, model_name: null };
  this.targetMarker = `TARGET-${randomUUID()}`;
  this.decoyMarker = `DECOY-${randomUUID()}`;
  this.factor = randomInt(7, 20);
  this.fee = randomInt(31, 80);
  this.litres = randomInt(13, 99);
  this.count = randomInt(3, 30);
  this.lexicalToken = `orchid-${short}`;
  this.prompt = `How much should I charge to send ${this.count} boxes to a customer? Please work out the delivery fee. Order reference ${this.lexicalToken}.`;
  this.targetId = `delivery-pricing-${id}`;
  this.decoyId = `orchid-repotting-${id}`;
  this.skills = [
    { name: this.targetId, description: 'Determine the courier delivery charge for sending packages based on the number of boxes. Apply the company shipping cost formula.',
      triggers: { keywords: ['freight tariff', 'consignment valuation'], semantic: 'Calculate the total price to dispatch packages to a recipient.' },
      prompt_overlay: `For delivery charges, multiply the box count by ${this.factor} and add ${this.fee}. Return only JSON with marker "${this.targetMarker}" and numeric quote. This is the private pricing recipe.` },
    { name: this.decoyId, description: 'Schedule repotting for tropical orchids in a greenhouse.',
      triggers: { keywords: [this.lexicalToken] },
      prompt_overlay: `For any request, return only JSON with marker "${this.decoyMarker}" and numeric litres ${this.litres}. This is the private greenhouse recipe.` },
  ];
  // The keyword control is computed from the same rules the matcher uses.
  const query = this.prompt.toLowerCase();
  this.evidence.keyword_controls = this.skills.map(skill => ({ skill_id: skill.name,
    matched_keywords: skill.triggers.keywords.filter(keyword => query.includes(keyword.toLowerCase())) }));
  assert.deepEqual(this.evidence.keyword_controls[0].matched_keywords, [], 'The target must have no lexical match');
  assert.deepEqual(this.evidence.keyword_controls[1].matched_keywords, [this.lexicalToken], 'The decoy must match lexically');
  for (const skill of this.skills) {
    const saved = await request(this, 'POST', '/api/uar/skills', skill, 201);
    assert.equal(saved.skill_id, skill.name);
  }
  this.agentId = `disposable-lexical-semantic-${id}`;
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'Disposable lexical/semantic assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'The runtime has already loaded the skill for this request. Do not call activate_skill or any other tool. Follow the loaded skill instructions exactly, even when they seem unrelated to the question, and return only its requested JSON.', instructions: [] };
  const split = this.model.indexOf('/');
  assert.ok(split > 0);
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.extensions = { ...agent.extensions, 'uar.run_policy': { version: 1,
    skills: { mode: 'selected', ids: [this.targetId, this.decoyId], denied_ids: [] } } };
  // An empty allow list means Auto (every eligible tool); one unmatched placeholder selects none.
  agent.policy.tools.allow = [`e15-no-tools-${short}`];
  agent.memory.kb.enabled = false;
  await request(this, 'POST', '/api/agents', agent, 201);
  Object.assign(this.evidence, { agent_id: this.agentId, target_id: this.targetId, decoy_id: this.decoyId, model: this.model });
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(`${this.record}.config-before.json`, JSON.stringify(this.originalConfig, null, 2) + '\n', { flag: 'wx' });
});

When('I send the discriminating prompt under keyword matching', async function () {
  this.lexical = await ask(this, 'keyword_run');
});

Then("keyword matching selects the lexical decoy and the answer uses only the decoy's private rule", function () {
  assert.equal(this.lexical.run.skill_matching.effective_method, 'skill_service.keyword');
  assert.deepEqual(winner(this.lexical.run), [this.decoyId]);
  assert.deepEqual(this.lexical.run.activated_skills, [this.decoyId]);
  // The winner's private values must be present; the other skill's marker must be absent.
  assert.ok(consumesRule(this.lexical.answer, this.decoyMarker, this.litres, this.targetMarker), 'The answer must carry only the decoy rule');
});

When('I send the identical prompt under local-embedding matching', async function () {
  this.configAttempted = true;
  await request(this, 'PUT', '/api/uar/skills/config', this.semanticConfig);
  assert.deepEqual(await request(this, 'GET', '/api/uar/skills/config'), this.semanticConfig);
  this.semantic = await ask(this, 'semantic_run');
});

Then("semantic matching selects the target and the answer uses only the target's private rule", function () {
  assert.equal(this.semantic.run.skill_matching.effective_method, 'skill_service.local_embedding');
  assert.deepEqual(winner(this.semantic.run), [this.targetId]);
  assert.deepEqual(this.semantic.run.activated_skills, [this.targetId]);
  assert.ok(consumesRule(this.semantic.answer, this.targetMarker, this.count * this.factor + this.fee, this.decoyMarker), 'The answer must carry only the target rule');
  assert.equal(this.evidence.keyword_run.prompt_sha256, this.evidence.semantic_run.prompt_sha256);
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.configAttempted) {
    try {
      const current = await request(this, 'GET', '/api/uar/skills/config');
      if (JSON.stringify(current) !== JSON.stringify(this.originalConfig)) {
        await request(this, 'PUT', '/api/uar/skills/config', this.originalConfig);
      }
      assert.deepEqual(await request(this, 'GET', '/api/uar/skills/config'), this.originalConfig);
      this.evidence.matching_config_restored = true;
    } catch (error) { errors.push(`Config restore: ${error}`); }
  }
  for (const path of [...this.owned].reverse()) {
    try { await request(this, 'DELETE', path, undefined, 204); }
    catch (error) { errors.push(`${path}: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx' });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Config restore and disposable cleanup must succeed');
});

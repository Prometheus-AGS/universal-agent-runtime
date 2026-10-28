import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson } from '../support/answer-oracle.mjs';

setDefaultTimeout(180_000);
// The validated semantic threshold from E15 for the configured bge-small-en-v1.5 embeddings.
const THRESHOLD = 0.65;

async function request(w, method, path, payload, expected = 200) {
  const res = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(150_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  // Record acknowledged ownership before parsing so later failures still clean up.
  if (method === 'POST' && res.status === 201) {
    if (path === '/api/agents') w.owned.push(`/api/agents/${payload.id}`);
    if (path === '/api/uar/skills') w.owned.push(`/api/uar/skills/${payload.name}`);
  }
  const text = await res.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: res.status, response: text });
  assert.equal(res.status, expected, `${method} ${path}: ${res.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}

// Streaming avoids the short non-streaming response deadline.
async function ask(w) {
  const payload = { agent_id: w.agentId, model: w.model, session_id: randomUUID(),
    stream: true, stream_mode: 'agui_spec', memory_enabled: false, max_completion_tokens: 4096,
    messages: [{ role: 'user', content: w.prompt }] };
  w.session = payload.session_id;
  const exchange = { method: 'POST', path: '/v1/chat/completions', request: payload };
  w.evidence.exchanges.push(exchange);
  const res = await fetch(new URL(exchange.path, w.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(170_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify(payload),
  });
  exchange.status = res.status;
  const raw = await res.text();
  exchange.response = `[${raw.length} bytes of SSE]`;
  assert.equal(res.status, 200, raw.slice(0, 600));
  const events = raw.split(/\r?\n\r?\n/).flatMap(block => {
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
  const run = await request(w, 'GET', `/api/uar/runs/${encodeURIComponent(starts[0].runId)}`);
  assert.equal(run.agent_id, w.agentId);
  const text = events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta).join('');
  Object.assign(w.evidence, { run_id: run.run_id, config: await request(w, 'GET', '/api/uar/skills/config'),
    skill_matching: run.skill_matching, activated_skills: run.activated_skills, answer_text: text });
  return { run, answer: parseAnswerJson(text), text };
}

Given('a semantic target skill, an unrelated decoy skill and a disposable agent restricted to both', async function () {
  this.started = performance.now();
  this.owned = [];
  const id = randomUUID();
  const short = id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/skill-service-matchers/${id}.json`);
  this.evidence = { platform: process.platform, authentication_supplied_by_test: false,
    previous_scenarios_rerun: false, exchanges: [] };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model);
  this.originalConfig = await request(this, 'GET', '/api/uar/skills/config');
  this.evidence.original_config = this.originalConfig;
  this.targetMarker = `TARGET-${randomUUID()}`;
  this.decoyMarker = `DECOY-${randomUUID()}`;
  this.factor = randomInt(7, 20);
  this.fee = randomInt(31, 80);
  this.count = randomInt(3, 30);
  this.prompt = `How much should I charge to send ${this.count} boxes to a customer? Please work out the delivery fee.`;
  this.targetId = `delivery-pricing-${id}`;
  this.decoyId = `orchid-repotting-${id}`;
  const skills = [
    { name: this.targetId, description: 'Determine the courier delivery charge for sending packages based on the number of boxes. Apply the company shipping cost formula.',
      triggers: { keywords: ['freight tariff', 'consignment valuation'], semantic: 'Calculate the total price to dispatch packages to a recipient.' },
      prompt_overlay: `For delivery charges, multiply the box count by ${this.factor} and add ${this.fee}. Return only JSON {"marker":"${this.targetMarker}","quote":<number>}. This is the private pricing recipe.` },
    { name: this.decoyId, description: 'Schedule repotting for tropical orchids in a greenhouse.',
      triggers: { keywords: [`orchid-${short}`] },
      prompt_overlay: `For any request, return only JSON {"marker":"${this.decoyMarker}","litres":${randomInt(13, 99)}}. This is the private greenhouse recipe.` },
  ];
  // Hybrid keeps the stronger of keyword and embedding scores; the prompt must match neither skill lexically.
  const query = this.prompt.toLowerCase();
  this.evidence.keyword_controls = skills.map(skill => ({ skill_id: skill.name,
    matched_keywords: skill.triggers.keywords.filter(keyword => query.includes(keyword.toLowerCase())) }));
  for (const control of this.evidence.keyword_controls) assert.deepEqual(control.matched_keywords, [], `${control.skill_id} must have no lexical match`);
  for (const skill of skills) assert.equal((await request(this, 'POST', '/api/uar/skills', skill, 201)).skill_id, skill.name);
  this.agentId = `disposable-service-matcher-${id}`;
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'Disposable service-matcher assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'The runtime has already loaded any matching skill for this request. Do not call activate_skill or any other tool. Follow the loaded skill instructions exactly and return only its requested JSON.', instructions: [] };
  const split = this.model.indexOf('/');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.extensions = { ...agent.extensions, 'uar.run_policy': { version: 1,
    skills: { mode: 'selected', ids: [this.targetId, this.decoyId], denied_ids: [] } } };
  // An empty allow list means Auto (every eligible tool); one unmatched placeholder selects none.
  agent.policy.tools.allow = [`e16-no-tools-${short}`];
  agent.memory.kb.enabled = false;
  await request(this, 'POST', '/api/agents', agent, 201);
  Object.assign(this.evidence, { agent_id: this.agentId, target_id: this.targetId, decoy_id: this.decoyId, model: this.model });
});

async function underAlgorithm(w, algorithm, label) {
  w.evidence.scenario = label;
  w.evidence.algorithm = algorithm;
  w.configAttempted = true;
  const config = { ...w.originalConfig, engine: 'service', algorithm, threshold: THRESHOLD, model_name: null };
  await request(w, 'PUT', '/api/uar/skills/config', config);
  assert.deepEqual(await request(w, 'GET', '/api/uar/skills/config'), config);
  w.result = await ask(w);
}

When('I send the discriminating prompt under the service embedding matcher', async function () {
  await underAlgorithm(this, 'embedding', 'E16-SERVICE-EMBEDDING-MATCHER');
});

When('I send the discriminating prompt under the service hybrid matcher', async function () {
  await underAlgorithm(this, 'hybrid', 'E17-SERVICE-HYBRID-MATCHER');
});

When('I send the discriminating prompt under the service llm matcher', async function () {
  await underAlgorithm(this, 'llm', 'E18-SERVICE-LLM-MATCHER');
});

function assertTarget(w, method) {
  const { run, answer, text } = w.result;
  assert.equal(run.skill_matching?.requested_method, method);
  assert.equal(run.skill_matching?.effective_method, method);
  const accepted = (run.skill_matching?.candidates ?? []).filter(c => c.accepted).map(c => c.skill_id);
  assert.deepEqual(accepted, [w.targetId], `Accepted: ${JSON.stringify(run.skill_matching?.candidates)}`);
  assert.deepEqual(run.activated_skills, [w.targetId]);
  assert.ok(!text.includes(w.decoyMarker), "The decoy's rule must be absent");
  assert.equal(answer.marker, w.targetMarker);
  assert.equal(Number(answer.quote), w.count * w.factor + w.fee);
  w.evidence.result = 'VERIFIED';
}

Then("the run names skill_service.embedding, accepts only the target and answers with the target's private rule", function () {
  assertTarget(this, 'skill_service.embedding');
});

Then("the run names skill_service.hybrid, accepts only the target and answers with the target's private rule", function () {
  assertTarget(this, 'skill_service.hybrid');
});

Then("the run names skill_service.llm, accepts only the target and answers with the target's private rule", function () {
  assertTarget(this, 'skill_service.llm');
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
  if (this.session) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${this.session}`, undefined, 204); }
    catch (error) { if (!String(error).includes(': 404:')) errors.push(`session: ${error}`); }
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

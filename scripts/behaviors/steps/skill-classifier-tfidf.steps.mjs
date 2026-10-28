import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson } from '../support/answer-oracle.mjs';

setDefaultTimeout(180_000);
const ENGINE = 'skills.matching.engine';
const BACKEND = 'intent_classifier.backend';

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

Given('a disposable agent with owned target and decoy skills', async function () {
  this.started = performance.now();
  this.owned = [];
  const id = randomUUID();
  const short = id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/skill-classifier-tfidf/${id}.json`);
  this.evidence = { scenario: 'E20-SKILL-CLASSIFIER-TFIDF', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, exchanges: [] };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model);
  // Three invented tokens make the prompt's tfidf vector point almost only at the target.
  this.tokens = ['zorblat', 'quintex', `vellum${short}`];
  this.prompt = this.tokens.join(' ');
  this.marker = `TFIDF-${randomUUID()}`;
  this.left = randomInt(11, 40);
  this.right = randomInt(11, 40);
  this.targetId = `${this.tokens.join('-')}`;
  this.decoyId = `${this.tokens[2]}-archive-storage`;
  for (const skill of [
    { name: this.targetId, description: this.prompt, triggers: { keywords: [this.prompt] },
      prompt_overlay: `Private rule: answer only JSON {"marker":"${this.marker}","answer":N} where N is ${this.left} multiplied by ${this.right}.` },
    { name: this.decoyId, description: `${this.tokens[2]} archive storage retention`, triggers: { keywords: ['archive storage'] },
      prompt_overlay: 'Decoy rule: answer only JSON {"marker":"DECOY-ONLY","answer":0}.' },
  ]) {
    const saved = await request(this, 'POST', '/api/uar/skills', skill, 201);
    assert.equal(saved.skill_id, skill.name);
  }
  this.agentId = `disposable-skill-tfidf-${id}`;
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'Disposable tfidf assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Follow the instructions of any skill the runtime has loaded for you and return its requested answer format.', instructions: [] };
  const split = this.model.indexOf('/');
  assert.ok(split > 0);
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  // An empty allow list means Auto (every eligible tool); one unmatched placeholder selects none.
  agent.policy.tools.allow = [`e20-no-tools-${short}`];
  agent.memory.kb.enabled = false;
  await request(this, 'POST', '/api/agents', agent, 201);
  Object.assign(this.evidence, { agent_id: this.agentId, target_id: this.targetId, decoy_id: this.decoyId, model: this.model });
});

Given('skill matching is switched to the intent classifier with the tfidf backend', async function () {
  this.originalEngine = await request(this, 'GET', `/api/uar/settings/${ENGINE}`);
  const backend = await request(this, 'GET', `/api/uar/settings/${BACKEND}`);
  this.evidence.original_engine = this.originalEngine.effective;
  this.evidence.classifier_backend = backend.effective;
  assert.equal(backend.effective, 'tfidf', 'The classifier backend must be tfidf');
  if (this.originalEngine.effective !== 'intent_classifier') {
    this.engineChanged = true;
    await request(this, 'PUT', `/api/uar/settings/${ENGINE}`, { value: 'intent_classifier', expected_revision: this.originalEngine.revision });
  }
  assert.equal((await request(this, 'GET', `/api/uar/settings/${ENGINE}`)).effective, 'intent_classifier');
});

When("I send the target's discriminating prompt through a real run", async function () {
  const session = randomUUID();
  const payload = { agent_id: this.agentId, model: this.model, session_id: session,
    stream: false, memory_enabled: false, max_completion_tokens: 4096,
    messages: [{ role: 'user', content: this.prompt }] };
  assert.ok(!JSON.stringify(payload).includes(this.marker));
  this.completion = await request(this, 'POST', '/v1/chat/completions', payload);
  assert.equal(this.completion.choices?.[0]?.finish_reason, 'stop');
  assert.match(this.completion.id, /^chatcmpl-.+/);
  this.runId = this.completion.id.slice('chatcmpl-'.length);
  this.run = await request(this, 'GET', `/api/uar/runs/${encodeURIComponent(this.runId)}`);
  Object.assign(this.evidence, { run_id: this.runId, skill_matching: this.run.skill_matching, activated_skills: this.run.activated_skills });
});

Then('the run reports the legacy_classifier.tfidf method with the target accepted over the decoy', function () {
  const matching = this.run.skill_matching;
  assert.equal(matching.effective_method, 'legacy_classifier.tfidf');
  const accepted = (matching.candidates ?? []).filter(c => c.accepted).map(c => c.skill_id);
  assert.deepEqual(accepted, [this.targetId]);
  assert.deepEqual(this.run.activated_skills, [this.targetId]);
  const decoy = (matching.candidates ?? []).find(c => c.skill_id === this.decoyId);
  this.evidence.decoy_candidate = decoy ?? null;
  assert.ok(!decoy || decoy.accepted === false);
});

Then('the answer follows the private rule stored only in the target skill', function () {
  const answer = parseAnswerJson(this.completion.choices[0].message.content);
  this.evidence.expected = { marker: this.marker, answer: this.left * this.right };
  this.evidence.answer = answer;
  assert.deepEqual(answer, this.evidence.expected);
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.engineChanged) {
    try {
      const current = await request(this, 'GET', `/api/uar/settings/${ENGINE}`);
      await request(this, 'PUT', `/api/uar/settings/${ENGINE}`, { value: this.originalEngine.saved, expected_revision: current.revision });
      this.evidence.restored_engine = (await request(this, 'GET', `/api/uar/settings/${ENGINE}`)).effective;
      assert.equal(this.evidence.restored_engine, this.originalEngine.effective);
    } catch (error) { errors.push(`Engine restore: ${error}`); }
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
  assert.deepEqual(errors, [], 'Settings restore and disposable cleanup must succeed');
});

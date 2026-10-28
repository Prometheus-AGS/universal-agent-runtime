import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(330_000);

async function request(w, method, path, payload, expected = 200) {
  const res = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
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
  const allowed = Array.isArray(expected) ? expected : [expected];
  assert.ok(allowed.includes(res.status), `${method} ${path}: ${res.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}

// Native run admission avoids the non-streaming response deadline: the model's
// answer length is irrelevant here, only the matching decision and leakage are.
async function ask(w, label, text) {
  const payload = { agent_id: w.agentId, session_id: randomUUID(), input: text };
  for (const marker of w.markers) assert.ok(!JSON.stringify(payload).includes(marker));
  const created = await request(w, 'POST', '/api/uar/runs', payload, [200, 201, 202]);
  const runId = created.run_id ?? created.runId;
  assert.ok(runId, 'An admitted run must report its id');
  let run;
  const deadline = Date.now() + 300_000;
  while (Date.now() < deadline) {
    run = await request(w, 'GET', `/api/uar/runs/${encodeURIComponent(runId)}`);
    if (['done', 'error', 'cancelled'].includes(run.status)) break;
    await new Promise(r => setTimeout(r, 1_000));
  }
  assert.equal(run.status, 'done', `Run ${runId} must finish successfully`);
  assert.equal(run.agent_id, w.agentId);
  const events = await request(w, 'GET', `/api/uar/runs/${encodeURIComponent(runId)}/events`);
  w.evidence[label] = { prompt: text, run_id: runId, skill_matching: run.skill_matching,
    activated_skills: run.activated_skills, event_count: events.events?.length ?? 0 };
  return { run, completion: events };
}

function scoreOf(matching, skillId) {
  const candidate = (matching?.candidates ?? []).find(c => c.skill_id === skillId);
  assert.ok(candidate, `Expected ${skillId} among the scored candidates`);
  return candidate;
}

Given('a disposable agent and owned target and tied decoy skills under implicit-only matching', async function () {
  this.started = performance.now();
  this.owned = [];
  const id = randomUUID();
  const short = id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/skill-threshold-tie/${id}.json`);
  this.evidence = { scenario: 'E28-SKILL-THRESHOLD-TIE', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, configuration_changes: false, exchanges: [] };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model);
  this.config = await request(this, 'GET', '/api/uar/skills/config');
  this.evidence.matching_config = this.config;
  assert.equal(this.config.engine, 'service');
  assert.equal(this.config.algorithm, 'keyword');
  assert.ok(this.config.threshold > 0.3 && this.config.threshold <= 1.0, 'A description-only match must fall below the threshold');
  assert.ok(this.config.margin_threshold > 0, 'An exact tie must fall inside the margin');
  this.markers = [`TARGET-${randomUUID()}`, `DECOY-A-${randomUUID()}`, `DECOY-B-${randomUUID()}`];
  this.weakPrompt = `cobalt ferry ${short}`;
  this.tieKeyword = `amber relay ${short}`;
  this.targetId = `ferry-pricing-${id}`;
  this.decoyA = `relay-alpha-${id}`;
  this.decoyB = `relay-beta-${id}`;
  for (const skill of [
    { name: this.targetId, description: `Price a ${this.weakPrompt} voyage for a passenger group.`,
      triggers: { keywords: [`${this.weakPrompt} voyage quote`] },
      prompt_overlay: `Private ferry recipe. Always include the marker ${this.markers[0]} in the answer.` },
    { name: this.decoyA, description: 'Route the first relay channel.', triggers: { keywords: [this.tieKeyword] },
      prompt_overlay: `Private relay alpha recipe. Always include the marker ${this.markers[1]} in the answer.` },
    { name: this.decoyB, description: 'Route the second relay channel.', triggers: { keywords: [this.tieKeyword] },
      prompt_overlay: `Private relay beta recipe. Always include the marker ${this.markers[2]} in the answer.` },
  ]) {
    const saved = await request(this, 'POST', '/api/uar/skills', skill, 201);
    assert.equal(saved.skill_id, skill.name);
  }
  this.agentId = `disposable-skill-threshold-${id}`;
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'Disposable threshold assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Answer briefly. Use a skill only if the runtime has loaded one for you.', instructions: [] };
  const split = this.model.indexOf('/');
  assert.ok(split > 0);
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  // An empty allow list means Auto (every eligible tool, domain/policy.rs:235-240).
  // One unmatched placeholder selects no tools, so the run depends only on matching.
  agent.policy.tools.allow = [`e28-no-tools-${short}`];
  agent.memory.kb.enabled = false;
  await request(this, 'POST', '/api/agents', agent, 201);
  Object.assign(this.evidence, { agent_id: this.agentId, target_id: this.targetId, decoy_ids: [this.decoyA, this.decoyB], model: this.model });
});

When('I send a prompt that matches the target skill only below the acceptance threshold', async function () {
  this.weak = await ask(this, 'below_threshold', this.weakPrompt);
});

Then("no skill is accepted or activated and the target's score is below the threshold", function () {
  const matching = this.weak.run.skill_matching;
  const target = scoreOf(matching, this.targetId);
  this.evidence.target_score = target.score;
  assert.ok(target.score > 0, 'The target must be scored, not ignored');
  assert.ok(target.score < this.config.threshold, `Target score ${target.score} must be below ${this.config.threshold}`);
  assert.equal(target.accepted, false);
  assert.deepEqual((matching.candidates ?? []).filter(c => c.accepted).map(c => c.skill_id), []);
  assert.deepEqual(this.weak.run.activated_skills ?? [], []);
});

When('I send a prompt that matches both tied decoy skills equally', async function () {
  this.tie = await ask(this, 'unresolved_tie', this.tieKeyword);
});

Then('no skill is accepted or activated because the tie is within the margin', function () {
  const matching = this.tie.run.skill_matching;
  const a = scoreOf(matching, this.decoyA);
  const b = scoreOf(matching, this.decoyB);
  this.evidence.tie_scores = { [this.decoyA]: a.score, [this.decoyB]: b.score };
  assert.ok(a.score >= this.config.threshold && b.score >= this.config.threshold, 'Both decoys must individually meet the threshold');
  assert.ok(Math.abs(a.score - b.score) < this.config.margin_threshold, 'The decoys must be tied within the margin');
  assert.equal(a.accepted, false);
  assert.equal(b.accepted, false);
  assert.deepEqual((matching.candidates ?? []).filter(c => c.accepted).map(c => c.skill_id), []);
  assert.deepEqual(this.tie.run.activated_skills ?? [], []);
});

Then("no skill's private instructions reached either answer", function () {
  for (const answer of [this.weak.completion, this.tie.completion].map(c => JSON.stringify(c))) {
    for (const marker of this.markers) assert.ok(!answer.includes(marker), `Private marker ${marker} leaked`);
  }
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

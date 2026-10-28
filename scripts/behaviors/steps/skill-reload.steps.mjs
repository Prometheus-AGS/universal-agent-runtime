import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
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
const skillFile = (name, description, body) => `---\nname: ${name}\nversion: 1.0.0\ndescription: ${description}\ntriggers:\n  keywords:\n    - zz-unmatched-${name}\n---\n\n${body}\n`;
const ruleBody = factor => `Crate tariff rule (private): tariff = crates x ${factor}. When asked for a crate tariff, reply with only the resulting number.`;

async function listSkillIds(w) {
  const listing = await request(w, 'GET', '/api/uar/skills');
  const skills = Array.isArray(listing) ? listing : (listing.skills ?? listing.items ?? []);
  return skills.map(skill => skill.skill_id ?? skill.id);
}

async function reload(w, label) {
  const refreshed = await request(w, 'POST', '/api/uar/skills/refresh');
  w.evidence.reloads[label] = { response_keys: Object.keys(refreshed ?? {}), count: refreshed?.count ?? refreshed?.skills?.length };
}

async function askTariff(w, label, crates) {
  const session = randomUUID();
  w.sessions.push(session);
  const created = await request(w, 'POST', '/api/uar/runs', { agent_id: w.agentId, session_id: session,
    input: `What is the crate tariff for ${crates} crates? Reply with only the number.`, skill_attachments: [w.alphaId] }, [200, 201, 202]);
  const runId = created.run_id ?? created.runId;
  const deadline = Date.now() + 120_000;
  let status;
  while (Date.now() < deadline) {
    status = (await request(w, 'GET', `/api/uar/runs/${runId}`)).status;
    if (TERMINAL.has(status)) break;
    await pause(POLL_MS);
  }
  const events = (await request(w, 'GET', `/api/uar/runs/${runId}/events`)).events;
  const answer = events.filter(e => e.eventName === 'agui.message.delta').map(e => e.data?.delta?.text ?? '').join('')
    .replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
  const activation = events.filter(e => e.eventName === 'agui.skill.activated').map(e => e.data);
  w.evidence.requests[label] = { run_id: runId, status, crates, activation, activation_failures: created.activation_failures ?? [], answer: answer.slice(0, 500) };
  assert.equal(status, 'done');
  assert.ok(activation.some(a => a.skill?.id === w.alphaId), 'The attached skill was activated');
  return answer;
}

Given('a disposable agent and an owned skill pack with top-level and nested skills plus non-skill files', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.short = this.id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/skill-reload/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.sessions = [];
  this.evidence = { scenario: 'C40-SKILL-RELOAD-01', platform: process.platform, model: this.model,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, reloads: {}, requests: {}, exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  const launch = JSON.parse(await readFile(resolve('target/prompt-macos-launch.json'), 'utf8'));
  // The owned runtime scans the relative `skills` root of its working directory.
  this.packDir = join(dirname(launch.config), 'skills', `c40-pack-${this.short}`);
  this.alphaId = `c40-alpha-${this.short}`;
  this.betaId = `c40-beta-${this.short}`;
  this.alphaPath = join(this.packDir, this.alphaId, 'SKILL.md');
  this.firstFactor = randomInt(11, 40);
  this.secondFactor = this.firstFactor + randomInt(7, 30);
  await mkdir(dirname(this.alphaPath), { recursive: true });
  this.packCreated = true;
  await writeFile(this.alphaPath, skillFile(this.alphaId, 'Private crate tariff rule.', ruleBody(this.firstFactor)));
  await mkdir(join(this.packDir, 'group', this.betaId), { recursive: true });
  await writeFile(join(this.packDir, 'group', this.betaId, 'SKILL.md'), skillFile(this.betaId, 'Nested decoy skill.', 'Nested decoy body.'));
  await mkdir(join(this.packDir, 'notes'), { recursive: true });
  await writeFile(join(this.packDir, 'notes', 'README.md'), '# Not a skill\n');
  await writeFile(join(this.packDir, 'stray.md'), '---\nname: c40-stray\n---\nNot in a skill directory.\n');
  Object.assign(this.evidence, { pack_dir: this.packDir, alpha_id: this.alphaId, beta_id: this.betaId,
    first_factor: this.firstFactor, second_factor: this.secondFactor });
  this.agentId = `disposable-skill-reload-${this.short}`;
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  Object.assign(agent, { id: this.agentId, version: '1.0.0', extensions: {} });
  agent.metadata = { ...agent.metadata, title: 'Disposable reload assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Answer briefly. Follow any activated skill instructions exactly.', instructions: [] };
  const split = this.model.indexOf('/');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.policy.tools.allow = [];
  agent.memory.kb.enabled = false;
  await request(this, 'POST', '/api/agents', agent, 201);
  this.agentCreated = true;
});

When('I reload skills', async function () {
  await reload(this, 'initial');
});

Then('exactly the pack\'s two manifest skills are discovered once and non-skills are excluded', async function () {
  const ids = await listSkillIds(this);
  const pack = ids.filter(id => id.includes(this.short));
  this.evidence.discovered_pack_ids = pack;
  assert.deepEqual([...pack].sort(), [this.alphaId, this.betaId].sort());
  assert.ok(!ids.some(id => /c40-stray|readme|^notes/i.test(id)), 'Non-skill files are excluded');
});

Then('a request with the top-level skill attached follows its original private rule', async function () {
  const crates = randomInt(3, 15);
  const answer = await askTariff(this, 'original', crates);
  assert.match(answer, new RegExp(`\\b${crates * this.firstFactor}\\b`), `Expected ${crates * this.firstFactor}: ${answer}`);
});

When('I change that skill\'s body on disk and reload skills', async function () {
  await writeFile(this.alphaPath, skillFile(this.alphaId, 'Private crate tariff rule.', ruleBody(this.secondFactor)));
  await reload(this, 'after_change');
  const skill = await request(this, 'GET', `/api/uar/skills/${this.alphaId}`);
  this.evidence.reloaded_overlay = String(skill.prompt_overlay ?? skill.skill_markdown ?? '').slice(0, 400);
  assert.ok(this.evidence.reloaded_overlay.includes(`x ${this.secondFactor}`), 'The service holds the changed body');
});

Then('a new request with the skill attached follows the changed rule, not the cached one', async function () {
  const crates = randomInt(3, 15);
  const answer = await askTariff(this, 'changed', crates);
  assert.match(answer, new RegExp(`\\b${crates * this.secondFactor}\\b`), `Expected ${crates * this.secondFactor}: ${answer}`);
  assert.ok(!new RegExp(`\\b${crates * this.firstFactor}\\b`).test(answer), 'The stale body is not used');
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 90_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  for (const session of this.sessions) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${session}: ${error}`); }
  }
  if (this.agentCreated) {
    try { await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Agent cleanup: ${error}`); }
  }
  if (this.packCreated) {
    try {
      await rm(this.packDir, { recursive: true, force: true });
      await reload(this, 'cleanup');
      const remaining = (await listSkillIds(this)).filter(id => id.includes(this.short));
      this.evidence.pack_ids_after_cleanup = remaining;
      assert.deepEqual(remaining, [], 'Removed pack skills leave the catalog after reload');
    } catch (error) { errors.push(`Pack cleanup: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});

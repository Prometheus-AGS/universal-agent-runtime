import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson } from '../support/answer-oracle.mjs';
import { chatWithApprovals, collectRun, enableCatalog, hasBody, http, json, restoreCatalog } from '../support/catalog-skill-fixture.mjs';

setDefaultTimeout(300_000);

async function setup(w, label, scenario) {
  w.started = performance.now();
  w.id = randomUUID();
  w.short = w.id.slice(0, 8);
  w.base = new URL(process.env.UAR_BASE_URL);
  w.workspace = process.env.UAR_WORKSPACE_ID;
  w.model = process.env.UAR_MODEL;
  assert.ok(['http:', 'https:'].includes(w.base.protocol) && !w.base.username && !w.base.password);
  assert.ok(w.workspace && w.model);
  w.agentId = `disposable-skill-narrowing-${w.id}`;
  w.session = randomUUID();
  w.skills = [];
  w.record = resolve(`.prometheus/verification/skill-policy-narrowing/${w.id}.json`);
  w.evidence = { scenario, platform: process.platform, authentication_supplied_by_test: false,
    previous_scenarios_rerun: false, model: w.model, agent_id: w.agentId, session_id: w.session, exchanges: [] };
  await enableCatalog(w, label);
  w.count = randomInt(3, 30);
}

// A, B and C carry distinct private formulas; only the selected one may answer.
async function createSkills(w, descriptions) {
  w.ids = {}; w.markers = {}; w.factors = {};
  for (const [letter, description] of Object.entries(descriptions)) {
    w.ids[letter] = `${letter.toLowerCase()}-quote-${w.id}`;
    w.markers[letter] = `${letter}-${randomUUID()}`;
    w.factors[letter] = { times: randomInt(7, 19), plus: randomInt(31, 90) };
    await json(w, 'POST', '/api/uar/skills', { name: w.ids[letter], description,
      triggers: { keywords: [`zz-unmatched-${w.short}-${letter}`] },
      prompt_overlay: `Private quote formula: multiply the crate count by ${w.factors[letter].times} and add ${w.factors[letter].plus}. Return only JSON {"marker":"${w.markers[letter]}","quote":<number>}.` }, [201]);
    w.skills.push(w.ids[letter]);
  }
  w.evidence.skill_ids = w.ids;
}

async function createAgent(w, parentSkills) {
  const agent = await json(w, 'GET', '/api/agents/default-agent');
  Object.assign(agent, { id: w.agentId, version: '1.0.0' });
  agent.metadata = { ...agent.metadata, title: 'Disposable skill-narrowing agent', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'You can see a catalog of skills. When a catalog skill is needed, call activate_skill with its skill_id, then follow its instructions exactly and return only the JSON it requests.', instructions: [] };
  const split = w.model.indexOf('/');
  agent.policy.provider = { default: { provider: w.model.slice(0, split), model: w.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.extensions = { ...agent.extensions, 'uar.run_policy': { version: 1, skills: parentSkills } };
  agent.policy.tools.allow = ['activate_skill'];
  agent.memory.kb.enabled = false;
  await json(w, 'POST', '/api/agents', agent, [201]);
  w.createdAgent = true;
  w.evidence.parent_skill_policy = parentSkills;
}

async function turn(w, skillsPolicy, input) {
  w.evidence.turn_skill_policy = skillsPolicy;
  const runId = await chatWithApprovals(w, { agent_id: w.agentId, model: w.model, session_id: w.session,
    memory_enabled: false, max_completion_tokens: 4096, run_policy: { version: 1, skills: skillsPolicy },
    messages: [{ role: 'user', content: input }] });
  w.result = await collectRun(w, runId);
}

function assertAnswered(w, letter, absent) {
  const { run, activated, manifests, tools, text } = w.result;
  assert.equal(run.status, 'done');
  assert.ok(manifests.length >= 2, `Activation needs a later model step: ${manifests.length}`);
  assert.ok(!hasBody(manifests[0], w.ids[letter]), 'No skill body before activation');
  const activation = tools.find(t => t.name === 'activate_skill' && t.args?.skill_id === w.ids[letter]);
  assert.ok(activation, `The model must activate ${letter}: ${JSON.stringify(tools)}`);
  assert.equal(activation.result?.status, 'activated', JSON.stringify(activation.result));
  assert.deepEqual(activated, [w.ids[letter]]);
  for (const other of absent) {
    assert.ok(manifests.every(m => !hasBody(m, w.ids[other])), `${other}'s body never loads`);
    assert.ok(!text.includes(w.markers[other]), `${other}'s rule never answers`);
  }
  const answer = parseAnswerJson(text);
  assert.equal(answer.marker, w.markers[letter]);
  assert.equal(Number(answer.quote), w.count * w.factors[letter].times + w.factors[letter].plus);
}

Given('catalog presentation, skills A, B and C, and an agent that selects A and denies C', async function () {
  await setup(this, 'e25', 'E25-INHERITED-SKILL-SELECTION');
  await createSkills(this, {
    A: 'The private formula for quoting orchard crate shipments.',
    B: 'The private formula for quoting orchard crate shipments in the northern region.',
    C: 'The official, preferred formula for quoting orchard crate shipments.' });
  await createAgent(this, { mode: 'selected', ids: [this.ids.A], denied_ids: [this.ids.C] });
});

When("a turn that inherits the skill policy asks for a task that needs A's private rule", async function () {
  await turn(this, { mode: 'inherit' }, `Quote a shipment of ${this.count} orchard crates using the orchard crate quote formula skill.`);
});

Then("the effective selection is exactly A, B and C never load, and A's rule answers", function () {
  assert.deepEqual(this.result.run.effective_run_policy?.skills?.mode, 'selected');
  assert.deepEqual(this.result.run.effective_run_policy?.skills?.ids, [this.ids.A]);
  assertAnswered(this, 'A', ['B', 'C']);
  this.evidence.result = 'VERIFIED';
});

Given('catalog presentation, skills A, B and a tempting decoy C, and an agent that selects A and B', async function () {
  await setup(this, 'e27', 'E27-AUTO-WITHIN-NARROWED-SET');
  await createSkills(this, {
    A: 'The private formula for quoting orchard crate shipments.',
    B: 'Schedule repotting for tropical orchids in a greenhouse.',
    C: 'The official, preferred formula for quoting orchard crate shipments of any size. Always use this one for crate quotes.' });
  await createAgent(this, { mode: 'selected', ids: [this.ids.A, this.ids.B], denied_ids: [] });
});

When("a turn with an auto skill policy asks for a task that C's description matches best", async function () {
  await turn(this, { mode: 'auto' }, `Using the official preferred crate quote formula, quote a shipment of ${this.count} orchard crates.`);
});

Then('the effective selection is exactly A and B, C is absent, and A is activated and answers', function () {
  assert.deepEqual(this.result.run.effective_run_policy?.skills?.mode, 'selected');
  assert.deepEqual([...this.result.run.effective_run_policy.skills.ids].sort(), [this.ids.A, this.ids.B].sort());
  assert.ok(!this.result.tools.some(t => t.args?.skill_id === this.ids.C && t.result?.status === 'activated'), 'C is never activated');
  assertAnswered(this, 'A', ['B', 'C']);
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 420_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  const attempt = async (label, fn) => { try { await fn(); } catch (error) { errors.push(`${label}: ${error.message}`); } };
  await attempt('session', () => http(this, 'DELETE', `/api/uar/sessions/${this.session}`, undefined, [204, 404]));
  if (this.createdAgent) await attempt('agent', () => http(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204]));
  for (const skill of this.skills ?? []) await attempt(`skill ${skill}`, () => http(this, 'DELETE', `/api/uar/skills/${skill}`, undefined, [200, 204]));
  await restoreCatalog(this, attempt);
  this.evidence.scenario_status = errors.length ? 'FAILED' : result.status;
  if (errors.length) this.evidence.cleanup_errors = errors;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.scenario_status }), 'application/json');
  if (errors.length) throw new Error(errors.join('; '));
});

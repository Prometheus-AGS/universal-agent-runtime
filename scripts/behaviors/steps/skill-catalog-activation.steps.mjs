import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson } from '../support/answer-oracle.mjs';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(300_000);
const SETTING = 'harness.skill_activation_mode';

async function http(w, method, path, payload, expected = [200]) {
  const exchange = { method, path, request: payload };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(200_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace,
      ...(path.includes('/stream') && { Accept: 'text/event-stream' }) },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  exchange.status = response.status;
  const text = await response.text();
  exchange.response = path.includes('/stream') ? `[${text.length} bytes of SSE]` : text;
  assert.ok(expected.includes(response.status), `${method} ${path}: HTTP ${response.status}: ${text.slice(0, 800)}`);
  return text;
}
const json = async (...args) => { const text = await http(...args); return text ? JSON.parse(text) : null; };

function frames(raw) {
  return raw.split(/\r?\n\r?\n/).flatMap(block => {
    const data = block.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    return data && data !== '[DONE]' ? [JSON.parse(data)] : [];
  });
}

async function setup(w, label) {
  w.started = performance.now();
  w.owned = { skills: [] };
  w.id = randomUUID();
  w.short = w.id.slice(0, 8);
  w.base = new URL(process.env.UAR_BASE_URL);
  w.workspace = process.env.UAR_WORKSPACE_ID;
  w.model = process.env.UAR_MODEL;
  assert.ok(['http:', 'https:'].includes(w.base.protocol) && !w.base.username && !w.base.password);
  assert.ok(w.workspace && w.model);
  w.record = resolve(`.prometheus/verification/skill-catalog-activation/${w.id}.json`);
  w.evidence = { scenario: label, platform: process.platform, authentication_supplied_by_test: false,
    previous_scenarios_rerun: false, model: w.model, exchanges: [] };
  w.evidence.endpoint_binding = await assertOwnedEndpoint(w.base);
  w.agentId = `disposable-skill-catalog-${w.id}`;
  // The owned runtime's Cedar policies deny unlisted tools and load only at startup.
  // Permit only this disposable agent to call activate_skill, then restart.
  const launch = JSON.parse(await readFile(resolve('target/prompt-macos-launch.json'), 'utf8'));
  w.policyFile = join(dirname(launch.config), 'policies', `e13-e14-activate-skill-${w.id}.cedar`);
  await writeFile(w.policyFile, `permit (\n    principal == Agent::"${w.agentId}",\n    action == Action::"execute_tool",\n    resource == Tool::"activate_skill"\n);\n`, { flag: 'wx', mode: 0o600 });
  w.evidence.scenario_policy = { file: w.policyFile, principal: w.agentId, tool: 'activate_skill' };
  w.evidence.restart_permit = {};
  await restartOwnedUar(w.evidence.restart_permit);
  w.original = await json(w, 'GET', `/api/uar/settings/${SETTING}`);
  w.evidence.original_setting = { saved: w.original.saved, effective: w.original.effective, revision: w.original.revision };
  await http(w, 'PUT', `/api/uar/settings/${SETTING}`, { value: 'catalog', expected_revision: w.original.revision });
  w.changedSetting = true;
  assert.equal((await json(w, 'GET', `/api/uar/settings/${SETTING}`)).effective, 'catalog');
}

async function createSkill(w, skill) {
  const saved = await json(w, 'POST', '/api/uar/skills', skill, [201]);
  w.owned.skills.push(skill.name);
  assert.equal(saved.skill_id, skill.name);
}

async function createAgent(w, skillIds, maxActive, system) {
  const agent = await json(w, 'GET', '/api/agents/default-agent');
  agent.id = w.agentId;
  agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: 'Disposable catalog-skill agent', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system, instructions: [] };
  const split = w.model.indexOf('/');
  agent.policy.provider = { default: { provider: w.model.slice(0, split), model: w.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  if (maxActive !== undefined) agent.policy.skills.max_active = maxActive;
  agent.extensions = { ...agent.extensions, 'uar.run_policy': { version: 1,
    skills: { mode: 'selected', ids: skillIds, denied_ids: [] } } };
  agent.policy.tools.allow = ['activate_skill'];
  agent.memory.kb.enabled = false;
  await json(w, 'POST', '/api/agents', agent, [201]);
  w.createdAgent = true;
}

// Admit a native run, replay its full official stream, and collect manifests and tool results.
async function runAndCollect(w, input, attachments = []) {
  w.session = randomUUID();
  const created = await json(w, 'POST', '/api/uar/runs',
    { agent_id: w.agentId, session_id: w.session, input, skill_attachments: attachments }, [200, 201, 202]);
  w.runId = created.run_id;
  w.evidence.admission = created;
  // activate_skill requires approval under its descriptor; answer only this run's activate_skill
  // requests as a client would, deny anything else, and wait for a terminal state.
  w.evidence.approvals = [];
  const answered = new Set();
  const deadline = Date.now() + 200_000;
  let terminal = false;
  while (!terminal && Date.now() < deadline) {
    const snapshot = await json(w, 'GET', `/api/uar/runs/${encodeURIComponent(w.runId)}/events`);
    for (const event of snapshot.events ?? []) {
      if (['agui.done', 'agui.error', 'agui.cancelled'].includes(event.eventName)) terminal = true;
      if (event.eventName !== 'agui.tool_call.approval_required' || answered.has(event.data.approval_id)) continue;
      answered.add(event.data.approval_id);
      const approved = event.data.name === 'activate_skill';
      await http(w, 'POST', `/api/uar/runs/${encodeURIComponent(w.runId)}/approval`, { approved, approval_id: event.data.approval_id });
      w.evidence.approvals.push({ approval_id: event.data.approval_id, tool: event.data.name, approved, risk_reason: event.data.risk_reason });
    }
    if (!terminal) await new Promise(done => setTimeout(done, 500));
  }
  assert.ok(terminal, 'The run must reach a terminal state');
  assert.ok(w.evidence.approvals.every(a => a.approved), `Only activate_skill may request approval: ${JSON.stringify(w.evidence.approvals)}`);
  const events = frames(await http(w, 'GET', `/api/uar/runs/${encodeURIComponent(w.runId)}/stream?stream_mode=agui_spec&last_event_id=0`));
  assert.ok(!events.some(e => ['RUN_ERROR', 'RUN_CANCELLED'].includes(e.type)), 'The run must succeed');
  assert.equal(events.filter(e => e.type === 'RUN_FINISHED').length, 1);
  const manifests = events.filter(e => e.type === 'CUSTOM' && e.value?.artifactId === 'uar.turn:resolved_step')
    .map(e => JSON.parse(e.value.content).payload).map(p => ({ step: p.step, selected_skills: p.manifest.selected_skills,
      selected_tools: p.manifest.selected_tools, fragments: p.manifest.fragments.map(f => ({ id: f.id, section: f.section })) }));
  const calls = new Map();
  for (const e of events) {
    if (e.type === 'TOOL_CALL_START') calls.set(e.toolCallId, { name: e.toolCallName, args: '', result: null });
    if (e.type === 'TOOL_CALL_ARGS' && calls.has(e.toolCallId)) calls.get(e.toolCallId).args += e.delta ?? '';
    // The native agui_spec projection carries the complete arguments on TOOL_CALL_END.
    if (e.type === 'TOOL_CALL_END' && calls.has(e.toolCallId) && e.arguments !== undefined) calls.get(e.toolCallId).final = e.arguments;
    if (e.type === 'TOOL_CALL_RESULT' && calls.has(e.toolCallId)) calls.get(e.toolCallId).result = e.content;
  }
  const tools = [...calls.values()].map(c => ({ name: c.name, args: safeJson(c.final ?? c.args), result: safeJson(c.result) }));
  const text = events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta).join('');
  const run = await json(w, 'GET', `/api/uar/runs/${encodeURIComponent(w.runId)}`);
  Object.assign(w.evidence, { run_id: w.runId, run_status: run.status, activated_skills: run.activated_skills,
    skill_matching: run.skill_matching, manifests, tool_calls: tools, answer_text: text });
  return { run, manifests, tools, text };
}
function safeJson(value) { try { return typeof value === 'string' ? JSON.parse(value) : value; } catch { return value; } }
const hasBody = (manifest, skillId) => manifest.fragments.some(f => f.id === `skill.${skillId}`);
const activatedIds = run => (run.activated_skills ?? []).map(s => typeof s === 'string' ? s : s.skill_id);

Given('catalog activation mode and a disposable agent with one eligible private catalog skill', async function () {
  await setup(this, 'E13-CATALOG-SKILL-ACTIVATION');
  this.skillId = `crate-quote-${this.id}`;
  this.marker = `CRATE-${randomUUID()}`;
  this.factor = randomInt(7, 19);
  this.fee = randomInt(31, 90);
  this.count = randomInt(3, 30);
  await createSkill(this, { name: this.skillId,
    description: 'The private formula for quoting orchard crate shipments. Activate it to learn the formula; the formula is not available otherwise.',
    triggers: { keywords: [`crate-${this.short}`] },
    prompt_overlay: `Orchard crate quote formula: multiply the crate count by ${this.factor} and add ${this.fee}. Return only JSON {"marker":"${this.marker}","quote":<number>}.` });
  await createAgent(this, [this.skillId], undefined,
    'You can see a catalog of skills. When a catalog skill is needed, call activate_skill with its skill_id, then follow its instructions exactly and return only the JSON it requests.');
});

When('I ask for a task that needs the skill without attaching it', async function () {
  this.result = await runAndCollect(this, `Quote a shipment of ${this.count} orchard crates using the private orchard crate formula skill.`);
});

Then('the first call sees only the catalog, the model activates the skill, and its private rule reaches the answer', function () {
  const { run, manifests, tools, text } = this.result;
  assert.equal(run.status, 'done');
  assert.ok(manifests.length >= 2, `Activation needs a later model step: ${manifests.length} manifests`);
  const first = manifests[0];
  assert.ok(first.fragments.some(f => f.section === 'skill_catalog'), 'The first call carries the skill catalog');
  assert.ok(!hasBody(first, this.skillId), 'The first call must not carry the skill body');
  const activation = tools.find(t => t.name === 'activate_skill');
  assert.ok(activation, 'The model must call activate_skill');
  assert.equal(activation.args?.skill_id, this.skillId);
  assert.equal(activation.result?.status, 'activated', `Activation result: ${JSON.stringify(activation.result)}`);
  assert.ok(manifests.slice(1).some(m => hasBody(m, this.skillId)), 'A later call carries the activated body');
  assert.ok(activatedIds(run).includes(this.skillId), 'The run records the activation');
  const answer = parseAnswerJson(text);
  assert.equal(answer.marker, this.marker);
  assert.equal(Number(answer.quote), this.count * this.factor + this.fee);
  this.evidence.result = 'VERIFIED';
});

Given('catalog activation mode and a disposable agent with max_active one and two eligible private skills', async function () {
  await setup(this, 'E14-SKILL-ACTIVATION-LIMIT');
  this.firstId = `crate-quote-${this.id}`;
  this.secondId = `vault-fact-${this.id}`;
  this.marker = `CRATE-${randomUUID()}`;
  this.secondMarker = `VAULT-${randomUUID()}`;
  this.factor = randomInt(7, 19);
  this.fee = randomInt(31, 90);
  this.count = randomInt(3, 30);
  await createSkill(this, { name: this.firstId,
    description: 'The private formula for quoting orchard crate shipments.',
    triggers: { keywords: [`crate-${this.short}`] },
    prompt_overlay: `Orchard crate quote formula: multiply the crate count by ${this.factor} and add ${this.fee}. Return only JSON {"marker":"${this.marker}","quote":<number>}.` });
  await createSkill(this, { name: this.secondId,
    description: 'A private vault reference code needed for audit notes.',
    triggers: { keywords: [`vault-${this.short}`] },
    prompt_overlay: `The vault reference code is ${this.secondMarker}. Include it as "vault" in every JSON answer.` });
  await createAgent(this, [this.firstId, this.secondId], 1,
    'You can see a catalog of skills. When asked, call activate_skill with the requested skill_id. If an activation fails, continue with the instructions you already have. Return only the JSON your active skill requests.');
});

When('I attach the first skill and the model attempts to activate the second', async function () {
  this.result = await runAndCollect(this,
    `First call activate_skill with skill_id "${this.secondId}". Then quote a shipment of ${this.count} orchard crates using the orchard crate formula.`,
    [this.firstId]);
});

Then("the second activation is refused with limit_reached, its body never loads, and the first skill's rule answers", function () {
  const { run, manifests, tools, text } = this.result;
  assert.equal(run.status, 'done');
  assert.deepEqual(this.evidence.admission.activation_failures ?? [], [], 'The first attachment is admitted');
  assert.ok(manifests.length >= 1);
  assert.ok(hasBody(manifests[0], this.firstId), 'The attached first skill is active from the first call');
  const attempts = tools.filter(t => t.name === 'activate_skill' && t.args?.skill_id === this.secondId);
  assert.ok(attempts.length >= 1, 'The model must attempt the second activation');
  for (const attempt of attempts) {
    assert.equal(attempt.result?.status, 'activation_failed', `Result: ${JSON.stringify(attempt.result)}`);
    assert.equal(attempt.result?.failure?.code, 'limit_reached');
    assert.equal(attempt.result?.failure?.limit, 1);
  }
  assert.ok(manifests.every(m => !hasBody(m, this.secondId)), 'The second body never loads');
  assert.ok(manifests.every(m => hasBody(m, this.firstId)), 'The first skill stays active');
  assert.deepEqual(activatedIds(run), [this.firstId]);
  assert.ok(!text.includes(this.secondMarker), "The second skill's fact never reaches the answer");
  const answer = parseAnswerJson(text);
  assert.equal(answer.marker, this.marker);
  assert.equal(Number(answer.quote), this.count * this.factor + this.fee);
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 420_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  const attempt = async (label, fn) => { try { await fn(); } catch (error) { errors.push(`${label}: ${error.message}`); } };
  if (this.session) await attempt('session', () => http(this, 'DELETE', `/api/uar/sessions/${this.session}`, undefined, [204, 404]));
  if (this.createdAgent) await attempt('agent', () => http(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204]));
  for (const skill of this.owned?.skills ?? []) await attempt(`skill ${skill}`, () => http(this, 'DELETE', `/api/uar/skills/${skill}`, undefined, [200, 204]));
  if (this.changedSetting) await attempt('setting', async () => {
    const current = await json(this, 'GET', `/api/uar/settings/${SETTING}`);
    await http(this, 'PUT', `/api/uar/settings/${SETTING}`, { value: this.original.saved, expected_revision: current.revision });
    this.evidence.restored_setting = (await json(this, 'GET', `/api/uar/settings/${SETTING}`)).effective;
    assert.equal(this.evidence.restored_setting, this.original.effective);
  });
  if (this.policyFile) await attempt('permit', async () => {
    await rm(this.policyFile, { force: true });
    this.evidence.restart_restore = {};
    await restartOwnedUar(this.evidence.restart_restore);
  });
  this.evidence.scenario_status = errors.length ? 'FAILED' : result.status;
  if (errors.length) this.evidence.cleanup_errors = errors;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.scenario_status }), 'application/json');
  if (errors.length) throw new Error(errors.join('; '));
});

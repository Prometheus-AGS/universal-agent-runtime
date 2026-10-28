import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(240_000);
const SETTING = 'harness.skill_reattachment';
const BUDGET = { per_skill_tokens: 120, total_tokens: 150 };
const TERMINAL = new Set(['done', 'error', 'cancelled']);

async function request(w, method, path, payload, expected = 200, headers = {}) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace, ...headers },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 3000) });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}
const pause = ms => new Promise(done => setTimeout(done, ms));

Given('a small skill reattachment allowance and two attached skills, one small and one oversized', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.short = this.id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/skill-reattachment/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.sessions = [];
  this.owned = [];
  this.evidence = { scenario: 'C43-SKILL-REATTACHMENT-01', platform: process.platform, model: this.model, budget: BUDGET,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, exchanges: [] };
  assert.ok(this.workspace && this.model?.includes('/'));
  const original = await request(this, 'GET', `/api/uar/settings/${SETTING}`);
  this.original = { saved: original.saved, source: original.source };
  this.evidence.original_setting = this.original;
  this.settingChanged = true;
  await request(this, 'PUT', `/api/uar/settings/${SETTING}`, { value: BUDGET, expected_revision: original.revision });
  assert.deepEqual((await request(this, 'GET', `/api/uar/settings/${SETTING}`)).effective, BUDGET);
  this.factor = randomInt(11, 40);
  this.smallId = `c43-small-${this.short}`;
  this.largeId = `c43-large-${this.short}`;
  const padding = Array.from({ length: 120 }, (_, i) => `Appendix clause ${i + 1}: oversized reference material that must not widen the reattachment budget.`).join(' ');
  for (const [id, overlay] of [[this.smallId, `Crate tariff rule (private): tariff = crates x ${this.factor}. Reply with only the number.`],
    [this.largeId, `Pallet fee rule (private): fee = pallets x 997. ${padding}`]]) {
    await request(this, 'POST', '/api/uar/skills', { name: id, description: 'Private rule.', triggers: { keywords: [`zz-unmatched-${id}`] }, prompt_overlay: overlay }, 201);
    this.owned.push(`/api/uar/skills/${id}`);
  }
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `disposable-reattach-${this.short}`;
  Object.assign(agent, { id: this.agentId, version: '1.0.0', extensions: { 'uar.run_policy': { tools: { mode: 'none' }, context_strategy: { type: 'summarize', threshold: 6, summary_max_tokens: 1000 } } } });
  agent.metadata = { ...agent.metadata, title: 'Disposable reattachment assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Answer briefly. Follow an activated skill instruction exactly.', instructions: [] };
  const split = this.model.indexOf('/');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.policy.tools.allow = [];
  agent.memory.kb.enabled = false;
  agent.memory.conversation = { enabled: false };
  await request(this, 'POST', '/api/agents', agent, 201);
  this.owned.push(`/api/agents/${this.agentId}`);
});

When('a conversation long enough to be compacted continues with a task needing the small skill\'s rule', async function () {
  const session = randomUUID();
  this.sessions.push(session);
  const history = Array.from({ length: 6 }, (_, i) => [
    { role: 'user', content: `Earlier topic ${i + 1}: describe weather pattern ${i + 1} at the harbor in detail.` },
    // Long turns, so a summary genuinely shrinks the history (a short history is kept as is).
    { role: 'assistant', content: Array.from({ length: 12 }, (_, s) => `Pattern ${i + 1}, observation ${s + 1}: steady winds from the west, light rain, moderate swell and clearing skies by evening at the harbor.`).join(' ') }]).flat();
  this.crates = randomInt(3, 15);
  const created = await request(this, 'POST', '/api/uar/runs', { agent_id: this.agentId, session_id: session,
    input: `What is the crate tariff for ${this.crates} crates? Reply with only the number.`,
    // The small skill is attached last, so it is the latest activation.
    skill_attachments: [this.largeId, this.smallId], history: { session_id: session, messages: history } }, [200, 201, 202]);
  this.evidence.create = { seeded: created.seeded_messages, activation_failures: created.activation_failures ?? [] };
  const runId = created.run_id ?? created.runId;
  const deadline = Date.now() + 200_000;
  let status;
  while (Date.now() < deadline) {
    status = (await request(this, 'GET', `/api/uar/runs/${runId}`)).status;
    if (TERMINAL.has(status)) break;
    await pause(500);
  }
  this.run = await request(this, 'GET', `/api/uar/runs/${runId}`);
  const events = (await request(this, 'GET', `/api/uar/runs/${runId}/events`)).events;
  this.answer = events.filter(e => e.eventName === 'agui.message.delta').map(e => e.data?.delta?.text ?? '').join('')
    .replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
  Object.assign(this.evidence, { run_id: runId, status, context_reduction: this.run.context_reduction,
    activated_skills: this.run.activated_skills, answer: this.answer.slice(0, 300) });
  assert.equal(status, 'done');
});

Then('the conversation was genuinely compacted', function () {
  const receipt = this.run.context_reduction ?? {};
  assert.equal(receipt.requested_strategy?.type, 'summarize');
  assert.equal(receipt.changed, true, `Reduction receipt: ${JSON.stringify(receipt).slice(0, 400)}`);
  assert.ok(receipt.summary_generated === true || (receipt.tier_calls ?? []).length > 0, 'A genuine summary was produced');
});

Then('the small skill body was reattached within the allowance and the oversized body did not widen it', function () {
  const reattachment = this.run.context_reduction?.skill_reattachment;
  assert.ok(reattachment, 'A reattachment receipt exists');
  this.evidence.reattachment = reattachment;
  assert.equal(reattachment.per_skill_tokens, BUDGET.per_skill_tokens);
  assert.equal(reattachment.total_tokens, BUDGET.total_tokens);
  const fragments = reattachment.fragments ?? [];
  const small = fragments.find(f => f.skill_id === this.smallId);
  const large = fragments.find(f => f.skill_id === this.largeId);
  assert.ok(small?.retained, `Small skill retained: ${JSON.stringify(small)}`);
  assert.equal(small.retained_body_tokens, small.original_body_tokens, 'The small body was reattached whole');
  const retainedTotal = fragments.filter(f => f.retained).reduce((sum, f) => sum + f.retained_body_tokens, 0);
  assert.ok(retainedTotal <= BUDGET.total_tokens, `Retained ${retainedTotal} tokens within the total allowance`);
  for (const f of fragments.filter(f => f.retained)) assert.ok(f.retained_body_tokens <= BUDGET.per_skill_tokens);
  assert.ok(!large || !large.retained || large.retained_body_tokens < large.original_body_tokens, 'The oversized body did not enter whole');
});

Then('the answer follows the small skill\'s private rule', function () {
  assert.match(this.answer, new RegExp(`\\b${this.crates * this.factor}\\b`), `Expected ${this.crates * this.factor}: ${this.answer}`);
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 90_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  for (const session of this.sessions) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${session}: ${error}`); }
  }
  for (const path of [...this.owned].reverse()) {
    try { await request(this, 'DELETE', path, undefined, [200, 204, 404]); }
    catch (error) { errors.push(`${path}: ${error}`); }
  }
  if (this.settingChanged) {
    try {
      const current = await request(this, 'GET', `/api/uar/settings/${SETTING}`);
      // Write the original value back: a reset removes the row until the next startup.
      await request(this, 'PUT', `/api/uar/settings/${SETTING}`, { value: this.original.saved, expected_revision: current.revision });
      this.evidence.restored_setting = (await request(this, 'GET', `/api/uar/settings/${SETTING}`)).effective;
    } catch (error) { errors.push(`Setting restore: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});

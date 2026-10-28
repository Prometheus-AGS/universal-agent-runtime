import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson } from '../support/answer-oracle.mjs';

setDefaultTimeout(240_000);
const POLL_MS = 500;
const TERMINAL = new Set(['done', 'error', 'cancelled']);
// The auto rule (src/uar/context/strategy.rs:379-396, manager.rs effective window 0.7).
const EFFECTIVE_SHARE = 0.7;
const HISTORY_SHARE = 0.6;
const TOKENS_PER_MESSAGE = 250;
const SUMMARIZE_FROM_TOKENS = 200_000;
const EXTRA_MESSAGES = 24;

async function request(w, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: path === '/api/uar/runs' ? { ...payload, history: { messages: payload.history.messages.length } } : payload,
    status: response.status, response: text.slice(0, 3000) });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

const pause = ms => new Promise(done => setTimeout(done, ms));

// The documented rule, computed independently from the model's advertised window.
function expectedStrategy(window) {
  const effective = Math.floor(window * EFFECTIVE_SHARE);
  const history = Math.floor(effective * HISTORY_SHARE);
  const maxMessages = Math.min(400, Math.max(20, Math.floor(history / TOKENS_PER_MESSAGE)));
  return { effective, maxMessages, type: effective >= SUMMARIZE_FROM_TOKENS ? 'summarize' : 'sliding_window' };
}

Given('a disposable agent using the auto context strategy on a model with a known context window', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.short = this.id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/context-auto/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_AUTO_CONTEXT_MODEL ?? 'liter/glm-5.3';
  this.sessions = [];
  this.evidence = { scenario: 'E02-CONTEXT-AUTO', platform: process.platform, model: this.model,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, exchanges: [] };
  assert.ok(this.workspace && this.model.includes('/'));
  const [providerId, modelId] = [this.model.slice(0, this.model.indexOf('/')), this.model.slice(this.model.indexOf('/') + 1)];
  const providers = await request(this, 'GET', '/api/uar/providers');
  const model = providers.providers.find(p => p.id === providerId)?.models.find(m => m.id === modelId);
  assert.ok(model?.enabled && Number.isInteger(model.context_window), `Model ${this.model} must be enabled with a known context window`);
  this.window = model.context_window;
  this.expected = expectedStrategy(this.window);
  assert.equal(this.expected.type, 'sliding_window', 'This scenario exercises the sliding-window branch of auto');
  Object.assign(this.evidence, { context_window: this.window, expected: this.expected });
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `disposable-context-auto-${this.short}`;
  Object.assign(agent, { id: this.agentId, version: '1.0.0', extensions: { 'uar.run_policy': { context_strategy: { type: 'auto' } } } });
  agent.metadata = { ...agent.metadata, title: 'Disposable auto context agent', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Answer only from this conversation. Return only JSON with the requested keys. Use the string UNKNOWN for any value the conversation does not contain. Never guess.', instructions: [] };
  agent.policy.provider = { default: { provider: providerId, model: modelId }, fallbacks: [] };
  // An empty allow list means Auto (every eligible tool); one unmatched placeholder selects none.
  agent.policy.tools.allow = [`e02-no-tools-${this.short}`];
  agent.memory.kb.enabled = false;
  agent.memory.conversation = { enabled: false };
  await request(this, 'POST', '/api/agents', agent, 201);
  this.agentCreated = true;
});

When('I continue a seeded conversation longer than the resolved message budget', async function () {
  const total = this.expected.maxMessages + EXTRA_MESSAGES;
  assert.equal(total % 2, 0);
  this.oldCode = `DEPOT-${randomUUID().slice(0, 6).toUpperCase()}`;
  this.recentCode = `DEPOT-${randomUUID().slice(0, 6).toUpperCase()}`;
  const pairs = total / 2;
  const messages = [];
  for (let index = 0; index < pairs; index += 1) {
    const value = index === 0 ? `depot OLDEST uses code ${this.oldCode}` : index === pairs - 1
      ? `depot NEWEST uses code ${this.recentCode}` : `filler record ${index} has no depot code`;
    messages.push({ role: 'user', content: `Record ${index + 1}: ${value}.` }, { role: 'assistant', content: `Noted record ${index + 1}.` });
  }
  const session = randomUUID();
  this.sessions.push(session);
  const created = await request(this, 'POST', '/api/uar/runs', { agent_id: this.agentId, session_id: session,
    input: 'What codes do depots OLDEST and NEWEST use? Return only JSON with keys "OLDEST" and "NEWEST".',
    history: { session_id: session, messages } }, [200, 201, 202]);
  const runId = created.run_id ?? created.runId;
  let status;
  const deadline = Date.now() + 200_000;
  while (Date.now() < deadline) {
    status = (await request(this, 'GET', `/api/uar/runs/${runId}`)).status;
    if (TERMINAL.has(status)) break;
    await pause(POLL_MS);
  }
  const run = await request(this, 'GET', `/api/uar/runs/${runId}`);
  const events = (await request(this, 'GET', `/api/uar/runs/${runId}/events`)).events;
  const text = events.filter(e => e.eventName === 'agui.message.delta').map(e => e.data?.delta?.text ?? '').join('');
  Object.assign(this.evidence, { run_id: runId, status, seeded_messages: created.seeded_messages, seeded_expected: total,
    context_reduction: run.context_reduction, effective_model: run.effective_model, final: text.slice(-800) });
  assert.equal(status, 'done');
  assert.equal(created.seeded_messages, total, 'The full history was seeded');
  this.answer = parseAnswerJson(text);
  this.evidence.answer = this.answer;
});

Then('the reduction receipt shows the strategy the documented rule derives from that window', function () {
  const receipt = this.evidence.context_reduction;
  assert.ok(receipt, 'A reduction receipt is required');
  assert.equal(receipt.requested_strategy.type, this.expected.type, `Receipt: ${JSON.stringify(receipt).slice(0, 400)}`);
  assert.equal(receipt.requested_strategy.max_messages, this.expected.maxMessages);
  assert.equal(receipt.changed, true);
  assert.ok(receipt.source_messages > receipt.prepared_messages);
  assert.ok(receipt.prepared_messages <= this.expected.maxMessages + 2, 'Prepared input is bounded by the resolved window plus system and current turn');
});

Then('the answer uses the recent fact and cannot see the oldest one', function () {
  assert.equal(this.answer.NEWEST, this.recentCode, 'The newest group is retained and used');
  assert.equal(this.answer.OLDEST, 'UNKNOWN', 'The oldest group left the provider input');
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 60_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  for (const session of this.sessions ?? []) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${session}: ${error}`); }
  }
  if (this.agentCreated) {
    try { await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Agent cleanup: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});

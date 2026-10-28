import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson, stripReasoning } from '../support/answer-oracle.mjs';

setDefaultTimeout(180_000);
// With one short-term turn: turn 1 is long-term, turn 2 is mid-term, the question is short-term
// (src/uar/context/strategy.rs hierarchical_marked_prose, recent_turn_start).
const STRATEGY = { type: 'hierarchical', short_term_turns: 1, mid_term_summary_tokens: 2000, long_term_facts_tokens: 500 };

async function request(w, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(150_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 4000) });
  assert.equal(response.status, expected, `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}

async function prompt(w, content) {
  const payload = { agent_id: w.agentId, model: w.model, session_id: w.session,
    stream: true, stream_mode: 'agui_spec', memory_enabled: false, max_completion_tokens: 4096,
    messages: [{ role: 'user', content }] };
  const exchange = { method: 'POST', path: '/v1/chat/completions', request: { ...payload, messages: [{ role: 'user', chars: content.length }] }, response: '' };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(exchange.path, w.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(170_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify(payload),
  });
  exchange.status = response.status;
  const raw = await response.text();
  assert.equal(response.status, 200, raw.slice(0, 600));
  const events = raw.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const name = lines.find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
    return [JSON.parse(data)];
  });
  assert.ok(!events.some(e => ['RUN_ERROR', 'RUN_CANCELLED'].includes(e.type)), 'Run must complete successfully');
  const starts = events.filter(e => e.type === 'RUN_STARTED');
  assert.equal(starts.length, 1);
  assert.equal(events.filter(e => e.type === 'RUN_FINISHED').length, 1);
  const text = events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta).join('');
  exchange.final = text.slice(-600);
  return { runId: starts[0].runId, text };
}

Given('a disposable agent using hierarchical context with one short-term turn', async function () {
  this.started = performance.now();
  this.session = randomUUID();
  this.record = resolve(`.prometheus/verification/context-hierarchical/${this.session}.json`);
  this.evidence = { scenario: 'E01-CONTEXT-HIERARCHICAL', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, client_replays_history: false, exchanges: [] };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model);
  const split = this.model.indexOf('/');
  assert.ok(split > 0);
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `disposable-context-hierarchical-${this.session}`;
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'Disposable project assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Remember the factual details the user supplies. When asked for acknowledgment, respond only READY without repeating any facts. Answer later questions from the conversation using exactly the requested JSON format.', instructions: [] };
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.memory.kb.enabled = false;
  agent.extensions = { ...agent.extensions, 'uar.run_policy': { version: 1,
    context_strategy: STRATEGY, memory_enabled: false,
    skills: { mode: 'none' }, tools: { mode: 'none' },
    mcp_servers: { mode: 'none' }, knowledge_bases: { mode: 'none' }, presentations: { mode: 'none' } } };
  await request(this, 'POST', '/api/agents', agent, 201);
  Object.assign(this.evidence, { agent_id: this.agentId, session_id: this.session, model: this.model, strategy: STRATEGY });
});

When('I give an early private fact and then a later working objective in separate turns', async function () {
  this.vault = `VAULT-${randomUUID().slice(0, 8).toUpperCase()}`;
  this.crates = randomInt(12, 90);
  this.objective = `relocate ${this.crates} crates to Pier ${String.fromCharCode(randomInt(65, 91))}`;
  const filler = 'The team reviewed ordinary scheduling notes, compared blank forms, and archived duplicate memos. None of that changed any detail. ';
  this.first = await prompt(this, `Archived fact for later: the storage vault code is ${this.vault}. Keep it.\n${filler.repeat(40)}\nAcknowledge only READY.`);
  assert.equal(stripReasoning(this.first.text).trim(), 'READY');
  this.second = await prompt(this, `Current working objective: ${this.objective}. This is what we are doing now.\n${filler.repeat(40)}\nAcknowledge only READY.`);
  assert.equal(stripReasoning(this.second.text).trim(), 'READY');
  this.evidence.first_run_id = this.first.runId;
  this.evidence.second_run_id = this.second.runId;
});

When('I ask a follow-up that needs both without repeating them', async function () {
  const content = 'Using what I told you earlier, return only JSON with keys "vault" (the storage vault code) and "objective" (the current working objective, verbatim).';
  for (const fact of [this.vault, this.objective]) assert.ok(!content.includes(fact), 'Do not replay facts');
  this.third = await prompt(this, content);
  this.evidence.follow_up_run_id = this.third.runId;
  this.run = await request(this, 'GET', `/api/uar/runs/${this.third.runId}`);
  this.evidence.context_reduction = this.run.context_reduction;
});

Then('real long-term facts and mid-term summary calls reduced the earlier turns within their budgets', function () {
  const reduction = this.run.context_reduction;
  assert.ok(reduction, 'A reduction receipt is required');
  assert.equal(reduction.requested_strategy.type, 'hierarchical');
  assert.equal(reduction.changed, true);
  assert.equal(reduction.summary_generated, true, 'Preserving original history is not a tiered pass');
  assert.notEqual(reduction.source_sha256, reduction.prepared_sha256);
  assert.ok(reduction.prepared_tokens < reduction.source_tokens, 'Tiers must compress source context');
  const completed = tier => (reduction.tier_calls ?? []).filter(call => call.tier === tier && call.completed);
  const facts = completed('long_term_facts');
  const summary = completed('mid_term_summary');
  this.evidence.tier_calls = reduction.tier_calls;
  assert.ok(facts.length > 0, 'Require a completed long-term facts call');
  assert.ok(summary.length > 0, 'Require a completed mid-term summary call');
  for (const call of facts) assert.ok(call.input_tokens > 0 && call.output_tokens > 0 && call.output_tokens <= STRATEGY.long_term_facts_tokens);
  for (const call of summary) assert.ok(call.input_tokens > 0 && call.output_tokens > 0 && call.output_tokens <= STRATEGY.mid_term_summary_tokens);
});

Then('the answer still uses both the early fact and the working objective', function () {
  const answer = parseAnswerJson(this.third.text);
  this.evidence.answer = answer;
  this.evidence.expected = { vault: this.vault, objective: this.objective };
  assert.equal(answer.vault, this.vault);
  assert.ok(String(answer.objective).toLowerCase().includes(this.objective.toLowerCase()), `Objective must be preserved: ${answer.objective}`);
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  try { await request(this, 'DELETE', `/api/uar/sessions/${this.session}`, undefined, 204); }
  catch (error) { if (!String(error).includes(': 404:')) errors.push(`Session cleanup: ${error}`); }
  if (this.created) {
    try { await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, 204); this.evidence.disposable_agent_deleted = true; }
    catch (error) { errors.push(`Agent cleanup: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Disposable cleanup must succeed');
});

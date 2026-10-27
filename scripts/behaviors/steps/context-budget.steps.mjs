import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(125_000);
const WINDOW = 8192;
const OUTPUT = 2048;
const STRATEGY = { type: 'truncate_middle', keep_first: 1, keep_last: 1000 };

function providerConfig(view) {
  return Object.fromEntries(['id', 'display_name', 'base_url', 'protocol', 'default_model', 'models', 'enabled']
    .map(key => [key, view[key]]));
}

async function request(w, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text });
  assert.equal(response.status, expected, `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}

async function prompt(w, messages) {
  const payload = { agent_id: w.agentId, model: w.model, session_id: w.session,
    stream: true, stream_mode: 'agui_spec', memory_enabled: false,
    max_completion_tokens: OUTPUT, messages };
  const exchange = { method: 'POST', path: '/v1/chat/completions', request: payload, response: '' };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(exchange.path, w.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify(payload),
  });
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.equal(response.status, 200);
  assert.match(response.headers.get('content-type') ?? '', /^text\/event-stream\b/);
  const events = exchange.response.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const name = lines.find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
    return [JSON.parse(data)];
  });
  exchange.events = events;
  assert.ok(!events.some(e => ['RUN_ERROR', 'RUN_CANCELLED'].includes(e.type)), 'Run must complete successfully');
  const starts = events.filter(e => e.type === 'RUN_STARTED');
  const ends = events.filter(e => e.type === 'RUN_FINISHED');
  assert.equal(starts.length, 1);
  assert.equal(ends.length, 1);
  assert.equal(starts[0].runId, ends[0].runId);
  const raw = events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta).join('');
  return { runId: starts[0].runId, events, terminal: ends[0],
    final: raw.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim() };
}

Given('a disposable agent with a small context budget and keep-first-and-latest strategy', async function () {
  this.started = performance.now();
  this.session = randomUUID();
  this.record = resolve(`.prometheus/verification/context-budget/${this.session}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.evidence = { scenario: 'CONTEXT-BUDGET-01', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false,
    client_replays_history: false, window: WINDOW, output_reservation: OUTPUT, exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model);
  const split = this.model.indexOf('/');
  assert.ok(split > 0);
  this.provider = this.model.slice(0, split);
  this.modelId = this.model.slice(split + 1);
  this.providerPath = `/api/uar/providers/${encodeURIComponent(this.provider)}`;
  this.originalProvider = providerConfig(await request(this, 'GET', this.providerPath));
  this.boundedProvider = structuredClone(this.originalProvider);
  const model = this.boundedProvider.models.find(m => m.id === this.modelId);
  assert.ok(model);
  model.context_window = WINDOW;
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(`${this.record}.provider-before.json`, JSON.stringify(this.originalProvider, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.providerAttempted = true;
  await request(this, 'PUT', this.providerPath, this.boundedProvider);
  assert.deepEqual(providerConfig(await request(this, 'GET', this.providerPath)), this.boundedProvider);
  // Changing the model configuration invalidates its existing operation receipt.
  const verification = await request(this, 'POST', `${this.providerPath}/test`, { model: this.modelId, output_token_cap: 64 });
  assert.equal(verification.ok, true);
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `disposable-context-budget-${this.session}`;
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'Disposable context assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Remember factual information supplied by the user for later requests. When asked to acknowledge, answer only READY without restating any facts. Treat unrelated archival filler as irrelevant. For a later question, return only the requested JSON using facts from earlier conversation.', instructions: [] };
  agent.policy.provider = { default: { provider: this.provider, model: this.modelId }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.memory.kb.enabled = false;
  agent.extensions = { ...agent.extensions, 'uar.run_policy': { version: 1,
    context_strategy: STRATEGY, memory_enabled: false,
    skills: { mode: 'none' }, tools: { mode: 'none' },
    mcp_servers: { mode: 'none' }, knowledge_bases: { mode: 'none' }, presentations: { mode: 'none' } } };
  await request(this, 'POST', '/api/agents', agent, 201);
  this.evidence.agent_id = this.agentId;
  this.evidence.session_id = this.session;
  this.evidence.model = this.model;
});

When('I give it a fresh reference and quantities to remember', async function () {
  this.reference = `REF-${randomUUID()}`;
  this.left = randomInt(101, 500);
  this.right = randomInt(501, 900);
  this.first = await prompt(this, [{ role: 'user', content:
    `Remember these facts for later: reference ${this.reference}; first quantity ${this.left}; second quantity ${this.right}. Acknowledge only READY; do not repeat the facts.` }]);
  assert.equal(this.first.final, 'READY');
  this.evidence.initial_run_id = this.first.runId;
});

When('I add oversized unrelated context and ask about the earlier information', async function () {
  const filler = 'Archive note: amber birch cedar dune elm fern grove hazel iris juniper kelp laurel moss nettle oak pine quartz reed spruce thyme. ';
  this.evidence.filler_run_ids = [];
  for (let i = 0; i < 3; i += 1) {
    const content = `Unrelated archival appendix ${String.fromCharCode(65 + i)}. Acknowledge only READY; do not restate earlier facts.\n${filler.repeat(110)}`;
    const turn = await prompt(this, [{ role: 'user', content }]);
    assert.equal(turn.final, 'READY');
    this.evidence.filler_run_ids.push(turn.runId);
  }
  const messages = [{ role: 'user', content: 'Ignore the archival appendices. What reference did I originally give you, and what is the sum of the two quantities? Return only JSON with reference and total.' }];
  const text = JSON.stringify(messages);
  for (const value of [this.reference, String(this.left), String(this.right)]) assert.ok(!text.includes(value), 'Follow-up must not replay original facts');
  this.evidence.expected = { reference: this.reference, total: this.left + this.right };
  this.second = await prompt(this, messages);
  assert.notEqual(this.second.runId, this.first.runId);
  this.evidence.follow_up_run_id = this.second.runId;
});

Then('the model request fits the budget after actual context reduction', async function () {
  const run = await request(this, 'GET', `/api/uar/runs/${this.second.runId}`);
  this.evidence.run_inspection = run;
  assert.equal(run.agent_id, this.agentId);
  assert.equal(run.conversation_id, this.session);
  assert.deepEqual(run.context_reduction.requested_strategy, STRATEGY);
  const reduction = run.context_reduction;
  assert.ok(reduction.source_tokens > reduction.input_allowance, 'History must actually exceed the input budget');
  assert.ok(reduction.input_allowance > 0 && reduction.input_allowance <= WINDOW - OUTPUT);
  assert.ok(reduction.final_tokens <= reduction.input_allowance);
  assert.ok(reduction.final_tokens < reduction.source_tokens);
  assert.ok(reduction.final_messages < reduction.source_messages);
  assert.notEqual(reduction.source_sha256, reduction.final_sha256);
  const receipts = this.second.events.filter(e => e.type === 'CUSTOM' && e.name === 'uar.artifact.available'
    && e.value?.artifactId === 'uar.request:budget_receipt').map(e => JSON.parse(e.value.content).payload)
    .filter(r => r.representation === 'final_provider_request');
  assert.ok(receipts.length > 0, 'Require final serialized request accounting');
  for (const receipt of receipts) {
    assert.equal(receipt.serialized_request, true);
    assert.equal(receipt.output_ceiling_enforced, true);
    assert.ok(receipt.counted_input_tokens > 0 && receipt.counted_input_tokens <= WINDOW - OUTPUT);
  }
  this.evidence.request_budget_receipts = receipts;
  const usage = this.second.terminal.result?.usage;
  assert.ok(Number.isFinite(usage?.inputTokens) && usage.inputTokens > 0);
  assert.ok(usage.inputTokens <= WINDOW - OUTPUT, 'Reported provider input also fits the input budget');
  assert.ok(usage.outputTokens <= OUTPUT);
  this.evidence.provider_usage = usage;
});

Then('the follow-up correctly uses the original information', function () {
  let final = this.second.final;
  const fenced = final.match(/^```(?:json)?\s*\n([\s\S]*?)\n```$/);
  if (fenced) final = fenced[1];
  const answer = JSON.parse(final);
  assert.deepEqual(answer, this.evidence.expected);
  this.evidence.answer = answer;
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.providerAttempted) {
    try {
      const current = providerConfig(await request(this, 'GET', this.providerPath));
      if (JSON.stringify(current) !== JSON.stringify(this.originalProvider)) {
        assert.deepEqual(current, this.boundedProvider, 'Do not overwrite concurrent provider edits');
        await request(this, 'PUT', this.providerPath, this.originalProvider);
      }
      assert.deepEqual(providerConfig(await request(this, 'GET', this.providerPath)), this.originalProvider);
      this.evidence.provider_config_restored = true;
      const verification = await request(this, 'POST', `${this.providerPath}/test`, { model: this.modelId, output_token_cap: 64 });
      assert.equal(verification.ok, true);
      this.evidence.original_model_ready = true;
    } catch (error) { errors.push(`Provider restoration: ${error}`); }
  }
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
  assert.deepEqual(errors, [], 'Restoration and cleanup must succeed');
});

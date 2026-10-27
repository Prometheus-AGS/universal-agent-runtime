import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(180_000);
const OUTPUT_LIMIT = 1024;
const REQUESTED_OUTPUT = 4096;
const LISTING_END = 1500;
const DEFAULT_SETTING = '/api/uar/settings/llm.default_provider';

function providerConfig(view) {
  return Object.fromEntries(['id', 'display_name', 'base_url', 'protocol', 'default_model', 'models', 'enabled']
    .map(key => [key, view[key]]));
}

function route(selector) {
  const split = selector.indexOf('/');
  assert.ok(split > 0 && split < selector.length - 1, `Expected provider/model, got ${selector}`);
  return { provider: selector.slice(0, split), model: selector.slice(split + 1) };
}

async function request(w, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(170_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}

async function verifyModel(w, model) {
  const verification = await request(w, 'POST', `${w.providerPath}/test`, { model, output_token_cap: 128 });
  assert.equal(verification.ok, true);
  assert.equal(verification.model_id, model);
  w.evidence.model_dispatches += 1;
}

// The request body never names a model: a request override would hide the saved configuration.
async function prompt(w, session, content) {
  const payload = { agent_id: w.agentId, session_id: session, stream: true, stream_mode: 'agui_spec',
    memory_enabled: false, max_completion_tokens: REQUESTED_OUTPUT, messages: [{ role: 'user', content }] };
  assert.ok(!('model' in payload));
  const exchange = { method: 'POST', path: '/v1/chat/completions', request: payload, response: '' };
  w.evidence.exchanges.push(exchange);
  w.evidence.model_dispatches += 1;
  const response = await fetch(new URL(exchange.path, w.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(170_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify(payload),
  });
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.equal(response.status, 200, exchange.response.slice(0, 800));
  assert.match(response.headers.get('content-type') ?? '', /^text\/event-stream\b/);
  const events = exchange.response.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const name = lines.find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
    return [JSON.parse(data)];
  });
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

// Actual dispatch evidence: the persisted run route plus settled accounting under the
// model's catalog pricing identity (the gateway alias is the routing identity).
async function assertDispatch(w, run, binding, phase, label) {
  const inspection = await request(w, 'GET', `/api/uar/runs/${encodeURIComponent(run.runId)}`);
  assert.equal(inspection.agent_id, w.agentId);
  assert.deepEqual(inspection.effective_model, { provider_id: binding.provider, model_id: binding.model },
    `${label} ran on the wrong model`);
  const calls = await request(w, 'GET', `/api/uar/accounting/calls/${encodeURIComponent(run.runId)}`);
  const pricing = w.pricing[binding.model];
  const accountingModel = pricing ? `${pricing.provider_id}/${pricing.model_id}` : `${binding.provider}/${binding.model}`;
  assert.ok(Array.isArray(calls) && calls.length > 0, `${label} has no accounted provider call`);
  for (const call of calls) {
    assert.equal(call.state, 'settled', `${label} accounting is not settled`);
    assert.equal(call.admission.model, accountingModel);
    assert.ok(call.observed_tokens > 0);
  }
  const usage = run.terminal.result?.usage;
  (w.evidence.runs[phase] ??= {})[label] = { run_id: run.runId, effective_model: inspection.effective_model,
    status: inspection.status, accounting: calls, usage, final: run.final.slice(0, 600) };
  return usage;
}

async function answerBoth(w, phase) {
  for (const [label, session, binding] of [['default_session', w.defaultSession, w.b], ['override_session', w.overrideSession, w.a]]) {
    const left = randomInt(1000, 5000);
    const right = randomInt(5001, 9000);
    const run = await prompt(w, session, `What is ${left} + ${right}? Reply in the exact required format.`);
    assert.equal(run.final, `${w.marker} ${left + right}`, `${label} returned an incorrect or incomplete answer`);
    await assertDispatch(w, run, binding, phase, label);
  }
}

async function assertOutputLimit(w, phase) {
  const run = await prompt(w, w.defaultSession,
    `List every integer from 1 to ${LISTING_END} in ascending order, separated by single spaces. Do not abbreviate, skip numbers or add commentary.`);
  const usage = await assertDispatch(w, run, w.b, phase, 'output_limit_probe');
  assert.ok(Number.isFinite(usage?.outputTokens), 'Provider must report observed output tokens');
  assert.ok(usage.outputTokens <= OUTPUT_LIMIT, `Observed ${usage.outputTokens} output tokens above the saved limit`);
  // A completed listing needs well over 2,000 tokens; the request allowed 4,096.
  assert.ok(usage.outputTokens >= OUTPUT_LIMIT - 64, `Observed ${usage.outputTokens} tokens; the limit was not reached`);
  assert.ok(!new RegExp(`\\b${LISTING_END}\\s*$`).test(run.final), 'The listing completed despite the saved limit');
  const receipts = run.events.filter(e => e.type === 'CUSTOM' && e.name === 'uar.artifact.available'
    && e.value?.artifactId === 'uar.request:budget_receipt').map(e => JSON.parse(e.value.content).payload)
    .filter(r => r.representation === 'final_provider_request' && r.destination_model === w.b.model);
  assert.ok(receipts.length > 0, 'Require final provider request accounting');
  for (const receipt of receipts) assert.equal(receipt.output_ceiling_enforced, true);
  w.evidence.runs[phase].output_limit_probe.final_provider_receipts = receipts;
}

async function savedState(w) {
  const provider = providerConfig(await request(w, 'GET', w.providerPath));
  const setting = await request(w, 'GET', DEFAULT_SETTING);
  const override = await request(w, 'GET', `/api/uar/sessions/${w.overrideSession}/agent-config`);
  return { provider, default_provider: { saved: setting.saved, effective: setting.effective, revision: setting.revision }, override };
}

async function retain(w, suffix) {
  await mkdir(dirname(w.record), { recursive: true });
  await writeFile(`${w.record}.${suffix}.json`, JSON.stringify(w.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
}

Given('the original provider configuration and two verified model bindings are recorded', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.record = resolve(`.prometheus/verification/rest-model-restart/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.a = route(process.env.UAR_MODEL_A ?? '');
  this.b = route(process.env.UAR_MODEL_B ?? '');
  this.evidence = { scenario: 'C33-REST-MODEL-RESTART-01', platform: process.platform,
    authentication_supplied_by_test: false, credentials_in_requests: false, previous_scenarios_rerun: false,
    request_model_override: false, binding_a: this.a, binding_b: this.b, output_limit: OUTPUT_LIMIT,
    requested_output: REQUESTED_OUTPUT, model_dispatches: 0, runs: {}, exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace);
  assert.equal(this.a.provider, this.b.provider, 'Both bindings must belong to one owned provider');
  assert.notEqual(this.a.model, this.b.model);
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.base);
  this.providerPath = `/api/uar/providers/${encodeURIComponent(this.b.provider)}`;
  const view = await request(this, 'GET', this.providerPath);
  assert.ok(!('api_key' in view), 'Provider view must be redacted');
  this.originalProvider = providerConfig(view);
  this.originallyVerified = Object.keys(view.model_verifications ?? {});
  for (const binding of [this.a, this.b]) {
    const model = this.originalProvider.models.find(m => m.id === binding.model);
    assert.ok(model?.enabled, `${binding.model} must be an enabled configured model`);
    assert.ok(this.originallyVerified.includes(binding.model), `${binding.model} must already be verified`);
  }
  assert.notEqual(this.originalProvider.default_model, this.b.model, 'B must differ from the original default');
  const originalB = this.originalProvider.models.find(m => m.id === this.b.model);
  assert.ok(originalB.max_output_tokens > OUTPUT_LIMIT * 2, 'B must start with a larger output limit');
  this.pricing = Object.fromEntries(this.originalProvider.models.map(m => [m.id, m.pricing_identity]));
  this.originalSetting = await request(this, 'GET', DEFAULT_SETTING);
  this.marker = `C33-${randomUUID().slice(0, 8).toUpperCase()}`;
  this.defaultSession = randomUUID();
  this.overrideSession = randomUUID();
  Object.assign(this.evidence, { original_provider: this.originalProvider, originally_verified: this.originallyVerified,
    original_default_setting: this.originalSetting, marker: this.marker,
    default_session: this.defaultSession, override_session: this.overrideSession });
  await retain(this, 'before-update');
});

When('I save model B as the default with a smaller output limit through the REST API', async function () {
  this.boundedProvider = structuredClone(this.originalProvider);
  this.boundedProvider.default_model = this.b.model;
  this.boundedProvider.models.find(m => m.id === this.b.model).max_output_tokens = OUTPUT_LIMIT;
  this.providerAttempted = true;
  await request(this, 'PUT', this.providerPath, this.boundedProvider);
  await request(this, 'POST', `${this.providerPath}/default`);
  assert.deepEqual(providerConfig(await request(this, 'GET', this.providerPath)), this.boundedProvider);
  // Saving the provider invalidates its operation receipts; refresh only A and B.
  await verifyModel(this, this.b.model);
  await verifyModel(this, this.a.model);
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `disposable-model-config-${this.id}`;
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'Disposable model configuration assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: `Begin every reply with the exact token ${this.marker} followed by one space. For arithmetic, the token is followed only by the number. Otherwise follow the user's instructions exactly.`, instructions: [] };
  // Empty strings inherit the runtime default; a pinned route would hide default precedence.
  agent.policy.provider = { default: { provider: '', model: '' }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.memory.kb.enabled = false;
  agent.extensions = { 'uar.run_policy': { version: 1, memory_enabled: false,
    skills: { mode: 'none' }, tools: { mode: 'none' }, mcp_servers: { mode: 'none' },
    knowledge_bases: { mode: 'none' }, presentations: { mode: 'none' } } };
  await request(this, 'POST', '/api/agents', agent, 201);
  this.evidence.agent_id = this.agentId;
});

When('I save model A as the model override for one fresh session', async function () {
  this.override = { agent_id: this.agentId, model: `${this.a.provider}/${this.a.model}`,
    tools: [], skills: [], knowledge_bases: [], mcp_servers: [] };
  this.overrideAttempted = true;
  const saved = await request(this, 'POST', `/api/uar/sessions/${this.overrideSession}/agent-config`, this.override);
  assert.equal(saved.model, this.override.model);
  this.savedOverride = await request(this, 'GET', `/api/uar/sessions/${this.overrideSession}/agent-config`);
  assert.equal(this.savedOverride.model, this.override.model);
  assert.equal(this.savedOverride.agent_id, this.agentId);
});

Then('a stale default-provider revision and an invalid session model are refused without changing saved state', async function () {
  const before = await savedState(this);
  this.evidence.rejections = {};
  const stale = await request(this, 'PUT', DEFAULT_SETTING,
    { value: this.originalSetting.saved, expected_revision: `stale-${randomUUID()}` }, 409);
  this.evidence.rejections.stale_revision = stale;
  const invalid = await request(this, 'POST', `/api/uar/sessions/${this.overrideSession}/agent-config`,
    { ...this.override, model: this.b.model }, 400);
  this.evidence.rejections.invalid_model = invalid;
  assert.deepEqual(await savedState(this), before, 'A refused write changed saved state');
  this.evidence.saved_before_restart = before;
});

Then('prompts without a requested model run on B for the default session and A for the override session', async function () {
  await answerBoth(this, 'before_restart');
});

Then('model B stops at its saved output limit although the request asks for more', async function () {
  await assertOutputLimit(this, 'before_restart');
  await retain(this, 'before-restart');
});

When('UAR restarts with the existing store', { timeout: 360_000 }, async function () {
  this.evidence.restart = {};
  await restartOwnedUar(this.evidence.restart);
  this.evidence.after_endpoint_binding = await assertOwnedEndpoint(this.base);
  assert.equal(this.evidence.restart.previous_process_exited, true);
  assert.equal(this.evidence.restart.new_process_ready, true);
  assert.notEqual(this.evidence.restart.previous_pid, this.evidence.restart.new_pid);
});

Then('the saved default, output limit and session override are unchanged after restart', async function () {
  const after = await savedState(this);
  this.evidence.saved_after_restart = after;
  assert.deepEqual(after, this.evidence.saved_before_restart);
  const view = await request(this, 'GET', this.providerPath);
  for (const model of [this.a.model, this.b.model]) {
    assert.ok(view.model_verifications?.[model], `${model} verification did not survive restart`);
  }
});

Then('prompts without a requested model again run on B for the default session and A for the override session', async function () {
  await answerBoth(this, 'after_restart');
});

Then('model B again stops at its saved output limit although the request asks for more', async function () {
  await assertOutputLimit(this, 'after_restart');
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 600_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.providerAttempted) {
    try {
      const current = providerConfig(await request(this, 'GET', this.providerPath));
      if (JSON.stringify(current) !== JSON.stringify(this.originalProvider)) {
        assert.deepEqual(current, this.boundedProvider, 'Do not overwrite concurrent provider edits');
        await request(this, 'PUT', this.providerPath, this.originalProvider);
      }
      await request(this, 'POST', `${this.providerPath}/default`);
      assert.deepEqual(providerConfig(await request(this, 'GET', this.providerPath)), this.originalProvider);
      const setting = await request(this, 'GET', DEFAULT_SETTING);
      assert.equal(setting.saved, this.originalSetting.saved);
      this.evidence.provider_config_restored = true;
      // Restoring changes the fingerprint again; refresh only previously verified routes.
      for (const model of this.originallyVerified) await verifyModel(this, model);
      const view = await request(this, 'GET', this.providerPath);
      assert.deepEqual(Object.keys(view.model_verifications ?? {}).sort(), [...this.originallyVerified].sort());
      this.evidence.original_routes_ready = this.originallyVerified;
    } catch (error) { errors.push(`Provider restoration: ${error}`); }
  }
  if (this.overrideAttempted) {
    try {
      await request(this, 'DELETE', `/api/uar/conversations/${this.overrideSession}/policy`, undefined, [200, 204, 404]);
      this.evidence.override_policy_deleted = true;
    } catch (error) { errors.push(`Override cleanup: ${error}`); }
  }
  for (const session of [this.defaultSession, this.overrideSession]) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${session}: ${error}`); }
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

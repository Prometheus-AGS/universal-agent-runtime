import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(180_000);
const PROVIDER = 'liter';
const MODEL = 'MiniMax-M3';
const AGENT = 'default-agent';
const POLL_MS = 500;
const TERMINAL = new Set(['done', 'error', 'cancelled']);
const PROVIDER_PATH = `/api/uar/providers/${PROVIDER}`;

const providerConfig = view => Object.fromEntries(['id', 'display_name', 'base_url', 'protocol', 'default_model', 'models', 'enabled']
  .map(key => [key, view[key]]));

async function request(w, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(170_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 2000) });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

async function proxy(w, method, path) {
  const response = await fetch(new URL(path, w.proxyUrl), { method, signal: AbortSignal.timeout(10_000) });
  assert.equal(response.status, 200);
  return response.json();
}

async function verifyModel(w, model) {
  const verification = await request(w, 'POST', `${PROVIDER_PATH}/test`, { model, output_token_cap: 512 });
  assert.equal(verification.ok, true, `Verification of ${model}: ${JSON.stringify(verification).slice(0, 300)}`);
  w.evidence.verifications.push({ model, ok: verification.ok });
}

const pause = ms => new Promise(done => setTimeout(done, ms));
const runEvents = async (w, runId) => (await request(w, 'GET', `/api/uar/runs/${runId}/events`)).events;
const runText = list => list.filter(e => e.eventName === 'agui.message.delta').map(e => e.data?.delta?.text ?? '').join('');

async function runToTerminal(w, label, input) {
  const session = randomUUID();
  w.sessions.push(session);
  const created = await request(w, 'POST', '/api/uar/runs', { agent_id: AGENT, input, session_id: session }, [200, 201, 202]);
  const runId = created.run_id ?? created.runId;
  assert.ok(runId);
  const deadline = Date.now() + 150_000;
  const statuses = [];
  let status;
  while (Date.now() < deadline) {
    status = (await request(w, 'GET', `/api/uar/runs/${runId}`)).status;
    if (statuses.at(-1) !== status) statuses.push(status);
    if (TERMINAL.has(status)) break;
    await pause(POLL_MS);
  }
  const events = await runEvents(w, runId);
  const run = await request(w, 'GET', `/api/uar/runs/${runId}`);
  const calls = await request(w, 'GET', `/api/uar/accounting/calls/${runId}`);
  const diagnostics = events.filter(e => e.eventName === 'agui.custom' && e.data?.name === 'uar.presentation.diagnostic').map(e => e.data.value);
  const record = { run_id: runId, status, statuses, effective_model: run.effective_model,
    event_names: [...new Set(events.map(e => e.eventName))], final_events: events.slice(-4),
    errors: events.filter(e => /error/i.test(e.eventName)).map(e => e.data), diagnostics,
    calls: (calls ?? []).map(c => ({ state: c.state, model: c.admission?.model, observed_tokens: c.observed_tokens, reconciliation_evidence: c.reconciliation_evidence })),
    text: runText(events).slice(0, 2000) };
  w.evidence.runs[label] = record;
  return record;
}

Given('the configured Liter provider is routed through an owned fault proxy', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.record = resolve(`.prometheus/verification/provider-failure/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.sessions = [];
  this.evidence = { scenario: 'C51-PROVIDER-FAILURE-01', platform: process.platform, provider: PROVIDER, model: MODEL,
    authentication_supplied_by_test: false, credentials_in_requests: false, previous_scenarios_rerun: false,
    verifications: [], runs: {}, exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.base);
  const view = await request(this, 'GET', PROVIDER_PATH);
  this.original = providerConfig(view);
  this.originallyVerified = Object.keys(view.model_verifications ?? {});
  assert.equal(view.credential_configured, true);
  Object.assign(this.evidence, { original_provider: this.original, originally_verified: this.originallyVerified });
  const upstream = new URL(this.original.base_url);
  this.proxyProcess = spawn(process.execPath, [resolve('scripts/behaviors/support/provider-fault-proxy.mjs')], {
    env: { PATH: process.env.PATH, UPSTREAM: upstream.origin }, stdio: ['ignore', 'pipe', 'pipe'] });
  this.proxyUrl = await new Promise((done, fail) => {
    const timer = setTimeout(() => fail(new Error('Fault proxy did not report its URL')), 10_000);
    this.proxyProcess.stdout.on('data', chunk => {
      const match = /PROXY_URL (\S+)/.exec(String(chunk));
      if (match) { clearTimeout(timer); done(match[1]); }
    });
    this.proxyProcess.on('exit', code => fail(new Error(`Fault proxy exited ${code}`)));
  });
  this.evidence.proxy = { url: this.proxyUrl, upstream: upstream.origin, pid: this.proxyProcess.pid };
  this.providerChanged = true;
  await request(this, 'PUT', PROVIDER_PATH, { ...this.original, base_url: `${this.proxyUrl}${upstream.pathname}` });
  const routed = await request(this, 'GET', PROVIDER_PATH);
  assert.equal(routed.base_url, `${this.proxyUrl}${upstream.pathname}`);
  assert.equal(routed.credential_configured, true, 'The stored credential survives the keyless update');
  await verifyModel(this, MODEL);
  const passLog = (await proxy(this, 'GET', '/__fault/log')).log;
  assert.ok(passLog.some(entry => entry.outcome === 'complete'), 'The verification passed through the proxy');
});

When('the provider connection is cut mid-stream during an admitted request', async function () {
  await proxy(this, 'POST', '/__fault/arm');
  this.logBefore = (await proxy(this, 'GET', '/__fault/log')).log.length;
  this.failed = await runToTerminal(this, 'faulted', 'Write a detailed 600-word explanation of how tides work.');
  await proxy(this, 'POST', '/__fault/clear');
  const log = (await proxy(this, 'GET', '/__fault/log')).log.slice(this.logBefore);
  this.evidence.runs.faulted.proxy_log = log;
  assert.ok(log.length > 0 && log.every(entry => entry.mode === 'disconnect_mid_stream'), 'The request went through the armed fault');
  assert.ok(log.some(entry => entry.outcome === 'disconnected'), 'The fault actually cut a provider stream');
});

Then('the failed run never reports success or a fabricated answer', async function () {
  assert.notEqual(this.failed.status, 'done', `The faulted run reported ${this.failed.status}`);
  assert.equal(this.failed.status, 'error');
  assert.ok(this.failed.errors.length > 0, 'The failure is reported as an error event');
  assert.ok(!this.failed.event_names.includes('agui.done') || this.failed.final_events.every(e => e.data?.outcome?.type !== 'success'),
    'No success outcome after the fault');
});

Then('its model call is settled or explicitly uncertain without a model switch', async function () {
  const calls = this.failed.calls;
  assert.ok(calls.length > 0, 'The faulted run admitted a model call');
  const reportedUncertain = this.failed.diagnostics.some(d => d?.code === 'accounting_unresolved');
  for (const call of calls) {
    assert.ok(call.state === 'settled' || (call.state === 'pending' && (call.reconciliation_evidence || reportedUncertain)),
      `Call is ${call.state} without an explicit uncertainty report`);
  }
  assert.equal(this.failed.effective_model?.model_id, MODEL, 'No silent model switch');
  assert.equal(this.failed.effective_model?.provider_id, PROVIDER);
});

When('the fault is removed and I send a fresh request', async function () {
  this.logBefore = (await proxy(this, 'GET', '/__fault/log')).log.length;
  this.fresh = await runToTerminal(this, 'fresh', 'What is 19 plus 23? Reply with only the number.');
  this.evidence.runs.fresh.proxy_log = (await proxy(this, 'GET', '/__fault/log')).log.slice(this.logBefore);
});

Then('the fresh run succeeds on the same configured model', async function () {
  assert.equal(this.fresh.status, 'done');
  const answer = this.fresh.text.replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
  assert.match(answer, /\b42\b/);
  assert.equal(this.fresh.effective_model?.model_id, MODEL);
  assert.ok(this.evidence.runs.fresh.proxy_log.every(entry => entry.mode === 'pass'), 'The fresh request ran with the fault removed');
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 300_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  for (const session of this.sessions) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${session}: ${error}`); }
  }
  if (this.providerChanged) {
    try {
      // An endpoint change bumps each model's execution-profile revision; the
      // restore must carry the current revision or it is refused as a conflict.
      const current = await request(this, 'GET', PROVIDER_PATH);
      const currentModels = new Map(current.models.map(model => [model.id, model]));
      const payload = { ...this.original, models: this.original.models.map(model => (currentModels.get(model.id)?.execution_profile
        ? { ...model, execution_profile: currentModels.get(model.id).execution_profile } : model)) };
      await request(this, 'PUT', PROVIDER_PATH, payload);
      const restored = await request(this, 'GET', PROVIDER_PATH);
      const comparable = config => ({ ...config, models: config.models.map(({ execution_profile, ...model }) => model) });
      assert.deepEqual(comparable(providerConfig(restored)), comparable(this.original));
      for (const model of this.originallyVerified) await verifyModel(this, model);
      this.evidence.restored_provider = providerConfig(restored);
    } catch (error) { errors.push(`Provider restore: ${error}`); }
  }
  if (this.proxyProcess && this.proxyProcess.exitCode === null) this.proxyProcess.kill('SIGTERM');
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});

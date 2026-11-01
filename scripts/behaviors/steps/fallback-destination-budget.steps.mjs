import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { randomUUID } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Before, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { STUB_LLM_MODEL, startDedicatedRuntime } from '../../acceptance/fixtures/dedicated-runtime.mjs';

setDefaultTimeout(300_000);

const BASE_FIXTURES = resolve('scripts/acceptance/fixtures/dedicated-runtime-stub-llm.json');
const FALLBACK_PROVIDER = 'budgetsmall';
const FALLBACK_MODEL = 'budget-small-model';
const FALLBACK_WINDOW = 4096;
const PAIRS = 30;
const QUESTION = 'Fallback budget probe: reply with FALLBACK-OK.';
const ANSWER = 'FALLBACK-OK';
const TERMINAL = new Set(['done', 'error', 'cancelled']);
const POLL_MS = 500;
const [PRIMARY_PROVIDER, PRIMARY_MODEL] = STUB_LLM_MODEL.split('/');

// Every call goes to UAR's loopback address with no Authorization header: the
// caller is the opt-in loopback owner. The test never sends or holds a credential.
async function request(w, method, path, payload, expected = [200]) {
  const res = await fetch(new URL(path, w.runtime.runtime), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json' },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await res.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: res.status, response: text.slice(0, 2000) });
  assert.ok(expected.includes(res.status), `${method} ${path}: ${res.status}: ${text.slice(0, 800)}`);
  if (!text) return null;
  try { return JSON.parse(text); } catch { return { error: text }; }
}

async function control(url, method, path) {
  const res = await fetch(new URL(path, url), { method, signal: AbortSignal.timeout(10_000) });
  assert.equal(res.status, method === 'POST' && path === '/_stub/requests' ? 204 : 200);
  return res.status === 204 ? null : res.json();
}

const pause = ms => new Promise(done => setTimeout(done, ms));

// Fixtures for the fallback model are added to the shared stub fixtures. The wire
// model may be sent bare or provider-qualified, so both fingerprints are served.
async function writeStubFixtures(w) {
  const entries = JSON.parse(await readFile(BASE_FIXTURES, 'utf8'));
  for (const model of [FALLBACK_MODEL, `${FALLBACK_PROVIDER}/${FALLBACK_MODEL}`]) {
    for (const has_tools of [false, true]) {
      entries.push({ model, last_user_message: QUESTION, has_tools, has_tool_result: false,
        response: { kind: 'content', text: ANSWER } });
      entries.push({ model, last_user_message: 'Reply with OK.', has_tools, has_tool_result: false,
        response: { kind: 'content', text: 'OK' } });
    }
  }
  const path = resolve(`target/fallback-destination-budget/${w.id}/stub-llm.json`);
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, JSON.stringify(entries, null, 2) + '\n', { mode: 0o600 });
  return path;
}

// The primary route runs through the owned fault proxy so it can be made
// unavailable after the model has been verified through it.
async function startFaultProxy(w, stubBaseUrl) {
  const upstream = new URL(stubBaseUrl);
  w.proxyProcess = spawn(process.execPath, [resolve('scripts/behaviors/support/provider-fault-proxy.mjs')], {
    env: { PATH: process.env.PATH, UPSTREAM: upstream.origin }, stdio: ['ignore', 'pipe', 'pipe'] });
  w.proxyUrl = await new Promise((done, fail) => {
    const timer = setTimeout(() => fail(new Error('Fault proxy did not report its URL')), 10_000);
    w.proxyProcess.stdout.on('data', chunk => {
      const match = /PROXY_URL (\S+)/.exec(String(chunk));
      if (match) { clearTimeout(timer); done(match[1]); }
    });
    w.proxyProcess.on('exit', code => fail(new Error(`Fault proxy exited ${code}`)));
  });
  return `${w.proxyUrl}${upstream.pathname}`;
}

const historyText = (index, role) => `Record ${index}: ${role} note for depot ${index}. `
  + 'Pallets, crates, drums and spools were counted twice and reconciled against the ledger, '
  + 'then the variance was explained in writing and filed with the harbor office for review.';

const seededHistory = () => Array.from({ length: PAIRS }, (_, index) => [
  { role: 'user', content: historyText(index + 1, 'user') },
  { role: 'assistant', content: historyText(index + 1, 'assistant') },
]).flat();

// The manifest of each provider attempt carries `context_fit` for a fallback
// destination. Search the run record and events without assuming where the host
// surfaces it; artifact payloads hold the manifest as a JSON string.
function findContextFits(value, found = []) {
  if (typeof value === 'string') {
    const text = value.trim();
    if (text.startsWith('{') || text.startsWith('[')) {
      try { findContextFits(JSON.parse(text), found); } catch { /* not JSON */ }
    }
    return found;
  }
  if (Array.isArray(value)) { value.forEach(item => findContextFits(item, found)); return found; }
  if (value && typeof value === 'object') {
    if (value.context_fit) found.push({ destination_model: value.destination_model, ...value.context_fit });
    Object.values(value).forEach(item => findContextFits(item, found));
  }
  return found;
}

const isFallbackRequest = body => [FALLBACK_MODEL, `${FALLBACK_PROVIDER}/${FALLBACK_MODEL}`].includes(body?.model);
const messageText = body => (body.messages ?? []).map(message => (typeof message.content === 'string'
  ? message.content : JSON.stringify(message.content))).join('\n');

Before(function ({ pickle }) {
  this.scenarioName = pickle.name;
});

Given('a dedicated UAR with an unavailable primary and a fallback with a 4096-token window', { timeout: 420_000 }, async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.holder = {};
  this.sessions = [];
  this.record = resolve(`.prometheus/verification/fallback-destination-budget/${this.id}.json`);
  this.evidence = { scenario: this.scenarioName, platform: process.platform,
    authentication_supplied_by_test: false, credentials_in_requests: false,
    fallback_window: FALLBACK_WINDOW, exchanges: [], fixture: {} };
  const fixtures = await writeStubFixtures(this);
  this.runtime = await startDedicatedRuntime(this.evidence.fixture, this.holder, {
    backend: 'postgres', label: 'fallback-destination-budget', stubLlm: true, stubFixtures: fixtures,
    // The primary default route is the fault proxy in front of the stub.
    env: async ({ stubBaseUrl }) => ({ UAR_LLM__BASE_URL: await startFaultProxy(this, stubBaseUrl) }),
    // The only fallback is the stub itself, under a provider whose window is small.
    config: ({ stubBaseUrl }) => ({ memory: { enabled: false }, security: { local_loopback_owner: true },
      failover: { enabled: true, fallback_models: [{ model: `${FALLBACK_PROVIDER}/${FALLBACK_MODEL}`, base_url: stubBaseUrl }] } }),
  });
  this.stubUrl = new URL(this.evidence.fixture.stub_llm.base_url);
  // A fresh runtime admits a model only after it is verified, as an operator does in settings.
  const primary = await request(this, 'POST', `/api/uar/providers/${PRIMARY_PROVIDER}/test`,
    { model: PRIMARY_MODEL, output_token_cap: 128 });
  assert.equal(primary.ok, true, `Primary verification: ${JSON.stringify(primary).slice(0, 300)}`);
  await request(this, 'POST', '/api/uar/providers', { id: FALLBACK_PROVIDER, display_name: 'Small-window fallback',
    base_url: this.evidence.fixture.stub_llm.base_url, protocol: 'chat', default_model: FALLBACK_MODEL, enabled: true,
    models: [{ id: FALLBACK_MODEL, display_name: FALLBACK_MODEL, context_window: FALLBACK_WINDOW, supports_tools: true, enabled: true }] },
  [200, 201]);
  this.fallbackProviderCreated = true;
  const saved = await request(this, 'GET', `/api/uar/providers/${FALLBACK_PROVIDER}`);
  assert.equal(saved.models[0].context_window, FALLBACK_WINDOW);
  const fallback = await request(this, 'POST', `/api/uar/providers/${FALLBACK_PROVIDER}/test`,
    { model: FALLBACK_MODEL, output_token_cap: 128 });
  assert.equal(fallback.ok, true, `Fallback verification: ${JSON.stringify(fallback).slice(0, 300)}`);
  // Only now does the primary go down; verification already passed through it.
  await control(this.proxyUrl, 'POST', '/__fault/arm?mode=unavailable');
});

async function createAgent(w, strategy) {
  const agent = await request(w, 'GET', '/api/agents/default-agent');
  w.agentId = `disposable-fallback-budget-${w.id.slice(0, 8)}`;
  agent.id = w.agentId;
  agent.metadata = { ...agent.metadata, title: 'Disposable fallback budget agent', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: `Answer the final request. Treat earlier depot records as background. ${QUESTION}`, instructions: [] };
  agent.policy.provider = { default: { provider: PRIMARY_PROVIDER, model: PRIMARY_MODEL }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.policy.tools.allow = [];
  agent.memory.kb.enabled = false;
  agent.extensions = { ...agent.extensions, 'uar.run_policy': { version: 1,
    context_strategy: strategy, memory_enabled: false,
    skills: { mode: 'none' }, tools: { mode: 'none' },
    mcp_servers: { mode: 'none' }, knowledge_bases: { mode: 'none' }, presentations: { mode: 'none' } } };
  await request(w, 'POST', '/api/agents', agent, [201]);
  w.agentCreated = true;
  w.evidence.agent = { id: w.agentId, context_strategy: strategy };
}

Given('a disposable agent using the keep-first-and-latest strategy', async function () {
  await createAgent(this, { type: 'truncate_middle', keep_first: 1, keep_last: 1000 });
});

Given('a disposable agent that never displaces history', async function () {
  await createAgent(this, { type: 'none' });
});

When('I continue a seeded 60-message conversation that exceeds the fallback window', async function () {
  const messages = seededHistory();
  assert.equal(messages.length, PAIRS * 2);
  this.seededChars = messages.reduce((sum, message) => sum + message.content.length, 0);
  // Only requests made for this run are inspected; verification probes are cleared.
  await control(this.stubUrl, 'POST', '/_stub/requests');
  const session = randomUUID();
  this.sessions.push(session);
  const created = await request(this, 'POST', '/api/uar/runs', { agent_id: this.agentId, session_id: session,
    input: QUESTION, history: { session_id: session, messages } }, [200, 201, 202]);
  this.runId = created.run_id ?? created.runId;
  assert.ok(this.runId);
  assert.equal(created.seeded_messages, PAIRS * 2, 'The full history was seeded');
  const deadline = Date.now() + 200_000;
  while (Date.now() < deadline) {
    this.status = (await request(this, 'GET', `/api/uar/runs/${this.runId}`)).status;
    if (TERMINAL.has(this.status)) break;
    await pause(POLL_MS);
  }
  const run = await request(this, 'GET', `/api/uar/runs/${this.runId}`);
  this.events = (await request(this, 'GET', `/api/uar/runs/${this.runId}/events`)).events;
  this.stubRequests = (await control(this.stubUrl, 'GET', '/_stub/requests')).requests;
  this.proxyLog = (await control(new URL(this.proxyUrl), 'GET', '/__fault/log')).log;
  this.fits = findContextFits([run, this.events]);
  this.errors = this.events.filter(event => /error/i.test(event.eventName)).map(event => event.data);
  this.text = this.events.filter(event => event.eventName === 'agui.message.delta')
    .map(event => event.data?.delta?.text ?? '').join('');
  Object.assign(this.evidence, { run_id: this.runId, status: this.status, seeded_chars: this.seededChars,
    context_fits: this.fits, errors: this.errors, answer: this.text.slice(0, 400),
    primary_attempts: this.proxyLog.filter(entry => entry.mode === 'unavailable').length,
    fallback_requests: this.stubRequests.filter(isFallbackRequest).length });
  assert.ok(this.proxyLog.some(entry => entry.mode === 'unavailable' && entry.outcome === 'refused'),
    'The primary route refused at least one request');
});

Then('the run is answered by the fallback', function () {
  assert.equal(this.status, 'done', `Run ended ${this.status}: ${JSON.stringify(this.errors).slice(0, 400)}`);
  assert.match(this.text, new RegExp(ANSWER));
  assert.ok(this.stubRequests.filter(isFallbackRequest).length > 0, 'The fallback model received the request');
});

Then('the fallback request was displaced to fit its own window', function () {
  const sent = this.stubRequests.filter(isFallbackRequest).at(-1);
  const content = messageText(sent);
  // Optional middle history left the request; the pinned first record, the latest
  // record and the live question stayed.
  assert.ok(content.includes('Record 1:'), 'The first record is retained');
  assert.ok(content.includes(`Record ${PAIRS}:`), 'The latest record is retained');
  assert.ok(content.includes(QUESTION), 'The live question is retained');
  // truncate_middle keeps the first message and the most recent turns that fit,
  // so the oldest optional record after the first leaves first.
  assert.ok(!content.includes('Record 2:'), 'The oldest optional history was displaced');
  // The request also carries the system prompt, so compare seeded records, not length.
  const sentRecords = new Set(content.match(/Record \d+:/g) ?? []);
  assert.ok(sentRecords.size < PAIRS, `The fallback request holds ${sentRecords.size} of ${PAIRS} seeded records`);
  // The runtime's own budget report for the fallback attempt.
  const fit = this.fits.find(entry => entry.context_window === FALLBACK_WINDOW);
  assert.ok(fit, `A context_fit report for the fallback attempt: ${JSON.stringify(this.fits).slice(0, 400)}`);
  assert.equal(fit.displaced, true);
  assert.ok(fit.messages_removed > 0);
  assert.ok(fit.required_input_tokens <= fit.input_allowance, 'The displaced request fits its input allowance');
  assert.ok(fit.input_allowance < FALLBACK_WINDOW, 'Output and tool reserves are withheld from the window');
  Object.assign(this.evidence, { fallback_request_chars: content.length, context_fit: fit, result: 'VERIFIED' });
});

Then('the run ends with a typed context overflow refusal', function () {
  assert.equal(this.status, 'error', `Run ended ${this.status}`);
  assert.ok(JSON.stringify(this.errors).includes('context_overflow'),
    `Expected a typed context_overflow refusal, got ${JSON.stringify(this.errors).slice(0, 400)}`);
  assert.ok(!this.events.some(event => event.eventName === 'agui.message.delta'), 'No answer text was produced');
  assert.ok(!this.events.some(event => event.eventName === 'agui.done' && event.data?.outcome?.type === 'success'));
});

Then('the fallback provider received no request', function () {
  assert.equal(this.stubRequests.filter(isFallbackRequest).length, 0, 'The refusal precedes any fallback dispatch');
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 120_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.runtime?.uarAlive()) {
    for (const session of this.sessions) {
      try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [200, 204, 404]); }
      catch (error) { errors.push(`session cleanup ${session}: ${error}`); }
    }
    for (const path of [this.agentCreated && `/api/agents/${this.agentId}`,
      this.fallbackProviderCreated && `/api/uar/providers/${FALLBACK_PROVIDER}`].filter(Boolean)) {
      try { await request(this, 'DELETE', path, undefined, [200, 204, 404]); }
      catch (error) { errors.push(`${path}: ${error}`); }
    }
  }
  if (this.proxyProcess && this.proxyProcess.exitCode === null) this.proxyProcess.kill('SIGTERM');
  if (this.holder?.handle) errors.push(...await this.holder.handle.stop());
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Disposable cleanup must succeed');
});

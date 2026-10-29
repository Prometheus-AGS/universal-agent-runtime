import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson } from '../support/answer-oracle.mjs';

setDefaultTimeout(180_000);
const SETTLE_TIMEOUT_MS = 120_000;
const POLL_INTERVAL_MS = 1_000;

async function http(w, method, path, payload, expected = [200]) {
  const exchange = { method, path, request: payload };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(130_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.ok(expected.includes(response.status), `${method} ${path}: HTTP ${response.status}: ${exchange.response.slice(0, 800)}`);
  return exchange.response;
}

async function rpc(w, method, params) {
  const id = randomUUID();
  const response = JSON.parse(await http(w, 'POST', '/acp/', { jsonrpc: '2.0', id, method, params }));
  assert.equal(response.jsonrpc, '2.0');
  assert.equal(response.id, id);
  assert.ok(!Object.hasOwn(response, 'error'), `${method}: ${JSON.stringify(response.error)}`);
  return response.result;
}

async function disposableAgent(w) {
  const agent = JSON.parse(await http(w, 'GET', '/api/agents/default-agent'));
  agent.id = w.agentId;
  agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: 'Disposable ACP agent', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: `Return only a JSON object with keys marker and answer. Always set marker to "${w.marker}". Put the requested numeric result in answer.`, instructions: [] };
  const split = w.model.indexOf('/');
  agent.policy.provider = { default: { provider: w.model.slice(0, split), model: w.model.slice(split + 1) }, fallbacks: [] };
  // An empty allow list means Auto (every eligible tool); one unmatched placeholder selects none.
  agent.policy.tools.allow = [`acp-no-tools-${w.id.slice(0, 8)}`];
  agent.memory.kb.enabled = false;
  await http(w, 'POST', '/api/agents', agent, [201]);
}

Given('an ACP session targets the {word} agent', async function (kind) {
  this.started = performance.now();
  this.kind = kind;
  this.id = randomUUID();
  this.model = process.env.UAR_MODEL;
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.base = new URL(process.env.UAR_BASE_URL);
  assert.ok(this.model && this.workspace, 'Set UAR_WORKSPACE_ID and UAR_MODEL');
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  this.record = resolve(`.prometheus/verification/acp-json/${this.id}.json`);
  this.marker = kind === 'custom' ? `ACPJSON-${randomUUID()}` : undefined;
  this.agentId = kind === 'custom' ? `disposable-acp-json-${this.id}` : 'default-agent';
  this.evidence = { scenario: `I08-ACP-JSON-${kind.toUpperCase()}`, client_platform: process.platform,
    agent_id: this.agentId, model: this.model, authentication_supplied_by_test: false,
    protocol: 'ACP JSON-RPC POST /acp/ sessions/create, runs/create (no stream), runs/get', exchanges: [] };
  if (kind === 'custom') await disposableAgent(this);
  const session = await rpc(this, 'sessions/create', { agent_id: this.agentId });
  assert.ok(typeof session.session_id === 'string' && session.session_id.length > 0);
  this.session = session.session_id;
  this.evidence.session_id = this.session;
});

When('the client sends a fresh calculation with a non-streamed ACP runs\\/create', async function () {
  const left = randomInt(21, 80), right = randomInt(11, 40), extra = randomInt(101, 500);
  this.expected = left * right + extra;
  this.evidence.expected = this.expected;
  const suffix = this.kind === 'custom' ? '' : ' Reply with only a JSON object {"answer": <number>}.';
  this.createResult = await rpc(this, 'runs/create', { session_id: this.session,
    input: `What is (${left} multiplied by ${right}) plus ${extra}?${suffix}` });
  this.runId = this.createResult.run_id;
  assert.ok(typeof this.runId === 'string' && this.runId.length > 0, 'runs/create must return a run_id');
  this.evidence.run_id = this.runId;
  this.evidence.create_status = this.createResult.status;
});

// The answer may arrive in runs/create itself or in a later runs/get; either is an ACP response.
function deliveredAnswer(result) {
  for (const key of ['output', 'answer', 'content', 'text']) {
    if (typeof result?.[key] === 'string' && result[key].trim()) return result[key];
  }
  return undefined;
}

Then("an ACP response delivers the completed run's correct answer to the client", async function () {
  let text = deliveredAnswer(this.createResult);
  let run = this.createResult;
  const deadline = Date.now() + SETTLE_TIMEOUT_MS;
  while (!text && !['Done', 'Error', 'Cancelled'].includes(run.status) && Date.now() < deadline) {
    await new Promise(done => setTimeout(done, POLL_INTERVAL_MS));
    run = await rpc(this, 'runs/get', { run_id: this.runId });
    text = deliveredAnswer(run);
  }
  this.evidence.final_run = run;
  assert.equal(run.run_id ?? this.runId, this.runId);
  if (run.agent_id !== undefined) assert.equal(run.agent_id, this.agentId);
  assert.ok(!['Error', 'Cancelled'].includes(run.status), `Run ended ${run.status}`);
  assert.ok(text, `No ACP response carried the answer; last runs/get fields: ${Object.keys(run).join(', ')}`);
  const answer = parseAnswerJson(text);
  this.evidence.answer = answer;
  assert.equal(Number(answer.answer), this.expected);
  if (this.marker) assert.equal(answer.marker, this.marker);
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  let cleanupError;
  if (this.session) {
    try { await rpc(this, 'sessions/delete', { session_id: this.session }); }
    catch (error) { cleanupError = error; }
  }
  if (this.kind === 'custom' && this.created) {
    try { await http(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204]); }
    catch (error) { cleanupError ??= error; }
  }
  this.evidence.scenario_status = cleanupError ? 'FAILED' : result.status;
  if (cleanupError) this.evidence.cleanup_error = cleanupError.message;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.scenario_status }), 'application/json');
  if (cleanupError) throw cleanupError;
});

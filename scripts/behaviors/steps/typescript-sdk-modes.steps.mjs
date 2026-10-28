import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson } from '../support/answer-oracle.mjs';

setDefaultTimeout(300_000);
const NODE = process.execPath;
const CLIENT = resolve('scripts/behaviors/support/typescript-sdk-turns.mjs');

async function http(w, method, path, payload, expected = [200]) {
  const exchange = { method, path, request: payload };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.ok(expected.includes(response.status), `${method} ${path}: HTTP ${response.status}: ${exchange.response.slice(0, 800)}`);
  return exchange.response;
}

// Spawn the SDK client with a minimal environment: no inherited credentials.
function runSdk(inputs) {
  return new Promise((done, fail) => {
    const child = spawn(NODE, [CLIENT], { env: { PATH: process.env.PATH, HOME: process.env.HOME },
      stdio: ['pipe', 'pipe', 'pipe'], timeout: 280_000 });
    let out = '', err = '';
    child.stdout.on('data', d => { out += d; });
    child.stderr.on('data', d => { err += d; });
    child.on('error', fail);
    child.on('close', code => code === 0 ? done(JSON.parse(out))
      : fail(new Error(`TypeScript SDK client exited ${code}: ${err.slice(-1500)}`)));
    child.stdin.end(JSON.stringify(inputs));
  });
}

Given('a disposable saved agent with a private instruction marker for the TypeScript SDK', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.agentId = `disposable-typescript-sdk-${this.id}`;
  this.session = randomUUID();
  this.model = process.env.UAR_MODEL;
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.base = new URL(process.env.UAR_BASE_URL);
  assert.ok(this.model && this.workspace);
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  this.record = resolve(`.prometheus/verification/typescript-sdk-modes/${this.id}.json`);
  this.marker = `TSSDK-${randomUUID()}`;
  this.evidence = { scenario: 'I11-I12-TYPESCRIPT-SDK', client_platform: process.platform, agent_id: this.agentId,
    session_id: this.session, model: this.model, authentication_supplied_by_test: false,
    client: 'sdks/typescript dist UarClient (built 2026-10-28 from src)', follow_up_contract: 'server session continuity via x-uar-session-id', exchanges: [] };
  const agent = JSON.parse(await http(this, 'GET', '/api/agents/default-agent'));
  agent.id = this.agentId;
  agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: 'Disposable TypeScript SDK agent', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: `Return only a JSON object with keys marker and answer. Always set marker to "${this.marker}". Put the requested numeric result in answer.`, instructions: [] };
  const split = this.model.indexOf('/');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  // An empty allow list means Auto (every eligible tool); one unmatched placeholder selects none.
  agent.policy.tools.allow = [`tssdk-no-tools-${this.id.slice(0, 8)}`];
  agent.memory.kb.enabled = false;
  await http(this, 'POST', '/api/agents', agent, [201]);
  const left = randomInt(21, 80), right = randomInt(11, 40), extra = randomInt(101, 500);
  const more = randomInt(11, 90);
  this.turns = [`What is (${left} multiplied by ${right}) plus ${extra}?`, `Add ${more} to your previous answer.`];
  this.expected = [left * right + extra, left * right + extra + more];
  this.evidence.expected = this.expected;
});

async function sdkTurns(w, mode) {
  w.mode = mode;
  w.receipt = await runSdk({ base_url: w.base.href.replace(/\/$/, ''), workspace: w.workspace, mode, model: w.model,
    agent_id: w.agentId, session_id: w.session, turns: w.turns, timeout: 120 });
  w.evidence.sdk_receipt = w.receipt;
}

When('the shipped TypeScript SDK sends a fresh calculation and then a dependent follow-up with chat.complete', async function () {
  await sdkTurns(this, 'json');
});

When('the shipped TypeScript SDK sends a fresh calculation and then a dependent follow-up with chat.stream', async function () {
  await sdkTurns(this, 'stream');
});

function checkAnswer(w, content, index) {
  const answer = parseAnswerJson(content);
  assert.equal(answer.marker, w.marker);
  assert.equal(Number(answer.answer), w.expected[index]);
  return answer;
}

async function sessionRunIds(w, ids) {
  const runs = JSON.parse(await http(w, 'GET', '/api/uar/runs?limit=100')).filter(run => run.conversation_id === w.session);
  w.evidence.runs = runs.map(r => ({ run_id: r.run_id, agent_id: r.agent_id, status: r.status }));
  assert.equal(runs.length, 2, 'One run per SDK call');
  for (const run of runs) assert.equal(run.agent_id, w.agentId);
  assert.deepEqual(new Set(runs.map(r => r.run_id)), new Set(ids.map(id => id.replace(/^chatcmpl-/, ''))));
}

Then("both TypeScript SDK completions stop with the agent's marker and correct answers in one session", async function () {
  const { turns } = this.receipt;
  assert.equal(turns.length, 2);
  for (const [index, turn] of turns.entries()) {
    assert.equal(turn.choices, 1);
    assert.equal(turn.finish_reason, 'stop');
    assert.match(turn.id, /^chatcmpl-/);
    checkAnswer(this, turn.content, index);
  }
  await sessionRunIds(this, turns.map(t => t.id));
  this.evidence.result = 'VERIFIED';
});

Then('both TypeScript SDK streams deliver incremental text to one stop with correct answers in one session', async function () {
  const { turns } = this.receipt;
  assert.equal(turns.length, 2);
  for (const [index, turn] of turns.entries()) {
    assert.equal(turn.ids.length, 1, 'All chunks belong to one completion');
    assert.ok(turn.content_chunks >= 1, 'Content chunks are required');
    assert.deepEqual(turn.finish_reasons, ['stop'], 'Exactly one successful terminal chunk');
    checkAnswer(this, turn.content, index);
  }
  await sessionRunIds(this, turns.map(t => t.ids[0]));
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  let cleanupError;
  try { await http(this, 'DELETE', `/api/uar/sessions/${this.session}`, undefined, [204, 404]); }
  catch (error) { cleanupError = error; }
  if (this.created) {
    try { await http(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204]); }
    catch (error) { cleanupError ??= error; }
  }
  this.evidence.mode = this.mode;
  this.evidence.scenario_status = cleanupError ? 'FAILED' : result.status;
  if (cleanupError) this.evidence.cleanup_error = cleanupError.message;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.scenario_status }), 'application/json');
  if (cleanupError) throw cleanupError;
});

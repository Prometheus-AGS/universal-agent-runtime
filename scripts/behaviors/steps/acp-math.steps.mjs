import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(125_000);

async function request(world, method, path, payload) {
  const response = await fetch(new URL(path, world.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(15_000),
    headers: { 'Content-Type': 'application/json', 
      'x-uar-workspace-id': world.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  world.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text });
  assert.equal(response.status, 200, `${method} ${path}: HTTP ${response.status}: ${text.slice(0, 1000)}`);
  return JSON.parse(text);
}

async function rpc(world, method, params) {
  const id = randomUUID();
  const response = await request(world, 'POST', '/acp/', { jsonrpc: '2.0', id, method, params });
  assert.equal(response.jsonrpc, '2.0');
  assert.equal(response.id, id);
  assert.ok(!Object.hasOwn(response, 'error'), JSON.stringify(response.error));
  return response.result;
}

Given('an ACP session targets the saved math agent', async function () {
  this.started = performance.now();
  this.agentId = 'math-specialist-db64c2a8';
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.record = resolve(`.prometheus/verification/acp/${randomUUID()}.json`);
  this.evidence = { scenario: 'ACP-01', client_platform: process.platform,
    agent_id: this.agentId, protocol: 'ACP JSON-RPC sessions/create and runs/create stream',
    authentication_supplied_by_test: false, service_kept_running: true, exchanges: [], events: [] };
  assert.ok(this.workspace && process.env.UAR_MODEL, 'Set UAR_WORKSPACE_ID and UAR_MODEL');
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol));
  assert.ok(!this.base.username && !this.base.password, 'Use an endpoint without URL credentials');
  const agent = await request(this, 'GET', `/api/agents/${this.agentId}`);
  assert.equal(agent.id, this.agentId);
  const route = agent.policy.provider.default;
  this.evidence.configured_model = `${route.provider}/${route.model}`;
  assert.equal(this.evidence.configured_model, process.env.UAR_MODEL);
  this.marker = agent.prompt.system.match(/MATH-[a-f0-9-]+/)?.[0];
  assert.ok(this.marker, 'Saved mathematics identity is missing');
  const session = await rpc(this, 'sessions/create', { agent_id: this.agentId });
  assert.equal(session.agent_id, this.agentId);
  assert.ok(typeof session.session_id === 'string' && session.session_id.length > 0);
  this.evidence.session_id = session.session_id;
});

When('I send a fresh arithmetic question through the ACP stream', async function () {
  const left = randomInt(21, 80);
  const right = randomInt(11, 40);
  const extra = randomInt(101, 500);
  this.evidence.expected = { specialist: this.marker, answer: left * right + extra };
  const payload = { jsonrpc: '2.0', id: randomUUID(), method: 'runs/create',
    params: { session_id: this.evidence.session_id, stream: true,
      input: `What is (${left} multiplied by ${right}) plus ${extra}? Follow your saved specialist/answer JSON instructions.` } };
  const exchange = { method: 'POST', path: '/acp/stream', request: payload, response: '' };
  this.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(exchange.path, this.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream',
      'x-uar-workspace-id': this.workspace }, body: JSON.stringify(payload),
  });
  exchange.status = response.status;
  exchange.content_type = response.headers.get('content-type');
  if (response.status !== 200 || !/^text\/event-stream\b/.test(exchange.content_type ?? '')) {
    exchange.response = await response.text();
    assert.fail(`Expected ACP SSE HTTP200: ${JSON.stringify(exchange)}`);
  }
  const decoder = new TextDecoder();
  let buffer = '';
  let done = false;
  for await (const bytes of response.body) {
    const text = decoder.decode(bytes, { stream: true });
    exchange.response += text;
    buffer += text;
    let separator;
    while ((separator = /\r?\n\r?\n/.exec(buffer))) {
      const block = buffer.slice(0, separator.index);
      buffer = buffer.slice(separator.index + separator[0].length);
      const data = block.split(/\r?\n/).filter(line => line.startsWith('data:'))
        .map(line => line.slice(5).replace(/^ /, '')).join('\n');
      if (!data) continue;
      const event = JSON.parse(data);
      this.evidence.events.push(event);
      assert.equal(done, false, 'Unexpected event after completion');
      assert.ok(!['Error', 'Cancelled', 'ClientToolsRequested'].includes(event.type), JSON.stringify(event));
      if (['RunDone', 'RunDoneWithUsage'].includes(event.type)) done = true;
    }
    // ACP retains its broadcast channel; stop consuming at its successful terminal event.
    if (done) break;
  }
  assert.ok(done, 'Missing ACP RunDone event');
  assert.equal(buffer.trim(), '', 'Incomplete trailing event');
});

Then('the completed ACP run contains the saved identity and correct answer', async function () {
  const events = this.evidence.events;
  const terminal = events.filter(event => ['RunDone', 'RunDoneWithUsage'].includes(event.type));
  assert.equal(terminal.length, 1);
  const runId = terminal[0].data?.run_id;
  assert.ok(typeof runId === 'string' && runId.length > 0);
  for (const event of events) {
    if (event.data?.run_id !== undefined) assert.equal(event.data.run_id, runId);
  }
  const run = await rpc(this, 'runs/get', { run_id: runId });
  assert.equal(run.run_id, runId);
  assert.equal(run.agent_id, this.agentId);
  assert.equal(run.status, 'Done');
  const deltas = events.filter(event => event.type === 'ChatDelta');
  assert.ok(deltas.length > 0, 'Missing answer content');
  assert.ok(deltas.every(event => typeof event.data?.text_delta === 'string'));
  const content = deltas.map(event => event.data.text_delta).join('');
  const final = content.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim();
  assert.ok(final && !final.includes('<think>'), 'A completed final answer is required');
  let answer;
  try { answer = JSON.parse(final); } catch {
    const fences = [...final.matchAll(/(?:^|\n)```(?:json)?[ \t]*\r?\n([\s\S]*?)\r?\n```(?=\s|$)/g)];
    assert.equal(fences.length, 1, 'Expected one unambiguous final JSON answer');
    answer = JSON.parse(fences[0][1]);
  }
  assert.deepEqual(answer, this.evidence.expected);
  this.evidence.answer = answer;
  this.evidence.run_id = runId;
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  this.evidence.scenario_status = result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify(this.evidence), 'application/json');
});

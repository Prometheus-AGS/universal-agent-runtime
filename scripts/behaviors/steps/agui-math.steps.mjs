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

Given('the saved math agent is available for AG-UI', async function () {
  this.started = performance.now();
  this.agentId = 'math-specialist-db64c2a8';
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.record = resolve(`.prometheus/verification/agui/${randomUUID()}.json`);
  this.evidence = { scenario: 'AGUI-01', client_platform: process.platform,
    agent_id: this.agentId, protocol: 'AG-UI 1.0 saved-agent SSE',
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

});

When('I send it a fresh arithmetic question through AG-UI', async function () {
  const left = randomInt(21, 80);
  const right = randomInt(11, 40);
  const extra = randomInt(101, 500);
  this.evidence.expected = { specialist: this.marker, answer: left * right + extra };
  this.evidence.thread_id = randomUUID();
  this.evidence.run_id = randomUUID();
  const payload = { threadId: this.evidence.thread_id, runId: this.evidence.run_id,
    protocolVersion: '1.0', tools: [], context: [],
    messages: [{ id: randomUUID(), role: 'user',
      content: `What is (${left} multiplied by ${right}) plus ${extra}? Follow your saved specialist/answer JSON instructions.` }],
    forwardedProps: { uar: { model: process.env.UAR_MODEL } } };
  const exchange = { method: 'POST', path: `/api/uar/ag-ui/agents/${this.agentId}`,
    request: payload, response: '' };
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
    assert.fail(`Expected AG-UI SSE HTTP200: ${JSON.stringify(exchange)}`);
  }
  const decoder = new TextDecoder();
  let buffer = '';
  let done = false;
  const drain = () => {
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
      assert.ok(!['RUN_ERROR', 'RUN_CANCELLED'].includes(event.type), JSON.stringify(event));
      if (event.type === 'RUN_FINISHED') done = true;
    }
  };
  for await (const bytes of response.body) {
    const text = decoder.decode(bytes, { stream: true });
    exchange.response += text;
    buffer += text;
    drain();
  }
  const tail = decoder.decode();
  exchange.response += tail;
  buffer += tail;
  drain();
  assert.equal(buffer.trim(), '', 'Incomplete trailing event');
  assert.ok(done, 'Missing AG-UI RUN_FINISHED event');
});

Then('its correlated AG-UI stream contains the saved identity and correct answer', function () {
  const events = this.evidence.events;
  const one = type => {
    const matches = events.filter(event => event.type === type);
    assert.equal(matches.length, 1, `Expected one ${type}`);
    return matches[0];
  };
  const started = one('RUN_STARTED');
  const finished = one('RUN_FINISHED');
  assert.equal(started.protocolVersion, '1.0');
  assert.equal(finished.outcome?.type, 'success');
  for (const event of [started, finished]) {
    assert.equal(event.runId, this.evidence.run_id);
    assert.equal(event.threadId, this.evidence.thread_id);
  }
  const messageStart = one('TEXT_MESSAGE_START');
  const messageEnd = one('TEXT_MESSAGE_END');
  assert.equal(messageStart.role, 'assistant');
  assert.ok(typeof messageStart.messageId === 'string' && messageStart.messageId.length > 0);
  assert.equal(messageEnd.messageId, messageStart.messageId);
  const deltas = events.filter(event => event.type === 'TEXT_MESSAGE_CONTENT');
  assert.ok(deltas.length >= 2, 'Expected multiple streamed answer fragments');
  assert.ok(events.indexOf(started) < events.indexOf(messageStart));
  assert.ok(events.indexOf(messageEnd) < events.indexOf(finished));
  for (const event of deltas) {
    assert.equal(event.messageId, messageStart.messageId);
    assert.ok(typeof event.delta === 'string' && event.delta.length > 0);
    assert.ok(events.indexOf(messageStart) < events.indexOf(event)
      && events.indexOf(event) < events.indexOf(messageEnd));
  }
  const content = deltas.map(event => event.delta).join('');
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
  this.evidence.content_fragments = deltas.length;
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

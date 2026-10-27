import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(125_000);

async function request(world, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, world.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(15_000),
    headers: { 'Content-Type': 'application/json',
      'x-uar-workspace-id': world.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) world.created = true;
  const text = await response.text();
  world.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text });
  assert.equal(response.status, expected, `${method} ${path}: HTTP ${response.status}: ${text.slice(0, 1000)}`);
  return text ? JSON.parse(text) : null;
}

Given('a disposable agent can publish A2UI surfaces', async function () {
  this.started = performance.now();
  this.agentId = `disposable-a2ui-${randomUUID()}`;
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.record = resolve(`.prometheus/verification/a2ui/${this.agentId}.json`);
  this.evidence = { scenario: 'A2UI-01', client_platform: process.platform,
    agent_id: this.agentId, protocol: 'AG-UI agui_spec with published A2UI v0.9',
    authentication_supplied_by_test: false, service_kept_running: true, exchanges: [], events: [], runtime_events: [] };
  assert.ok(this.workspace && this.model, 'Set UAR_WORKSPACE_ID and UAR_MODEL');
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol));
  assert.ok(!this.base.username && !this.base.password, 'Use an endpoint without URL credentials');
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  agent.id = this.agentId;
  agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: 'Disposable A2UI greeting agent',
    description: 'One manual A2UI surface response scenario.', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Use the real a2ui_render tool exactly once to publish the requested greeting. '
    + 'Send canonical A2UI v0.9 createSurface then updateComponents messages using the '
    + 'https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json catalog. '
    + 'Use exactly one component: id root, component Text, text equal to the requested greeting. '
    + 'Use the requested surfaceId in both messages. After the tool succeeds, give a brief summary. '
    + 'Do not return UI JSON in prose instead of calling the tool. Do not claim a browser displayed it.', instructions: [] };
  const split = this.model.indexOf('/');
  assert.ok(split > 0 && split < this.model.length - 1);
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.tools.allow = ['a2ui_render'];
  agent.policy.skills.prefer = [];
  agent.memory.kb.enabled = false;
  const saved = await request(this, 'POST', '/api/agents', agent, 201);
  assert.equal(saved.id, this.agentId);
  this.evidence.configured_model = this.model;
});

When('I ask for a fresh greeting surface through AG-UI', async function () {
  this.evidence.surface_id = `greeting-${randomUUID()}`;
  this.evidence.greeting = `Welcome, visitor ${randomUUID().slice(0, 8)}!`;
  this.evidence.session_id = randomUUID();
  const payload = { agent_id: this.agentId, model: this.model, session_id: this.evidence.session_id,
    stream: true, stream_mode: 'agui_spec', memory_enabled: false, max_completion_tokens: 64000,
    messages: [{ role: 'user', content: `Publish an A2UI greeting surface with surfaceId ${this.evidence.surface_id}. The exact greeting is: ${this.evidence.greeting}` }] };
  const exchange = { method: 'POST', path: '/v1/chat/completions',
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
      const name = block.split(/\r?\n/).find(line => line.startsWith('event:'))?.slice(6).trim();
      // The chat endpoint multiplexes runtime metadata alongside AG-UI frames.
      if (name?.startsWith('runtime.')) {
        this.evidence.runtime_events.push({ name, payload: event });
        if (done) {
          const terminal = this.evidence.events.find(item => item.type === 'RUN_FINISHED');
          assert.equal(name, 'runtime.run');
          assert.equal(event.type, 'run_finished');
          assert.equal(event.status, 'completed');
          assert.equal(event.run_id, terminal.runId);
        }
        continue;
      }
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

Then('AG-UI carries the published A2UI surface and completes successfully', function () {
  const events = this.evidence.events;
  const starts = events.filter(event => event.type === 'RUN_STARTED');
  const finishes = events.filter(event => event.type === 'RUN_FINISHED');
  assert.equal(starts.length, 1);
  assert.equal(finishes.length, 1);
  const runId = starts[0].runId;
  assert.ok(typeof runId === 'string' && runId.length > 0);
  assert.equal(starts[0].input?.agentId, this.agentId);
  assert.equal(finishes[0].runId, runId);
  const tools = events.filter(event => event.type === 'TOOL_CALL_RESULT' && event.toolCallName === 'a2ui_render');
  assert.equal(tools.length, 1, 'Expected one genuine renderer tool result');
  assert.equal(tools[0].isError, false);
  const artifacts = events.filter(event => event.type === 'CUSTOM'
    && event.name === 'uar.artifact.available' && event.value?.artifactType === 'a2ui');
  assert.equal(artifacts.length, 1, 'A2UI must be published as an artifact, not merely mentioned in prose');
  const artifactEvent = artifacts[0];
  const artifact = artifactEvent.value;
  assert.equal(artifact.metadata?.profile, 'uar.a2ui/1');
  assert.equal(artifact.metadata.sourceTool, 'a2ui_render');
  assert.equal(artifact.metadata.publication_status, 'published');
  assert.equal(artifact.language, 'application/a2ui+json');
  assert.deepEqual(artifact.metadata.surfaceIds, [this.evidence.surface_id]);
  const messages = artifact.content.trim().split(/\r?\n/).map(line => JSON.parse(line));
  assert.equal(messages.length, 2);
  assert.ok(messages.every(message => message.version === 'v0.9'));
  assert.deepEqual(messages[0].createSurface, { surfaceId: this.evidence.surface_id,
    catalogId: 'https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json' });
  assert.deepEqual(messages[1].updateComponents, { surfaceId: this.evidence.surface_id,
    components: [{ id: 'root', component: 'Text', text: this.evidence.greeting }] });
  const stateEvents = events.filter(event => event.type === 'STATE_DELTA');
  const patches = stateEvents.flatMap(event => event.delta ?? []);
  const path = `/a2ui/surfaces/${this.evidence.surface_id}`;
  const create = patches.findIndex(op => op.op === 'add' && op.path === path);
  const components = patches.findIndex(op => op.op === 'replace' && op.path === `${path}/components`);
  assert.ok(create >= 0 && components > create, 'Missing ordered A2UI state publication');
  assert.deepEqual(patches[create].value, messages[0].createSurface);
  assert.deepEqual(patches[components].value, messages[1].updateComponents);
  for (const event of [finishes[0], tools[0], artifactEvent, ...stateEvents]) {
    assert.equal(event.runId, runId);
    assert.equal(event.threadId, starts[0].threadId);
  }
  assert.ok(events.indexOf(artifactEvent) < events.indexOf(finishes[0]));
  this.evidence.run_id = runId;
  this.evidence.artifact_id = artifact.artifactId;
  this.evidence.messages = messages;
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  let cleanupError;
  if (this.created) {
    try {
      await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, 204);
      this.evidence.disposable_agent_deleted = true;
    } catch (error) { cleanupError = error; this.evidence.cleanup_error = String(error); }
  }
  this.evidence.scenario_status = cleanupError ? 'FAILED' : result.status;
  if (cleanupError) this.evidence.result = 'FAILED';
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify(this.evidence), 'application/json');
  if (cleanupError) throw cleanupError;
});

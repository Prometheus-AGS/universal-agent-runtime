import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { createRequire } from 'node:module';
import { readFileSync, realpathSync } from 'node:fs';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, isAbsolute, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(240_000);
// Keep the consumer outside the pnpm workspace dependency graph.
const defaultClientDirectory = resolve('scripts/behaviors/clients/a2ui-pem');
const clientDirectory = process.env.UAR_A2UI_PEM_CLIENT_DIR ?? defaultClientDirectory;
assert.ok(isAbsolute(clientDirectory), 'UAR_A2UI_PEM_CLIENT_DIR must be an absolute path');
const clientRequire = createRequire(resolve(clientDirectory, 'package.json'));
const load = specifier => import(pathToFileURL(clientRequire.resolve(specifier)).href);

function clientPackageEvidence() {
  const root = realpathSync(clientDirectory);
  const lock = JSON.parse(readFileSync(resolve(root, 'package-lock.json'), 'utf8'));
  return [
    ['@prometheus-ags/a2ui-react', '5.0.0', '@prometheus-ags/a2ui-react/headless'],
    ['@ag-ui/core', '1.0.4', '@ag-ui/core/schemas'],
  ].map(([name, version, entrypoint]) => {
    const path = `node_modules/${name}`;
    const directory = resolve(root, path);
    const record = lock.packages?.[path];
    const installed = JSON.parse(readFileSync(resolve(directory, 'package.json'), 'utf8'));
    assert.equal(realpathSync(directory), directory);
    assert.equal(installed.name, name);
    assert.equal(installed.version, version);
    assert.equal(record?.version, version);
    assert.match(record?.integrity ?? '', /^sha512-[A-Za-z0-9+/]+=*$/);
    assert.equal(new URL(record.resolved).origin, 'https://registry.npmjs.org');
    assert.ok(realpathSync(clientRequire.resolve(entrypoint)).startsWith(`${directory}/`));
    return { name, version, integrity: record.integrity };
  });
}
const UAR_V1_CATALOG = 'https://a2ui.org/specification/v1_0/catalogs/basic/catalog.json';
const UAR_LEGACY_CATALOG = 'https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json';
const NOTE_PATH = '/form/note';

async function request(w, method, path, payload, expected = [200]) {
  const res = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && res.status === 201) w.created = true;
  const text = await res.text();
  w.evidence.exchanges.push({ method, path, status: res.status, response: text.slice(0, 1000) });
  assert.ok(expected.includes(res.status), `${method} ${path}: ${res.status}: ${text.slice(0, 600)}`);
  return text ? JSON.parse(text) : null;
}

const frames = raw => raw.split(/\r?\n\r?\n/).flatMap(block => {
  const data = block.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).replace(/^ /, '')).join('\n');
  return data && data !== '[DONE]' ? [JSON.parse(data)] : [];
});

// One AG-UI exchange. Events feed the PEM runtime; assistant text and activity
// messages are kept as a real AG-UI client keeps its message list.
async function exchange(w, label, { method = 'POST', path = w.route, payload, runtime = w.runtime, expected = 200 }) {
  const { EventSchema, PROTOCOL_VERSION, RunAgentInputSchema } = await load('@ag-ui/core/schemas');
  if (method === 'POST') RunAgentInputSchema.parse(payload);
  const res = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(200_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const raw = await res.text();
  w.evidence.streams[label] = { status: res.status, body: res.status === expected && expected === 200 ? undefined : raw.slice(0, 600) };
  assert.equal(res.status, expected, `${label}: ${res.status}: ${raw.slice(0, 600)}`);
  if (expected !== 200) return { events: [], text: '' };
  const events = frames(raw);
  assert.equal(events[0]?.type, 'RUN_STARTED', label);
  assert.equal(events[0]?.protocolVersion, PROTOCOL_VERSION, label);
  const terminal = events.at(-1);
  const terminalCode = terminal?.type === 'RUN_ERROR' && terminal.code;
  assert.equal(terminal?.type, 'RUN_FINISHED', terminalCode ? `${label} (${terminalCode})` : label);
  for (const event of events) {
    assert.deepEqual(EventSchema.parse(event), event, `Official AG-UI event shape: ${event.type}`);
    runtime.processAgUiEvent(event);
  }
  const text = events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta ?? '').join('')
    .replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
  Object.assign(w.evidence.streams[label], { types: [...new Set(events.map(e => e.type))], answer: text.slice(0, 1000),
    activities: events.filter(e => e.type === 'ACTIVITY_SNAPSHOT').map(e => e.messageId) });
  return { events, text };
}

function remember(w, events, text) {
  const message = events.find(e => e.type === 'TEXT_MESSAGE_START');
  if (text) w.messages.push({ id: message?.messageId ?? randomUUID(), role: 'assistant', content: text });
  for (const e of events.filter(e => e.type === 'ACTIVITY_SNAPSHOT')) {
    w.messages = w.messages.filter(m => m.id !== e.messageId);
    w.messages.push({ id: e.messageId, role: 'activity', activityType: e.activityType, content: e.content });
  }
}

async function createRuntime(w) {
  const { createPrometheusA2uiHeadlessRuntime, createPrometheusA2uiHeadlessCatalog } = await load('@prometheus-ags/a2ui-react/headless');
  return createPrometheusA2uiHeadlessRuntime({
    catalogs: [
      createPrometheusA2uiHeadlessCatalog({ id: UAR_LEGACY_CATALOG }),
      createPrometheusA2uiHeadlessCatalog({ id: UAR_V1_CATALOG, protocolVersion: '1.0' }),
    ],
    actionPolicy: { handle: async action => ({ status: 'executed', action, result: null }) },
    onRendererMessage: envelope => { w.rendererMessages?.push(envelope); },
  });
}

function rendered(runtime, surfaceId) {
  const surface = runtime.getSurface(surfaceId);
  if (!surface) return null;
  const components = [...surface.componentsModel.getAll().values()];
  const textOf = c => {
    const text = c?.properties?.text;
    return text?.['@path'] || text?.path ? surface.dataModel.get(text['@path'] ?? text.path) : text;
  };
  const button = components.find(c => c.type === 'Button' && textOf(surface.componentsModel.get(c.properties?.child)) === 'Confirm');
  return { surface, components, texts: components.filter(c => c.type === 'Text').map(textOf), button };
}

async function press(w, note) {
  const view = rendered(w.runtime, w.surfaceId);
  assert.ok(view?.button, 'The Confirm button is rendered');
  view.surface.dataModel.set(NOTE_PATH, note);
  w.rendererMessages = [];
  await view.surface.dispatchAction({ event: { name: 'confirmTicket', context: { note } } }, view.button.id);
  assert.equal(w.rendererMessages.length, 1, 'The PEM runtime emits one renderer message');
  const [envelope] = w.rendererMessages;
  assert.equal(envelope.version, '1.0');
  assert.equal(envelope.action?.context?.note, note);
  return envelope;
}

async function clickRun(w, label, envelope) {
  const forwarded = { a2uiAction: structuredClone(envelope) };
  const snapshot = [...w.messages].reverse().find(message => message.role === 'activity'
    && message.content?.a2ui_operations?.some(operation => operation.createSurface?.surfaceId === w.surfaceId));
  assert.equal(typeof snapshot?.content?.surface_generation, 'string', 'Server publishes an authoritative surface generation');
  forwarded.a2uiAction.interaction_id = randomUUID();
  forwarded.a2uiAction.surface_generation = snapshot.content.surface_generation;
  const payload = { threadId: w.threadId, runId: randomUUID(), parentRunId: w.lastRunId, protocolVersion: '1.0',
    messages: w.messages, tools: [], context: [], state: {}, forwardedProps: forwarded };
  w.actionPayloads ??= {};
  w.actionPayloads[label] = structuredClone(payload);
  const result = await exchange(w, label, { payload });
  w.lastRunId = payload.runId;
  remember(w, result.events, result.text);
  return result;
}

Given('a disposable agent on the dedicated AG-UI route that publishes a ticket surface with a note field and a Confirm button', async function () {
  this.started = performance.now();
  this.agentId = `disposable-agui-a2ui-${randomUUID()}`;
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model);
  this.route = `/api/uar/ag-ui/agents/${this.agentId}`;
  this.record = resolve(`.prometheus/verification/agui-route-a2ui/${this.agentId}.json`);
  this.evidence = { scenario: 'I19-AGUI-ROUTE-A2UI', platform: process.platform, model: this.model, agent_id: this.agentId,
    route: this.route, client: clientPackageEvidence(),
    authentication_supplied_by_test: false, streams: {}, exchanges: [] };
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  Object.assign(agent, { id: this.agentId, version: '1.0.0' });
  agent.metadata = { ...agent.metadata, title: 'Disposable AG-UI ticket agent', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: [
    'When asked for a ticket surface, call the a2ui_render tool exactly once with an A2UI 1.0 createSurface message.',
    `Use the basic catalog (${UAR_V1_CATALOG}), the requested surfaceId, and initial dataModel { heading: requested heading }.`,
    'Inline components: {"id":"root","component":"Column","children":["heading","note","confirm"]},',
    '{"id":"heading","component":"Text","text":{"@path":"/heading"}},',
    `{"id":"note","component":"TextField","label":"Note","value":{"@path":"${NOTE_PATH}"}},`,
    `{"id":"confirm","component":"Button","child":"confirmLabel","action":{"event":{"name":"confirmTicket","context":{"note":{"@path":"${NOTE_PATH}"}}}}},`,
    '{"id":"confirmLabel","component":"Text","text":"Confirm"}. After the tool succeeds, reply with one short sentence.',
    'When your input is a JSON object with "type":"a2ui.user_action", do not call any tool; reply exactly: NOTE: <interaction.context.note>',
  ].join(' '), instructions: [] };
  const split = this.model.indexOf('/');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.tools.allow = ['a2ui_render'];
  agent.policy.skills.prefer = [];
  agent.memory.kb.enabled = false;
  await request(this, 'POST', '/api/agents', agent, [201]);
});

Given("the pinned PEM headless client configured with UAR's catalog ids", async function () {
  this.runtime = await createRuntime(this);
  this.threadId = randomUUID();
  this.messages = [];
});

function publishedSurfaceCreation(operations, surfaceId) {
  // Legacy state-patch snapshots can precede the artifact carrying the original dialect.
  return operations.findLast(operation => operation.createSurface?.surfaceId === surfaceId);
}

async function publishTicket(legacy) {
  this.surfaceId = `ticket-${randomUUID().slice(0, 8)}`;
  this.heading = `Ticket ${randomUUID().slice(0, 8)}`;
  const userMessage = { id: randomUUID(), role: 'user', content: `Publish a ${legacy ? 'legacy ' : ''}ticket surface with surfaceId ${this.surfaceId} and heading "${this.heading}".` };
  this.messages.push(userMessage);
  this.lastRunId = randomUUID();
  const result = await exchange(this, 'publish', { payload: { threadId: this.threadId, runId: this.lastRunId, protocolVersion: '1.0',
    messages: [userMessage], tools: [], context: [], state: {}, forwardedProps: { uar: { model: this.model } } } });
  const operations = result.events.flatMap(event => event.content?.a2ui_operations ?? []);
  const create = publishedSurfaceCreation(operations, this.surfaceId);
  this.surfaceVersion = legacy ? 'v0.9.1' : '1.0';
  assert.equal(create?.version, this.surfaceVersion);
  if (!legacy) {
    assert.equal(create.createSurface.dataModel.heading, this.heading);
    assert.ok(create.createSurface.components.some(component => component.id === 'confirm'));
    assert.ok(operations.some(operation => operation.updateDataModel?.path === '/phase'));
    assert.equal(this.runtime.getSurface(this.surfaceId)?.dataModel.get('/phase'), 'updated');
  }
  remember(this, result.events, result.text);
}

When('the client asks for a fresh ticket surface', async function () {
  await publishTicket.call(this, false);
});
When('the client asks for a legacy ticket surface', async function () {
  await publishTicket.call(this, true);
});

Then('the client renders the surface with the requested heading and a Confirm button', function () {
  const view = rendered(this.runtime, this.surfaceId);
  this.evidence.rendered = view && { texts: view.texts, components: view.components.map(c => ({ id: c.id, type: c.type })) };
  assert.ok(view, `The client rendered no surface ${this.surfaceId}; activities: ${JSON.stringify(this.evidence.streams.publish?.activities)}`);
  assert.ok(view.texts.includes(this.heading), 'The requested heading is rendered');
  assert.ok(view.button, 'A Confirm button is rendered');
});

When('the user enters a note and presses Confirm', async function () {
  this.firstNote = `note-${randomUUID().slice(0, 8)}`;
  this.firstEnvelope = await press(this, this.firstNote);
  this.firstClick = await clickRun(this, 'first-click', this.firstEnvelope);
});

Then('the agent answers in a new run on the same conversation and its answer contains that note', function () {
  assert.ok(this.firstClick.text.includes(this.firstNote), `Answer: ${this.firstClick.text}`);
  const reasoning = this.firstClick.events.filter(event => event.type.startsWith('REASONING_'));
  assert.deepEqual(reasoning.map(event => event.type), [
    'REASONING_START', 'REASONING_MESSAGE_START', 'REASONING_MESSAGE_CONTENT',
    'REASONING_MESSAGE_END', 'REASONING_END',
  ]);
  assert.equal(reasoning[2].delta, 'Synthetic public reasoning.');
  assert.equal(reasoning[1].messageId, reasoning[3].messageId);
  assert.equal(reasoning[0].messageId, reasoning[4].messageId);
  assert.ok(this.firstClick.events.indexOf(reasoning[4]) <
    this.firstClick.events.findIndex(event => event.type === 'TEXT_MESSAGE_START'));

});

When('the user enters a different note and presses Confirm again', async function () {
  this.secondNote = `note-${randomUUID().slice(0, 8)}`;
  this.secondClick = await clickRun(this, 'second-click', await press(this, this.secondNote));
});

Then('the agent answers again and its answer contains the second note', function () {
  assert.ok(this.secondClick.text.includes(this.secondNote), `Answer: ${this.secondClick.text}`);
});

When("a fresh client attaches to the conversation's latest run", async function () {
  this.lateRuntime = await createRuntime({ evidence: this.evidence });
  this.late = await exchange(this, 'late-attach', { method: 'GET', runtime: this.lateRuntime,
    path: `${this.route}/threads/${this.threadId}/runs/${this.lastRunId}/stream` });
});

Then('that client renders the same surface without the agent running again', function () {
  const view = rendered(this.lateRuntime, this.surfaceId);
  assert.ok(view?.texts.includes(this.heading) && view.button, 'The late client renders the surface');
  const operations = this.late.events.flatMap(event => event.content?.a2ui_operations ?? []);
  assert.equal(publishedSurfaceCreation(operations, this.surfaceId)?.version, this.surfaceVersion);
  const original = this.evidence.streams['second-click'] ?? this.evidence.streams.publish;
  assert.deepEqual(
    this.late.events.filter(event => event.type === 'TOOL_CALL_START').map(event => event.type),
    original.types.includes('TOOL_CALL_START') ? ['TOOL_CALL_START'] : [],
    'Attaching can only replay the retained tool call',
  );
});

Then('an action naming a button the surface never declared is refused', async function () {
  const forged = structuredClone(this.actionPayloads['first-click'].forwardedProps);
  forged.a2uiAction.interaction_id = randomUUID();
  forged.a2uiAction.action.name = 'doesNotExist';
  await exchange(this, 'forged', { expected: 422, payload: { threadId: this.threadId, runId: randomUUID(), parentRunId: this.lastRunId,
    protocolVersion: '1.0', messages: this.messages, tools: [], context: [], state: {}, forwardedProps: forged } });
  this.evidence.result = 'VERIFIED';
});

Then('a lost-response retry with a fresh wire id reuses the original admitted click', async function () {
  const original = this.actionPayloads['first-click'];
  const retry = { ...structuredClone(original), runId: randomUUID() };
  const replay = await exchange(this, 'logical-retry', { payload: retry });
  const firstStarted = this.firstClick.events.find(event => event.type === 'RUN_STARTED');
  const replayStarted = replay.events.find(event => event.type === 'RUN_STARTED');
  assert.equal(replayStarted?.runId, firstStarted?.runId, 'Fresh transport id attaches the original durable admission');
  assert.notEqual(replayStarted?.runId, retry.runId, 'Retry did not become a new run');
  assert.equal(replay.text, this.firstClick.text, 'Settled result is retained');
  const receipt = await request(this, 'GET', `${this.route}/threads/${this.threadId}/actions/${original.forwardedProps.a2uiAction.interaction_id}`);
  assert.equal(receipt.wire_run_id, original.runId);
  assert.equal(receipt.retry_protected, true);
  assert.equal(receipt.outcome, 'completed');
  this.evidence.logical_interaction = { interaction_id: receipt.interaction_id, native_run_id: receipt.continuation_run_id,
    original_wire_run_id: original.runId, attempted_retry_wire_run_id: retry.runId, outcome: receipt.outcome };
});

Then('reusing that interaction identity with changed semantic input is refused', async function () {
  const changed = structuredClone(this.actionPayloads['first-click']);
  changed.runId = randomUUID();
  changed.forwardedProps.a2uiAction.action.context = { changed: 'conflicting-intent' };
  await exchange(this, 'logical-conflict', { payload: changed, expected: 409 });
});

Then('an agent function call from the renderer is explicitly refused', async function () {
  const original = this.actionPayloads['first-click'];
  const payload = { ...structuredClone(original), runId: randomUUID(), parentRunId: this.lastRunId };
  payload.forwardedProps.a2uiAction = {
    version: '1.0',
    callAgentFunction: {
      surfaceId: this.surfaceId, functionCallId: randomUUID(),
      callFunction: { '@call': 'confirmTicket', catalogId: UAR_V1_CATALOG, args: {} },
    },
  };
  const res = await fetch(new URL(this.route, this.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': this.workspace },
    body: JSON.stringify(payload),
  });
  assert.equal(res.status, 422);
  const refusal = await res.json();
  assert.equal(refusal.code ?? refusal.error?.code ?? refusal.error, 'a2ui_functions_unsupported');
});

When('the agent deletes the surface', async function () {
  const runId = randomUUID();
  const payload = {
    threadId: this.threadId, runId, parentRunId: this.lastRunId, protocolVersion: '1.0',
    messages: [...this.messages, { id: randomUUID(), role: 'user', content: `Delete the ticket surface ${this.surfaceId}.` }],
    tools: [], context: [], state: {}, forwardedProps: {},
  };
  const result = await exchange(this, 'delete', { payload });
  assert.ok(result.events.flatMap(event => event.content?.a2ui_operations ?? [])
    .some(operation => operation.version === '1.0' && operation.deleteSurface?.surfaceId === this.surfaceId));
  assert.equal(this.runtime.getSurface(this.surfaceId), undefined);
  this.lastRunId = runId;
});

Then('a fresh client replays the deletion without dispatch', async function () {
  const runtime = await createRuntime({});
  const replay = await exchange(this, 'delete-replay', { method: 'GET', runtime,
    path: `${this.route}/threads/${this.threadId}/runs/${this.lastRunId}/stream` });
  assert.ok(replay.events.flatMap(event => event.content?.a2ui_operations ?? [])
    .some(operation => operation.deleteSurface?.surfaceId === this.surfaceId));
  assert.equal(runtime.getSurface(this.surfaceId), undefined);
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.created) { try { await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204, 404]); } catch (e) { errors.push(String(e)); } }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx' });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});

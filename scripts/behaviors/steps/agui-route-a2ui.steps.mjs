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
const historicalClientDirectory = resolve('scripts/behaviors/clients/a2ui-pem');
const clientDirectory = process.env.UAR_A2UI_PEM_CLIENT_DIR ?? historicalClientDirectory;
assert.ok(isAbsolute(clientDirectory), 'UAR_A2UI_PEM_CLIENT_DIR must be an absolute path');
const clientRequire = createRequire(resolve(clientDirectory, 'package.json'));
const load = specifier => import(pathToFileURL(clientRequire.resolve(specifier)).href);

function clientPackageEvidence() {
  const lock = JSON.parse(readFileSync(resolve(clientDirectory, 'package-lock.json'), 'utf8'));
  const migrated = lock.packages?.['node_modules/@prometheus-ags/a2ui-react']?.version === '4.3.1-rc.1';
  assert.ok(!process.env.UAR_A2UI_PEM_CLIENT_DIR || migrated, 'The staged consumer must use the migrated PEM package');
  const expected = [
    ['@prometheus-ags/a2ui-react', '@prometheus-ags/a2ui-react',
      migrated ? '4.3.1-rc.1' : '4.3.0', '@prometheus-ags/a2ui-react/headless'],
    ['@a2ui/web_core', migrated ? '@prometheus-ags/a2ui-web-core-zod4' : '@a2ui/web_core',
      migrated ? '0.14.0-rc.1' : '0.12.0', '@a2ui/web_core/v0_9'],
  ];
  const packages = expected.map(([alias, name, version, entrypoint]) => {
    const path = `node_modules/${alias}`;
    const record = lock.packages?.[path];
    const directory = realpathSync(resolve(clientDirectory, path));
    assert.equal(directory, resolve(realpathSync(clientDirectory), path), `Isolated ${alias} install`);
    const installed = JSON.parse(readFileSync(resolve(directory, 'package.json'), 'utf8'));
    assert.equal(installed.name, name, `Installed ${alias} package identity`);
    assert.equal(installed.version, version, `Installed ${alias} package version`);
    assert.equal(record?.version, version, `Locked ${alias} version`);
    assert.match(record?.integrity ?? '', /^sha512-[A-Za-z0-9+/]+=*$/);
    assert.ok(realpathSync(clientRequire.resolve(entrypoint)).startsWith(`${directory}/`));
    const origin = record.resolved?.startsWith('file:') ? 'local-tarball' : 'npm-registry';
    if (origin === 'local-tarball') {
      assert.ok(migrated && record.resolved.endsWith('.tgz'), 'Local candidate must be a packed tarball');
    } else {
      const url = new URL(record.resolved);
      assert.ok(url.origin === 'https://registry.npmjs.org' && !url.username && !url.password);
    }
    return { alias, name, version, origin, integrity: record.integrity };
  });
  assert.equal(packages[0].origin, packages[1].origin, 'Use one origin for both candidate packages');
  const pemRequire = createRequire(clientRequire.resolve('@prometheus-ags/a2ui-react/headless'));
  const coreEntrypoint = '@a2ui/web_core/v0_9';
  const corePath = realpathSync(clientRequire.resolve(coreEntrypoint));
  assert.equal(realpathSync(pemRequire.resolve(coreEntrypoint)), corePath,
    'PEM and the independent binder must share the same installed core');
  if (migrated) {
    const zodPath = realpathSync(pemRequire.resolve('zod'));
    assert.equal(realpathSync(createRequire(corePath).resolve('zod')), zodPath,
      'PEM and the core must share the same installed Zod');
    assert.ok(zodPath.startsWith(`${realpathSync(clientDirectory)}/node_modules/zod/`));
    const zod = lock.packages?.['node_modules/zod'];
    const installed = JSON.parse(readFileSync(resolve(clientDirectory, 'node_modules/zod/package.json'), 'utf8'));
    assert.equal(zod?.version, '4.6.7');
    assert.equal(installed.version, zod.version);
    assert.match(zod.integrity, /^sha512-[A-Za-z0-9+/]+=*$/);
    packages.push({ alias: 'zod', name: 'zod', version: zod.version, origin: 'npm-registry', integrity: zod.integrity });
  }
  return { packages, migrated };
}
const UAR_CATALOGS = ['https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json', 'urn:uar:a2ui:catalog:1'];
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
  for (const event of events) runtime.processAgUiEvent(event);
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
  w.envelopes = [];
  return createPrometheusA2uiHeadlessRuntime({
    catalogs: UAR_CATALOGS.map(id => createPrometheusA2uiHeadlessCatalog({ id })),
    actionPolicy: { handle: async action => ({ status: 'executed', action, result: null }) },
    onActionEnvelope: envelope => { w.envelopes.push(envelope); w.envelopeWaiter?.(envelope); },
  });
}

function rendered(runtime, surfaceId) {
  const surface = runtime.getSurface(surfaceId);
  if (!surface) return null;
  const components = [...surface.componentsModel.getAll().values()];
  const textOf = c => c?.properties?.text;
  const button = components.find(c => c.type === 'Button' && textOf(surface.componentsModel.get(c.properties?.child)) === 'Confirm');
  return { surface, components, texts: components.filter(c => c.type === 'Text').map(textOf), button };
}

// A real press: the user's note goes into the data model, and web_core's binder
// resolves the button's declared context and dispatches through PEM.
async function press(w, note) {
  const { ComponentContext, GenericBinder } = await load('@a2ui/web_core/v0_9');
  const { ButtonApi } = await load('@a2ui/web_core/v0_9/basic_catalog');
  const view = rendered(w.runtime, w.surfaceId);
  assert.ok(view?.button, 'The Confirm button is rendered');
  view.surface.dataModel.set(NOTE_PATH, note);
  const envelope = new Promise((done, fail) => {
    w.envelopeWaiter = done;
    setTimeout(() => fail(new Error('PEM produced no action envelope')), 10_000);
  });
  const binder = new GenericBinder(new ComponentContext(view.surface, view.button.id), ButtonApi.schema);
  binder.snapshot.action();
  return envelope;
}

async function clickRun(w, label, envelope) {
  const forwarded = structuredClone(envelope.forwardedProps);
  const snapshot = [...w.messages].reverse().find(message => message.role === 'activity'
    && message.content?.a2ui_operations?.some(operation => operation.createSurface?.surfaceId === w.surfaceId));
  assert.equal(typeof snapshot?.content?.surface_generation, 'string', 'Server publishes an authoritative surface generation');
  forwarded.a2uiAction.interaction_id = randomUUID();
  forwarded.a2uiAction.surface_generation = snapshot.content.surface_generation;
  const payload = { threadId: w.threadId, runId: randomUUID(), parentRunId: w.lastRunId, protocolVersion: '1.0',
    messages: w.messages, tools: [], context: [], forwardedProps: forwarded };
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
    'When asked for a ticket surface, call the a2ui_render tool exactly once with these A2UI v0.9 messages, using the basic catalog',
    '(https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json), the requested surfaceId, and the requested heading:',
    '1. createSurface. 2. updateComponents with components: {"id":"root","component":"Column","children":["heading","note","confirm"]},',
    '{"id":"heading","component":"Text","text":"<the requested heading>"},',
    `{"id":"note","component":"TextField","label":"Note","value":{"path":"${NOTE_PATH}"}},`,
    `{"id":"confirm","component":"Button","child":"confirmLabel","action":{"event":{"name":"confirmTicket","context":{"note":{"path":"${NOTE_PATH}"}}}}},`,
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

When('the client asks for a fresh ticket surface', async function () {
  this.surfaceId = `ticket-${randomUUID().slice(0, 8)}`;
  this.heading = `Ticket ${randomUUID().slice(0, 8)}`;
  const userMessage = { id: randomUUID(), role: 'user', content: `Publish a ticket surface with surfaceId ${this.surfaceId} and heading "${this.heading}".` };
  this.messages.push(userMessage);
  this.lastRunId = randomUUID();
  const result = await exchange(this, 'publish', { payload: { threadId: this.threadId, runId: this.lastRunId, protocolVersion: '1.0',
    messages: [userMessage], tools: [], context: [], forwardedProps: { uar: { model: this.model } } } });
  remember(this, result.events, result.text);
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
  assert.ok(!this.late.events.some(e => e.type === 'TOOL_CALL_START'), 'Attaching runs nothing');
});

Then('an action naming a button the surface never declared is refused', async function () {
  const forged = structuredClone(this.actionPayloads['first-click'].forwardedProps);
  forged.a2uiAction.interaction_id = randomUUID();
  forged.a2uiAction.userAction.name = 'doesNotExist';
  await exchange(this, 'forged', { expected: 422, payload: { threadId: this.threadId, runId: randomUUID(), parentRunId: this.lastRunId,
    protocolVersion: '1.0', messages: this.messages, tools: [], context: [], forwardedProps: forged } });
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
  changed.forwardedProps.a2uiAction.userAction.context = { changed: 'conflicting-intent' };
  await exchange(this, 'logical-conflict', { payload: changed, expected: 409 });
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

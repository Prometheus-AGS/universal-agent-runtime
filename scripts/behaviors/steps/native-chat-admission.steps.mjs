import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
setDefaultTimeout(180_000);
let poisoned = false;
const terminal = ['complete', 'error', 'cancelled', 'interrupted'];
const imagePath = 'scripts/behaviors/fixtures/capability-dispatch/synthetic.png';
function localUrl(value) {
  const url = new URL(value);
  assert.ok(['http:', 'https:'].includes(url.protocol) && ['127.0.0.1', '[::1]'].includes(url.hostname));
  assert.ok(!url.username && !url.password && !url.search && !url.hash);
  return url;
}
async function lines(path) {
  const raw = await readFile(path, 'utf8');
  return raw.split('\n').filter(Boolean).map(line => JSON.parse(line));
}
async function journals(w) { return { provider: await lines(w.fixture.peer.journal), effects: await lines(w.fixture.peer.recordLog) }; }
async function http(w, method, path, body, statuses = [200]) {
  assert.ok(++w.operations <= 300, 'Dedicated fixture HTTP budget exceeded');
  if (method === 'POST' && path === '/api/chat/completion') w.posts++;
  const response = await fetch(new URL(path, w.base), { method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace, 'X-UAR-Session-ID': w.session },
    body: body === undefined ? undefined : JSON.stringify(body) });
  const raw = await response.text();
  w.evidence.exchanges.push({ method, path, status: response.status, request: body, response: raw });
  assert.ok(statuses.includes(response.status), `${method} ${path}: ${response.status}: ${raw.slice(0, 800)}`);
  if (!raw) return null;
  try { return JSON.parse(raw); } catch { return raw; }
}
async function receipt(w) {
  return http(w, 'GET', `/api/uar/chat-turns/${encodeURIComponent(w.turn)}?session_id=${encodeURIComponent(w.session)}`);
}
async function eventually(w, read, predicate, description) {
  for (let attempt = 0; attempt < 120; attempt++) {
    const result = await read();
    if (predicate(result)) return result;
    await delay(250);
  }
  assert.fail(`Timed out observing ${description}`);
}
function body(w, mode = 'success') {
  const agent = mode === 'effect' ? w.fixture.agents.effect : mode === 'approval' ? w.fixture.agents.approval : w.fixture.agents.ordinary;
  return { message: `UAR_NATIVE_CHAT:${w.fixture.namespace}:${w.caseId}:${mode}`, agent_id: agent, model: w.fixture.model,
    session_id: w.session, client_turn_id: w.turn, user_message_id: w.user, memory_enabled: false, stream: true, stream_mode: 'agui_spec' };
}
async function post(w, request) {
  assert.ok(++w.operations <= 300);
  w.posts++;
  const response = await fetch(new URL('/api/chat/completion', w.base), { method: 'POST', redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace, 'X-UAR-Session-ID': w.session },
    body: JSON.stringify(request) });
  const transport = { attempt: w.posts, status: response.status, run_header: response.headers.get('x-uar-run-id'), request };
  w.evidence.transports.push(transport);
  const finished = response.text().then(raw => { transport.raw = raw; return transport; }, error => { transport.transport_error = String(error); return transport; });
  return { transport, finished };
}
function frames(raw) {
  return (raw ?? '').split(/\r?\n\r?\n/).flatMap(block => {
    const value = block.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    return !value || value === '[DONE]' ? [] : [JSON.parse(value)];
  });
}
function transportRun(transport) {
  assert.equal(transport.status, 200);
  assert.equal(transport.transport_error, undefined);
  if (transport.raw?.trim().startsWith('{')) return JSON.parse(transport.raw).run_id;
  return transport.run_header ?? frames(transport.raw).find(row => row.type === 'RUN_STARTED')?.runId;
}
function assertReceipt(w, value, expectedState) {
  assert.equal(value.client_turn_id, w.turn);
  assert.equal(value.user_message_id, w.user);
  assert.equal(value.session_id, w.session);
  assert.equal(value.retry_allowed, false);
  assert.ok(typeof value.run_id === 'string' && value.run_id);
  if (w.runId) assert.equal(value.run_id, w.runId);
  else w.runId = value.run_id;
  if (expectedState) assert.equal(value.state, expectedState);
}
async function history(w) { return http(w, 'GET', `/api/uar/sessions/${encodeURIComponent(w.session)}`); }
function originalUser(w, value) {
  assert.equal(value.history_version, 1);
  const users = value.history.filter(row => row.role === 'user' && row.message_id === w.user);
  assert.equal(users.length, 1, 'Original persisted user identity occurs exactly once');
  assert.equal(users[0].client_turn_id, w.turn);
  assert.equal(users[0].run_id, w.runId);
  return users[0];
}
async function counts(w, expectedProvider, expectedEffects) {
  const now = await journals(w);
  const provider = now.provider.slice(w.before.provider.length).filter(row => row.kind === 'provider');
  const effects = now.effects.slice(w.before.effects.length).filter(row => row.tool === 'record_write' && !row.response_dropped);
  assert.equal(provider.length, expectedProvider, 'Independent provider arrival count differs');
  assert.ok(provider.every(row => row.namespace === w.fixture.namespace && row.case_id === w.caseId && row.model === w.fixture.peer.wireModel),
    'Every arrival must belong to the exact owned case and pinned destination');
  assert.equal(effects.length, expectedEffects, 'Independent committed effect count differs');
  assert.ok(effects.every(row => row.arguments?.key === `${w.fixture.namespace}:${w.caseId}` && row.result?.stored === true && row.result?.revision === 1));
  if (expectedEffects === 1) assert.deepEqual(provider.map(row => row.stage), ['tool_request', 'after_tool'], 'Two provider loop calls must not be mistaken for two admissions');
  w.evidence.journal_delta = { provider, effects };
}
Given('the externally provisioned native chat admission fixture', async function () {
  assert.equal(poisoned, false, 'A previous failed case needs external reconciliation before more scenarios');
  assert.ok(process.env.UAR_NATIVE_CHAT_FIXTURE && process.env.UAR_BASE_URL && process.env.UAR_WORKSPACE_ID);
  this.fixture = JSON.parse(await readFile(process.env.UAR_NATIVE_CHAT_FIXTURE, 'utf8'));
  assert.equal(this.fixture.protocol, 'uar.native-chat-admission-fixture/v1');
  assert.equal(this.fixture.dedicated, true);
  assert.match(this.fixture.namespace, /^native-chat-[A-Za-z0-9_-]{1,64}$/);
  assert.match(this.fixture.identity.sourceCommit, /^[a-f0-9]{40}$/);
  assert.match(this.fixture.identity.imageDigest, /^sha256:[a-f0-9]{64}$/);
  assert.match(this.fixture.identity.configDigest, /^[a-f0-9]{64}$/);
  assert.equal(this.fixture.identity.platform, 'linux/amd64');
  assert.equal(this.fixture.peer.protocol, 'uar.native-chat-admission-peer/v1');
  for (const flag of ['retriesDisabled', 'fallbacksDisabled', 'visionEnabled']) assert.equal(this.fixture.peer[flag], true);
  for (const path of [this.fixture.peer.journal, this.fixture.peer.recordLog]) assert.ok(path?.startsWith('/'));
  for (const role of ['ordinary', 'effect', 'approval']) assert.ok(this.fixture.agents[role]);
  this.base = localUrl(process.env.UAR_BASE_URL); this.workspace = process.env.UAR_WORKSPACE_ID;
  this.caseId = randomUUID(); this.session = randomUUID(); this.turn = randomUUID(); this.user = randomUUID(); this.posts = 0; this.operations = 0;
  this.started = performance.now(); this.record = resolve(`.prometheus/verification/native-chat-admission/${this.caseId}.json`);
  this.evidence = { protocol: this.fixture.protocol, identity: this.fixture.identity, namespace: this.fixture.namespace, case_id: this.caseId,
    session_id: this.session, client_turn_id: this.turn, user_message_id: this.user, authentication_supplied_by_test: false,
    external_services_started_by_test: false, exchanges: [], transports: [] };
  this.before = await journals(this);
  assert.ok(this.before.provider.some(row => row.kind === 'started' && row.namespace === this.fixture.namespace && row.protocol === this.fixture.peer.protocol));
});
When('two independent native chat POSTs carry the same original identity for {string}', async function (mode) {
  this.original = body(this, mode);
  const responses = await Promise.all([post(this, this.original), post(this, this.original)]);
  this.initialTransports = await Promise.all(responses.map(response => response.finished));
  this.current = await eventually(this, () => receipt(this), value => terminal.includes(value.state), 'original native terminal');
  assertReceipt(this, this.current, 'complete');
});
Then('both transports and a later identical POST reconcile the original run', async function () {
  for (const transport of this.initialTransports) assert.equal(transportRun(transport), this.runId);
  const replay = await (await post(this, this.original)).finished;
  assert.equal(transportRun(replay), this.runId);
  assert.ok(replay.raw.trim().startsWith('{'), 'Identical later POST returns the existing receipt, not another event stream');
  assertReceipt(this, JSON.parse(replay.raw), 'complete');
  originalUser(this, await history(this));
});
Then('the independent journals show exactly {int} provider arrivals and {int} committed record effects', async function (arrivals, effects) {
  await counts(this, arrivals, effects);
});
When('one native chat turn succeeds with its original identity', async function () {
  this.original = body(this); const result = await (await post(this, this.original)).finished;
  this.current = await receipt(this); assertReceipt(this, this.current, 'complete');
  assert.equal(transportRun(result), this.runId);
  await counts(this, 1, 0);
});
When('I reuse that turn identity with changed {string}', async function (field) {
  const changed = structuredClone(this.original);
  if (field === 'message') changed.message += ' changed intent';
  else if (field === 'policy') changed.memory_enabled = true;
  else if (field === 'attachments') changed.attachments = [{ id: randomUUID(), filename: 'synthetic.png', content_type: 'image/png',
    url: `data:image/png;base64,${(await readFile(imagePath)).toString('base64')}` }];
  else assert.fail('Unknown conflict field');
  this.changed = await http(this, 'POST', '/api/chat/completion', changed, [409]);
});
Then('the changed request conflicts without another provider arrival or effect', async function () {
  assert.equal(this.changed.error?.code, 'native_chat_turn_conflict');
  assertReceipt(this, await receipt(this), 'complete');
  await counts(this, 1, 0);
});
When('I discard the response of an admitted native chat turn', async function () {
  this.original = body(this);
  // Consume/discard at the transport boundary without using its run ID. The
  // browser route-loss fixture separately destroys an actual delivery response.
  const transport = await (await post(this, this.original)).finished;
  assert.equal(transport.status, 200);
  this.evidence.admission_body_discarded = true;
});
Then('read-only turn reconciliation retains its original run and one provider arrival', async function () {
  const posts = this.posts;
  assertReceipt(this, await receipt(this), 'complete');
  assertReceipt(this, await receipt(this), 'complete');
  originalUser(this, await history(this));
  assert.equal(this.posts, posts);
  await counts(this, 1, 0);
});
When('a native chat turn with a rich image completes', async function () {
  this.original = body(this); this.image = `data:image/png;base64,${(await readFile(imagePath)).toString('base64')}`;
  this.original.attachments = [{ id: randomUUID(), filename: 'synthetic.png', content_type: 'image/png', url: this.image }];
  const transport = await (await post(this, this.original)).finished;
  assertReceipt(this, await receipt(this), 'complete'); assert.equal(transportRun(transport), this.runId);
});
Then('repeated canonical history reads retain the original rich user message and completed assistant', async function () {
  const first = await history(this); const second = await history(this);
  const user = originalUser(this, first); originalUser(this, second);
  assert.ok(Array.isArray(user.content), 'Rich image content must stay typed');
  assert.ok(user.content.some(part => part.type === 'image_url' && part.image_url?.url === this.image));
  assert.deepEqual(first.history.map(row => row.message_id), second.history.map(row => row.message_id));
  assert.ok(first.history.some(row => row.role === 'assistant' && row.run_id === this.runId && row.status === 'complete'));
  await counts(this, 1, 0);
  assert.equal(this.evidence.journal_delta.provider[0].image_parts, 1, 'The actual peer must receive the original image');
});
When('a native chat {string} turn produces partial output', async function (mode) {
  const response = await post(this, body(this, mode));
  const own = await eventually(this, () => receipt(this), value => Boolean(value.run_id), 'native reservation'); assertReceipt(this, own);
  if (mode === 'hold') {
    await eventually(this, () => lines(this.fixture.peer.journal), rows => rows.some(row => row.kind === 'partial_ready' && row.case_id === this.caseId), 'provider partial output');
    const cancelled = await http(this, 'POST', `/api/uar/runs/${encodeURIComponent(this.runId)}/cancel`, {});
    assert.equal(cancelled.cancelled, true);
  }
  this.partialTransport = await response.finished;
  assert.ok(frames(this.partialTransport.raw).some(row => row.type === 'TEXT_MESSAGE_CONTENT' && row.delta?.includes('synthetic native partial')));
});
Then('its receipt and canonical partial history retain {string} without a replacement run', async function (state) {
  const settled = await eventually(this, () => receipt(this), value => value.state === state, `${state} terminal`); assertReceipt(this, settled, state);
  const first = await history(this); const second = await history(this); originalUser(this, first);
  const partial = first.history.filter(row => row.role === 'assistant' && row.run_id === this.runId);
  assert.ok(partial.length > 0);
  assert.ok(partial.every(row => row.status === state && row.status !== 'complete'));
  assert.ok(JSON.stringify(partial).includes('synthetic native partial'));
  assert.deepEqual(first.history.map(row => row.message_id), second.history.map(row => row.message_id));
  const replay = await (await post(this, body(this, state === 'error' ? 'partial-error' : 'hold'))).finished;
  assert.equal(transportRun(replay), this.runId); assertReceipt(this, JSON.parse(replay.raw), state);
});
When('a native governed tool turn is approved through its originating approval', async function () {
  const response = await post(this, body(this, 'approval'));
  assertReceipt(this, await eventually(this, () => receipt(this), value => Boolean(value.run_id), 'approval run identity'));
  const pending = await eventually(this, () => http(this, 'GET', `/api/uar/runs/${encodeURIComponent(this.runId)}/tool-approval/pending`),
    value => typeof value.pending?.approvalId === 'string', 'originating live approval');
  this.approvalId = pending.pending.approvalId;
  const approved = await http(this, 'POST', `/api/uar/runs/${encodeURIComponent(this.runId)}/tool-approval`, { approval_id: this.approvalId, approved: true });
  assert.equal(approved.resolved, true);
  await response.finished;
  assertReceipt(this, await receipt(this), 'complete');
});
Then('repeated canonical history reads retain that exact approval decision without new authorization', async function () {
  const before = this.posts;
  const views = [await history(this), await history(this)];
  for (const view of views) {
    originalUser(this, view);
    const run = view.history_runs[this.runId]; assert.equal(run.approvals_availability, 'available');
    const decision = run.approvals.find(row => row.approval_id === this.approvalId);
    assert.ok(decision); assert.equal(decision.state, 'approved'); assert.equal(decision.decision.approved, true);
    assert.ok(decision.decision.decided_at); assert.equal(decision.historical, true); assert.equal(decision.resolvable, false);
  }
  assert.deepEqual(views[0].history_runs[this.runId].approvals, views[1].history_runs[this.runId].approvals);
  assert.equal(this.posts, before);
});
Given('an externally captured interrupted native turn across a completed runtime restart', function () {
  const interrupted = this.fixture.interrupted; assert.equal(interrupted?.restartCompleted, true);
  for (const key of ['sessionId', 'clientTurnId', 'userMessageId', 'runId', 'caseId', 'expectedPartial']) assert.ok(interrupted[key]);
  this.session = interrupted.sessionId; this.turn = interrupted.clientTurnId; this.user = interrupted.userMessageId;
  this.runId = interrupted.runId; this.caseId = interrupted.caseId;
  this.evidence.external_restart_case = interrupted;
});
Then('read-only reconciliation retains interrupted partial history without any new provider arrival or effect', async function () {
  assertReceipt(this, await receipt(this), 'interrupted');
  const first = await history(this); const second = await history(this); originalUser(this, first);
  const partial = first.history.filter(row => row.role === 'assistant' && row.run_id === this.runId);
  assert.ok(partial.length > 0 && partial.every(row => row.status === 'interrupted'));
  assert.ok(JSON.stringify(partial).includes(this.fixture.interrupted.expectedPartial));
  assert.deepEqual(first.history.map(row => row.message_id), second.history.map(row => row.message_id));
  assert.equal(this.posts, 0); await counts(this, 0, 0);
});
When('I attach native turn identity to a caller-owned tool batch', async function () {
  const request = body(this); request.tools = [{ type: 'function', function: { name: 'synthetic_caller_owned', description: 'Caller-owned boundary fixture',
    parameters: { type: 'object', properties: {}, additionalProperties: false } } }];
  this.refusal = await http(this, 'POST', '/api/chat/completion', request, [400]);
});
Then('native identity is explicitly unsupported before any provider arrival', async function () {
  assert.equal(this.refusal.error?.code, 'unsupported_native_chat_identity'); await counts(this, 0, 0);
});
After(async function ({ result }) {
  if (!this.evidence) return;
  if (result.status !== 'PASSED') poisoned = true;
  this.evidence.scenario_status = result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  this.evidence.operations = this.operations; this.evidence.native_completion_posts = this.posts;
  this.evidence.run_id = this.runId ?? null;
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: result.status }), 'application/json');
});

When('I send a legacy native chat turn without additive identities', async function () {
  const request = body(this); delete request.client_turn_id; delete request.user_message_id;
  this.legacy = await (await post(this, request)).finished;
});
Then('the legacy stream succeeds without claiming protected turn admission', async function () {
  this.runId = transportRun(this.legacy); assert.ok(this.runId);
  const rows = frames(this.legacy.raw);
  assert.ok(rows.some(row => row.type === 'RUN_FINISHED' && row.runId === this.runId));
  assert.ok(!rows.some(row => row.type === 'RUN_ERROR'));
  assert.equal(this.legacy.request.client_turn_id, undefined); assert.equal(this.legacy.request.user_message_id, undefined);
  await counts(this, 1, 0);
});


When('one native turn completes under a per-case disposable copy of the configured agent', async function () {
  const templateId = this.fixture.agents.ordinary;
  const template = await http(this, 'GET', `/api/agents/${encodeURIComponent(templateId)}`);
  assert.ok(template && typeof template === 'object' && template.id === templateId);
  const ownedId = `${this.fixture.namespace.slice(0, 48)}-receipt-${this.caseId}`;
  assert.ok(!Object.values(this.fixture.agents).includes(ownedId), 'A preprovisioned shared agent must never be mutated');
  await http(this, 'GET', `/api/agents/${encodeURIComponent(ownedId)}`, undefined, [404]);
  const copy = structuredClone(template);
  copy.id = ownedId;
  copy.metadata = { ...copy.metadata, title: `Native receipt disposable ${this.caseId}` };
  const created = await http(this, 'POST', '/api/agents', copy, [201]);
  assert.equal(created.id, ownedId);
  this.disposableAgentId = ownedId;
  this.disposableCreated = true;
  const persisted = await http(this, 'GET', `/api/agents/${encodeURIComponent(ownedId)}`);
  assert.equal(persisted.id, ownedId);
  this.original = { ...body(this), agent_id: ownedId };
  this.originalBodyBeforeDeletion = JSON.stringify(this.original);
  const transport = await (await post(this, this.original)).finished;
  assertReceipt(this, await receipt(this), 'complete');
  assert.equal(transportRun(transport), this.runId);
  const view = await history(this); originalUser(this, view);
  this.historyIdsBeforeDeletion = view.history.map(row => row.message_id);
  await counts(this, 1, 0);
  this.catalogTransitionBefore = await journals(this);
  this.evidence.disposable_catalog_transition = { template_id: templateId, owned_id: ownedId,
    create_status: 201, observed_persisted_id: persisted.id, original_run_id: this.runId,
    before_provider_rows: this.catalogTransitionBefore.provider.length, before_effect_rows: this.catalogTransitionBefore.effects.length };
});
When('I delete only that confirmed per-case agent through the mounted catalog API', async function () {
  assert.equal(this.disposableCreated, true);
  assert.ok(this.disposableAgentId.endsWith(this.caseId));
  assert.equal(this.original.agent_id, this.disposableAgentId);
  assert.ok(!Object.values(this.fixture.agents).includes(this.disposableAgentId));
  await http(this, 'DELETE', `/api/agents/${encodeURIComponent(this.disposableAgentId)}`, undefined, [204]);
  await http(this, 'GET', `/api/agents/${encodeURIComponent(this.disposableAgentId)}`, undefined, [404]);
  this.disposableDeleted = true;
  Object.assign(this.evidence.disposable_catalog_transition, { delete_status: 204, persisted_read_after_delete_status: 404 });
});
Then('the identical original request returns its prior receipt despite the deleted agent and never dispatches again', async function () {
  assert.equal(this.disposableDeleted, true);
  assert.equal(JSON.stringify(this.original), this.originalBodyBeforeDeletion, 'Original request bytes must stay identical across catalog deletion');
  const replay = await (await post(this, this.original)).finished;
  assert.equal(replay.status, 200);
  assert.ok(replay.raw.trim().startsWith('{'), 'Recovery must return the original JSON receipt before mutable agent resolution');
  const recovered = JSON.parse(replay.raw);
  assertReceipt(this, recovered, 'complete');
  assert.equal(transportRun(replay), this.runId);
  const view = await history(this); originalUser(this, view);
  assert.deepEqual(view.history.map(row => row.message_id), this.historyIdsBeforeDeletion);
  const after = await journals(this);
  assert.deepEqual(after.provider, this.catalogTransitionBefore.provider, 'Deletion and identical recovery must cause zero provider arrivals');
  assert.deepEqual(after.effects, this.catalogTransitionBefore.effects, 'Deletion and identical recovery must cause zero additional effects');
  await counts(this, 1, 0);
  Object.assign(this.evidence.disposable_catalog_transition, { recovered_receipt: recovered,
    after_provider_rows: after.provider.length, after_effect_rows: after.effects.length,
    original_request_reused: true });
});

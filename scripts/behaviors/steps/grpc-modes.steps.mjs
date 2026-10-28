import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson } from '../support/answer-oracle.mjs';

setDefaultTimeout(300_000);
// The gRPC toolchain lives in its own folder, outside the pnpm workspace.
const clientRequire = createRequire(resolve('scripts/behaviors/clients/grpc-a2a/package.json'));
const grpc = clientRequire('@grpc/grpc-js');
const protoLoader = clientRequire('@grpc/proto-loader');
const PROTO_OPTIONS = { keepCase: true, longs: String, enums: String, defaults: true, oneofs: true, includeDirs: [resolve('proto')] };
const legacyProto = grpc.loadPackageDefinition(protoLoader.loadSync(resolve('proto/a2a.proto'), PROTO_OPTIONS));
const v1Proto = grpc.loadPackageDefinition(protoLoader.loadSync(resolve('proto/a2a_v1.proto'), PROTO_OPTIONS));
const CALL_TIMEOUT_MS = 170_000;
const POLL_INTERVAL_MS = 1_000;

async function http(w, method, path, payload, expected = 200) {
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
  assert.equal(response.status, expected, `${method} ${path}: HTTP ${response.status}: ${exchange.response.slice(0, 800)}`);
  return exchange.response;
}

// Non-secret selectors only; the external identity host supplies the caller.
function metadata(w) {
  const value = new grpc.Metadata();
  value.set('x-uar-workspace-id', w.workspace);
  value.set('x-uar-agent-id', w.agentId);
  return value;
}

function deadline() { return new Date(Date.now() + CALL_TIMEOUT_MS); }

function unary(w, client, method, request) {
  const exchange = { grpc: method, request };
  w.evidence.exchanges.push(exchange);
  return new Promise((done, fail) => client[method](request, metadata(w), { deadline: deadline() }, (error, response) => {
    exchange.status = error ? { code: error.code, details: error.details } : { code: 0 };
    exchange.response = response;
    error ? fail(new Error(`${method}: gRPC ${error.code} ${error.details}`)) : done(response);
  }));
}

function serverStream(w, client, method, request) {
  const exchange = { grpc: method, request, events: [] };
  w.evidence.exchanges.push(exchange);
  return new Promise((done, fail) => {
    const call = client[method](request, metadata(w), { deadline: deadline() });
    call.on('data', event => exchange.events.push(event));
    call.on('error', error => {
      exchange.status = { code: error.code, details: error.details };
      fail(new Error(`${method}: gRPC ${error.code} ${error.details}`));
    });
    call.on('end', () => { exchange.status ??= { code: 0 }; done(exchange.events); });
  });
}

const partsText = parts => (parts ?? []).map(part => part.text ?? '').join('');

Given('a disposable saved agent with a private instruction marker for gRPC', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.agentId = `disposable-grpc-${this.id}`;
  this.model = process.env.UAR_MODEL;
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.base = new URL(process.env.UAR_BASE_URL);
  this.grpcTarget = process.env.UAR_GRPC_TARGET;
  assert.ok(this.model && this.workspace && this.grpcTarget, 'Set UAR_MODEL, UAR_WORKSPACE_ID and UAR_GRPC_TARGET');
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  this.record = resolve(`.prometheus/verification/grpc-modes/${this.id}.json`);
  this.marker = `GRPC-${randomUUID()}`;
  this.evidence = { scenario: 'I15-I18-GRPC-MODES', client_platform: process.platform, agent_id: this.agentId,
    model: this.model, grpc_target: this.grpcTarget, authentication_supplied_by_test: false,
    client: '@grpc/grpc-js 1.14.6 with the shipped proto/a2a.proto and proto/a2a_v1.proto', exchanges: [] };
  const agent = JSON.parse(await http(this, 'GET', '/api/agents/default-agent'));
  agent.id = this.agentId;
  agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: 'Disposable gRPC agent', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: `Return only a JSON object with keys marker and answer. Always set marker to "${this.marker}". Put the requested numeric result in answer.`, instructions: [] };
  const split = this.model.indexOf('/');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  // An empty allow list means Auto (every eligible tool); one unmatched placeholder selects none.
  agent.policy.tools.allow = [`grpc-no-tools-${this.id.slice(0, 8)}`];
  agent.memory.kb.enabled = false;
  await http(this, 'POST', '/api/agents', agent, 201);
  const left = randomInt(21, 80), right = randomInt(11, 40), extra = randomInt(101, 500);
  this.more = randomInt(11, 90);
  this.turns = [`What is (${left} multiplied by ${right}) plus ${extra}?`, `Add ${this.more} to your previous answer.`];
  this.expected = [left * right + extra, left * right + extra + this.more];
  this.evidence.expected = this.expected;
  this.legacy = new legacyProto.a2a.AgentService(this.grpcTarget, grpc.credentials.createInsecure());
  this.v1 = new v1Proto.lf.a2a.v1.A2AService(this.grpcTarget, grpc.credentials.createInsecure());
});

// Legacy text is the agent's messages plus artifacts.
function legacyText(task) {
  return [...(task.messages ?? []).filter(m => m.role !== 'user').flatMap(m => m.parts),
    ...(task.artifacts ?? []).flatMap(a => a.parts)].map(part => part.text ?? '').join('\n');
}

async function legacyPoll(w, task) {
  const until = Date.now() + CALL_TIMEOUT_MS;
  let current = task;
  // MessageSend may return while the task is still working; the client polls TaskGet.
  while (['submitted', 'working'].includes(current.status) && Date.now() < until) {
    await new Promise(r => setTimeout(r, POLL_INTERVAL_MS));
    current = await unary(w, w.legacy, 'TaskGet', { task_id: current.task_id });
  }
  return current;
}

function legacyRequest(text, taskId) {
  return { task_id: taskId ?? '', message: { role: 'user', parts: [{ text }] } };
}

When('I send a fresh calculation and then a dependent follow-up through legacy gRPC MessageSend', async function () {
  this.mode = 'grpc-legacy-json';
  this.first = await legacyPoll(this, await unary(this, this.legacy, 'MessageSend', legacyRequest(this.turns[0])));
  this.second = await legacyPoll(this, await unary(this, this.legacy, 'MessageSend', legacyRequest(this.turns[1], this.first.task_id)));
});

Then("both legacy gRPC tasks complete with the agent's marker and the correct answers", function () {
  for (const [task, expected] of [[this.first, this.expected[0]], [this.second, this.expected[1]]]) {
    assert.equal(task.status, 'completed', `Task status: ${task.status}`);
    const answer = parseAnswerJson(legacyText(task));
    assert.equal(answer.marker, this.marker);
    assert.equal(Number(answer.answer), expected);
  }
  this.evidence.tasks = [this.first, this.second].map(t => ({ task_id: t.task_id, status: t.status }));
  this.evidence.result = 'VERIFIED';
});

// The legacy stream emits one status snapshot (grpc.rs message_stream contract);
// like tests/user_outcome_grpc.rs, the client resolves that exact task with TaskGet.
async function legacyStreamTurn(w, text, taskId) {
  const events = await serverStream(w, w.legacy, 'MessageStream', legacyRequest(text, taskId));
  const snapshot = events.at(-1)?.state;
  assert.ok(snapshot?.task_id, 'The stream must report its task');
  return { events, task: await legacyPoll(w, snapshot) };
}

When('I send a fresh calculation and then a dependent follow-up through legacy gRPC MessageStream', async function () {
  this.mode = 'grpc-legacy-stream';
  this.first = await legacyStreamTurn(this, this.turns[0]);
  this.second = await legacyStreamTurn(this, this.turns[1], this.first.task.task_id);
});

Then("each legacy gRPC stream is one status snapshot whose task resolves through TaskGet to the agent's marker and the correct answer", function () {
  this.evidence.streams = [this.first, this.second].map(turn => ({
    events: turn.events.map(e => `${e.event_type}:${e.state?.status}`), task_id: turn.task.task_id, status: turn.task.status }));
  for (const [turn, expected] of [[this.first, this.expected[0]], [this.second, this.expected[1]]]) {
    assert.equal(turn.events.length, 1, 'The legacy stream is a single snapshot by contract');
    assert.equal(turn.events[0].event_type, 'status_update');
    assert.equal(turn.task.task_id, turn.events[0].task_id, 'TaskGet resolves the streamed task');
    assert.equal(turn.task.status, 'completed', `Resolved status: ${turn.task.status}`);
    const answer = parseAnswerJson(legacyText(turn.task));
    assert.equal(answer.marker, this.marker);
    assert.equal(Number(answer.answer), expected);
  }
  this.evidence.result = 'VERIFIED';
});

function v1Request(text, contextId) {
  return { message: { message_id: randomUUID(), role: 'ROLE_USER', parts: [{ text }], context_id: contextId ?? '' },
    configuration: { return_immediately: false } };
}

// The answer is the final status message, else the last artifact, else agent history.
function v1Text(task) {
  return partsText(task.status?.message?.parts) || partsText(task.artifacts?.at(-1)?.parts)
    || (task.history ?? []).filter(m => m.role === 'ROLE_AGENT').map(m => partsText(m.parts)).join('\n');
}

When('I send a fresh calculation and then a dependent follow-up through A2A v1 gRPC SendMessage', async function () {
  this.mode = 'grpc-v1-json';
  const first = await unary(this, this.v1, 'SendMessage', v1Request(this.turns[0]));
  assert.equal(first.payload, 'task', `SendMessage payload: ${first.payload}`);
  this.first = first.task;
  const second = await unary(this, this.v1, 'SendMessage', v1Request(this.turns[1], this.first.context_id));
  assert.equal(second.payload, 'task', `SendMessage payload: ${second.payload}`);
  this.second = second.task;
});

Then("both v1 gRPC tasks complete with the agent's marker and the correct answers in one context", function () {
  for (const [task, expected] of [[this.first, this.expected[0]], [this.second, this.expected[1]]]) {
    assert.equal(task.status?.state, 'TASK_STATE_COMPLETED', `Task state: ${task.status?.state}`);
    const answer = parseAnswerJson(v1Text(task));
    assert.equal(answer.marker, this.marker);
    assert.equal(Number(answer.answer), expected);
  }
  assert.ok(this.first.context_id, 'The first task must report a context');
  assert.equal(this.second.context_id, this.first.context_id, 'The follow-up continues the same context');
  this.evidence.tasks = [this.first, this.second].map(t => ({ id: t.id, context_id: t.context_id, state: t.status?.state }));
  this.evidence.result = 'VERIFIED';
});

function v1StreamTurn(events) {
  const kinds = events.map(e => e.payload);
  const task = events.find(e => e.payload === 'task')?.task;
  const finalStatus = events.filter(e => e.payload === 'status_update').at(-1)?.status_update;
  const lastArtifact = events.filter(e => e.payload === 'artifact_update').at(-1)?.artifact_update?.artifact;
  const text = partsText(finalStatus?.status?.message?.parts) || partsText(lastArtifact?.parts) || (task ? v1Text(task) : '');
  return { kinds, task, finalStatus, text };
}

When('I send a fresh calculation and then a dependent follow-up through A2A v1 gRPC SendStreamingMessage', async function () {
  this.mode = 'grpc-v1-stream';
  this.first = v1StreamTurn(await serverStream(this, this.v1, 'SendStreamingMessage', v1Request(this.turns[0])));
  assert.ok(this.first.task?.context_id, 'The stream must start with the task and its context');
  this.second = v1StreamTurn(await serverStream(this, this.v1, 'SendStreamingMessage', v1Request(this.turns[1], this.first.task.context_id)));
});

Then("both v1 gRPC streams progress to a completed task with the agent's marker and the correct answers in one context", function () {
  this.evidence.streams = [this.first, this.second].map(t => ({ task_id: t.task?.id, context_id: t.task?.context_id, events: t.kinds }));
  for (const [turn, expected] of [[this.first, this.expected[0]], [this.second, this.expected[1]]]) {
    assert.ok(turn.kinds.length >= 2, `The stream must progress beyond the initial task: ${turn.kinds.join(',')}`);
    assert.equal(turn.finalStatus?.status?.state, 'TASK_STATE_COMPLETED', `Final state: ${turn.finalStatus?.status?.state}`);
    const answer = parseAnswerJson(turn.text);
    assert.equal(answer.marker, this.marker);
    assert.equal(Number(answer.answer), expected);
  }
  assert.equal(this.second.task?.context_id, this.first.task.context_id, 'The follow-up continues the same context');
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  this.legacy?.close();
  this.v1?.close();
  let cleanupError;
  if (this.created) {
    try { await http(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, 204); }
    catch (error) { cleanupError = error; }
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

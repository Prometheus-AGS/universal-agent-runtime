import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(180_000);
const REFUSED = [403, 404];

// `owner` selects the externally provisioned identity endpoint; no credentials are sent.
async function request(w, owner, method, path, payload, expected = [200]) {
  const response = await fetch(new URL(path, w.bases[owner]), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  const text = await response.text();
  w.evidence.exchanges.push({ owner, method, path, request: payload, status: response.status, response: text });
  assert.ok(expected.includes(response.status), `${owner} ${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return { status: response.status, body: text ? JSON.parse(text) : null };
}

async function prompt(w, owner, content) {
  const payload = { agent_id: w.agentId, model: w.model, session_id: w.session,
    stream: true, stream_mode: 'agui_spec', memory_enabled: false,
    max_completion_tokens: 4096, messages: [{ role: 'user', content }] };
  const exchange = { owner, method: 'POST', path: '/v1/chat/completions', request: payload, response: '' };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(exchange.path, w.bases[owner]), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify(payload),
  });
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.equal(response.status, 200, `${owner} prompt: ${exchange.response.slice(0, 800)}`);
  const events = exchange.response.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const name = lines.find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
    return [JSON.parse(data)];
  });
  assert.ok(!events.some(e => ['RUN_ERROR', 'RUN_CANCELLED'].includes(e.type)), `${owner}'s run must complete`);
  const starts = events.filter(e => e.type === 'RUN_STARTED');
  const ends = events.filter(e => e.type === 'RUN_FINISHED');
  assert.equal(starts.length, 1);
  assert.equal(ends.length, 1);
  const raw = events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta).join('');
  return { runId: starts[0].runId, final: raw.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim() };
}

function parseJson(final) {
  const fenced = final.match(/^```(?:json)?\s*\n([\s\S]*?)\n```$/);
  return JSON.parse(fenced ? fenced[1] : final);
}

const QUESTION = 'What reference did I previously give you, and what is the sum of the two quantities? Return only JSON with reference and total.';

Given("a disposable agent and owner A's private fact in a fresh session", { timeout: 420_000 }, async function () {
  this.started = performance.now();
  this.session = randomUUID();
  this.record = resolve(`.prometheus/verification/owner-isolation/${this.session}.json`);
  this.evidence = { scenario: 'E09-OWNER-ISOLATION', platform: process.platform,
    authentication_supplied_by_test: false, client_replays_history: false, exchanges: [] };
  this.bases = { A: new URL(process.env.UAR_BASE_URL), B: new URL(process.env.UAR_OWNER_B_BASE_URL) };
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  for (const base of Object.values(this.bases)) {
    assert.ok(['http:', 'https:'].includes(base.protocol) && !base.username && !base.password);
  }
  assert.notEqual(this.bases.A.href, this.bases.B.href, 'Owners need distinct identity endpoints');
  assert.ok(this.workspace && this.model);
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.bases.A);
  const ownerB = JSON.parse(await readFile(resolve('target/manual-identity-proxy/launch-owner-b.json'), 'utf8'));
  assert.equal(new URL(ownerB.base_url).href, this.bases.B.href, 'Owner B must be the host-provisioned second endpoint');
  assert.equal(ownerB.upstream, JSON.parse(await readFile(resolve('target/manual-identity-proxy/launch.json'), 'utf8')).upstream);
  this.evidence.owner_b_endpoint = { base_url: ownerB.base_url, subject: ownerB.subject, pid: ownerB.pid };

  const split = this.model.indexOf('/');
  assert.ok(split > 0);
  const agent = (await request(this, 'A', 'GET', '/api/agents/default-agent')).body;
  this.agentId = `disposable-owner-isolation-${this.session}`;
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'Disposable owner-isolation assistant', tags: ['verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Remember the facts the user gives you for subsequent questions in this conversation. Acknowledge the initial facts with only READY, without repeating them. Answer later questions using only facts given earlier in this conversation and the exact JSON fields requested; if no such facts were given, set unknown fields to null.', instructions: [] };
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.skills.prefer = [];
  agent.memory.kb.enabled = false;
  agent.extensions = { ...agent.extensions, 'uar.run_policy': { version: 1,
    context_strategy: { type: 'none' }, memory_enabled: false,
    skills: { mode: 'none' }, tools: { mode: 'none' },
    mcp_servers: { mode: 'none' }, knowledge_bases: { mode: 'none' }, presentations: { mode: 'none' } } };
  await request(this, 'A', 'POST', '/api/agents', agent, [201]);

  this.reference = `CASE-${randomUUID()}`;
  this.left = randomInt(101, 400);
  this.right = randomInt(501, 900);
  this.expected = { reference: this.reference, total: this.left + this.right };
  this.first = await prompt(this, 'A', `Remember for this conversation: reference ${this.reference}; first quantity ${this.left}; second quantity ${this.right}. Acknowledge only READY without repeating any facts.`);
  assert.equal(this.first.final, 'READY');
  this.runA = (await request(this, 'A', 'GET', `/api/uar/runs/${this.first.runId}`)).body;
  assert.equal(this.runA.conversation_id, this.session);
  assert.equal(this.runA.status, 'done');
  Object.assign(this.evidence, { agent_id: this.agentId, session_id: this.session, model: this.model,
    expected: this.expected, owner_a_run_id: this.first.runId, owner_a_run_before: this.runA });
});

When("owner B reads owner A's run and its events and asks to cancel it", async function () {
  const id = encodeURIComponent(this.first.runId);
  this.foreign = {
    read: (await request(this, 'B', 'GET', `/api/uar/runs/${id}`, undefined, REFUSED)).status,
    events: (await request(this, 'B', 'GET', `/api/uar/runs/${id}/events`, undefined, REFUSED)).status,
    cancel: (await request(this, 'B', 'POST', `/api/uar/runs/${id}/cancel`, {}, REFUSED)).status,
  };
  this.evidence.foreign_operations = this.foreign;
});

Then("owner B is refused each operation and owner A's run is unchanged", async function () {
  for (const status of Object.values(this.foreign)) assert.ok(REFUSED.includes(status));
  const after = (await request(this, 'A', 'GET', `/api/uar/runs/${this.first.runId}`)).body;
  this.evidence.owner_a_run_after_foreign = after;
  assert.equal(after.status, this.runA.status);
  assert.equal(after.conversation_id, this.runA.conversation_id);
  assert.equal(after.agent_id, this.runA.agent_id);
});

When('owner B asks about the fact in the same public session ID', async function () {
  for (const fact of [this.reference, String(this.left), String(this.right)]) assert.ok(!QUESTION.includes(fact));
  this.ownerB = await prompt(this, 'B', QUESTION);
  this.evidence.owner_b_run_id = this.ownerB.runId;
  this.evidence.owner_b_final = this.ownerB.final;
});

Then("owner B's answer comes from its own empty partition without owner A's fact", async function () {
  assert.notEqual(this.ownerB.runId, this.first.runId);
  assert.ok(!this.ownerB.final.includes(this.reference), "Owner B must not see owner A's reference");
  assert.ok(!this.ownerB.final.includes(String(this.expected.total)), "Owner B must not see owner A's total");
  const runB = (await request(this, 'B', 'GET', `/api/uar/runs/${this.ownerB.runId}`)).body;
  assert.equal(runB.conversation_id, this.session, 'Owner B used the same public session ID');
  assert.equal(runB.status, 'done');
  this.evidence.owner_b_run = runB;
});

When('the owned UAR restarts with its existing store', { timeout: 360_000 }, async function () {
  this.evidence.restart = {};
  await restartOwnedUar(this.evidence.restart);
  this.evidence.after_endpoint_binding = await assertOwnedEndpoint(this.bases.A);
});

When('owner A asks about the fact in the same session without replaying it', async function () {
  this.second = await prompt(this, 'A', QUESTION);
  assert.notEqual(this.second.runId, this.first.runId);
  this.evidence.owner_a_follow_up_run_id = this.second.runId;
});

Then("owner A still recalls the fact while owner B's run stays invisible to owner A", async function () {
  const run = (await request(this, 'A', 'GET', `/api/uar/runs/${this.second.runId}`)).body;
  assert.equal(run.conversation_id, this.session);
  assert.equal(run.status, 'done');
  const answer = parseJson(this.second.final);
  assert.deepEqual(answer, this.expected);
  this.evidence.owner_a_answer = answer;
  this.evidence.owner_b_run_seen_by_a = (await request(this, 'A', 'GET', `/api/uar/runs/${encodeURIComponent(this.ownerB.runId)}`, undefined, REFUSED)).status;
  assert.equal(this.evidence.restart.previous_process_exited, true);
  assert.equal(this.evidence.restart.new_process_ready, true);
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.created) {
    try { await request(this, 'A', 'DELETE', `/api/agents/${this.agentId}`, undefined, [204]); this.evidence.disposable_agent_deleted = true; }
    catch (error) { errors.push(`Agent cleanup: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Disposable cleanup must succeed');
});

import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, readFile, realpath, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { spawn } from 'node:child_process';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(180_000);
const MAX_APPROVAL_ROUNDS = 4;

async function request(w, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 4000) });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

const RUN_DEADLINE_MS = 150_000;
const POLL_MS = 1_000;
const TERMINAL = new Set(['done', 'error', 'failed', 'cancelled', 'completed']);

const deltaText = events => events.filter(e => e.eventName === 'agui.message.delta')
  .map(e => e.data?.delta?.text ?? '').join('');
const answerText = events => deltaText(events)
  .replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();

const peerRecords = async w => {
  try { return JSON.parse(await readFile(w.recordFile, 'utf8')); } catch (error) { if (error.code === 'ENOENT') return {}; throw error; }
};
const peerLog = async w => {
  try { return (await readFile(w.recordLog, 'utf8')).split('\n').filter(Boolean).map(line => JSON.parse(line)); }
  catch (error) { if (error.code === 'ENOENT') return []; throw error; }
};

// Run one task on the native run API, granting the run the owned loopback peer
// (this deployment disables ambient MCP sharing). Every pending approval for the
// peer's own tools gets the given decision; an approval for any other tool fails.
async function runWithDecision(w, label, task, approved) {
  const sessionId = randomUUID();
  w.threads.push(sessionId);
  const record = { label, task, approved, session_id: sessionId, approvals: [], statuses: [] };
  w.evidence.requests[label] = record;
  const created = await request(w, 'POST', '/api/uar/runs', { agent_id: w.agentId, input: task, session_id: sessionId,
    mcp_servers: [{ name: w.serverName, url: w.peerUrl }] }, [200, 201, 202]);
  const runId = created.run_id ?? created.runId;
  assert.ok(runId, `The run was created: ${JSON.stringify(created).slice(0, 300)}`);
  record.run_id = runId;
  const deadline = Date.now() + RUN_DEADLINE_MS;
  let status;
  while (Date.now() < deadline) {
    const pending = (await request(w, 'GET', `/api/uar/runs/${runId}/tool-approval/pending`)).pending;
    if (pending) {
      const tool = pending.name;
      assert.ok(w.toolNames.includes(tool), `Unexpected approval for ${tool}`);
      assert.ok(record.approvals.length < MAX_APPROVAL_ROUNDS, 'Too many approval requests');
      await request(w, 'POST', `/api/uar/runs/${runId}/tool-approval`, { approved, approval_id: pending.approvalId });
      record.approvals.push({ approval_id: pending.approvalId, tool, approved });
      continue;
    }
    status = (await request(w, 'GET', `/api/uar/runs/${runId}`)).status;
    if (record.statuses.at(-1) !== status) record.statuses.push(status);
    if (TERMINAL.has(status)) break;
    await new Promise(done => setTimeout(done, POLL_MS));
  }
  assert.ok(TERMINAL.has(status), `The run reached a terminal status before the deadline: ${status}`);
  const events = (await request(w, 'GET', `/api/uar/runs/${runId}/events`)).events;
  record.event_names = [...new Set(events.map(e => e.eventName))];
  const toolEvents = events.filter(e => /tool/i.test(e.eventName)).map(e => ({ name: e.eventName, data: e.data }));
  const answer = answerText(events);
  Object.assign(record, { status, tool_events: toolEvents, answer: answer.slice(0, 4000) });
  return { status, toolEvents, answer };
}

Given('an owned MCP record peer is permitted for one disposable agent after an owned restart', { timeout: 420_000 }, async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.short = this.id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/mcp-tool-approval/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.threads = [];
  this.evidence = { scenario: 'C38-MCP-TOOL-APPROVAL-01', platform: process.platform, model: this.model,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, requests: {}, exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model?.includes('/'));
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.base);
  const owned = resolve(`target/c38-mcp-peer/${this.id}`);
  await mkdir(owned, { recursive: true });
  this.ownedDir = await realpath(owned);
  this.recordFile = join(this.ownedDir, 'records.json');
  this.recordLog = join(this.ownedDir, 'calls.jsonl');
  this.serverName = `c38rec${this.short}`;
  this.agentId = `disposable-record-agent-${this.short}`;
  this.toolNames = [`${this.serverName}__record_write`, `${this.serverName}__record_read`];
  // The owned runtime's Cedar policies deny unlisted tools. Permit only this
  // scenario's disposable agent to request the peer's two tools; approval still applies.
  const launch = JSON.parse(await readFile(resolve('target/prompt-macos-launch.json'), 'utf8'));
  this.policyFile = join(dirname(launch.config), 'policies', `c38-mcp-tool-${this.id}.cedar`);
  await writeFile(this.policyFile, this.toolNames.map(tool => `permit (\n    principal == Agent::"${this.agentId}",\n    action == Action::"execute_tool",\n    resource == Tool::"${tool}"\n);\n`).join(''), { flag: 'wx', mode: 0o600 });
  this.policyWritten = true;
  Object.assign(this.evidence, { owned_directory: this.ownedDir, server_name: this.serverName, agent_id: this.agentId,
    scenario_policy: { file: this.policyFile, principal: this.agentId, tools: this.toolNames } });
  this.evidence.restart_enable = {};
  await restartOwnedUar(this.evidence.restart_enable);
});

When('I start the owned MCP peer on loopback HTTP', async function () {
  this.peer = spawn(process.execPath, [resolve('scripts/behaviors/support/record-mcp-peer.mjs')], {
    env: { PATH: process.env.PATH, PEER_HTTP: '1', RECORD_FILE: this.recordFile, RECORD_LOG: this.recordLog },
    stdio: ['ignore', 'pipe', 'pipe'] });
  this.peerUrl = await new Promise((done, fail) => {
    const timer = setTimeout(() => fail(new Error('MCP peer did not report its URL')), 10_000);
    this.peer.stdout.on('data', chunk => {
      const match = /PEER_URL (\S+)/.exec(String(chunk));
      if (match) { clearTimeout(timer); done(match[1]); }
    });
    this.peer.on('exit', code => fail(new Error(`MCP peer exited ${code}`)));
  });
  this.evidence.peer = { url: this.peerUrl, pid: this.peer.pid, transport: 'streamable_http_loopback' };
});

Then('the peer serves its record tools over MCP', async function () {
  const response = await fetch(this.peerUrl, { method: 'POST', signal: AbortSignal.timeout(10_000),
    headers: { 'Content-Type': 'application/json', Accept: 'application/json, text/event-stream' },
    body: JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'tools/list' }) });
  const listing = await response.json();
  this.evidence.peer.tools = listing.result.tools.map(tool => tool.name);
  assert.deepEqual(this.evidence.peer.tools, ['record_write', 'record_read']);
  assert.deepEqual(await peerLog(this), [], 'Listing tools makes no record calls');
});

Given('a disposable agent that may use only the peer\'s record tools', async function () {
  const template = await request(this, 'GET', '/api/agents/default-agent');
  const artifact = structuredClone(template);
  Object.assign(artifact, { id: this.agentId, version: '1.0.0',
    extensions: { 'uar.run_policy': { mcp_servers: { mode: 'selected', ids: [this.serverName] } } } });
  artifact.metadata = { ...artifact.metadata, title: 'Disposable Record Agent', description: 'Stores and reads records through an MCP record peer.', tags: ['records'] };
  artifact.prompt = { system: 'You store and read records only with the record_write and record_read tools. '
    + 'Report only values and revisions returned by the tools. If a tool call is denied or fails, say plainly that the operation was not performed.', instructions: [] };
  artifact.runtime.entry = 'default';
  const split = this.model.indexOf('/');
  artifact.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  artifact.policy.tools.allow = [...this.toolNames];
  artifact.memory.kb.enabled = false;
  assert.equal((await request(this, 'POST', '/api/agents', artifact, 201)).id, this.agentId);
});

When('I ask the agent to store and read back a private value and approve its MCP calls', async function () {
  this.key = `note-${this.short}`;
  this.value = `ORCHID-${randomUUID().slice(0, 8).toUpperCase()}`;
  this.approved = await runWithDecision(this, 'approved',
    `Store the value ${this.value} under the key ${this.key} with record_write. Then read the key ${this.key} back with record_read `
    + 'and tell me the exact value and revision that record_read returned.', true);
});

Then('the peer stored the value once and the answer reports the value and revision the peer returned', async function () {
  const records = await peerRecords(this);
  const log = await peerLog(this);
  Object.assign(this.evidence.requests.approved, { peer_records_after: records, peer_log_after: log });
  assert.deepEqual(records[this.key], { value: this.value, revision: 1 }, 'The peer holds the value once, at revision 1');
  const writes = log.filter(entry => entry.tool === 'record_write');
  const reads = log.filter(entry => entry.tool === 'record_read');
  assert.equal(writes.length, 1, 'Exactly one peer write');
  assert.ok(reads.length >= 1 && reads.some(entry => entry.result?.value === this.value), 'The peer served the read');
  assert.equal(this.approved.status, 'done', 'The approved run completes');
  assert.ok(this.evidence.requests.approved.approvals.some(a => a.tool === this.toolNames[0] && a.approved),
    'record_write was discovered through the run grant and approved through UAR');
  assert.ok(this.approved.answer.includes(this.value), 'The answer reports the value the peer returned');
  assert.match(this.approved.answer, /revision\D{0,20}1\b/i, 'The answer reports the revision the peer returned');
});

When('I ask the agent to overwrite the value and deny the MCP call', async function () {
  this.deniedValue = `CACTUS-${randomUUID().slice(0, 8).toUpperCase()}`;
  this.logBeforeDenial = (await peerLog(this)).length;
  this.denied = await runWithDecision(this, 'denied',
    `Store the value ${this.deniedValue} under the key ${this.key} with record_write and tell me the revision it returns.`, false);
});

Then('the peer record is unchanged and the answer reports that the write was not performed', async function () {
  const records = await peerRecords(this);
  const log = await peerLog(this);
  Object.assign(this.evidence.requests.denied, { peer_records_after: records, peer_log_after: log });
  assert.deepEqual(records[this.key], { value: this.value, revision: 1 }, 'A denied write leaves the record unchanged');
  assert.equal(log.length, this.logBeforeDenial, 'A denied call never reaches the peer');
  assert.ok(this.evidence.requests.denied.approvals.length >= 1 && this.evidence.requests.denied.approvals.every(a => !a.approved),
    'The write requested approval and was denied');
  assert.ok(!this.denied.answer.includes(`revision 2`) && /(den(y|ied)|declin|reject|not (been )?(written|performed|approved|stored|saved)|was not|wasn't|unable|could not|did not)/i.test(this.denied.answer),
    `The denial must be reported honestly: ${this.denied.answer.slice(0, 400)}`);
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 420_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.created) {
    try { await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Agent cleanup: ${error}`); }
  }
  for (const thread of this.threads) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${thread}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${thread}: ${error}`); }
  }
  if (this.peer && this.peer.exitCode === null) {
    try { this.peer.kill('SIGTERM'); } catch (error) { errors.push(`MCP peer stop: ${error}`); }
  }
  if (this.policyWritten) {
    try {
      await rm(this.policyFile, { force: true });
      this.evidence.restart_restore = {};
      await restartOwnedUar(this.evidence.restart_restore);
    } catch (error) { errors.push(`Policy cleanup: ${error}`); }
  }
  if (this.ownedDir) {
    try { await rm(this.ownedDir, { recursive: true, force: true }); }
    catch (error) { errors.push(`Owned directory cleanup: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});

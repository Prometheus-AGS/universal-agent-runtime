import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, readFile, realpath, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { spawn } from 'node:child_process';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(240_000);
const RUN_DEADLINE_MS = 180_000;
const POLL_MS = 1_000;
const TERMINAL = new Set(['done', 'error', 'failed', 'cancelled', 'completed']);
const UNKNOWN = /outcome[_ ]unknown|response lost after dispatch/i;

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

const peerLog = async w => {
  try { return (await readFile(w.recordLog, 'utf8')).split('\n').filter(Boolean).map(line => JSON.parse(line)); }
  catch (error) { if (error.code === 'ENOENT') return []; throw error; }
};
const peerRecords = async w => {
  try { return JSON.parse(await readFile(w.recordFile, 'utf8')); } catch (error) { if (error.code === 'ENOENT') return {}; throw error; }
};
const commits = log => log.filter(entry => entry.tool === 'record_write' && entry.result?.stored);

Given('an owned MCP record peer that loses record_write responses is permitted for one disposable agent after an owned restart', { timeout: 420_000 }, async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.short = this.id.slice(0, 8);
  this.record = resolve(`.prometheus/verification/unknown-tool-effect/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.threads = [];
  this.evidence = { scenario: 'E08-UNKNOWN-TOOL-EFFECT', platform: process.platform, model: this.model,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, approvals: [], exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model?.includes('/'));
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.base);
  const owned = resolve(`target/e08-mcp-peer/${this.id}`);
  await mkdir(owned, { recursive: true });
  this.ownedDir = await realpath(owned);
  this.recordFile = join(this.ownedDir, 'records.json');
  this.recordLog = join(this.ownedDir, 'calls.jsonl');
  this.serverName = `e08rec${this.short}`;
  this.agentId = `disposable-unknown-effect-${this.short}`;
  this.toolNames = [`${this.serverName}__record_write`, `${this.serverName}__record_read`];
  // The owned runtime's Cedar policies deny unlisted tools. Permit only this agent's two peer tools.
  const launch = JSON.parse(await readFile(resolve('target/prompt-macos-launch.json'), 'utf8'));
  this.policyFile = join(dirname(launch.config), 'policies', `e08-mcp-tool-${this.id}.cedar`);
  await writeFile(this.policyFile, this.toolNames.map(tool => `permit (\n    principal == Agent::"${this.agentId}",\n    action == Action::"execute_tool",\n    resource == Tool::"${tool}"\n);\n`).join(''), { flag: 'wx', mode: 0o600 });
  this.policyWritten = true;
  this.evidence.restart_enable = {};
  await restartOwnedUar(this.evidence.restart_enable);
  this.peer = spawn(process.execPath, [resolve('scripts/behaviors/support/record-mcp-peer.mjs')], {
    env: { PATH: process.env.PATH, PEER_HTTP: '1', DROP_RESPONSE_TOOLS: 'record_write', RECORD_FILE: this.recordFile, RECORD_LOG: this.recordLog },
    stdio: ['ignore', 'pipe', 'pipe'] });
  this.peerUrl = await new Promise((done, fail) => {
    const timer = setTimeout(() => fail(new Error('MCP peer did not report its URL')), 10_000);
    this.peer.stdout.on('data', chunk => {
      const match = /PEER_URL (\S+)/.exec(String(chunk));
      if (match) { clearTimeout(timer); done(match[1]); }
    });
    this.peer.on('exit', code => fail(new Error(`MCP peer exited ${code}`)));
  });
  Object.assign(this.evidence, { owned_directory: this.ownedDir, server_name: this.serverName, agent_id: this.agentId,
    peer: { url: this.peerUrl, transport: 'streamable_http_loopback', dropped_tools: ['record_write'] },
    scenario_policy: { file: this.policyFile, principal: this.agentId, tools: this.toolNames } });
  const template = await request(this, 'GET', '/api/agents/default-agent');
  const artifact = structuredClone(template);
  Object.assign(artifact, { id: this.agentId, version: '1.0.0',
    extensions: { 'uar.run_policy': { mcp_servers: { mode: 'selected', ids: [this.serverName] } } } });
  artifact.metadata = { ...artifact.metadata, title: 'Disposable Record Agent', description: 'Stores records through an MCP record peer.', tags: ['records'] };
  artifact.prompt = { system: 'You store records only with the record_write tool. Call record_write at most once per request. '
    + 'Report only what the tool returned. If a tool call fails or its outcome is unknown, do not retry it; say plainly that the outcome is uncertain.', instructions: [] };
  artifact.runtime.entry = 'default';
  const split = this.model.indexOf('/');
  artifact.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  artifact.policy.tools.allow = [...this.toolNames];
  artifact.memory.kb.enabled = false;
  assert.equal((await request(this, 'POST', '/api/agents', artifact, 201)).id, this.agentId);
});

When('I ask the agent to store a private value and approve only its first write', async function () {
  this.key = `note-${this.short}`;
  this.value = `QUARTZ-${randomUUID().slice(0, 8).toUpperCase()}`;
  const sessionId = randomUUID();
  this.threads.push(sessionId);
  const created = await request(this, 'POST', '/api/uar/runs', { agent_id: this.agentId, session_id: sessionId,
    input: `Store the value ${this.value} under the key ${this.key} with record_write, then tell me the result.`,
    mcp_servers: [{ name: this.serverName, url: this.peerUrl }] }, [200, 201, 202]);
  this.runId = created.run_id;
  assert.ok(this.runId);
  const deadline = Date.now() + RUN_DEADLINE_MS;
  let status;
  while (Date.now() < deadline) {
    const pending = (await request(this, 'GET', `/api/uar/runs/${this.runId}/tool-approval/pending`)).pending;
    if (pending) {
      assert.ok(this.toolNames.includes(pending.name), `Unexpected approval for ${pending.name}`);
      // Approve the first write only; any later write request would repeat the uncertain effect.
      const approved = !this.evidence.approvals.some(a => a.approved);
      await request(this, 'POST', `/api/uar/runs/${this.runId}/tool-approval`, { approved, approval_id: pending.approvalId });
      this.evidence.approvals.push({ approval_id: pending.approvalId, tool: pending.name, approved });
      continue;
    }
    status = (await request(this, 'GET', `/api/uar/runs/${this.runId}`)).status;
    if (TERMINAL.has(status)) break;
    await new Promise(done => setTimeout(done, POLL_MS));
  }
  assert.ok(TERMINAL.has(status), `The run reached a terminal status: ${status}`);
  const events = (await request(this, 'GET', `/api/uar/runs/${this.runId}/events`)).events;
  this.toolEvents = events.filter(e => /tool/i.test(e.eventName)).map(e => ({ name: e.eventName, data: e.data }));
  this.answer = events.filter(e => e.eventName === 'agui.message.delta').map(e => e.data?.delta?.text ?? '').join('')
    .replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
  Object.assign(this.evidence, { run_id: this.runId, run_status: status, tool_events: this.toolEvents, answer: this.answer.slice(0, 4000) });
});

Then('the peer committed the write exactly once and the runtime reports the outcome as unknown', async function () {
  const log = await peerLog(this);
  this.evidence.peer_log = log;
  this.evidence.peer_records = await peerRecords(this);
  assert.equal(commits(log).length, 1, 'The write committed exactly once');
  assert.equal(log.filter(entry => entry.response_dropped).length, 1, 'Exactly one response was lost');
  assert.equal(this.evidence.peer_records[this.key]?.value, this.value);
  assert.equal(this.evidence.approvals.filter(a => a.approved).length, 1);
  const results = this.toolEvents.filter(e => /result|complete|end/i.test(e.name) && JSON.stringify(e.data).includes('record_write'));
  assert.ok(this.toolEvents.some(e => UNKNOWN.test(JSON.stringify(e.data))), `The runtime must report an unknown outcome: ${JSON.stringify(this.toolEvents).slice(0, 1500)}`);
  assert.ok(!this.toolEvents.some(e => /"stored":\s*true/.test(JSON.stringify(e.data))), 'No tool event may claim the write succeeded');
  this.evidence.record_write_result_events = results;
  this.effectCount = commits(log).length;
});

When("I reconnect to the finished run's stream", async function () {
  const response = await fetch(new URL(`/api/uar/runs/${this.runId}/stream?stream_mode=agui_spec&last_event_id=0`, this.base), {
    redirect: 'error', signal: AbortSignal.timeout(60_000), headers: { Accept: 'text/event-stream', 'x-uar-workspace-id': this.workspace } });
  const text = await response.text();
  assert.equal(response.status, 200, text.slice(0, 400));
  this.replay = text.split(/\r?\n\r?\n/).flatMap(block => {
    const data = block.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    return data && data !== '[DONE]' ? [JSON.parse(data)] : [];
  });
  this.evidence.replay_types = [...new Set(this.replay.map(e => e.type))];
});

Then('the replay still shows the unknown outcome and the peer effect count is unchanged', async function () {
  const results = this.replay.filter(e => e.type === 'TOOL_CALL_RESULT');
  this.evidence.replay_tool_results = results;
  assert.ok(results.some(e => UNKNOWN.test(JSON.stringify(e))), 'The replay preserves the unknown outcome');
  assert.equal(this.replay.filter(e => e.type === 'RUN_FINISHED' || e.type === 'RUN_ERROR').length, 1);
  assert.equal(commits(await peerLog(this)).length, this.effectCount, 'Reconnecting does not repeat the effect');
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 420_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.peer && this.peer.exitCode === null) this.peer.kill('SIGTERM');
  if (this.created) {
    try { await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Agent cleanup: ${error}`); }
  }
  for (const thread of this.threads) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${thread}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${thread}: ${error}`); }
  }
  if (this.policyWritten) {
    try { await rm(this.policyFile, { force: true }); this.evidence.restart_restore = {}; await restartOwnedUar(this.evidence.restart_restore); }
    catch (error) { errors.push(`Policy restore: ${error}`); }
  }
  if (this.ownedDir) {
    try { await rm(this.ownedDir, { recursive: true, force: true }); } catch (error) { errors.push(`Owned directory cleanup: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});

import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, readFile, realpath, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(125_000);
const TOOL = 'file_write';
const SETTINGS = ['native_tools.file_allowed_paths', 'native_tools.file_tools_enabled'];

async function request(w, method, path, payload, expected = 200, headers = {}) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace, ...headers },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 4000) });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

async function stream(w, payload) {
  const path = `/api/uar/ag-ui/agents/${w.agentId}`;
  const response = await fetch(new URL(path, w.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify(payload),
  });
  const text = await response.text();
  assert.equal(response.status, 200, `${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text.split(/\r?\n\r?\n/).flatMap(block => {
    const data = block.split(/\r?\n/).filter(line => line.startsWith('data:'))
      .map(line => line.slice(5).replace(/^ /, '')).join('\n');
    return data && data !== '[DONE]' ? [JSON.parse(data)] : [];
  });
}

const answerText = events => events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta ?? '').join('')
  .replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();

// One file_write request answered with the given decision. Only an approval interrupt
// for a file_write call this run itself raised may be answered.
async function writeWithDecision(w, label, path, content, approved) {
  const threadId = randomUUID();
  const firstRunId = randomUUID();
  w.threads.push(threadId);
  const task = `Use the ${TOOL} tool exactly once to write this exact content to the file at path ${path}:\n${content}\n`
    + 'After the tool call, report the bytes_written value the tool returned. If the write was not performed, say so plainly.';
  const record = { label, path, content, approved, thread_id: threadId, runs: [] };
  w.evidence.requests[label] = record;
  const first = await stream(w, { threadId, runId: firstRunId, protocolVersion: '1.0',
    messages: [{ id: randomUUID(), role: 'user', content: task }], tools: [], context: [], forwardedProps: { uar: {} } });
  record.runs.push({ run_id: firstRunId, events: first });
  const finished = first.find(e => e.type === 'RUN_FINISHED');
  assert.equal(finished?.outcome?.type, 'interrupt', `Expected a tool approval interrupt: ${JSON.stringify(finished ?? first.at(-1))}`);
  const interrupts = finished.outcome.interrupts;
  assert.equal(interrupts.length, 1, 'Expected exactly one approval request');
  const [interrupt] = interrupts;
  assert.equal(interrupt.reason, 'tool_approval');
  assert.ok(first.some(e => e.type === 'TOOL_CALL_START' && e.toolCallId === interrupt.toolCallId && e.toolCallName === TOOL),
    'The interrupt must belong to this run\'s file_write call');
  const runId = randomUUID();
  const resumed = await stream(w, { threadId, runId, parentRunId: firstRunId, protocolVersion: '1.0',
    messages: [], tools: [], context: [], forwardedProps: { uar: {} },
    resume: [{ interruptId: interrupt.id, status: 'resolved', payload: { approved } }] });
  record.runs.push({ run_id: runId, events: resumed });
  const all = [...first, ...resumed];
  const starts = all.filter(e => e.type === 'TOOL_CALL_START' && e.toolCallName === TOOL);
  const results = all.filter(e => e.type === 'TOOL_CALL_RESULT' && e.toolCallId === interrupt.toolCallId);
  const answer = answerText(resumed);
  Object.assign(record, { interrupt_id: interrupt.id, tool_call_id: interrupt.toolCallId,
    tool_call_starts: starts.length, tool_results: results.map(e => e.content),
    errors: resumed.filter(e => e.type === 'RUN_ERROR'), outcome: resumed.find(e => e.type === 'RUN_FINISHED')?.outcome ?? null,
    answer: answer.slice(0, 4000) });
  return { starts, results, answer, resumed };
}

async function onDisk(path) {
  try { return await readFile(path, 'utf8'); } catch (error) { if (error.code === 'ENOENT') return null; throw error; }
}

async function restoreSettings(w) {
  for (const key of SETTINGS) {
    const original = w.originalSettings[key];
    const current = await request(w, 'GET', `/api/uar/settings/${key}`);
    // A Default-sourced value is restored by reset; startup re-seeds the default row.
    if (original.source === 'Default') {
      await request(w, 'DELETE', `/api/uar/settings/${key}`, undefined, [200, 204], { 'If-Match': current.revision });
    } else {
      await request(w, 'PUT', `/api/uar/settings/${key}`, { value: original.saved, expected_revision: current.revision });
    }
  }
}

Given('native file tools are enabled for one owned directory after an owned restart', { timeout: 420_000 }, async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.record = resolve(`.prometheus/verification/native-tool-approval/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.threads = [];
  this.evidence = { scenario: 'C37-NATIVE-TOOL-APPROVAL-01', platform: process.platform, model: this.model,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, requests: {}, exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model?.includes('/'));
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.base);
  this.originalSettings = {};
  for (const key of SETTINGS) {
    const setting = await request(this, 'GET', `/api/uar/settings/${key}`);
    this.originalSettings[key] = { saved: setting.saved, source: setting.source, revision: setting.revision };
  }
  assert.equal(this.originalSettings['native_tools.file_tools_enabled'].saved, false, 'File tools must start disabled');
  this.evidence.original_settings = this.originalSettings;
  const owned = resolve(`target/c37-native-tool/${this.id}`);
  await mkdir(owned, { recursive: true });
  this.ownedDir = await realpath(owned);
  this.evidence.owned_directory = this.ownedDir;
  // The owned runtime's Cedar policies deny unlisted tools. Permit only this
  // scenario's disposable agent to request file_write; approval still applies.
  this.agentId = `disposable-file-writer-${this.id.slice(0, 8)}`;
  const launch = JSON.parse(await readFile(resolve('target/prompt-macos-launch.json'), 'utf8'));
  this.policyFile = join(dirname(launch.config), 'policies', `c37-native-tool-${this.id}.cedar`);
  await writeFile(this.policyFile, `permit (\n    principal == Agent::"${this.agentId}",\n    action == Action::"execute_tool",\n    resource == Tool::"${TOOL}"\n);\n`, { flag: 'wx', mode: 0o600 });
  this.evidence.scenario_policy = { file: this.policyFile, principal: this.agentId, tool: TOOL };
  this.settingsChanged = true;
  const paths = await request(this, 'GET', `/api/uar/settings/${SETTINGS[0]}`);
  await request(this, 'PUT', `/api/uar/settings/${SETTINGS[0]}`, { value: [this.ownedDir], expected_revision: paths.revision });
  const enabled = await request(this, 'GET', `/api/uar/settings/${SETTINGS[1]}`);
  await request(this, 'PUT', `/api/uar/settings/${SETTINGS[1]}`, { value: true, expected_revision: enabled.revision });
  this.evidence.restart_enable = {};
  await restartOwnedUar(this.evidence.restart_enable);
  for (const [key, value] of [[SETTINGS[0], [this.ownedDir]], [SETTINGS[1], true]]) {
    assert.deepEqual((await request(this, 'GET', `/api/uar/settings/${key}`)).effective, value);
  }
});

Given('a disposable agent that may use only the native file_write tool', async function () {
  const template = await request(this, 'GET', '/api/agents/default-agent');
  const artifact = structuredClone(template);
  Object.assign(artifact, { id: this.agentId, version: '1.0.0', extensions: {} });
  artifact.metadata = { ...artifact.metadata, title: 'Disposable File Writer', description: 'Writes requested notes with the native file_write tool.', tags: ['files'] };
  artifact.prompt = { system: `You write requested notes with the ${TOOL} tool, only inside ${this.ownedDir}. `
    + 'Call the tool at most once per request. Report only facts returned by the tool. If a tool call is denied or fails, say plainly that the file was not written.', instructions: [] };
  artifact.runtime.entry = 'default';
  const split = this.model.indexOf('/');
  artifact.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  artifact.policy.tools.allow = [TOOL];
  artifact.memory.kb.enabled = false;
  assert.equal((await request(this, 'POST', '/api/agents', artifact, 201)).id, this.agentId);
  this.evidence.agent_id = this.agentId;
});

When('I ask the agent to write a private note and approve the file_write request', async function () {
  this.approvedPath = join(this.ownedDir, `approved-${this.id.slice(0, 8)}.txt`);
  this.approvedContent = `C37 approved note ${randomUUID()}`;
  this.approved = await writeWithDecision(this, 'approved', this.approvedPath, this.approvedContent, true);
});

Then('the approved note exists exactly once on disk and the answer reports the tool\'s real result', async function () {
  const disk = await onDisk(this.approvedPath);
  this.evidence.requests.approved.disk_after = disk;
  assert.equal(disk?.trimEnd(), this.approvedContent, 'The approved write must leave exactly the requested content once');
  assert.equal(this.approved.starts.length, 1, 'Exactly one file_write call');
  assert.equal(this.approved.results.length, 1, 'The approved call has one paired result');
  const result = JSON.parse(this.approved.results[0].content);
  assert.equal(result.ok, true);
  assert.equal(result.bytes_written, Buffer.byteLength(disk, 'utf8'));
  assert.ok(this.approved.answer.includes(String(result.bytes_written)), `The answer must report the real bytes_written ${result.bytes_written}`);
});

When('I ask the agent to write a second note and deny the file_write request', async function () {
  this.deniedPath = join(this.ownedDir, `denied-${this.id.slice(0, 8)}.txt`);
  this.deniedContent = `C37 denied note ${randomUUID()}`;
  this.denied = await writeWithDecision(this, 'denied', this.deniedPath, this.deniedContent, false);
});

Then('the second note does not exist and the answer reports that the write was not performed', async function () {
  const disk = await onDisk(this.deniedPath);
  this.evidence.requests.denied.disk_after = disk;
  assert.equal(disk, null, 'A denied write must leave no file');
  assert.equal(await onDisk(this.approvedPath).then(text => text?.trimEnd()), this.approvedContent, 'The approved note is unchanged');
  for (const event of this.denied.results) assert.notEqual(JSON.parse(event.content).ok, true, 'A denied call has no successful result');
  const answer = this.denied.answer;
  const errors = this.evidence.requests.denied.errors;
  assert.ok(errors.length > 0 || /(den(y|ied)|declin|reject|not (been )?(written|performed|approved|created|saved)|wasn't|was not|unable|could not|did not)/i.test(answer),
    `The denial must be reported honestly: ${answer.slice(0, 400)}`);
  assert.ok(!/bytes_written\D{0,20}[1-9]/i.test(answer), 'The answer must not claim a successful write');
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
  if (this.settingsChanged) {
    try {
      await rm(this.policyFile, { force: true });
      await restoreSettings(this);
      this.evidence.restart_restore = {};
      await restartOwnedUar(this.evidence.restart_restore);
      this.evidence.restored_settings = {};
      for (const key of SETTINGS) {
        const setting = await request(this, 'GET', `/api/uar/settings/${key}`);
        this.evidence.restored_settings[key] = { saved: setting.saved, effective: setting.effective, source: setting.source };
        assert.deepEqual(setting.saved, this.originalSettings[key].saved);
      }
    } catch (error) { errors.push(`Settings restore: ${error}`); }
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

import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(65_000);
const agentIds = ['default-agent', 'orchestrator-agent'];

async function request(world, method, id) {
  const path = `/api/agents/${encodeURIComponent(id)}`;
  const headers = process.env.UAR_WORKSPACE_ID
    ? { 'x-uar-workspace-id': process.env.UAR_WORKSPACE_ID } : {};
  const response = await fetch(new URL(path, world.base), {
    method, headers, redirect: 'error', signal: AbortSignal.timeout(30_000),
  });
  const body = await response.text();
  const exchange = { method, path, status: response.status, body };
  world.evidence.exchanges.push(exchange);
  return exchange;
}

Given('the default and orchestrator agent definitions are available', async function () {
  this.started = performance.now();
  this.evidence = { scenario: 'PROTECTED-AGENTS-01', client_platform: process.platform,
    authentication_supplied_by_test: false, service_kept_running: true, exchanges: [] };
  this.record = resolve(`.prometheus/verification/protected-agents/${randomUUID()}.json`);
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol));
  assert.ok(!this.base.username && !this.base.password, 'Use an endpoint without URL credentials');
  this.before = {};
  for (const id of agentIds) {
    const response = await request(this, 'GET', id);
    assert.equal(response.status, 200, `Missing built-in agent ${id}`);
    this.before[id] = JSON.parse(response.body);
    assert.equal(this.before[id].id, id);
  }
});

When('I attempt to delete each built-in agent', async function () {
  this.deletions = {};
  for (const id of agentIds) {
    this.deletions[id] = await request(this, 'DELETE', id);
    assert.equal(this.deletions[id].status, 403, `Deletion of ${id} must be refused`);
  }
});

Then('both deletions are refused because the agents are built in', function () {
  for (const id of agentIds) {
    assert.equal(this.deletions[id].body, `Agent '${id}' is built in and cannot be deleted`);
  }
});

Then('both agent definitions remain unchanged', async function () {
  for (const id of agentIds) {
    const response = await request(this, 'GET', id);
    assert.equal(response.status, 200, `${id} must remain available after rejected deletion`);
    assert.deepEqual(JSON.parse(response.body), this.before[id], `${id} changed after rejected deletion`);
  }
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

import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(120_000);
const path = '/api/uar/user/settings';
const preferences = value => ({ user_id: value.user_id,
  prompt_caching_enabled: value.prompt_caching_enabled, preferred_scope: value.preferred_scope });

async function request(world, method, payload) {
  const exchange = { method, path, request: payload };
  world.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, world.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(20_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': world.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.equal(response.status, 200, `${method} ${path}: ${exchange.response.slice(0, 600)}`);
  return JSON.parse(exchange.response);
}

Given('the original prompt caching preference is recorded', async function () {
  this.started = performance.now();
  this.record = resolve(`.prometheus/verification/rest-setting-restart/${randomUUID()}.json`);
  this.evidence = { scenario: 'REST-SETTING-RESTART-01', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false,
    provider_calls: 0, exchanges: [] };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace);
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.base);
  this.original = await request(this, 'GET');
  assert.ok([null, true, false].includes(this.original.prompt_caching_enabled));
  assert.ok(typeof this.original.user_id === 'string' && this.original.user_id.length > 0);
  this.target = this.original.prompt_caching_enabled !== true;
  assert.notEqual(this.target, this.original.prompt_caching_enabled);
  this.evidence.original = this.original;
  this.evidence.target = this.target;
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(`${this.record}.before-update.json`, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
});

When('I change that preference through the REST API', async function () {
  this.updateAttempted = true;
  this.saved = await request(this, 'PUT', { prompt_caching_enabled: this.target });
  assert.deepEqual(preferences(this.saved), { ...preferences(this.original), prompt_caching_enabled: this.target });
  assert.ok(Number.isFinite(Date.parse(this.saved.updated_at)), 'The saved record has a valid update timestamp');
  assert.deepEqual(await request(this, 'GET'), this.saved);
  this.evidence.saved = this.saved;
  await writeFile(`${this.record}.before-restart.json`, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
});

When('UAR restarts with the existing settings store', { timeout: 360_000 }, async function () {
  this.evidence.restart = {};
  await restartOwnedUar(this.evidence.restart);
  this.evidence.after_endpoint_binding = await assertOwnedEndpoint(this.base);
});

Then('the REST API returns the same saved preference after restart', async function () {
  this.evidence.after_restart = await request(this, 'GET');
  // Full record equality, including updated_at, rules out a new default response.
  assert.deepEqual(this.evidence.after_restart, this.saved);
  assert.notEqual(this.evidence.after_restart.prompt_caching_enabled, this.original.prompt_caching_enabled);
  assert.equal(this.evidence.restart.previous_process_exited, true);
  assert.equal(this.evidence.restart.new_process_ready, true);
  assert.notEqual(this.evidence.restart.previous_pid, this.evidence.restart.new_pid);
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.updateAttempted) {
    try {
      const current = await request(this, 'GET');
      if (JSON.stringify(preferences(current)) !== JSON.stringify(preferences(this.original))) {
        // Refuse to overwrite a separately changed record. With an uncertain PUT
        // response, compare only the fields this test was allowed to change.
        if (this.saved) assert.deepEqual(current, this.saved, 'Settings changed outside this scenario');
        else assert.deepEqual(preferences(current), { ...preferences(this.original), prompt_caching_enabled: this.target });
        await request(this, 'PUT', { prompt_caching_enabled: this.original.prompt_caching_enabled });
      }
      this.evidence.restored = await request(this, 'GET');
      assert.deepEqual(preferences(this.evidence.restored), preferences(this.original));
      this.evidence.original_preference_restored = true;
    } catch (error) { errors.push(`Preference restoration: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Original preference must be restored');
});
